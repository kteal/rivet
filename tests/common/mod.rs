use rivet::ast::{Param, Type};
use rivet::lexer::Span;

pub const fn span() -> Span {
    Span { start: 0, end: 0 }
}

#[allow(dead_code)]
pub const fn span_from(start: usize, end: usize) -> Span {
    Span { start, end }
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
