use rivet::ast::{
    BinaryOp, Expr, Initializer, IntLiteralBase, IntLiteralSuffix, Statement, Type, UnaryOp,
};
use rivet::lexer::lex;
use rivet::parser::parse;

fn parse_source(source: &str) -> rivet::ast::Program {
    let tokens = lex(source).expect("lexing should succeed");
    parse(tokens).expect("parsing should succeed")
}

fn parse_source_err(source: &str) -> rivet::parser::ParseError {
    let tokens = lex(source).expect("lexing should succeed");
    parse(tokens).expect_err("parsing should fail")
}

fn main_body(source: &str) -> Vec<Statement> {
    let program = parse_source(source);
    program.functions[0].body.clone()
}

fn only_statement(source: &str) -> Statement {
    let body = main_body(source);
    assert_eq!(body.len(), 1);
    body.into_iter().next().expect("expected statement")
}

fn only_return_expr(source: &str) -> Expr {
    let statement = only_statement(source);
    let Statement::Return(expr) = statement else {
        panic!("expected return statement");
    };
    expr
}

#[test]
fn parses_basic_main_function() {
    let program = parse_source("int main() { return 42; }");

    assert_eq!(program.functions.len(), 1);
    assert_eq!(program.functions[0].return_type, Type::Int);
    assert_eq!(program.functions[0].name, "main");
    assert!(matches!(
        program.functions[0].body[0],
        Statement::Return(Expr::IntLiteral { value: 42, .. })
    ));
}

#[test]
fn parses_integer_literal_suffixes() {
    let body = main_body("int main() { 1U; 2u; 3L; 4l; 5UL; 6ul; 7LU; 8lu; return 0; }");

    let expected = [
        (1, IntLiteralSuffix::Unsigned),
        (2, IntLiteralSuffix::Unsigned),
        (3, IntLiteralSuffix::Long),
        (4, IntLiteralSuffix::Long),
        (5, IntLiteralSuffix::UnsignedLong),
        (6, IntLiteralSuffix::UnsignedLong),
        (7, IntLiteralSuffix::UnsignedLong),
        (8, IntLiteralSuffix::UnsignedLong),
    ];

    for (statement, (expected_value, expected_suffix)) in body.iter().zip(expected) {
        let Statement::ExprStatement(Expr::IntLiteral { value, suffix, .. }) = statement else {
            panic!("expected integer literal expression statement");
        };

        assert_eq!(*value, expected_value);
        assert_eq!(*suffix, expected_suffix);
    }
}

#[test]
fn parses_large_integer_literal_magnitudes() {
    let statement = only_statement("int main() { return 4294967295U; }");

    let Statement::Return(Expr::IntLiteral { value, suffix, .. }) = statement else {
        panic!("expected integer literal return");
    };

    assert_eq!(value, 4_294_967_295);
    assert_eq!(suffix, IntLiteralSuffix::Unsigned);
}

#[test]
fn parses_hex_integer_literals() {
    let body = main_body("int main() { 0xff; 0XFFU; 0xffffUL; 0XffffLU; return 0; }");

    let expected = [
        (255, IntLiteralSuffix::None),
        (255, IntLiteralSuffix::Unsigned),
        (65_535, IntLiteralSuffix::UnsignedLong),
        (65_535, IntLiteralSuffix::UnsignedLong),
    ];

    for (statement, (expected_value, expected_suffix)) in body.iter().zip(expected) {
        let Statement::ExprStatement(Expr::IntLiteral {
            value,
            suffix,
            base,
            ..
        }) = statement
        else {
            panic!("expected integer literal expression statement");
        };

        assert_eq!(*value, expected_value);
        assert_eq!(*suffix, expected_suffix);
        assert_eq!(*base, IntLiteralBase::Hex);
    }
}

#[test]
fn parses_function_returning_binary_op() {
    let statement = only_statement("int main() { return 1 + 2; }");

    assert!(matches!(
        statement,
        Statement::Return(Expr::Binary {
            op: BinaryOp::Add,
            ..
        })
    ));
}

#[test]
fn parses_pointer_return_type() {
    let program = parse_source("int *id(int *p) { return p; }");
    let function = &program.functions[0];

    assert_eq!(function.return_type, Type::Pointer(Box::new(Type::Int)));
    assert_eq!(function.name, "id");
    assert_eq!(function.params[0].ty, Type::Pointer(Box::new(Type::Int)));
    assert_eq!(function.params[0].name, "p");
}

#[test]
fn parses_function_calls() {
    let statement = only_statement("int main() { return helper() + 2; }");

    let Statement::Return(Expr::Binary {
        op: BinaryOp::Add,
        left,
        right,
        ..
    }) = statement
    else {
        panic!("expected binary return");
    };

    assert!(matches!(
        left.as_ref(),
        Expr::Call { name, args, .. } if name == "helper" && args.is_empty()
    ));
    assert!(matches!(right.as_ref(), Expr::IntLiteral { value: 2, .. }));
}

#[test]
fn parses_expression_and_empty_statements() {
    let body = main_body("int main() { ; helper(); 1 + 2; }");

    assert!(matches!(body[0], Statement::Empty));
    assert!(matches!(
        &body[1],
        Statement::ExprStatement(Expr::Call { name, args, .. })
            if name == "helper" && args.is_empty()
    ));
    assert!(matches!(
        body[2],
        Statement::ExprStatement(Expr::Binary {
            op: BinaryOp::Add,
            ..
        })
    ));
}

#[test]
fn parses_char_literals_as_int_literals() {
    let body = main_body("int main() { char c = '\\n'; return 'A'; }");

    assert!(matches!(
        &body[0],
        Statement::VarDecl {
            ty: Type::Char,
            name,
            init: Some(Initializer::Expr(Expr::IntLiteral { value: 10, .. })),
            ..
        } if name == "c"
    ));
    assert!(matches!(
        body[1],
        Statement::Return(Expr::IntLiteral { value: 65, .. })
    ));
}

#[test]
fn parses_function_parameters_and_argument_lists() {
    let program = parse_source("int add(int x, char y) { return add(x, y); }");
    let function = &program.functions[0];

    assert_eq!(function.params[0].ty, Type::Int);
    assert_eq!(function.params[0].name, "x");
    assert_eq!(function.params[1].ty, Type::Char);
    assert_eq!(function.params[1].name, "y");

    let Statement::Return(Expr::Call { name, args, .. }) = &function.body[0] else {
        panic!("expected call return");
    };

    assert_eq!(name, "add");
    assert_eq!(args.len(), 2);
}

#[test]
fn rejects_trailing_commas_in_lists() {
    assert_eq!(
        parse_source_err("int main() { return add(1,); }").message,
        "trailing comma"
    );
    assert_eq!(
        parse_source_err("int add(int x,) { return x; }").message,
        "trailing comma"
    );
}

#[test]
fn parses_variable_declarations() {
    let body = main_body("int main() { int x = 1; char y; return x; }");

    assert!(matches!(
        &body[0],
        Statement::VarDecl {
            ty: Type::Int,
            name,
            init: Some(Initializer::Expr(Expr::IntLiteral { value: 1, .. })),
            ..
        } if name == "x"
    ));
    assert!(matches!(
        &body[1],
        Statement::VarDecl {
            ty: Type::Char,
            name,
            init: None,
            ..
        } if name == "y"
    ));
}

#[test]
fn parses_prefix_postfix_and_unary_expressions() {
    assert!(matches!(
        only_return_expr("int main() { return ++x; }"),
        Expr::PrefixInc { .. }
    ));
    assert!(matches!(
        only_return_expr("int main() { return x--; }"),
        Expr::PostfixDec { .. }
    ));
    assert!(matches!(
        only_return_expr("int main() { return !~ -x; }"),
        Expr::Unary {
            op: UnaryOp::LogicalNot,
            ..
        }
    ));
}

#[test]
fn parses_expression_precedence() {
    let expr = only_return_expr("int main() { return 1 + 2 * 3; }");
    let Expr::Binary {
        op: BinaryOp::Add,
        right,
        ..
    } = expr
    else {
        panic!("expected addition at expression root");
    };
    assert!(matches!(
        right.as_ref(),
        Expr::Binary {
            op: BinaryOp::Multiply,
            ..
        }
    ));

    let expr = only_return_expr("int main() { return (1 + 2) * 3; }");
    let Expr::Binary {
        op: BinaryOp::Multiply,
        left,
        ..
    } = expr
    else {
        panic!("expected multiplication at expression root");
    };
    assert!(matches!(
        left.as_ref(),
        Expr::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));

    let expr = only_return_expr("int main() { return 1 | 2 && 3 || 4; }");
    assert!(matches!(
        expr,
        Expr::Binary {
            op: BinaryOp::LogicalOr,
            ..
        }
    ));
}

#[test]
fn parses_left_associative_arithmetic() {
    let expr = only_return_expr("int main() { return 5 - 2 - 1; }");
    let Expr::Binary {
        op: BinaryOp::Subtract,
        left,
        ..
    } = expr
    else {
        panic!("expected subtraction at expression root");
    };
    assert!(matches!(
        left.as_ref(),
        Expr::Binary {
            op: BinaryOp::Subtract,
            ..
        }
    ));

    let expr = only_return_expr("int main() { return 1 + 2 + 3; }");
    let Expr::Binary {
        op: BinaryOp::Add,
        left,
        ..
    } = expr
    else {
        panic!("expected addition at expression root");
    };
    assert!(matches!(
        left.as_ref(),
        Expr::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));
}

#[test]
fn parses_assignment_expressions() {
    let body = main_body("int main() { x = 1; x += 2; return x; }");

    assert!(matches!(
        body[0],
        Statement::ExprStatement(Expr::Assign { .. })
    ));
    assert!(matches!(
        body[1],
        Statement::ExprStatement(Expr::CompoundAssign {
            op: BinaryOp::Add,
            ..
        })
    ));
}

#[test]
fn parses_loop_and_jump_statements() {
    let body = main_body(
        "int main() { while (x) break; do continue; while (x); for (i = 0; i < 3; i++) { x += i; } return 0; }",
    );

    assert!(matches!(body[0], Statement::While { .. }));
    assert!(matches!(body[1], Statement::DoWhile { .. }));
    assert!(matches!(body[2], Statement::For { .. }));
    assert!(matches!(body[3], Statement::Return(_)));
}

#[test]
fn rejects_malformed_control_flow_statements() {
    assert!(
        parse_source_err("int main() { if x return 1; }")
            .message
            .contains("expected LParen")
    );
    assert!(
        parse_source_err("int main() { while x return 1; }")
            .message
            .contains("expected LParen")
    );
    assert!(
        parse_source_err("int main() { do { } while (0) }")
            .message
            .contains("expected Semicolon")
    );
    assert!(
        parse_source_err("int main() { break }")
            .message
            .contains("expected Semicolon")
    );
    assert!(
        parse_source_err("int main() { continue }")
            .message
            .contains("expected Semicolon")
    );
}

#[test]
fn parses_block_statements() {
    let body = main_body("int main() { { int x = 1; { x += 2; } } return x; }");

    assert!(matches!(body[0], Statement::Block(_)));
    assert!(matches!(body[1], Statement::Return(_)));
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
fn parses_dereference_assignment_expression_statement() {
    let program = parse_source("int write_first(char *p) { *p = 42; return *p; }");
    let body = &program.functions[0].body;

    let Statement::ExprStatement(Expr::Assign { target, value, .. }) = &body[0] else {
        panic!("expected dereference assignment expression statement");
    };

    assert!(matches!(
        target.as_ref(),
        Expr::Unary {
            op: UnaryOp::Dereference,
            ..
        }
    ));
    assert!(matches!(value.as_ref(), Expr::IntLiteral { value: 42, .. }));
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

#[test]
fn parses_signed_type_specifiers() {
    let program = parse_source(
        "int main() { signed a; signed int b; signed long c; signed long int d; return 0; }",
    );

    let body = &program.functions[0].body;

    let Statement::VarDecl { ty: a_ty, .. } = &body[0] else {
        panic!("expected first local declaration");
    };
    assert_eq!(*a_ty, Type::Int);

    let Statement::VarDecl { ty: b_ty, .. } = &body[1] else {
        panic!("expected second local declaration");
    };
    assert_eq!(*b_ty, Type::Int);

    let Statement::VarDecl { ty: c_ty, .. } = &body[2] else {
        panic!("expected third local declaration");
    };
    assert_eq!(*c_ty, Type::Long);

    let Statement::VarDecl { ty: d_ty, .. } = &body[3] else {
        panic!("expected fourth local declaration");
    };
    assert_eq!(*d_ty, Type::Long);
}

#[test]
fn parses_array_local_declarations() {
    let program = parse_source("int main() { char buf[3]; int nums[10]; return 0; }");

    let body = &program.functions[0].body;

    let Statement::VarDecl { ty: buf_ty, .. } = &body[0] else {
        panic!("expected first local declaration");
    };
    assert_eq!(
        *buf_ty,
        Type::Array {
            element: Box::new(Type::Char),
            len: 3,
        }
    );

    let Statement::VarDecl { ty: nums_ty, .. } = &body[1] else {
        panic!("expected second local declaration");
    };
    assert_eq!(
        *nums_ty,
        Type::Array {
            element: Box::new(Type::Int),
            len: 10,
        }
    );
}

#[test]
fn parses_array_initializer_list() {
    let program = parse_source("int main() { char buf[3] = {1, 2, 3}; return 0; }");

    let Statement::VarDecl { init, .. } = &program.functions[0].body[0] else {
        panic!("expected local declaration");
    };

    let Some(Initializer::List(values)) = init else {
        panic!("expected initializer list");
    };

    assert_eq!(values.len(), 3);
    assert!(matches!(values[0], Expr::IntLiteral { value: 1, .. }));
    assert!(matches!(values[1], Expr::IntLiteral { value: 2, .. }));
    assert!(matches!(values[2], Expr::IntLiteral { value: 3, .. }));
}

#[test]
fn parses_empty_array_initializer_list() {
    let program = parse_source("int main() { int nums[2] = {}; return 0; }");

    let Statement::VarDecl { init, .. } = &program.functions[0].body[0] else {
        panic!("expected local declaration");
    };

    let Some(Initializer::List(values)) = init else {
        panic!("expected initializer list");
    };

    assert!(values.is_empty());
}

#[test]
fn parses_array_of_pointers_declaration() {
    let program = parse_source("int main() { char *bufs[4]; return 0; }");

    let Statement::VarDecl { ty, name, .. } = &program.functions[0].body[0] else {
        panic!("expected local declaration");
    };

    assert_eq!(name, "bufs");
    assert_eq!(
        *ty,
        Type::Array {
            element: Box::new(Type::Pointer(Box::new(Type::Char))),
            len: 4,
        }
    );
}

#[test]
fn rejects_zero_length_array_declaration() {
    let err = parse_source_err("int main() { char buf[0]; return 0; }");

    assert_eq!(err.message, "array size must be greater than 0, got '0'");
}
