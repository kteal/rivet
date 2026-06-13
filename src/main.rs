use std::env;
use std::fs;
use std::path::Path;
use std::process;

use rivet::codegen::{CodegenTarget, generate};
use rivet::lexer::Span;
use rivet::parser::parse;
use rivet::preprocess::PreprocessFileError;
use rivet::preprocess::preprocess_file;
use rivet::sema::check;

fn line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;

    for (byte_index, ch) in source.char_indices() {
        if byte_index >= offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    (line, col)
}

fn print_error(path: &str, source: &str, span: Span, message: &str) {
    let (line, col) = line_col(source, span.start);
    eprintln!("{path}:{line}:{col}: error: {message}");
}

fn main() {
    let mut args = env::args();
    let program_name = args.next().unwrap_or_else(|| "rivet".to_string());

    let Some(path) = args.next() else {
        eprintln!("usage: {program_name} <source.c>");
        process::exit(2);
    };

    if args.next().is_some() {
        eprintln!("usage: {program_name} <source.c>");
        process::exit(2);
    }

    // For diagnostic message purposes
    let source = fs::read_to_string(&path).unwrap_or_else(|err| {
        eprintln!("failed to read {path}: {err}");
        process::exit(1);
    });

    let tokens = preprocess_file(Path::new(&path)).unwrap_or_else(|err| {
        match err {
            PreprocessFileError::Io { path, message } => {
                eprintln!("failed to read path '{}': {message}", path.display());
            }
            PreprocessFileError::Lex {
                path,
                source,
                span,
                message,
            }
            | PreprocessFileError::Preprocess {
                path,
                source,
                span,
                message,
            } => {
                print_error(&path.display().to_string(), &source, span, &message);
            }
        }
        process::exit(1);
    });

    let program = parse(tokens).unwrap_or_else(|err| {
        print_error(&path, &source, err.span, &err.message);
        process::exit(1);
    });

    let typed_program = check(&program).unwrap_or_else(|err| {
        print_error(&path, &source, err.span, &err.message);
        process::exit(1);
    });

    print!("{}", generate(&typed_program, CodegenTarget::Rv32));
}
