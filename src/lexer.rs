use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    KwInt,
    KwReturn,
    KwIf,
    KwElse,
    KwWhile,
    KwBreak,
    KwContinue,
    KwFor,
    Ident(String),
    IntLiteral(i32),
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Bang,
    Tilde,
    Ampersand,
    Caret,
    Pipe,
    LessLess,
    GreaterGreater,
    Comma,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
}

struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            tokens: Vec::new(),
        }
    }

    fn lex(&mut self) -> Result<Vec<Token>, LexError> {
        while let Some(ch) = self.chars.peek().copied() {
            match ch {
                ' ' | '\n' | '\r' | '\t' => {
                    self.advance();
                }
                ',' => self.advance_and_push(Token::Comma),
                '(' => self.advance_and_push(Token::LParen),
                ')' => self.advance_and_push(Token::RParen),
                '{' => self.advance_and_push(Token::LBrace),
                '}' => self.advance_and_push(Token::RBrace),
                ';' => self.advance_and_push(Token::Semicolon),
                '0'..='9' => {
                    let token = self.lex_int_literal()?;
                    self.push(token);
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let token = self.lex_word();
                    self.push(token);
                }
                '+' => self.advance_and_push(Token::Plus),
                '-' => self.advance_and_push(Token::Minus),
                '*' => self.advance_and_push(Token::Star),
                '/' => {
                    self.advance();
                    if self.consume_if('/') {
                        self.skip_line_comment();
                    } else if self.consume_if('*') {
                        self.skip_block_comment()?
                    } else {
                        self.push(Token::Slash)
                    }
                }
                '%' => self.advance_and_push(Token::Percent),
                '=' => {
                    self.advance();
                    if self.consume_if('=') {
                        self.push(Token::EqualEqual);
                    } else {
                        self.push(Token::Equal);
                    }
                }
                '!' => {
                    self.advance();
                    if self.consume_if('=') {
                        self.push(Token::BangEqual);
                    } else {
                        self.push(Token::Bang);
                    }
                }
                '<' => {
                    self.advance();
                    if self.consume_if('=') {
                        self.push(Token::LessEqual);
                    } else if self.consume_if('<') {
                        self.push(Token::LessLess);
                    } else {
                        self.push(Token::Less);
                    }
                }
                '>' => {
                    self.advance();
                    if self.consume_if('=') {
                        self.push(Token::GreaterEqual);
                    } else if self.consume_if('>') {
                        self.push(Token::GreaterGreater);
                    } else {
                        self.push(Token::Greater);
                    }
                }
                '~' => self.advance_and_push(Token::Tilde),
                '&' => self.advance_and_push(Token::Ampersand),
                '^' => self.advance_and_push(Token::Caret),
                '|' => self.advance_and_push(Token::Pipe),
                _ => {
                    return Err(LexError {
                        message: format!("unexpected character {ch:?}"),
                    });
                }
            }
        }

        self.push(Token::Eof);
        Ok(std::mem::take(&mut self.tokens))
    }

    fn lex_int_literal(&mut self) -> Result<Token, LexError> {
        let mut text = String::new();

        while let Some('0'..='9') = self.peek() {
            text.push(self.advance().expect("peeked character should exist"));
        }

        let value = text.parse::<i32>().map_err(|_| LexError {
            message: format!("integer literal out of range: {text}"),
        })?;

        Ok(Token::IntLiteral(value))
    }

    fn lex_word(&mut self) -> Token {
        let mut text = String::new();

        while let Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') = self.peek() {
            text.push(self.advance().expect("peeked character should exist"));
        }

        match text.as_str() {
            "int" => Token::KwInt,
            "return" => Token::KwReturn,
            "if" => Token::KwIf,
            "else" => Token::KwElse,
            "while" => Token::KwWhile,
            "break" => Token::KwBreak,
            "continue" => Token::KwContinue,
            "for" => Token::KwFor,
            _ => Token::Ident(text),
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(next_char) = self.advance() {
            if next_char == '\n' {
                break;
            }
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), LexError> {
        while let Some(char) = self.advance() {
            if char == '*' {
                if self.consume_if('/') {
                    break;
                }
            }
        }
        if self.eof() {
            return Err(LexError {
                message: format!("unterminated block comment"),
            });
        }
        Ok(())
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        self.chars.next()
    }

    fn consume_if(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            return true;
        }
        false
    }

    fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }

    fn advance_and_push(&mut self, token: Token) {
        self.advance();
        self.push(token);
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

    #[test]
    fn lexes_basic_program() {
        let source = "int main() {\n    return 42;\n}";

        let tokens = lex_with_struct(source).expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwInt,
                Token::Ident("main".to_string()),
                Token::LParen,
                Token::RParen,
                Token::LBrace,
                Token::KwReturn,
                Token::IntLiteral(42),
                Token::Semicolon,
                Token::RBrace,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lexes_plus() {
        let tokens = lex_with_struct("return 1 + 2;").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwReturn,
                Token::IntLiteral(1),
                Token::Plus,
                Token::IntLiteral(2),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lexes_arithmetic() {
        let tokens = lex_with_struct("return 1 - 2 * 3 / 4 % 5;").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwReturn,
                Token::IntLiteral(1),
                Token::Minus,
                Token::IntLiteral(2),
                Token::Star,
                Token::IntLiteral(3),
                Token::Slash,
                Token::IntLiteral(4),
                Token::Percent,
                Token::IntLiteral(5),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lexes_variable_declaration() {
        let tokens = lex_with_struct("int x = 5;").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwInt,
                Token::Ident("x".to_string()),
                Token::Equal,
                Token::IntLiteral(5),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lexes_commas_in_parameter_and_argument_lists() {
        let tokens = lex_with_struct("int add(int x, int y) { return add(x, y); }")
            .expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwInt,
                Token::Ident("add".to_string()),
                Token::LParen,
                Token::KwInt,
                Token::Ident("x".to_string()),
                Token::Comma,
                Token::KwInt,
                Token::Ident("y".to_string()),
                Token::RParen,
                Token::LBrace,
                Token::KwReturn,
                Token::Ident("add".to_string()),
                Token::LParen,
                Token::Ident("x".to_string()),
                Token::Comma,
                Token::Ident("y".to_string()),
                Token::RParen,
                Token::Semicolon,
                Token::RBrace,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lexes_comparison_operators() {
        let tokens = lex_with_struct("return 1 == 2 != 3 < 4 <= 5 > 6 >= 7;")
            .expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwReturn,
                Token::IntLiteral(1),
                Token::EqualEqual,
                Token::IntLiteral(2),
                Token::BangEqual,
                Token::IntLiteral(3),
                Token::Less,
                Token::IntLiteral(4),
                Token::LessEqual,
                Token::IntLiteral(5),
                Token::Greater,
                Token::IntLiteral(6),
                Token::GreaterEqual,
                Token::IntLiteral(7),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lexes_unary_operators() {
        let tokens = lex_with_struct("return -x + !0 + ~1;").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwReturn,
                Token::Minus,
                Token::Ident("x".to_string()),
                Token::Plus,
                Token::Bang,
                Token::IntLiteral(0),
                Token::Plus,
                Token::Tilde,
                Token::IntLiteral(1),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lexes_bitwise_operators() {
        let tokens =
            lex_with_struct("return a & b | c ^ d << 2 >> 1;").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwReturn,
                Token::Ident("a".to_string()),
                Token::Ampersand,
                Token::Ident("b".to_string()),
                Token::Pipe,
                Token::Ident("c".to_string()),
                Token::Caret,
                Token::Ident("d".to_string()),
                Token::LessLess,
                Token::IntLiteral(2),
                Token::GreaterGreater,
                Token::IntLiteral(1),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lexes_shift_operators_separately_from_comparisons() {
        let tokens = lex_with_struct("return a < b <= c << d > e >= f >> g;")
            .expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwReturn,
                Token::Ident("a".to_string()),
                Token::Less,
                Token::Ident("b".to_string()),
                Token::LessEqual,
                Token::Ident("c".to_string()),
                Token::LessLess,
                Token::Ident("d".to_string()),
                Token::Greater,
                Token::Ident("e".to_string()),
                Token::GreaterEqual,
                Token::Ident("f".to_string()),
                Token::GreaterGreater,
                Token::Ident("g".to_string()),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lexes_while_keyword() {
        let tokens = lex_with_struct("while (x) return x;").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwWhile,
                Token::LParen,
                Token::Ident("x".to_string()),
                Token::RParen,
                Token::KwReturn,
                Token::Ident("x".to_string()),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lexes_break_and_continue_keywords() {
        let tokens = lex_with_struct("break; continue;").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwBreak,
                Token::Semicolon,
                Token::KwContinue,
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn lexes_division_after_comment_handling() {
        let tokens = lex_with_struct("return 6 / 2;").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwReturn,
                Token::IntLiteral(6),
                Token::Slash,
                Token::IntLiteral(2),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn skips_line_comments() {
        let tokens = lex_with_struct("return 1; // comment").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwReturn,
                Token::IntLiteral(1),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn skips_block_comments() {
        let tokens = lex_with_struct("return /* comment */ 1;").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwReturn,
                Token::IntLiteral(1),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn skips_block_comments_with_stars_not_followed_by_slashes() {
        let tokens = lex_with_struct("return /* *a ** b* c */ 1;").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwReturn,
                Token::IntLiteral(1),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn skips_line_comment_after_division_expression() {
        let tokens =
            lex_with_struct("return 8 / 2; // trailing comment").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwReturn,
                Token::IntLiteral(8),
                Token::Slash,
                Token::IntLiteral(2),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn skips_block_comment_between_tokens_without_whitespace() {
        let tokens = lex_with_struct("return/* comment */1;").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwReturn,
                Token::IntLiteral(1),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn skips_multiple_block_comments() {
        let tokens =
            lex_with_struct("return /* first */ /* second */ 1;").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwReturn,
                Token::IntLiteral(1),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn skips_empty_block_comments() {
        let tokens = lex_with_struct("return /**/ 1;").expect("lexing should succeed");

        assert_eq!(
            tokens,
            vec![
                Token::KwReturn,
                Token::IntLiteral(1),
                Token::Semicolon,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn rejects_unterminated_block_comment() {
        let err = lex_with_struct("return /* unterminated").expect_err("lexing should fail");

        assert_eq!(err.message, "unterminated block comment");
    }

    #[test]
    fn rejects_unknown_characters() {
        let err = lex_with_struct("int main @").expect_err("lexing should fail");

        assert_eq!(err.message, "unexpected character '@'");
    }
}
