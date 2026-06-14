#![allow(dead_code)]

use rivet::ast::{ExternalDecl, Function, Param, Program, Type};
use rivet::source::{DUMMY_FILE_ID, Span};

use rivet::typed_ast::{TypedExternalDecl, TypedFunction, TypedProgram};

pub const fn span() -> Span {
    Span {
        file_id: DUMMY_FILE_ID,
        start: 0,
        end: 0,
    }
}

#[allow(dead_code)]
pub const fn span_from(start: usize, end: usize) -> Span {
    Span {
        file_id: DUMMY_FILE_ID,
        start,
        end,
    }
}

pub fn param(name: &str) -> Param {
    param_with_span(Type::Int, name, span())
}

pub fn param_with_span(ty: Type, name: &str, name_span: Span) -> Param {
    Param {
        ty,
        name: name.to_string(),
        name_span,
    }
}

pub fn program_with_functions(functions: Vec<Function>) -> Program {
    Program {
        declarations: functions.into_iter().map(ExternalDecl::Function).collect(),
        eof_span: span(),
    }
}

pub fn functions(program: &Program) -> Vec<&Function> {
    program
        .declarations
        .iter()
        .filter_map(|decl| match decl {
            ExternalDecl::Function(function) => Some(function),
            ExternalDecl::Typedef(_) => None,
        })
        .collect()
}

pub fn first_function(program: &Program) -> &Function {
    function_at(program, 0)
}

pub fn function_at(program: &Program, index: usize) -> &Function {
    functions(program)
        .into_iter()
        .nth(index)
        .expect("expected function declaration")
}

pub fn typed_functions(program: &TypedProgram) -> Vec<&TypedFunction> {
    program
        .declarations
        .iter()
        .filter_map(|decl| match decl {
            TypedExternalDecl::Function(function) => Some(function),
            TypedExternalDecl::Typedef => None,
        })
        .collect()
}

pub fn first_typed_function(program: &TypedProgram) -> &TypedFunction {
    typed_function_at(program, 0)
}

pub fn typed_function_at(program: &TypedProgram, index: usize) -> &TypedFunction {
    typed_functions(program)
        .into_iter()
        .nth(index)
        .expect("expected typed function declaration")
}
