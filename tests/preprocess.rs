use rivet::ast::{IntLiteralBase, IntLiteralSuffix};
use rivet::lexer::{TokenKind, lex};
use rivet::preprocess::preprocess;

fn preprocess_kinds(source: &str) -> Vec<TokenKind> {
    let tokens = lex(source).expect("lexing should succeed");
    preprocess(tokens)
        .expect("preprocessing should succeed")
        .into_iter()
        .map(|token| token.kind)
        .collect()
}

#[test]
fn expands_object_like_macro_tokens() {
    assert_eq!(
        preprocess_kinds("#define BASE 65521U\nint main() { return BASE; }\n"),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 65_521,
                suffix: IntLiteralSuffix::Unsigned,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn expands_empty_object_like_macro_to_no_tokens() {
    assert_eq!(
        preprocess_kinds("#define ZEXPORT\nint ZEXPORT main() { return 7; }\n"),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 7,
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
fn expands_nested_object_like_macro_tokens() {
    assert_eq!(
        preprocess_kinds("#define A B\n#define B 3\nint main() { return A; }\n"),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 3,
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
fn expands_function_like_macro_tokens() {
    assert_eq!(
        preprocess_kinds("#define ADD(x, y) x + y\nint main() { return ADD(2, 3); }\n"),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Plus,
            TokenKind::IntLiteral {
                value: 3,
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
fn expands_zero_arg_function_like_macro_tokens() {
    assert_eq!(
        preprocess_kinds("#define VALUE() 7\nint main() { return VALUE(); }\n"),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 7,
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
fn expands_nested_function_like_macro_tokens() {
    assert_eq!(
        preprocess_kinds(
            "#define DO1(buf, i) buf[i]\n#define DO2(buf, i) DO1(buf, i) + DO1(buf, i + 1)\nint main() { return DO2(a, 0); }\n"
        ),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::Ident("a".to_string()),
            TokenKind::LBracket,
            TokenKind::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::RBracket,
            TokenKind::Plus,
            TokenKind::Ident("a".to_string()),
            TokenKind::LBracket,
            TokenKind::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Plus,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::RBracket,
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn leaves_function_like_macro_name_without_call_unchanged() {
    assert_eq!(
        preprocess_kinds("#define VALUE() 7\nint main() { return VALUE; }\n"),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::Ident("VALUE".to_string()),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn eof_terminates_define_directive() {
    assert_eq!(preprocess_kinds("#define FOO 123"), vec![TokenKind::Eof]);
}

#[test]
fn rejects_define_without_macro_name() {
    let tokens = lex("#define 123 abc\n").expect("lexing should succeed");
    let err = preprocess(tokens).expect_err("preprocessing should fail");

    assert_eq!(
        err.message,
        "expected identifier token, got 'IntLiteral { value: 123, suffix: None, base: Decimal }'"
    );
}

#[test]
fn rejects_wrong_function_like_macro_arg_count() {
    let tokens = lex("#define ADD(x, y) x + y\nint main() { return ADD(1); }\n")
        .expect("lexing should succeed");
    let err = preprocess(tokens).expect_err("preprocessing should fail");

    assert_eq!(
        err.message,
        "macro was defined with '2' parameters, cannot be called with '1' arguments"
    );
}

#[test]
fn rejects_unsupported_preprocessor_directive() {
    let tokens = lex("#include <foo>\n").expect("lexing should succeed");
    let err = preprocess(tokens).expect_err("preprocessing should fail");

    assert_eq!(err.message, "unsupported preprocessor directive 'include'");
}
