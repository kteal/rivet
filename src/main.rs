use std::env;
use std::fs;
use std::process;

use rivet::codegen::{CodegenTarget, generate};
use rivet::lexer::lex;
use rivet::parser::parse;
use rivet::sema::check;

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

    let source = fs::read_to_string(&path).unwrap_or_else(|err| {
        eprintln!("failed to read {path}: {err}");
        process::exit(1);
    });

    let tokens = lex(&source).unwrap_or_else(|err| {
        eprintln!("lex error: {}", err.message);
        process::exit(1);
    });

    let program = parse(tokens).unwrap_or_else(|err| {
        eprintln!("parse error: {}", err.message);
        process::exit(1);
    });

    if let Err(err) = check(&program) {
        eprintln!("semantic analysis error: {}", err.message);
        process::exit(1);
    }

    print!("{}", generate(&program, CodegenTarget::Rv32));
}
