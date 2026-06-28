use std::{iter::Peekable, str::Chars};

use crate::ast::{IntLiteralBase, IntLiteralSuffix};
use crate::source::{FileId, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    KwInt,
    KwChar,
    KwUnsigned,
    KwSigned,
    KwLong,
    KwConst,
    KwReturn,
    KwIf,
    KwElse,
    KwWhile,
    KwBreak,
    KwContinue,
    KwFor,
    KwDo,
    KwTypedef,
    KwSizeof,
    KwVoid,
    KwStatic,
    KwExtern,
    KwStruct,
    Ident(String),
    IntLiteral {
        value: u64,
        suffix: IntLiteralSuffix,
        base: IntLiteralBase,
    },
    CharLiteral(i32),
    StringLiteral(Vec<u8>),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Semicolon,
    Plus,
    PlusEqual,
    PlusPlus,
    Minus,
    MinusEqual,
    MinusMinus,
    Star,
    StarEqual,
    Slash,
    SlashEqual,
    Percent,
    PercentEqual,
    Equal,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    LessLess,
    LessLessEqual,
    Greater,
    GreaterEqual,
    GreaterGreater,
    GreaterGreaterEqual,
    Bang,
    Tilde,
    Ampersand,
    AmpersandEqual,
    AmpersandAmpersand,
    Caret,
    CaretEqual,
    Pipe,
    PipeEqual,
    PipePipe,
    Comma,
    Eof,
    Hash,
    Newline,
    Dot,
    Arrow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    offset: usize,
    file_id: FileId,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str, file_id: FileId) -> Self {
        Self {
            chars: source.chars().peekable(),
            tokens: Vec::new(),
            offset: 0,
            file_id,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn lex(&mut self) -> Result<Vec<Token>, LexError> {
        while let Some(ch) = self.chars.peek().copied() {
            match ch {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                ',' => self.advance_and_push(TokenKind::Comma),
                '(' => self.advance_and_push(TokenKind::LParen),
                ')' => self.advance_and_push(TokenKind::RParen),
                '{' => self.advance_and_push(TokenKind::LBrace),
                '}' => self.advance_and_push(TokenKind::RBrace),
                '[' => self.advance_and_push(TokenKind::LBracket),
                ']' => self.advance_and_push(TokenKind::RBracket),
                ';' => self.advance_and_push(TokenKind::Semicolon),
                '0'..='9' => {
                    let token = self.lex_int_literal()?;
                    self.push_token(token);
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let token = self.lex_word();
                    self.push_token(token);
                }
                '\'' => {
                    let token = self.lex_char_literal()?;
                    self.push_token(token);
                }
                '+' => self.lex_one_or_two_char_operator(
                    TokenKind::Plus,
                    &[('=', TokenKind::PlusEqual), ('+', TokenKind::PlusPlus)],
                ),
                '-' if self.peek_second() == Some('>') => {
                    self.advance();
                    self.advance_and_push(TokenKind::Arrow);
                }
                '-' => self.lex_one_or_two_char_operator(
                    TokenKind::Minus,
                    &[('=', TokenKind::MinusEqual), ('-', TokenKind::MinusMinus)],
                ),
                '*' => self
                    .lex_one_or_two_char_operator(TokenKind::Star, &[('=', TokenKind::StarEqual)]),
                '/' => {
                    let start = self.offset;
                    self.advance();
                    if self.consume_if('/') {
                        self.skip_line_comment();
                    } else if self.consume_if('*') {
                        self.skip_block_comment(start)?;
                    } else if self.consume_if('=') {
                        let end = self.offset;
                        self.push(TokenKind::SlashEqual, self.span(start, end));
                    } else {
                        let end = self.offset;
                        self.push(TokenKind::Slash, self.span(start, end));
                    }
                }
                '%' => self.lex_one_or_two_char_operator(
                    TokenKind::Percent,
                    &[('=', TokenKind::PercentEqual)],
                ),
                '=' => self.lex_one_or_two_char_operator(
                    TokenKind::Equal,
                    &[('=', TokenKind::EqualEqual)],
                ),
                '!' => self
                    .lex_one_or_two_char_operator(TokenKind::Bang, &[('=', TokenKind::BangEqual)]),
                '<' => self.lex_shift_or_compare(
                    '<',
                    TokenKind::Less,
                    TokenKind::LessEqual,
                    TokenKind::LessLess,
                    TokenKind::LessLessEqual,
                ),
                '>' => self.lex_shift_or_compare(
                    '>',
                    TokenKind::Greater,
                    TokenKind::GreaterEqual,
                    TokenKind::GreaterGreater,
                    TokenKind::GreaterGreaterEqual,
                ),
                '~' => self.advance_and_push(TokenKind::Tilde),
                '&' => self.lex_one_or_two_char_operator(
                    TokenKind::Ampersand,
                    &[
                        ('=', TokenKind::AmpersandEqual),
                        ('&', TokenKind::AmpersandAmpersand),
                    ],
                ),
                '^' => self.lex_one_or_two_char_operator(
                    TokenKind::Caret,
                    &[('=', TokenKind::CaretEqual)],
                ),
                '|' => self.lex_one_or_two_char_operator(
                    TokenKind::Pipe,
                    &[('=', TokenKind::PipeEqual), ('|', TokenKind::PipePipe)],
                ),
                '#' => self.advance_and_push(TokenKind::Hash),
                '\n' => self.advance_and_push(TokenKind::Newline),
                '"' => {
                    let token = self.lex_string_literal()?;
                    self.push_token(token);
                }
                '.' => self.advance_and_push(TokenKind::Dot),
                _ => {
                    let start = self.offset;
                    let ch = self.advance().unwrap();

                    return Err(self.error(start, &format!("unexpected character '{ch}'")));
                }
            }
        }

        self.push(TokenKind::Eof, self.span(self.offset, self.offset));
        Ok(std::mem::take(&mut self.tokens))
    }

    fn lex_int_literal(&mut self) -> Result<Token, LexError> {
        let start = self.offset;
        let mut text = String::new();
        let mut unsigned = false;
        let mut long = false;

        let base = if self.peek() == Some('0') && matches!(self.peek_second(), Some('x' | 'X')) {
            self.advance();
            let hex_x = self.advance().unwrap();
            while let Some('0'..='9' | 'a'..='f' | 'A'..='F') = self.peek() {
                text.push(self.advance().expect("peeked character should exist"));
            }
            if text.is_empty() {
                return Err(self.error(start, &format!("expected hex digit after '0{hex_x}'")));
            }
            IntLiteralBase::Hex
        } else {
            while let Some('0'..='9') = self.peek() {
                text.push(self.advance().expect("peeked character should exist"));
            }
            IntLiteralBase::Decimal
        };

        while let Some('u' | 'U' | 'l' | 'L') = self.peek() {
            match self.advance() {
                Some('u' | 'U') => {
                    if unsigned {
                        return Err(self.error(start, "duplicate 'U' integer suffix"));
                    }
                    unsigned = true;
                }
                Some('l' | 'L') => {
                    if long {
                        return Err(
                            self.error(start, "'long long' integer suffix is not supported")
                        );
                    }
                    long = true;
                }
                _ => unreachable!(),
            }
        }

        let value = u64::from_str_radix(&text, base.radix())
            .map_err(|_| self.error(start, &format!("integer literal out of range: {text}")))?;

        Ok(Token {
            kind: TokenKind::IntLiteral {
                value,
                suffix: match (unsigned, long) {
                    (false, false) => IntLiteralSuffix::None,
                    (true, false) => IntLiteralSuffix::Unsigned,
                    (false, true) => IntLiteralSuffix::Long,
                    (true, true) => IntLiteralSuffix::UnsignedLong,
                },
                base,
            },
            span: self.span(start, self.offset),
        })
    }

    fn lex_char_literal(&mut self) -> Result<Token, LexError> {
        let start = self.offset;
        self.advance();

        if self.eof() {
            return Err(self.error(start, "unterminated character constant"));
        }

        if self.peek() == Some('\'') {
            self.advance();
            return Err(self.error(start, "empty character constant"));
        }

        let value = if self.peek() == Some('\\') {
            self.advance();

            let Some(escaped) = self.peek() else {
                return Err(self.error(start, "unterminated character constant"));
            };

            let value = match escaped {
                'n' => '\n' as i32,
                't' => '\t' as i32,
                'r' => '\r' as i32,
                '0' => '\0' as i32,
                '\'' => '\'' as i32,
                '"' => '"' as i32,
                '\\' => '\\' as i32,
                c => {
                    self.advance();
                    return Err(self.error(start, &format!("unknown escape sequence '\\{c}'")));
                }
            };

            self.advance();
            value
        } else {
            let Some(ch) = self.peek() else {
                return Err(self.error(start, "unterminated character constant"));
            };
            self.advance();
            ch as i32
        };

        if self.eof() {
            return Err(self.error(start, "unterminated character constant"));
        }

        if self.peek() != Some('\'') {
            while !self.eof() && self.peek() != Some('\'') {
                self.advance();
            }

            if self.peek() == Some('\'') {
                self.advance();
            }

            return Err(self.error(start, "multi-character constants are not supported"));
        }

        self.advance();

        Ok(Token {
            kind: TokenKind::CharLiteral(value),
            span: self.span(start, self.offset),
        })
    }

    fn lex_string_literal(&mut self) -> Result<Token, LexError> {
        let start = self.offset;
        self.advance();

        let mut bytes = Vec::new();

        loop {
            match self.peek() {
                Some('"') => {
                    self.advance();
                    break;
                }
                Some('\\') => {
                    self.advance();

                    let Some(escaped) = self.peek() else {
                        return Err(self.error(start, "unterminated string literal"));
                    };

                    let byte = match escaped {
                        'n' => b'\n',
                        't' => b'\t',
                        'r' => b'\r',
                        '0' => b'\0',
                        '\'' => b'\'',
                        '"' => b'"',
                        '\\' => b'\\',
                        c => {
                            self.advance();
                            return Err(
                                self.error(start, &format!("unknown escape sequence '\\{c}'"))
                            );
                        }
                    };

                    self.advance();
                    bytes.push(byte);
                }
                Some('\n') | None => {
                    return Err(self.error(start, "unterminated string literal"));
                }
                Some(ch) => {
                    self.advance();

                    if !ch.is_ascii() {
                        return Err(
                            self.error(start, "non-ASCII string literals are not supported")
                        );
                    }

                    bytes.push(ch as u8);
                }
            }
        }

        Ok(Token {
            kind: TokenKind::StringLiteral(bytes),
            span: self.span(start, self.offset),
        })
    }

    fn lex_word(&mut self) -> Token {
        let start = self.offset;
        let mut text = String::new();

        while let Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') = self.peek() {
            text.push(self.advance().expect("peeked character should exist"));
        }

        let end = self.offset;
        let kind = match text.as_str() {
            "int" => TokenKind::KwInt,
            "char" => TokenKind::KwChar,
            "unsigned" => TokenKind::KwUnsigned,
            "signed" => TokenKind::KwSigned,
            "long" => TokenKind::KwLong,
            "const" => TokenKind::KwConst,
            "return" => TokenKind::KwReturn,
            "if" => TokenKind::KwIf,
            "else" => TokenKind::KwElse,
            "while" => TokenKind::KwWhile,
            "break" => TokenKind::KwBreak,
            "continue" => TokenKind::KwContinue,
            "for" => TokenKind::KwFor,
            "do" => TokenKind::KwDo,
            "typedef" => TokenKind::KwTypedef,
            "sizeof" => TokenKind::KwSizeof,
            "void" => TokenKind::KwVoid,
            "static" => TokenKind::KwStatic,
            "extern" => TokenKind::KwExtern,
            "struct" => TokenKind::KwStruct,
            _ => TokenKind::Ident(text),
        };

        Token {
            kind,
            span: self.span(start, end),
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(next_char) = self.advance() {
            if next_char == '\n' {
                break;
            }
        }
    }

    fn skip_block_comment(&mut self, start: usize) -> Result<(), LexError> {
        while let Some(char) = self.advance() {
            if char == '*' && self.consume_if('/') {
                return Ok(());
            }
        }

        Err(self.error(start, "unterminated block comment"))
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn peek_second(&self) -> Option<char> {
        let mut chars = self.chars.clone();
        chars.next()?;
        chars.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.next()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }

    fn consume_if(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            return true;
        }
        false
    }

    fn push(&mut self, kind: TokenKind, span: Span) {
        self.tokens.push(Token { kind, span });
    }

    fn push_token(&mut self, token: Token) {
        self.tokens.push(token);
    }

    fn advance_and_push(&mut self, kind: TokenKind) {
        let start = self.offset;
        self.advance();
        let end = self.offset;

        self.push(kind, self.span(start, end));
    }

    fn lex_one_or_two_char_operator(
        &mut self,
        single: TokenKind,
        alternatives: &[(char, TokenKind)],
    ) {
        let start = self.offset;
        self.advance();

        for (ch, kind) in alternatives {
            if self.consume_if(*ch) {
                let end = self.offset;
                self.push(kind.clone(), self.span(start, end));
                return;
            }
        }

        let end = self.offset;
        self.push(single, self.span(start, end));
    }

    fn lex_shift_or_compare(
        &mut self,
        ch: char,
        single: TokenKind,
        equal: TokenKind,
        shift: TokenKind,
        shift_equal: TokenKind,
    ) {
        let start = self.offset;
        self.advance();

        let end = self.offset;
        if self.consume_if('=') {
            self.push(equal, self.span(start, end));
        } else if self.consume_if(ch) {
            if self.consume_if('=') {
                self.push(shift_equal, self.span(start, end));
            } else {
                self.push(shift, self.span(start, end));
            }
        } else {
            self.push(single, self.span(start, end));
        }
    }

    fn error(&self, start: usize, message: &str) -> LexError {
        LexError {
            message: message.to_string(),
            span: self.span(start, self.offset),
        }
    }

    const fn span(&self, start: usize, end: usize) -> Span {
        Span::new(self.file_id, start, end)
    }

    fn eof(&mut self) -> bool {
        self.chars.peek().is_none()
    }
}

/// Lexes C source text into tokens.
///
/// # Errors
///
/// Returns a [`LexError`] when the source contains an unknown character,
/// malformed character constant, or unterminated block comment.
pub fn lex(source: &str, file_id: FileId) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(source, file_id);
    lexer.lex()
}
