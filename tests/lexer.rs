use rivet::ast::{IntLiteralBase, IntLiteralSuffix};
use rivet::lexer::{LexError, Span, Token, TokenKind, lex};

fn lex_with_struct(source: &str) -> Result<Vec<Token>, LexError> {
    lex(source)
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
            TokenKind::IntLiteral {
                value: 42,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Plus,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Minus,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Star,
            TokenKind::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Slash,
            TokenKind::IntLiteral {
                value: 4,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Percent,
            TokenKind::IntLiteral {
                value: 5,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
            TokenKind::IntLiteral {
                value: 5,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
    let tokens =
        lex_with_struct("return 1 == 2 != 3 < 4 <= 5 > 6 >= 7;").expect("lexing should succeed");

    assert_eq!(
        token_kinds(&tokens),
        vec![
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::EqualEqual,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::BangEqual,
            TokenKind::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Less,
            TokenKind::IntLiteral {
                value: 4,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::LessEqual,
            TokenKind::IntLiteral {
                value: 5,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Greater,
            TokenKind::IntLiteral {
                value: 6,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::GreaterEqual,
            TokenKind::IntLiteral {
                value: 7,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
            TokenKind::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Plus,
            TokenKind::Tilde,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_bitwise_operators() {
    let tokens = lex_with_struct("return a & b | c ^ d << 2 >> 1;").expect("lexing should succeed");

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
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::GreaterGreater,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::Ident("x".to_string()),
            TokenKind::MinusEqual,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::Ident("x".to_string()),
            TokenKind::StarEqual,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::Ident("x".to_string()),
            TokenKind::SlashEqual,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::Ident("x".to_string()),
            TokenKind::PercentEqual,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::Ident("x".to_string()),
            TokenKind::AmpersandEqual,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::Ident("x".to_string()),
            TokenKind::PipeEqual,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::Ident("x".to_string()),
            TokenKind::CaretEqual,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::Ident("x".to_string()),
            TokenKind::LessLessEqual,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::Ident("x".to_string()),
            TokenKind::GreaterGreaterEqual,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
    let tokens =
        lex_with_struct("return a < b <= c << d > e >= f >> g;").expect("lexing should succeed");

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
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
            TokenKind::IntLiteral {
                value: 6,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Slash,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
            TokenKind::IntLiteral {
                value: 8,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Slash,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::AmpersandAmpersand,
            TokenKind::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::PipePipe,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
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
            TokenKind::IntLiteral {
                value: 6,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Ampersand,
            TokenKind::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Pipe,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}
