#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    KwInt,
    KwReturn,
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
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let mut chars = source.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(ch) = chars.peek().copied() {
        match ch {
            ' ' | '\n' | '\r' | '\t' => {
                chars.next();
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            '{' => {
                chars.next();
                tokens.push(Token::LBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::RBrace);
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }
            '0'..='9' => tokens.push(lex_int_literal(&mut chars)?),
            'a'..='z' | 'A'..='Z' | '_' => tokens.push(lex_word(&mut chars)),
            '+' => {
                chars.next();
                tokens.push(Token::Plus)
            }
            '-' => {
                chars.next();
                tokens.push(Token::Minus)
            }
            '*' => {
                chars.next();
                tokens.push(Token::Star)
            }
            '/' => {
                chars.next();
                if chars.peek().copied() == Some('/') {
                    while let Some(next_char) = chars.next() {
                        if next_char == '\n' {
                            break;
                        }
                    }
                } else if chars.peek().copied() == Some('*') {
                    chars.next();
                    while let Some(next_char) = chars.next() {
                        if next_char == '*' {
                            if Some('/') == chars.next() {
                                break;
                            }
                        }
                    }
                    if chars.peek().is_none() {
                        return Err(LexError {
                            message: format!("unterminated block comment"),
                        });
                    }
                } else {
                    tokens.push(Token::Slash)
                }
            }
            '%' => {
                chars.next();
                tokens.push(Token::Percent)
            }
            '=' => {
                chars.next();
                if chars.peek().copied() == Some('=') {
                    chars.next();
                    tokens.push(Token::EqualEqual);
                } else {
                    tokens.push(Token::Equal);
                }
            }
            '!' => {
                chars.next();
                if chars.peek().copied() == Some('=') {
                    chars.next();
                    tokens.push(Token::BangEqual);
                } else {
                    return Err(LexError {
                        message: format!("unexpected character '!'"),
                    });
                }
            }
            '<' => {
                chars.next();
                if chars.peek().copied() == Some('=') {
                    chars.next();
                    tokens.push(Token::LessEqual);
                } else {
                    tokens.push(Token::Less);
                }
            }
            '>' => {
                chars.next();
                if chars.peek().copied() == Some('=') {
                    chars.next();
                    tokens.push(Token::GreaterEqual);
                } else {
                    tokens.push(Token::Greater);
                }
            }
            _ => {
                return Err(LexError {
                    message: format!("unexpected character {ch:?}"),
                });
            }
        }
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

fn lex_int_literal<I>(chars: &mut std::iter::Peekable<I>) -> Result<Token, LexError>
where
    I: Iterator<Item = char>,
{
    let mut text = String::new();

    while let Some('0'..='9') = chars.peek() {
        text.push(chars.next().expect("peeked character should exist"));
    }

    let value = text.parse::<i32>().map_err(|_| LexError {
        message: format!("integer literal out of range: {text}"),
    })?;

    Ok(Token::IntLiteral(value))
}

fn lex_word<I>(chars: &mut std::iter::Peekable<I>) -> Token
where
    I: Iterator<Item = char>,
{
    let mut text = String::new();

    while let Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') = chars.peek() {
        text.push(chars.next().expect("peeked character should exist"));
    }

    match text.as_str() {
        "int" => Token::KwInt,
        "return" => Token::KwReturn,
        _ => Token::Ident(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_basic_program() {
        let source = "int main() {\n    return 42;\n}";

        let tokens = lex(source).expect("lexing should succeed");

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
        let tokens = lex("return 1 + 2;").expect("lexing should succeed");

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
        let tokens = lex("return 1 - 2 * 3 / 4 % 5;").expect("lexing should succeed");

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
        let tokens = lex("int x = 5;").expect("lexing should succeed");

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
    fn lexes_comparison_operators() {
        let tokens = lex("return 1 == 2 != 3 < 4 <= 5 > 6 >= 7;").expect("lexing should succeed");

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
    fn lexes_division_after_comment_handling() {
        let tokens = lex("return 6 / 2;").expect("lexing should succeed");

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
        let tokens = lex("return 1; // comment").expect("lexing should succeed");

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
        let tokens = lex("return /* comment */ 1;").expect("lexing should succeed");

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
        let tokens = lex("return /**/ 1;").expect("lexing should succeed");

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
        let err = lex("return /* unterminated").expect_err("lexing should fail");

        assert_eq!(err.message, "unterminated block comment");
    }

    #[test]
    fn rejects_lone_bang() {
        let err = lex("return !1;").expect_err("lexing should fail");

        assert_eq!(err.message, "unexpected character '!'");
    }

    #[test]
    fn rejects_unknown_characters() {
        let err = lex("int main @").expect_err("lexing should fail");

        assert_eq!(err.message, "unexpected character '@'");
    }
}
