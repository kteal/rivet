#![allow(dead_code)]

use rivet::ast::{
    ExternalDecl, FunctionDef, Initializer, LocalDecl, Param, ParamDecl, Program, Statement, Type,
};
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

pub fn param_decl(name: &str) -> ParamDecl {
    ParamDecl {
        ty: Type::Int,
        name: Some(name.to_string()),
        name_span: Some(span()),
    }
}

pub fn param_with_span(ty: Type, name: &str, name_span: Span) -> Param {
    Param {
        ty,
        name: name.to_string(),
        name_span,
    }
}

pub fn local_decl(ty: Type, name: &str, init: Option<Initializer>) -> LocalDecl {
    LocalDecl {
        ty,
        name: name.to_string(),
        name_span: span(),
        init,
    }
}

pub fn decl(ty: Type, name: &str, init: Option<Initializer>) -> Statement {
    Statement::Decl(vec![local_decl(ty, name, init)])
}

pub fn program_with_functions(functions: Vec<FunctionDef>) -> Program {
    Program {
        declarations: functions
            .into_iter()
            .map(ExternalDecl::FunctionDef)
            .collect(),
        eof_span: span(),
    }
}

pub fn functions(program: &Program) -> Vec<&FunctionDef> {
    program
        .declarations
        .iter()
        .filter_map(|decl| match decl {
            ExternalDecl::FunctionDef(function) => Some(function),
            ExternalDecl::Typedef(_) | ExternalDecl::FunctionDecl(_) | ExternalDecl::Global(_) => {
                None
            }
        })
        .collect()
}

pub fn first_function(program: &Program) -> &FunctionDef {
    function_at(program, 0)
}

pub fn function_at(program: &Program, index: usize) -> &FunctionDef {
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
            TypedExternalDecl::Typedef | TypedExternalDecl::Global(_) => None,
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
