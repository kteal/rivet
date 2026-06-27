use rivet::ast::{IntLiteralBase, IntLiteralSuffix};
use rivet::lexer::{Token, TokenKind, lex};
use rivet::preprocess::{PreprocessError, preprocess, preprocess_file, splice_escaped_newlines};
use rivet::source::DUMMY_FILE_ID;
use std::fs;

fn preprocess_kinds(source: &str) -> Vec<TokenKind> {
    preprocess_source(source)
        .expect("preprocessing should succeed")
        .into_iter()
        .map(|token| token.kind)
        .collect()
}

fn preprocess_source(source: &str) -> Result<Vec<Token>, PreprocessError> {
    let source = splice_escaped_newlines(source);
    let tokens = lex(&source, DUMMY_FILE_ID).expect("lexing should succeed");
    preprocess(tokens)
}

fn preprocess_file_kinds(source: &str) -> Vec<TokenKind> {
    let tempdir = tempfile::tempdir().expect("failed to create temporary directory");
    let path = tempdir.path().join("source.c");
    fs::write(&path, source).expect("failed to write temporary source file");

    preprocess_file(&path)
        .expect("preprocessing file should succeed")
        .tokens
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
fn splices_escaped_newlines_before_macro_expansion() {
    assert_eq!(
        preprocess_kinds("#define VALUE \\\n7\nint main() { return VALUE; }\n"),
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
fn splices_crlf_escaped_newlines() {
    assert_eq!(
        splice_escaped_newlines("#define VALUE \\\r\n7\r\n"),
        "#define VALUE 7\r\n"
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
    let err = preprocess_source("#define 123 abc\n").expect_err("preprocessing should fail");

    assert_eq!(
        err.message,
        "expected identifier token, got 'IntLiteral { value: 123, suffix: None, base: Decimal }'"
    );
}

#[test]
fn rejects_wrong_function_like_macro_arg_count() {
    let err = preprocess_source("#define ADD(x, y) x + y\nint main() { return ADD(1); }\n")
        .expect_err("preprocessing should fail");

    assert_eq!(
        err.message,
        "macro was defined with '2' parameters, cannot be called with '1' arguments"
    );
}

#[test]
fn rejects_unsupported_preprocessor_directive() {
    let err = preprocess_source("#unsupported <foo>\n").expect_err("preprocessing should fail");

    assert_eq!(
        err.message,
        "unsupported preprocessor directive 'unsupported'"
    );
}

#[test]
fn ifdef_keeps_defined_branch() {
    assert_eq!(
        preprocess_kinds("#define A\n#ifdef A\nint x;\n#endif\n"),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("x".to_string()),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn ifdef_skips_undefined_branch() {
    assert_eq!(
        preprocess_kinds("#ifdef A\nint x;\n#endif\nint y;\n"),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("y".to_string()),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn ifndef_keeps_undefined_branch() {
    assert_eq!(
        preprocess_kinds("#ifndef A\nint x;\n#endif\n"),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("x".to_string()),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn ifndef_skips_defined_branch() {
    assert_eq!(
        preprocess_kinds("#define A\n#ifndef A\nint x;\n#endif\nint y;\n"),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("y".to_string()),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn else_switches_to_untaken_branch() {
    assert_eq!(
        preprocess_kinds("#ifdef A\nint x;\n#else\nint y;\n#endif\n"),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("y".to_string()),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn else_skips_when_first_branch_was_taken() {
    assert_eq!(
        preprocess_kinds("#define A\n#ifdef A\nint x;\n#else\nint y;\n#endif\n"),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("x".to_string()),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn nested_conditionals_stay_inactive_under_inactive_parent() {
    assert_eq!(
        preprocess_kinds(
            "#ifdef OUTER\n#ifdef INNER\nint x;\n#else\nint y;\n#endif\n#else\nint z;\n#endif\n"
        ),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("z".to_string()),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn inactive_define_does_not_define_macro() {
    assert_eq!(
        preprocess_kinds("#ifdef MISSING\n#define VALUE 7\n#endif\nint x = VALUE;\n"),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("x".to_string()),
            TokenKind::Equal,
            TokenKind::Ident("VALUE".to_string()),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn inactive_unsupported_directive_is_skipped() {
    assert_eq!(
        preprocess_kinds("#ifdef MISSING\n#include <foo>\n#endif\nint x;\n"),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("x".to_string()),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn angle_include_uses_reduced_include_directory() {
    assert_eq!(
        preprocess_file_kinds("#include <angle_test.h>\nint y;\n"),
        vec![
            TokenKind::KwInt,
            TokenKind::Ident("included".to_string()),
            TokenKind::Semicolon,
            TokenKind::KwInt,
            TokenKind::Ident("y".to_string()),
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn rejects_else_without_open_conditional() {
    let err = preprocess_source("#else\nint x;\n").expect_err("preprocessing should fail");

    assert_eq!(
        err.message,
        "cannot use #else without opening conditional macro"
    );
}

#[test]
fn rejects_endif_without_open_conditional() {
    let err = preprocess_source("#endif\nint x;\n").expect_err("preprocessing should fail");

    assert_eq!(
        err.message,
        "cannot use #endif without opening conditional macro"
    );
}

#[test]
fn rejects_duplicate_else() {
    let err = preprocess_source("#ifdef A\n#else\n#else\n#endif\n")
        .expect_err("preprocessing should fail");

    assert_eq!(err.message, "cannot use duplicate #else");
}

#[test]
fn rejects_unterminated_conditional() {
    let err = preprocess_source("#ifdef A\nint x;\n").expect_err("preprocessing should fail");

    assert_eq!(err.message, "unterminated conditional directive");
}
