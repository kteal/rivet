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
fn eof_terminates_define_directive() {
    assert_eq!(
        preprocess_kinds("#define FOO 123"),
        vec![TokenKind::Eof]
    );
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
