use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    KwInt,
    KwChar,
    KwReturn,
    KwIf,
    KwElse,
    KwWhile,
    KwBreak,
    KwContinue,
    KwFor,
    KwDo,
    Ident(String),
    IntLiteral(i32),
    CharLiteral(i32),
    LParen,
    RParen,
    LBrace,
    RBrace,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
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
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            tokens: Vec::new(),
            offset: 0,
        }
    }

    fn lex(&mut self) -> Result<Vec<Token>, LexError> {
        while let Some(ch) = self.chars.peek().copied() {
            match ch {
                ' ' | '\n' | '\r' | '\t' => {
                    self.advance();
                }
                ',' => self.advance_and_push(TokenKind::Comma),
                '(' => self.advance_and_push(TokenKind::LParen),
                ')' => self.advance_and_push(TokenKind::RParen),
                '{' => self.advance_and_push(TokenKind::LBrace),
                '}' => self.advance_and_push(TokenKind::RBrace),
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
                        self.skip_block_comment(start)?
                    } else if self.consume_if('=') {
                        let end = self.offset;
                        self.push(TokenKind::SlashEqual, Span { start, end });
                    } else {
                        let end = self.offset;
                        self.push(TokenKind::Slash, Span { start, end })
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
                _ => {
                    let start = self.offset;
                    let ch = self.advance().unwrap();
                    let end = self.offset;

                    return Err(LexError {
                        message: format!("unexpected character {ch:?}"),
                        span: Span { start, end },
                    });
                }
            }
        }

        self.push(
            TokenKind::Eof,
            Span {
                start: self.offset,
                end: self.offset,
            },
        );
        Ok(std::mem::take(&mut self.tokens))
    }

    fn lex_int_literal(&mut self) -> Result<Token, LexError> {
        let start = self.offset;
        let mut text = String::new();

        while let Some('0'..='9') = self.peek() {
            text.push(self.advance().expect("peeked character should exist"));
        }

        let end = self.offset;
        let value = text.parse::<i32>().map_err(|_| LexError {
            message: format!("integer literal out of range: {text}"),
            span: Span { start, end },
        })?;

        Ok(Token {
            kind: TokenKind::IntLiteral(value),
            span: Span { start, end },
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
                    return Err(LexError {
                        message: format!("unknown escape sequence '\\{c}'"),
                        span: Span {
                            start,
                            end: self.offset,
                        },
                    });
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
            span: Span {
                start,
                end: self.offset,
            },
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
            "return" => TokenKind::KwReturn,
            "if" => TokenKind::KwIf,
            "else" => TokenKind::KwElse,
            "while" => TokenKind::KwWhile,
            "break" => TokenKind::KwBreak,
            "continue" => TokenKind::KwContinue,
            "for" => TokenKind::KwFor,
            "do" => TokenKind::KwDo,
            _ => TokenKind::Ident(text),
        };

        Token {
            kind,
            span: Span { start, end },
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

        Err(LexError {
            message: format!("unterminated block comment"),
            span: Span {
                start,
                end: self.offset,
            },
        })
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
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
        self.tokens.push(token)
    }

    fn advance_and_push(&mut self, kind: TokenKind) {
        let start = self.offset;
        self.advance();
        let end = self.offset;

        self.push(kind, Span { start, end });
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
                self.push(kind.clone(), Span { start, end });
                return;
            }
        }

        let end = self.offset;
        self.push(single, Span { start, end });
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

        if self.consume_if('=') {
            let end = self.offset;
            self.push(equal, Span { start, end });
        } else if self.consume_if(ch) {
            if self.consume_if('=') {
                let end = self.offset;
                self.push(shift_equal, Span { start, end });
            } else {
                let end = self.offset;
                self.push(shift, Span { start, end });
            }
        } else {
            let end = self.offset;
            self.push(single, Span { start, end });
        }
    }

    fn error(&self, start: usize, message: &str) -> LexError {
        LexError {
            message: message.to_string(),
            span: Span {
                start,
                end: self.offset,
            },
        }
    }

    fn eof(&mut self) -> bool {
        self.chars.peek().is_none()
    }
}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(source);
    lexer.lex()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex_with_struct(source: &str) -> Result<Vec<Token>, LexError> {
        let mut lexer = Lexer::new(source);
        lexer.lex()
    }

    fn token_kinds(tokens: &[Token]) -> Vec<TokenKind> {
        tokens.iter().map(|token| token.kind.clone()).collect()
    }

    fn token_spans(tokens: &[Token]) -> Vec<Span> {
        tokens.iter().map(|token| token.span).collect()
    }

    #[test]
    fn lexes_token_spans() {
        let tokens = lex_with_struct("int x;").expect("lexing should succeed");

        assert_eq!(
            token_spans(&tokens),
            vec![
                Span { start: 0, end: 3 },
                Span { start: 4, end: 5 },
                Span { start: 5, end: 6 },
                Span { start: 6, end: 6 },
            ]
        );
    }

    #[test]
    fn lexes_basic_program() {
        let source = "int main() {\n    return 42;\n}";

        let tokens = lex_with_struct(source).expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwInt,
                TokenKind::Ident("main".to_string()),
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::LBrace,
                TokenKind::KwReturn,
                TokenKind::IntLiteral(42),
                TokenKind::Semicolon,
                TokenKind::RBrace,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_plus() {
        let tokens = lex_with_struct("return 1 + 2;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::IntLiteral(1),
                TokenKind::Plus,
                TokenKind::IntLiteral(2),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_arithmetic() {
        let tokens = lex_with_struct("return 1 - 2 * 3 / 4 % 5;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::IntLiteral(1),
                TokenKind::Minus,
                TokenKind::IntLiteral(2),
                TokenKind::Star,
                TokenKind::IntLiteral(3),
                TokenKind::Slash,
                TokenKind::IntLiteral(4),
                TokenKind::Percent,
                TokenKind::IntLiteral(5),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_variable_declaration() {
        let tokens = lex_with_struct("int x = 5;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwInt,
                TokenKind::Ident("x".to_string()),
                TokenKind::Equal,
                TokenKind::IntLiteral(5),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_commas_in_parameter_and_argument_lists() {
        let tokens = lex_with_struct("int add(int x, int y) { return add(x, y); }")
            .expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwInt,
                TokenKind::Ident("add".to_string()),
                TokenKind::LParen,
                TokenKind::KwInt,
                TokenKind::Ident("x".to_string()),
                TokenKind::Comma,
                TokenKind::KwInt,
                TokenKind::Ident("y".to_string()),
                TokenKind::RParen,
                TokenKind::LBrace,
                TokenKind::KwReturn,
                TokenKind::Ident("add".to_string()),
                TokenKind::LParen,
                TokenKind::Ident("x".to_string()),
                TokenKind::Comma,
                TokenKind::Ident("y".to_string()),
                TokenKind::RParen,
                TokenKind::Semicolon,
                TokenKind::RBrace,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_comparison_operators() {
        let tokens = lex_with_struct("return 1 == 2 != 3 < 4 <= 5 > 6 >= 7;")
            .expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::IntLiteral(1),
                TokenKind::EqualEqual,
                TokenKind::IntLiteral(2),
                TokenKind::BangEqual,
                TokenKind::IntLiteral(3),
                TokenKind::Less,
                TokenKind::IntLiteral(4),
                TokenKind::LessEqual,
                TokenKind::IntLiteral(5),
                TokenKind::Greater,
                TokenKind::IntLiteral(6),
                TokenKind::GreaterEqual,
                TokenKind::IntLiteral(7),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_unary_operators() {
        let tokens = lex_with_struct("return -x + !0 + ~1;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::Minus,
                TokenKind::Ident("x".to_string()),
                TokenKind::Plus,
                TokenKind::Bang,
                TokenKind::IntLiteral(0),
                TokenKind::Plus,
                TokenKind::Tilde,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_bitwise_operators() {
        let tokens =
            lex_with_struct("return a & b | c ^ d << 2 >> 1;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::Ident("a".to_string()),
                TokenKind::Ampersand,
                TokenKind::Ident("b".to_string()),
                TokenKind::Pipe,
                TokenKind::Ident("c".to_string()),
                TokenKind::Caret,
                TokenKind::Ident("d".to_string()),
                TokenKind::LessLess,
                TokenKind::IntLiteral(2),
                TokenKind::GreaterGreater,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_compound_assignment_operators() {
        let tokens = lex_with_struct(
            "x += 1; x -= 1; x *= 1; x /= 1; x %= 1; x &= 1; x |= 1; x ^= 1; x <<= 1; x >>= 1;",
        )
        .expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::Ident("x".to_string()),
                TokenKind::PlusEqual,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Ident("x".to_string()),
                TokenKind::MinusEqual,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Ident("x".to_string()),
                TokenKind::StarEqual,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Ident("x".to_string()),
                TokenKind::SlashEqual,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Ident("x".to_string()),
                TokenKind::PercentEqual,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Ident("x".to_string()),
                TokenKind::AmpersandEqual,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Ident("x".to_string()),
                TokenKind::PipeEqual,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Ident("x".to_string()),
                TokenKind::CaretEqual,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Ident("x".to_string()),
                TokenKind::LessLessEqual,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Ident("x".to_string()),
                TokenKind::GreaterGreaterEqual,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_increment_and_decrement_operators() {
        let tokens = lex_with_struct("x++; ++x; x--; --x;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::Ident("x".to_string()),
                TokenKind::PlusPlus,
                TokenKind::Semicolon,
                TokenKind::PlusPlus,
                TokenKind::Ident("x".to_string()),
                TokenKind::Semicolon,
                TokenKind::Ident("x".to_string()),
                TokenKind::MinusMinus,
                TokenKind::Semicolon,
                TokenKind::MinusMinus,
                TokenKind::Ident("x".to_string()),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_shift_operators_separately_from_comparisons() {
        let tokens = lex_with_struct("return a < b <= c << d > e >= f >> g;")
            .expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::Ident("a".to_string()),
                TokenKind::Less,
                TokenKind::Ident("b".to_string()),
                TokenKind::LessEqual,
                TokenKind::Ident("c".to_string()),
                TokenKind::LessLess,
                TokenKind::Ident("d".to_string()),
                TokenKind::Greater,
                TokenKind::Ident("e".to_string()),
                TokenKind::GreaterEqual,
                TokenKind::Ident("f".to_string()),
                TokenKind::GreaterGreater,
                TokenKind::Ident("g".to_string()),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_while_keyword() {
        let tokens = lex_with_struct("while (x) return x;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwWhile,
                TokenKind::LParen,
                TokenKind::Ident("x".to_string()),
                TokenKind::RParen,
                TokenKind::KwReturn,
                TokenKind::Ident("x".to_string()),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_do_keyword() {
        let tokens = lex_with_struct("do x = x - 1; while (x);").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwDo,
                TokenKind::Ident("x".to_string()),
                TokenKind::Equal,
                TokenKind::Ident("x".to_string()),
                TokenKind::Minus,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::KwWhile,
                TokenKind::LParen,
                TokenKind::Ident("x".to_string()),
                TokenKind::RParen,
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_char_keyword() {
        let tokens = lex_with_struct("char x;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwChar,
                TokenKind::Ident("x".to_string()),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_char_literals() {
        let tokens = lex_with_struct("return 'A';").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::CharLiteral(65),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_char_literal_escapes() {
        let tokens = lex_with_struct("'\\n' '\\0' '\\'' '\\\\' '\\t' '\\r' '\"'")
            .expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::CharLiteral(10),
                TokenKind::CharLiteral(0),
                TokenKind::CharLiteral(39),
                TokenKind::CharLiteral(92),
                TokenKind::CharLiteral(9),
                TokenKind::CharLiteral(13),
                TokenKind::CharLiteral(34),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_char_literal_spans() {
        let tokens = lex_with_struct("'A' '\\n'").expect("lexing should succeed");

        assert_eq!(
            token_spans(&tokens),
            vec![
                Span { start: 0, end: 3 },
                Span { start: 4, end: 8 },
                Span { start: 8, end: 8 },
            ]
        );
    }

    #[test]
    fn rejects_empty_char_literal() {
        let err = lex_with_struct("''").expect_err("lexing should fail");

        assert_eq!(err.message, "empty character constant");
        assert_eq!(err.span, Span { start: 0, end: 2 });
    }

    #[test]
    fn rejects_unterminated_char_literal() {
        let err = lex_with_struct("'A").expect_err("lexing should fail");

        assert_eq!(err.message, "unterminated character constant");
        assert_eq!(err.span, Span { start: 0, end: 2 });
    }

    #[test]
    fn rejects_unknown_char_literal_escape() {
        let err = lex_with_struct("'\\q'").expect_err("lexing should fail");

        assert_eq!(err.message, "unknown escape sequence '\\q'");
        assert_eq!(err.span, Span { start: 0, end: 3 });
    }

    #[test]
    fn rejects_multi_character_literal() {
        let err = lex_with_struct("'ab'").expect_err("lexing should fail");

        assert_eq!(err.message, "multi-character constants are not supported");
        assert_eq!(err.span, Span { start: 0, end: 4 });
    }

    #[test]
    fn lexes_break_and_continue_keywords() {
        let tokens = lex_with_struct("break; continue;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwBreak,
                TokenKind::Semicolon,
                TokenKind::KwContinue,
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_division_after_comment_handling() {
        let tokens = lex_with_struct("return 6 / 2;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::IntLiteral(6),
                TokenKind::Slash,
                TokenKind::IntLiteral(2),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_line_comments() {
        let tokens = lex_with_struct("return 1; // comment").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_block_comments() {
        let tokens = lex_with_struct("return /* comment */ 1;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_block_comments_with_stars_not_followed_by_slashes() {
        let tokens = lex_with_struct("return /* *a ** b* c */ 1;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_line_comment_after_division_expression() {
        let tokens =
            lex_with_struct("return 8 / 2; // trailing comment").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::IntLiteral(8),
                TokenKind::Slash,
                TokenKind::IntLiteral(2),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_block_comment_between_tokens_without_whitespace() {
        let tokens = lex_with_struct("return/* comment */1;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_multiple_block_comments() {
        let tokens =
            lex_with_struct("return /* first */ /* second */ 1;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_empty_block_comments() {
        let tokens = lex_with_struct("return /**/ 1;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn rejects_unterminated_block_comment() {
        let source = "return /* unterminated";
        let err = lex_with_struct(source).expect_err("lexing should fail");

        assert_eq!(err.message, "unterminated block comment");
        assert_eq!(
            err.span,
            Span {
                start: 7,
                end: source.len()
            }
        );
    }

    #[test]
    fn rejects_unknown_characters() {
        let err = lex_with_struct("int main @").expect_err("lexing should fail");

        assert_eq!(err.message, "unexpected character '@'");
        assert_eq!(err.span, Span { start: 9, end: 10 });
    }

    #[test]
    fn lexes_logical_and_or_operators() {
        let tokens = lex_with_struct("return 1 && 0 || 2;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::IntLiteral(1),
                TokenKind::AmpersandAmpersand,
                TokenKind::IntLiteral(0),
                TokenKind::PipePipe,
                TokenKind::IntLiteral(2),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lexes_single_bitwise_operators_after_logical_operator_support() {
        let tokens = lex_with_struct("return 6&3|1;").expect("lexing should succeed");

        assert_eq!(
            token_kinds(&tokens),
            vec![
                TokenKind::KwReturn,
                TokenKind::IntLiteral(6),
                TokenKind::Ampersand,
                TokenKind::IntLiteral(3),
                TokenKind::Pipe,
                TokenKind::IntLiteral(1),
                TokenKind::Semicolon,
                TokenKind::Eof,
            ]
        );
    }
}
