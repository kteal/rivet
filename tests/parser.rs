mod common;

use common::{first_function, functions};
use rivet::ast::{
    BinaryOp, Expr, ExternalDecl, FunctionType, Initializer, IntLiteralBase, IntLiteralSuffix,
    LocalDecl, MemberAccessKind, Statement, Type, UnaryOp,
};
use rivet::lexer::lex;
use rivet::parser::parse;
use rivet::preprocess::preprocess;
use rivet::source::DUMMY_FILE_ID;

fn parse_source(source: &str) -> rivet::ast::Program {
    let tokens = lex(source, DUMMY_FILE_ID).expect("lexing should succeed");
    let tokens = preprocess(tokens).expect("preprocessing should succeed");
    parse(tokens).expect("parsing should succeed")
}

fn parse_source_err(source: &str) -> rivet::parser::ParseError {
    let tokens = lex(source, DUMMY_FILE_ID).expect("lexing should succeed");
    let tokens = preprocess(tokens).expect("preprocessing should succeed");
    parse(tokens).expect_err("parsing should fail")
}

fn main_body(source: &str) -> Vec<Statement> {
    let program = parse_source(source);
    first_function(&program).body.clone()
}

fn only_statement(source: &str) -> Statement {
    let body = main_body(source);
    assert_eq!(body.len(), 1);
    body.into_iter().next().expect("expected statement")
}

fn only_return_expr(source: &str) -> Expr {
    let statement = only_statement(source);
    let Statement::Return {
        expr: Some(expr), ..
    } = statement
    else {
        panic!("expected return statement");
    };
    expr
}

fn return_expr(statement: &Statement) -> &Expr {
    let Statement::Return {
        expr: Some(expr), ..
    } = statement
    else {
        panic!("expected return statement with expression");
    };
    expr
}

fn single_decl(statement: &Statement) -> &LocalDecl {
    let Statement::Decl(decls) = statement else {
        panic!("expected declaration statement");
    };
    assert_eq!(decls.len(), 1);
    &decls[0]
}

#[test]
fn parse_errors_use_unexpected_token_span() {
    let source = "int main() { return ); }";
    let err = parse_source_err(source);
    let right_paren = source
        .rfind(')')
        .expect("source should contain right paren");

    assert_eq!(err.message, "expected expression, found RParen");
    assert_eq!(err.span.file_id, DUMMY_FILE_ID);
    assert_eq!(err.span.start, right_paren);
    assert_eq!(err.span.end, right_paren + 1);
}

#[test]
fn missing_semicolon_errors_point_at_following_token() {
    let source = "int main() { x return 0; }";
    let err = parse_source_err(source);
    let return_start = source.find("return").expect("source should contain return");

    assert!(err.message.contains("expected Semicolon"));
    assert_eq!(err.span.file_id, DUMMY_FILE_ID);
    assert_eq!(err.span.start, return_start);
    assert_eq!(err.span.end, return_start + "return".len());
}

#[test]
fn trailing_comma_errors_point_at_right_paren() {
    let source = "int main() { return add(1,); }";
    let err = parse_source_err(source);
    let right_paren = source.find(");").expect("source should contain );");

    assert_eq!(err.message, "trailing comma");
    assert_eq!(err.span.file_id, DUMMY_FILE_ID);
    assert_eq!(err.span.start, right_paren);
    assert_eq!(err.span.end, right_paren + 1);
}

#[test]
fn binary_expression_preserves_operator_span() {
    let source = "int main() { return x + y; }";
    let expr = only_return_expr(source);
    let plus = source.find('+').expect("source should contain plus");

    let Expr::Binary { op, op_span, .. } = expr else {
        panic!("expected binary expression");
    };

    assert_eq!(op, BinaryOp::Add);
    assert_eq!(op_span.file_id, DUMMY_FILE_ID);
    assert_eq!(op_span.start, plus);
    assert_eq!(op_span.end, plus + 1);
}

#[test]
fn parameter_name_spans_are_preserved() {
    let source = "int add(int x, char y) { return x; }";
    let program = parse_source(source);
    let params = &first_function(&program).params;
    let x_start = source.find('x').expect("source should contain x");
    let y_start = source.find('y').expect("source should contain y");

    assert_eq!(params[0].name, "x");
    assert_eq!(params[0].name_span.file_id, DUMMY_FILE_ID);
    assert_eq!(params[0].name_span.start, x_start);
    assert_eq!(params[0].name_span.end, x_start + 1);

    assert_eq!(params[1].name, "y");
    assert_eq!(params[1].name_span.file_id, DUMMY_FILE_ID);
    assert_eq!(params[1].name_span.start, y_start);
    assert_eq!(params[1].name_span.end, y_start + 1);
}

#[test]
fn parses_basic_main_function() {
    let program = parse_source("int main() { return 42; }");

    assert_eq!(functions(&program).len(), 1);
    assert_eq!(first_function(&program).return_type, Type::Int);
    assert_eq!(first_function(&program).name, "main");
    assert!(matches!(
        first_function(&program).body[0],
        Statement::Return {
            expr: Some(Expr::IntLiteral { value: 42, .. }),
            ..
        }
    ));
}

#[test]
fn parses_function_prototype_with_unnamed_parameter() {
    let program = parse_source("int helper(int); int main() { return helper(3); }");

    assert_eq!(program.declarations.len(), 2);
    let ExternalDecl::FunctionDecl(function) = &program.declarations[0] else {
        panic!("expected function declaration");
    };

    assert_eq!(function.return_type, Type::Int);
    assert_eq!(function.name, "helper");
    assert_eq!(function.params.len(), 1);
    assert_eq!(function.params[0].ty, Type::Int);
    assert_eq!(function.params[0].name, None);
    assert_eq!(function.params[0].name_span, None);
}

#[test]
fn parses_function_prototype_with_unnamed_pointer_parameter() {
    let program = parse_source("int helper(int *, char **); int main() { return 0; }");

    assert_eq!(program.declarations.len(), 2);
    let ExternalDecl::FunctionDecl(function) = &program.declarations[0] else {
        panic!("expected function declaration");
    };

    assert_eq!(function.name, "helper");
    assert_eq!(function.params.len(), 2);
    assert_eq!(function.params[0].ty, Type::Pointer(Box::new(Type::Int)));
    assert_eq!(function.params[0].name, None);
    assert_eq!(function.params[0].name_span, None);
    assert_eq!(
        function.params[1].ty,
        Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Char))))
    );
    assert_eq!(function.params[1].name, None);
    assert_eq!(function.params[1].name_span, None);
}

#[test]
fn parses_void_parameter_list_as_no_parameters() {
    let program = parse_source("int helper(void); int main(void) { return helper(); }");

    let ExternalDecl::FunctionDecl(function_decl) = &program.declarations[0] else {
        panic!("expected function declaration");
    };
    assert_eq!(function_decl.name, "helper");
    assert!(function_decl.params.is_empty());

    let function_def = first_function(&program);
    assert_eq!(function_def.name, "main");
    assert!(function_def.params.is_empty());
}

#[test]
fn parses_void_pointer_parameter_as_object_parameter() {
    let program = parse_source("int has_pointer(void *p) { return p != 0; }");
    let function = first_function(&program);

    assert_eq!(function.params.len(), 1);
    assert_eq!(function.params[0].ty, Type::Pointer(Box::new(Type::Void)));
    assert_eq!(function.params[0].name, "p");
}

#[test]
fn rejects_plain_void_parameter_forms() {
    assert_eq!(
        parse_source_err("int f(void x) { return 0; }").message,
        "cannot have 'void' parameter type"
    );
    assert_eq!(
        parse_source_err("int f(void, int x) { return 0; }").message,
        "cannot have 'void' parameter type"
    );
    assert_eq!(
        parse_source_err("int f(int x, void) { return 0; }").message,
        "cannot have 'void' parameter type"
    );
}

#[test]
fn rejects_function_definition_with_unnamed_parameter() {
    let err = parse_source_err("int helper(int) { return 1; }");

    assert_eq!(
        err.message,
        "expected parameter name in function definition"
    );
}

#[test]
fn parses_global_declaration_without_initializer() {
    let program = parse_source("int g; int main() { return 0; }");

    assert_eq!(program.declarations.len(), 2);
    let ExternalDecl::Global(global) = &program.declarations[0] else {
        panic!("expected global declaration");
    };

    assert_eq!(global.ty, Type::Int);
    assert_eq!(global.name, "g");
    assert_eq!(global.init, None);
}

#[test]
fn parses_global_declaration_with_scalar_initializer() {
    let program = parse_source("int g = 3; int main() { return 0; }");

    assert_eq!(program.declarations.len(), 2);
    let ExternalDecl::Global(global) = &program.declarations[0] else {
        panic!("expected global declaration");
    };

    assert_eq!(global.ty, Type::Int);
    assert_eq!(global.name, "g");
    assert!(matches!(
        global.init,
        Some(Initializer::Expr(Expr::IntLiteral { value: 3, .. }))
    ));
}

#[test]
fn parses_pointer_global_declaration() {
    let program = parse_source("int *p; int main() { return 0; }");

    assert_eq!(program.declarations.len(), 2);
    let ExternalDecl::Global(global) = &program.declarations[0] else {
        panic!("expected global declaration");
    };

    assert_eq!(global.ty, Type::Pointer(Box::new(Type::Int)));
    assert_eq!(global.name, "p");
    assert_eq!(global.init, None);
}

#[test]
fn parses_array_global_declaration_with_initializer_list() {
    let program = parse_source("int arr[3] = {1, 2, 3}; int main() { return 0; }");

    assert_eq!(program.declarations.len(), 2);
    let ExternalDecl::Global(global) = &program.declarations[0] else {
        panic!("expected global declaration");
    };

    assert_eq!(
        global.ty,
        Type::Array {
            element: Box::new(Type::Int),
            len: 3,
        }
    );
    assert_eq!(global.name, "arr");
    let Some(Initializer::List(elements)) = &global.init else {
        panic!("expected initializer list");
    };
    assert_eq!(elements.len(), 3);
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

    let Statement::Return {
        expr: Some(Expr::IntLiteral { value, suffix, .. }),
        ..
    } = statement
    else {
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
        Statement::Return {
            expr: Some(Expr::Binary {
                op: BinaryOp::Add,
                ..
            }),
            ..
        }
    ));
}

#[test]
fn parses_pointer_return_type() {
    let program = parse_source("int *id(int *p) { return p; }");
    let function = &first_function(&program);

    assert_eq!(function.return_type, Type::Pointer(Box::new(Type::Int)));
    assert_eq!(function.name, "id");
    assert_eq!(function.params[0].ty, Type::Pointer(Box::new(Type::Int)));
    assert_eq!(function.params[0].name, "p");
}

#[test]
fn parses_function_calls() {
    let statement = only_statement("int main() { return helper() + 2; }");

    let Statement::Return {
        expr:
            Some(Expr::Binary {
                op: BinaryOp::Add,
                left,
                right,
                ..
            }),
        ..
    } = statement
    else {
        panic!("expected binary return");
    };

    assert!(matches!(
        left.as_ref(),
        Expr::Call { callee, args, .. }
            if matches!(callee.as_ref(), Expr::Variable { name, .. } if name == "helper")
                && args.is_empty()
    ));
    assert!(matches!(right.as_ref(), Expr::IntLiteral { value: 2, .. }));
}

#[test]
fn parses_function_pointer_calls_as_postfix_calls() {
    let expr = only_return_expr("int main() { return fp(3); }");

    let Expr::Call { callee, args, .. } = expr else {
        panic!("expected call expression");
    };

    assert!(matches!(
        callee.as_ref(),
        Expr::Variable { name, .. } if name == "fp"
    ));
    assert_eq!(args.len(), 1);
    assert!(matches!(args[0], Expr::IntLiteral { value: 3, .. }));
}

#[test]
fn parses_explicitly_dereferenced_function_pointer_calls() {
    let expr = only_return_expr("int main() { return (*fp)(3); }");

    let Expr::Call { callee, args, .. } = expr else {
        panic!("expected call expression");
    };

    assert!(matches!(
        callee.as_ref(),
        Expr::Unary {
            op: UnaryOp::Dereference,
            expr,
            ..
        } if matches!(expr.as_ref(), Expr::Variable { name, .. } if name == "fp")
    ));
    assert_eq!(args.len(), 1);
    assert!(matches!(args[0], Expr::IntLiteral { value: 3, .. }));
}

#[test]
fn parses_expression_and_empty_statements() {
    let body = main_body("int main() { ; helper(); 1 + 2; }");

    assert!(matches!(body[0], Statement::Empty));
    assert!(matches!(
        &body[1],
        Statement::ExprStatement(Expr::Call { callee, args, .. })
            if matches!(callee.as_ref(), Expr::Variable { name, .. } if name == "helper")
                && args.is_empty()
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
    let decl = single_decl(&body[0]);

    assert!(matches!(
        decl,
        LocalDecl {
            ty: Type::Char,
            name,
            init: Some(Initializer::Expr(Expr::IntLiteral { value: 10, .. })),
            ..
        } if name == "c"
    ));
    assert!(matches!(
        body[1],
        Statement::Return {
            expr: Some(Expr::IntLiteral { value: 65, .. }),
            ..
        }
    ));
}

#[test]
fn parses_function_parameters_and_argument_lists() {
    let program = parse_source("int add(int x, char y) { return add(x, y); }");
    let function = &first_function(&program);

    assert_eq!(function.params[0].ty, Type::Int);
    assert_eq!(function.params[0].name, "x");
    assert_eq!(function.params[1].ty, Type::Char);
    assert_eq!(function.params[1].name, "y");

    let Statement::Return {
        expr: Some(Expr::Call { callee, args, .. }),
        ..
    } = &function.body[0]
    else {
        panic!("expected call return");
    };

    assert!(matches!(
        callee.as_ref(),
        Expr::Variable { name, .. } if name == "add"
    ));
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
    let first_decl = single_decl(&body[0]);
    let second_decl = single_decl(&body[1]);

    assert!(matches!(
        first_decl,
        LocalDecl {
            ty: Type::Int,
            name,
            init: Some(Initializer::Expr(Expr::IntLiteral { value: 1, .. })),
            ..
        } if name == "x"
    ));
    assert!(matches!(
        second_decl,
        LocalDecl {
            ty: Type::Char,
            name,
            init: None,
            ..
        } if name == "y"
    ));
}

#[test]
fn parses_multiple_local_declarators() {
    let body = main_body("int main() { int a, *p, arr[4]; return 0; }");

    let Statement::Decl(decls) = &body[0] else {
        panic!("expected declaration statement");
    };
    assert_eq!(decls.len(), 3);

    assert_eq!(decls[0].name, "a");
    assert_eq!(decls[0].ty, Type::Int);
    assert_eq!(decls[0].init, None);

    assert_eq!(decls[1].name, "p");
    assert_eq!(decls[1].ty, Type::Pointer(Box::new(Type::Int)));
    assert_eq!(decls[1].init, None);

    assert_eq!(decls[2].name, "arr");
    assert_eq!(
        decls[2].ty,
        Type::Array {
            element: Box::new(Type::Int),
            len: 4,
        }
    );
    assert_eq!(decls[2].init, None);
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
    assert!(matches!(
        only_return_expr("int main() { return &x; }"),
        Expr::Unary {
            op: UnaryOp::AddressOf,
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
fn parses_assignment_to_non_variable_expression() {
    let expr = only_return_expr("int main() { return (1 + 2) = 3; }");

    let Expr::Assign { target, value, .. } = expr else {
        panic!("expected assignment expression");
    };

    assert!(matches!(
        target.as_ref(),
        Expr::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));
    assert!(matches!(value.as_ref(), Expr::IntLiteral { value: 3, .. }));
}

#[test]
fn parses_compound_assignment_to_non_variable_expression() {
    let expr = only_return_expr("int main() { return (1 + 2) += 3; }");

    let Expr::CompoundAssign {
        target, op, value, ..
    } = expr
    else {
        panic!("expected compound assignment expression");
    };

    assert_eq!(op, BinaryOp::Add);
    assert!(matches!(
        target.as_ref(),
        Expr::Binary {
            op: BinaryOp::Add,
            ..
        }
    ));
    assert!(matches!(value.as_ref(), Expr::IntLiteral { value: 3, .. }));
}

#[test]
fn parses_loop_and_jump_statements() {
    let body = main_body(
        "int main() { while (x) break; do continue; while (x); for (i = 0; i < 3; i++) { x += i; } return 0; }",
    );

    assert!(matches!(body[0], Statement::While { .. }));
    assert!(matches!(body[1], Statement::DoWhile { .. }));
    assert!(matches!(body[2], Statement::For { .. }));
    assert!(matches!(body[3], Statement::Return { .. }));
}

#[test]
fn parses_bare_return_statement() {
    let statement = only_statement("void f() { return; }");

    assert!(matches!(statement, Statement::Return { expr: None, .. }));
}

#[test]
fn parses_for_loop_clause_shapes() {
    let body = main_body(
        "int main() { for (;;) ; for (i = 0; i < 10; i = i + 1) ; for (int i = 0; i < 10; i = i + 1) ; for (; i < 10;) ; return 0; }",
    );

    let Statement::For {
        init,
        cond,
        post,
        body: for_body,
    } = &body[0]
    else {
        panic!("expected for statement");
    };
    assert!(init.is_none());
    assert!(cond.is_none());
    assert!(post.is_none());
    assert!(matches!(for_body.as_ref(), Statement::Empty));

    let Statement::For {
        init, cond, post, ..
    } = &body[1]
    else {
        panic!("expected for statement");
    };
    assert!(matches!(
        init.as_deref(),
        Some(Statement::ExprStatement(Expr::Assign { .. }))
    ));
    assert!(matches!(
        cond,
        Some(Expr::Binary {
            op: BinaryOp::Less,
            ..
        })
    ));
    assert!(matches!(post, Some(Expr::Assign { .. })));

    let Statement::For {
        init, cond, post, ..
    } = &body[2]
    else {
        panic!("expected for statement");
    };
    assert!(matches!(init.as_deref(), Some(Statement::Decl(_))));
    assert!(matches!(
        cond,
        Some(Expr::Binary {
            op: BinaryOp::Less,
            ..
        })
    ));
    assert!(matches!(post, Some(Expr::Assign { .. })));

    let Statement::For {
        init, cond, post, ..
    } = &body[3]
    else {
        panic!("expected for statement");
    };
    assert!(init.is_none());
    assert!(matches!(
        cond,
        Some(Expr::Binary {
            op: BinaryOp::Less,
            ..
        })
    ));
    assert!(post.is_none());
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
    assert!(matches!(body[1], Statement::Return { .. }));
}

#[test]
fn parses_declarations_mixed_with_statements() {
    let body = main_body("int main() { int x = 1; x = x + 1; int y = x + 2; return y; }");

    assert_eq!(body.len(), 4);
    assert!(matches!(body[0], Statement::Decl(_)));
    assert!(matches!(body[1], Statement::ExprStatement(_)));
    assert!(matches!(body[2], Statement::Decl(_)));
    assert!(matches!(body[3], Statement::Return { .. }));

    let y_decl = single_decl(&body[2]);
    assert_eq!(y_decl.name, "y");
    assert!(matches!(
        y_decl.init,
        Some(Initializer::Expr(Expr::Binary {
            op: BinaryOp::Add,
            ..
        }))
    ));
}

#[test]
fn parses_pointer_parameter_and_dereference_expression() {
    let program = parse_source("int first(char *buf) { return *buf; }");

    let function = &first_function(&program);
    assert_eq!(function.params[0].ty, Type::Pointer(Box::new(Type::Char)));

    let expr = return_expr(&function.body[0]);

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
    let body = &first_function(&program).body;

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

    let body = &first_function(&program).body;

    assert_eq!(
        single_decl(&body[0]).ty,
        Type::Pointer(Box::new(Type::Char))
    );

    assert_eq!(
        single_decl(&body[1]).ty,
        Type::Pointer(Box::new(Type::Pointer(Box::new(Type::Int))))
    );

    assert_eq!(
        single_decl(&body[2]).ty,
        Type::Pointer(Box::new(Type::UnsignedInt))
    );
}

#[test]
fn parses_signed_type_specifiers() {
    let program = parse_source(
        "int main() { signed a; signed int b; signed long c; signed long int d; return 0; }",
    );

    let body = &first_function(&program).body;

    assert_eq!(single_decl(&body[0]).ty, Type::Int);

    assert_eq!(single_decl(&body[1]).ty, Type::Int);

    assert_eq!(single_decl(&body[2]).ty, Type::Long);

    assert_eq!(single_decl(&body[3]).ty, Type::Long);
}

#[test]
fn parses_array_local_declarations() {
    let program =
        parse_source("int main() { char buf[3]; int nums[10]; char text[] = \"abc\"; return 0; }");

    let body = &first_function(&program).body;

    assert_eq!(
        single_decl(&body[0]).ty,
        Type::Array {
            element: Box::new(Type::Char),
            len: 3,
        }
    );

    assert_eq!(
        single_decl(&body[1]).ty,
        Type::Array {
            element: Box::new(Type::Int),
            len: 10,
        }
    );

    assert_eq!(
        single_decl(&body[2]).ty,
        Type::IncompleteArray {
            element: Box::new(Type::Char),
        }
    );
}

#[test]
fn parses_transparent_parenthesized_declarator() {
    let program = parse_source("int main() { int (x); int ((*p)); return 0; }");

    let body = &first_function(&program).body;

    assert_eq!(single_decl(&body[0]).name, "x");
    assert_eq!(single_decl(&body[0]).ty, Type::Int);

    assert_eq!(single_decl(&body[1]).name, "p");
    assert_eq!(single_decl(&body[1]).ty, Type::Pointer(Box::new(Type::Int)));
}

#[test]
fn parses_parenthesized_pointer_to_array_declaration() {
    let program = parse_source("int main() { int (*p)[3]; return 0; }");
    let decl = single_decl(&first_function(&program).body[0]);

    assert_eq!(decl.name, "p");
    assert_eq!(
        decl.ty,
        Type::Pointer(Box::new(Type::Array {
            element: Box::new(Type::Int),
            len: 3,
        }))
    );
}

#[test]
fn parenthesized_pointer_to_array_differs_from_array_of_pointers() {
    let program = parse_source("int main() { int (*p)[3]; int *q[3]; return 0; }");
    let body = &first_function(&program).body;

    assert_eq!(
        single_decl(&body[0]).ty,
        Type::Pointer(Box::new(Type::Array {
            element: Box::new(Type::Int),
            len: 3,
        }))
    );
    assert_eq!(
        single_decl(&body[1]).ty,
        Type::Array {
            element: Box::new(Type::Pointer(Box::new(Type::Int))),
            len: 3,
        }
    );
}

#[test]
fn parses_parenthesized_function_pointer_declaration() {
    let program = parse_source("int main() { int (*fp)(int, char *); return 0; }");
    let decl = single_decl(&first_function(&program).body[0]);

    assert_eq!(decl.name, "fp");
    assert_eq!(
        decl.ty,
        Type::Pointer(Box::new(Type::Function(Box::new(FunctionType {
            return_type: Box::new(Type::Int),
            params: vec![Type::Int, Type::Pointer(Box::new(Type::Char))],
        }))))
    );
}

#[test]
fn parses_array_initializer_list() {
    let program = parse_source("int main() { char buf[3] = {1, 2, 3}; return 0; }");

    let init = &single_decl(&first_function(&program).body[0]).init;

    let Some(Initializer::List(values)) = init else {
        panic!("expected initializer list");
    };

    assert_eq!(values.len(), 3);
    assert!(matches!(values[0], Expr::IntLiteral { value: 1, .. }));
    assert!(matches!(values[1], Expr::IntLiteral { value: 2, .. }));
    assert!(matches!(values[2], Expr::IntLiteral { value: 3, .. }));
}

#[test]
fn parses_array_initializer_list_with_trailing_comma() {
    let program = parse_source("int main() { char buf[3] = {1, 2, 3,}; return 0; }");

    let init = &single_decl(&first_function(&program).body[0]).init;

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

    let init = &single_decl(&first_function(&program).body[0]).init;

    let Some(Initializer::List(values)) = init else {
        panic!("expected initializer list");
    };

    assert!(values.is_empty());
}

#[test]
fn parses_array_of_pointers_declaration() {
    let program = parse_source("int main() { char *bufs[4]; return 0; }");

    let decl = single_decl(&first_function(&program).body[0]);

    assert_eq!(decl.name, "bufs");
    assert_eq!(
        decl.ty,
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

#[test]
fn parses_const_arg() {
    let program = parse_source("unsigned long f(const unsigned char *buf) { return buf[0];}");

    let function = &first_function(&program);
    assert_eq!(function.return_type, Type::UnsignedLong);
    assert_eq!(function.name, "f");
    assert_eq!(function.params.len(), 1);
    assert_eq!(
        function.params[0].ty,
        Type::Pointer(Box::new(Type::UnsignedChar))
    );
    assert_eq!(function.params[0].name, "buf");
}

#[test]
fn parses_const_decl() {
    let program =
        parse_source("unsigned long f(unsigned char *buf) { const int x = 4; return x; }");

    let function = &first_function(&program);
    assert_eq!(function.return_type, Type::UnsignedLong);
    assert_eq!(function.name, "f");
    assert_eq!(function.params.len(), 1);
    assert_eq!(
        function.params[0].ty,
        Type::Pointer(Box::new(Type::UnsignedChar))
    );
    assert_eq!(function.params[0].name, "buf");
}

#[test]
fn parses_scalar_cast_expressions() {
    let expr = only_return_expr("int main() { return (unsigned)1; }");

    let Expr::Cast { ty, expr, .. } = expr else {
        panic!("expected cast expression");
    };

    assert_eq!(ty, Type::UnsignedInt);
    assert!(matches!(expr.as_ref(), Expr::IntLiteral { value: 1, .. }));
}

#[test]
fn parses_const_qualified_cast_expressions() {
    let expr = only_return_expr("int main() { return (const unsigned long)1; }");

    let Expr::Cast { ty, expr, .. } = expr else {
        panic!("expected cast expression");
    };

    assert_eq!(ty, Type::UnsignedLong);
    assert!(matches!(expr.as_ref(), Expr::IntLiteral { value: 1, .. }));
}

#[test]
fn cast_has_unary_precedence() {
    let expr = only_return_expr("int main() { return (unsigned)-1 + 2; }");

    let Expr::Binary {
        op: BinaryOp::Add,
        left,
        right,
        ..
    } = expr
    else {
        panic!("expected addition expression");
    };

    assert!(matches!(
        left.as_ref(),
        Expr::Cast {
            ty: Type::UnsignedInt,
            expr,
            ..
        } if matches!(expr.as_ref(), Expr::Unary { op: UnaryOp::Negate, .. })
    ));
    assert!(matches!(right.as_ref(), Expr::IntLiteral { value: 2, .. }));
}

#[test]
fn parses_sizeof_type_and_expression_forms() {
    let type_expr = only_return_expr("int main() { return sizeof(int); }");
    let Expr::SizeOfType { ty, .. } = type_expr else {
        panic!("expected sizeof type expression");
    };
    assert_eq!(ty, Type::Int);

    let pointer_type_expr = only_return_expr("int main() { return sizeof(char *); }");
    let Expr::SizeOfType { ty, .. } = pointer_type_expr else {
        panic!("expected sizeof pointer type expression");
    };
    assert_eq!(ty, Type::Pointer(Box::new(Type::Char)));

    let body = main_body("int main() { int x; return sizeof x; }");
    let value_expr = return_expr(&body[1]);
    let Expr::SizeOfExpr { expr, .. } = value_expr else {
        panic!("expected sizeof expression");
    };
    assert!(matches!(expr.as_ref(), Expr::Variable { name, .. } if name == "x"));
}

#[test]
fn parses_string_literal_expressions() {
    let expr = only_return_expr("int main() { return \"abc\"; }");
    let Expr::StringLiteral { bytes, .. } = expr else {
        panic!("expected string literal expression");
    };

    assert_eq!(bytes, b"abc".to_vec());
}

#[test]
fn parses_adjacent_string_literals_as_one_expression() {
    let expr = only_return_expr("int main() { return \"foo\" \"bar\"; }");
    let Expr::StringLiteral { bytes, .. } = expr else {
        panic!("expected string literal expression");
    };

    assert_eq!(bytes, b"foobar".to_vec());
}

#[test]
fn parses_string_literal_postfix_indexing() {
    let expr = only_return_expr("int main() { return \"abc\"[1]; }");
    let Expr::Index { base, index, .. } = expr else {
        panic!("expected index expression");
    };

    assert!(matches!(
        base.as_ref(),
        Expr::StringLiteral { bytes, .. } if bytes == b"abc"
    ));
    assert!(matches!(index.as_ref(), Expr::IntLiteral { value: 1, .. }));
}

#[test]
fn parses_sizeof_string_literal_as_expression() {
    let expr = only_return_expr("int main() { return sizeof(\"abc\"); }");
    let Expr::SizeOfExpr { expr, .. } = expr else {
        panic!("expected sizeof expression");
    };

    assert!(matches!(
        expr.as_ref(),
        Expr::StringLiteral { bytes, .. } if bytes == b"abc"
    ));
}

#[test]
fn sizeof_parenthesized_identifier_uses_typedef_lookup() {
    let type_expr = only_return_expr("typedef int T; int main() { return sizeof(T); }");
    assert!(matches!(type_expr, Expr::SizeOfType { ty: Type::Int, .. }));

    let body = main_body("typedef int T; int main() { int T; return sizeof(T); }");
    let value_expr = return_expr(&body[1]);
    let Expr::SizeOfExpr { expr, .. } = value_expr else {
        panic!("expected sizeof expression after object shadows typedef");
    };
    assert!(matches!(expr.as_ref(), Expr::Variable { name, .. } if name == "T"));
}

#[test]
fn sizeof_has_unary_precedence() {
    let body = main_body("int main() { int x; return sizeof x + 1; }");
    let expr = return_expr(&body[1]);

    let Expr::Binary {
        op: BinaryOp::Add,
        left,
        right,
        ..
    } = expr
    else {
        panic!("expected addition expression");
    };

    assert!(matches!(left.as_ref(), Expr::SizeOfExpr { .. }));
    assert!(matches!(right.as_ref(), Expr::IntLiteral { value: 1, .. }));
}

#[test]
fn parses_typedef_declarations() {
    let program = parse_source(
        "typedef unsigned long uLong;\ntypedef unsigned char *BytefPtr;\nint main() { return 0; }",
    );

    assert_eq!(program.declarations.len(), 3);

    let ExternalDecl::Typedef(first) = &program.declarations[0] else {
        panic!("expected first declaration to be typedef");
    };
    assert_eq!(first.name, "uLong");
    assert_eq!(first.ty, Type::UnsignedLong);

    let ExternalDecl::Typedef(second) = &program.declarations[1] else {
        panic!("expected second declaration to be typedef");
    };
    assert_eq!(second.name, "BytefPtr");
    assert_eq!(second.ty, Type::Pointer(Box::new(Type::UnsignedChar)));

    let function = first_function(&program);
    assert_eq!(function.name, "main");
}

#[test]
fn parses_multiple_typedef_declarators() {
    let program = parse_source("typedef unsigned long uLong, *uLongp;\nint main() { return 0; }");

    assert_eq!(program.declarations.len(), 3);

    let ExternalDecl::Typedef(first) = &program.declarations[0] else {
        panic!("expected first declaration to be typedef");
    };
    assert_eq!(first.name, "uLong");
    assert_eq!(first.ty, Type::UnsignedLong);

    let ExternalDecl::Typedef(second) = &program.declarations[1] else {
        panic!("expected second declaration to be typedef");
    };
    assert_eq!(second.name, "uLongp");
    assert_eq!(second.ty, Type::Pointer(Box::new(Type::UnsignedLong)));

    let function = first_function(&program);
    assert_eq!(function.name, "main");
}

#[test]
fn parses_function_pointer_typedef() {
    let program = parse_source("typedef int (*handler)(int, char *);\nint main() { return 0; }");

    assert_eq!(program.declarations.len(), 2);

    let ExternalDecl::Typedef(typedef) = &program.declarations[0] else {
        panic!("expected typedef declaration");
    };
    assert_eq!(typedef.name, "handler");
    assert_eq!(
        typedef.ty,
        Type::Pointer(Box::new(Type::Function(Box::new(FunctionType {
            return_type: Box::new(Type::Int),
            params: vec![Type::Int, Type::Pointer(Box::new(Type::Char))],
        }))))
    );

    let function = first_function(&program);
    assert_eq!(function.name, "main");
}

#[test]
fn parses_anonymous_struct_typedef() {
    let program = parse_source("typedef struct { int x; char y; } Pair;\nint main() { return 0; }");

    let ExternalDecl::Typedef(typedef) = &program.declarations[0] else {
        panic!("expected typedef declaration");
    };
    assert_eq!(typedef.name, "Pair");

    let Type::Struct { fields } = &typedef.ty else {
        panic!("expected struct typedef type");
    };
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "x");
    assert_eq!(fields[0].ty, Type::Int);
    assert_eq!(fields[1].name, "y");
    assert_eq!(fields[1].ty, Type::Char);
}

#[test]
fn parses_local_anonymous_struct_object() {
    let body = main_body("int main() { struct { int x; char y; } value; return sizeof(value); }");
    let decl = single_decl(&body[0]);

    let Type::Struct { fields } = &decl.ty else {
        panic!("expected local struct object type");
    };
    assert_eq!(decl.name, "value");
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "x");
    assert_eq!(fields[0].ty, Type::Int);
    assert_eq!(fields[1].name, "y");
    assert_eq!(fields[1].ty, Type::Char);
}

#[test]
fn parses_pointer_to_struct_typedef() {
    let body = main_body(
        "typedef struct { char *ptr; unsigned long num_left; } Ctx;\nint main() { Ctx *p; return sizeof(p); }",
    );
    let decl = single_decl(&body[0]);

    let Type::Pointer(inner) = &decl.ty else {
        panic!("expected pointer to struct type");
    };
    let Type::Struct { fields } = inner.as_ref() else {
        panic!("expected pointer to struct type");
    };
    assert_eq!(decl.name, "p");
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "ptr");
    assert_eq!(fields[0].ty, Type::Pointer(Box::new(Type::Char)));
    assert_eq!(fields[1].name, "num_left");
    assert_eq!(fields[1].ty, Type::UnsignedLong);
}

#[test]
fn parses_direct_struct_member_access() {
    let expr = only_return_expr("int main() { return value.x; }");

    let Expr::Member {
        base,
        access,
        field,
        ..
    } = expr
    else {
        panic!("expected member access expression");
    };

    assert!(matches!(
        base.as_ref(),
        Expr::Variable { name, .. } if name == "value"
    ));
    assert_eq!(access, MemberAccessKind::Direct);
    assert_eq!(field, "x");
}

#[test]
fn parses_pointer_struct_member_access() {
    let expr = only_return_expr("int main() { return ctx->ptr; }");

    let Expr::Member {
        base,
        access,
        field,
        ..
    } = expr
    else {
        panic!("expected member access expression");
    };

    assert!(matches!(
        base.as_ref(),
        Expr::Variable { name, .. } if name == "ctx"
    ));
    assert_eq!(access, MemberAccessKind::Pointer);
    assert_eq!(field, "ptr");
}

#[test]
fn parses_member_access_as_postfix_expression() {
    let expr = only_return_expr("int main() { return ctx->ptr[0]; }");

    let Expr::Index { base, index, .. } = expr else {
        panic!("expected indexed expression");
    };
    let Expr::Member {
        base,
        access,
        field,
        ..
    } = base.as_ref()
    else {
        panic!("expected member access expression");
    };

    assert!(matches!(
        base.as_ref(),
        Expr::Variable { name, .. } if name == "ctx"
    ));
    assert_eq!(*access, MemberAccessKind::Pointer);
    assert_eq!(field, "ptr");
    assert!(matches!(index.as_ref(), Expr::IntLiteral { value: 0, .. }));
}

#[test]
fn parses_typedef_names_as_types() {
    let program = parse_source(
        "typedef unsigned long uLong;\ntypedef unsigned char Bytef;\nuLong f(const Bytef *buf) { return buf[0]; }",
    );

    let function = first_function(&program);
    assert_eq!(function.return_type, Type::UnsignedLong);
    assert_eq!(function.name, "f");
    assert_eq!(
        function.params[0].ty,
        Type::Pointer(Box::new(Type::UnsignedChar))
    );
}

#[test]
fn parses_local_typedef_declarations() {
    let body = main_body("int main() { typedef char T; T x; return 0; }");

    assert!(matches!(body[0], Statement::Empty));
    let decl = single_decl(&body[1]);
    assert_eq!(decl.name, "x");
    assert_eq!(decl.ty, Type::Char);
}

#[test]
fn local_typedef_shadows_outer_typedef() {
    let body = main_body("typedef int T; int main() { typedef char T; T x; return 0; }");

    assert!(matches!(body[0], Statement::Empty));
    let decl = single_decl(&body[1]);
    assert_eq!(decl.name, "x");
    assert_eq!(decl.ty, Type::Char);
}

#[test]
fn typedef_scope_is_restored_after_block() {
    let body = main_body("typedef int T; int main() { { typedef char T; T x; } T y; return 0; }");

    let Statement::Block(block) = &body[0] else {
        panic!("expected block statement");
    };
    assert!(matches!(block[0], Statement::Empty));
    let inner_decl = single_decl(&block[1]);
    assert_eq!(inner_decl.name, "x");
    assert_eq!(inner_decl.ty, Type::Char);

    let outer_decl = single_decl(&body[1]);
    assert_eq!(outer_decl.name, "y");
    assert_eq!(outer_decl.ty, Type::Int);
}

#[test]
fn object_declaration_shadows_typedef_name() {
    let body = main_body("typedef int T; int main() { int T = 3; return T; }");

    let decl = single_decl(&body[0]);
    assert_eq!(decl.name, "T");
    assert_eq!(decl.ty, Type::Int);
    assert!(matches!(return_expr(&body[1]), Expr::Variable { name, .. } if name == "T"));
}

#[test]
fn parameter_name_shadows_typedef_name_in_function_body() {
    let program = parse_source("typedef int T; int main(int T) { return T; }");
    let function = first_function(&program);

    assert_eq!(function.params[0].name, "T");
    assert!(matches!(
        return_expr(&function.body[0]),
        Expr::Variable { name, .. } if name == "T"
    ));
}

#[test]
fn for_init_object_name_scope_is_restored_after_loop() {
    let body = main_body(
        "typedef int T; int main() { for (int T = 0; T < 1; T = T + 1) { } T y; return 0; }",
    );

    assert!(matches!(body[0], Statement::For { .. }));
    let decl = single_decl(&body[1]);
    assert_eq!(decl.name, "y");
    assert_eq!(decl.ty, Type::Int);
}

#[test]
fn rejects_duplicate_typedef_names() {
    let err = parse_source_err("typedef unsigned long uLong;\ntypedef unsigned int *uLong;");

    assert_eq!(err.message, "duplicate typedef with name 'uLong'");
}
