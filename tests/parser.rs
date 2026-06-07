use rivet::ast::{Expr, Statement, Type, UnaryOp};
use rivet::lexer::lex;
use rivet::parser::parse;

fn parse_source(source: &str) -> rivet::ast::Program {
    let tokens = lex(source).expect("lexing should succeed");
    parse(tokens).expect("parsing should succeed")
}

#[test]
fn parses_pointer_parameter_and_dereference_expression() {
    let program = parse_source("int first(char *buf) { return *buf; }");

    let function = &program.functions[0];
    assert_eq!(function.params[0].ty, Type::Pointer(Box::new(Type::Char)));

    let Statement::Return(expr) = &function.body[0] else {
        panic!("expected return statement");
    };

    let Expr::Unary {
        op, expr: operand, ..
    } = expr
    else {
        panic!("expected unary expression");
    };

    assert_eq!(*op, UnaryOp::Dereference);
    assert!(matches!(
        operand.as_ref(),
        Expr::Variable { name, .. } if name == "buf"
    ));
}

#[test]
fn parses_pointer_local_declarations() {
    let program =
        parse_source("int main() { char *buf; int **cursor; unsigned int *sums; return 0; }");

    let body = &program.functions[0].body;

    let Statement::VarDecl { ty: buf_ty, .. } = &body[0] else {
        panic!("expected first local declaration");
    };
    assert_eq!(*buf_ty, Type::Pointer(Box::new(Type::Char)));

    let Statement::VarDecl { ty: cursor_ty, .. } = &body[1] else {
        panic!("expected second local declaration");
    };
    assert_eq!(
        *cursor_ty,
        Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Int))))
    );

    let Statement::VarDecl { ty: sums_ty, .. } = &body[2] else {
        panic!("expected third local declaration");
    };
    assert_eq!(*sums_ty, Type::Pointer(Box::new(Type::UnsignedInt)));
}
