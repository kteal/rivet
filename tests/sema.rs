mod common;

use common::{param, param_with_span, span, span_from};
use rivet::ast::{BinaryOp, Expr, Function, Program, Statement, Type};
use rivet::sema::check;
use rivet::typed_ast::{TypedExprKind, TypedStatement};

fn main_program(body: Vec<Statement>) -> Program {
    Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body,
        }],
        eof_span: span(),
    }
}

fn function(name: &str, body: Vec<Statement>) -> Function {
    Function {
        return_type: Type::Int,
        name_span: span(),
        name: name.to_string(),
        params: vec![],
        body,
    }
}

fn function_with_params(name: &str, params: &[&str], body: Vec<Statement>) -> Function {
    Function {
        return_type: Type::Int,
        name_span: span(),
        name: name.to_string(),
        params: params.iter().map(|name| param(name)).collect(),
        body,
    }
}

#[test]
fn accepts_multiple_functions() {
    let program = Program {
        functions: vec![
            function(
                "helper",
                vec![Statement::Return(Expr::IntLiteral {
                    value: 1,
                    span: span(),
                })],
            ),
            function(
                "main",
                vec![Statement::Return(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                })],
            ),
        ],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_same_local_name_in_different_functions() {
    let program = Program {
        functions: vec![
            function(
                "first",
                vec![
                    Statement::VarDecl {
                        ty: Type::Int,
                        name_span: span(),
                        name: "x".to_string(),
                        init: Some(Expr::IntLiteral {
                            value: 1,
                            span: span(),
                        }),
                    },
                    Statement::Return(Expr::Variable {
                        name: "x".to_string(),
                        span: span(),
                    }),
                ],
            ),
            function(
                "second",
                vec![
                    Statement::VarDecl {
                        ty: Type::Int,
                        name_span: span(),
                        name: "x".to_string(),
                        init: Some(Expr::IntLiteral {
                            value: 2,
                            span: span(),
                        }),
                    },
                    Statement::Return(Expr::Variable {
                        name: "x".to_string(),
                        span: span(),
                    }),
                ],
            ),
            function(
                "main",
                vec![Statement::Return(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                })],
            ),
        ],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_duplicate_function_names() {
    let program = Program {
        functions: vec![
            function(
                "main",
                vec![Statement::Return(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                })],
            ),
            function(
                "main",
                vec![Statement::Return(Expr::IntLiteral {
                    value: 1,
                    span: span(),
                })],
            ),
        ],
        eof_span: span(),
    };

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "duplicate function 'main'");
}

#[test]
fn rejects_program_without_main_function() {
    let program = Program {
        functions: vec![function(
            "helper",
            vec![Statement::Return(Expr::IntLiteral {
                value: 1,
                span: span(),
            })],
        )],
        eof_span: span(),
    };

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "no 'main' function found");
}

#[test]
fn accepts_call_to_declared_function() {
    let program = Program {
        functions: vec![
            function(
                "helper",
                vec![Statement::Return(Expr::IntLiteral {
                    value: 1,
                    span: span(),
                })],
            ),
            function(
                "main",
                vec![Statement::Return(Expr::Call {
                    name_span: span(),
                    name: "helper".to_string(),
                    args: vec![],
                })],
            ),
        ],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_forward_call_to_later_function() {
    let program = Program {
        functions: vec![
            function(
                "main",
                vec![Statement::Return(Expr::Call {
                    name_span: span(),
                    name: "helper".to_string(),
                    args: vec![],
                })],
            ),
            function(
                "helper",
                vec![Statement::Return(Expr::IntLiteral {
                    value: 1,
                    span: span(),
                })],
            ),
        ],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_call_to_undeclared_function() {
    let program = main_program(vec![Statement::Return(Expr::Call {
        name_span: span(),
        name: "helper".to_string(),
        args: vec![],
    })]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared function 'helper'");
}

#[test]
fn accepts_empty_statement() {
    let program = main_program(vec![
        Statement::Empty,
        Statement::Return(Expr::IntLiteral {
            value: 0,
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_expression_statement() {
    let program = Program {
        functions: vec![
            function(
                "helper",
                vec![Statement::Return(Expr::IntLiteral {
                    value: 1,
                    span: span(),
                })],
            ),
            function(
                "main",
                vec![
                    Statement::ExprStatement(Expr::Call {
                        name_span: span(),
                        name: "helper".to_string(),
                        args: vec![],
                    }),
                    Statement::Return(Expr::IntLiteral {
                        value: 0,
                        span: span(),
                    }),
                ],
            ),
        ],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_expression_statement_with_undeclared_variable() {
    let program = main_program(vec![
        Statement::ExprStatement(Expr::Variable {
            name: "x".to_string(),
            span: span(),
        }),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared local variable 'x'");
}

#[test]
fn accepts_parameter_usage_as_local() {
    let program = Program {
        functions: vec![function_with_params(
            "main",
            &["x", "y"],
            vec![Statement::Return(Expr::Binary {
                op: BinaryOp::Add,
                op_span: span(),
                left: Box::new(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                }),
                right: Box::new(Expr::Variable {
                    name: "y".to_string(),
                    span: span(),
                }),
            })],
        )],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_duplicate_parameter_names() {
    let program = Program {
        functions: vec![function_with_params(
            "main",
            &["x", "x"],
            vec![Statement::Return(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            })],
        )],
        eof_span: span(),
    };

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "duplicate local variable 'x'");
}

#[test]
fn duplicate_parameter_errors_point_at_duplicate_parameter() {
    let duplicate_span = span_from(20, 21);
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span_from(4, 8),
            name: "main".to_string(),
            params: vec![
                param_with_span(Type::Int, "x", span_from(13, 14)),
                param_with_span(Type::Int, "x", duplicate_span),
            ],
            body: vec![Statement::Return(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            })],
        }],
        eof_span: span(),
    };

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "duplicate local variable 'x'");
    assert_eq!(err.span, duplicate_span);
}

#[test]
fn rejects_local_redeclaring_parameter_in_function_scope() {
    let program = Program {
        functions: vec![function_with_params(
            "main",
            &["x"],
            vec![
                Statement::VarDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "x".to_string(),
                    init: Some(Expr::IntLiteral {
                        value: 1,
                        span: span(),
                    }),
                },
                Statement::Return(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                }),
            ],
        )],
        eof_span: span(),
    };

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "duplicate local variable 'x'");
}

#[test]
fn accepts_inner_block_shadowing_parameter() {
    let program = Program {
        functions: vec![function_with_params(
            "main",
            &["x"],
            vec![Statement::Block(vec![
                Statement::VarDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "x".to_string(),
                    init: Some(Expr::IntLiteral {
                        value: 1,
                        span: span(),
                    }),
                },
                Statement::Return(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                }),
            ])],
        )],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_call_with_matching_argument_count() {
    let program = Program {
        functions: vec![
            function_with_params(
                "add",
                &["x", "y"],
                vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Add,
                    op_span: span(),
                    left: Box::new(Expr::Variable {
                        name: "x".to_string(),
                        span: span(),
                    }),
                    right: Box::new(Expr::Variable {
                        name: "y".to_string(),
                        span: span(),
                    }),
                })],
            ),
            function(
                "main",
                vec![Statement::Return(Expr::Call {
                    name_span: span(),
                    name: "add".to_string(),
                    args: vec![
                        Expr::IntLiteral {
                            value: 1,
                            span: span(),
                        },
                        Expr::IntLiteral {
                            value: 2,
                            span: span(),
                        },
                    ],
                })],
            ),
        ],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_call_with_too_few_arguments() {
    let program = Program {
        functions: vec![
            function_with_params(
                "add",
                &["x", "y"],
                vec![Statement::Return(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                })],
            ),
            function(
                "main",
                vec![Statement::Return(Expr::Call {
                    name_span: span(),
                    name: "add".to_string(),
                    args: vec![Expr::IntLiteral {
                        value: 1,
                        span: span(),
                    }],
                })],
            ),
        ],
        eof_span: span(),
    };

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "function call of 'add' has 1 arguments, declaration has 2"
    );
}

#[test]
fn rejects_call_with_too_many_arguments_for_signature() {
    let program = Program {
        functions: vec![
            function_with_params(
                "id",
                &["x"],
                vec![Statement::Return(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                })],
            ),
            function(
                "main",
                vec![Statement::Return(Expr::Call {
                    name_span: span(),
                    name: "id".to_string(),
                    args: vec![
                        Expr::IntLiteral {
                            value: 1,
                            span: span(),
                        },
                        Expr::IntLiteral {
                            value: 2,
                            span: span(),
                        },
                    ],
                })],
            ),
        ],
        eof_span: span(),
    };

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "function call of 'id' has 2 arguments, declaration has 1"
    );
}

#[test]
fn rejects_function_with_more_than_eight_parameters() {
    let program = Program {
        functions: vec![function_with_params(
            "main",
            &["a", "b", "c", "d", "e", "f", "g", "h", "i"],
            vec![Statement::Return(Expr::IntLiteral {
                value: 0,
                span: span(),
            })],
        )],
        eof_span: span(),
    };

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "too many parameters in function main, got 9, max 8"
    );
}

#[test]
fn too_many_parameter_errors_point_at_ninth_parameter() {
    let ninth_span = span_from(50, 51);
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span_from(4, 8),
            name: "main".to_string(),
            params: vec![
                param_with_span(Type::Int, "a", span_from(13, 14)),
                param_with_span(Type::Int, "b", span_from(18, 19)),
                param_with_span(Type::Int, "c", span_from(23, 24)),
                param_with_span(Type::Int, "d", span_from(28, 29)),
                param_with_span(Type::Int, "e", span_from(33, 34)),
                param_with_span(Type::Int, "f", span_from(38, 39)),
                param_with_span(Type::Int, "g", span_from(43, 44)),
                param_with_span(Type::Int, "h", span_from(48, 49)),
                param_with_span(Type::Int, "i", ninth_span),
            ],
            body: vec![Statement::Return(Expr::IntLiteral {
                value: 0,
                span: span(),
            })],
        }],
        eof_span: span(),
    };

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "too many parameters in function main, got 9, max 8"
    );
    assert_eq!(err.span, ninth_span);
}

#[test]
fn rejects_call_with_more_than_eight_arguments() {
    let program = Program {
        functions: vec![
            function(
                "helper",
                vec![Statement::Return(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                })],
            ),
            function(
                "main",
                vec![Statement::Return(Expr::Call {
                    name_span: span(),
                    name: "helper".to_string(),
                    args: vec![
                        Expr::IntLiteral {
                            value: 1,
                            span: span(),
                        },
                        Expr::IntLiteral {
                            value: 2,
                            span: span(),
                        },
                        Expr::IntLiteral {
                            value: 3,
                            span: span(),
                        },
                        Expr::IntLiteral {
                            value: 4,
                            span: span(),
                        },
                        Expr::IntLiteral {
                            value: 5,
                            span: span(),
                        },
                        Expr::IntLiteral {
                            value: 6,
                            span: span(),
                        },
                        Expr::IntLiteral {
                            value: 7,
                            span: span(),
                        },
                        Expr::IntLiteral {
                            value: 8,
                            span: span(),
                        },
                        Expr::IntLiteral {
                            value: 9,
                            span: span(),
                        },
                    ],
                })],
            ),
        ],
        eof_span: span(),
    };

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "too many arguments in call to function helper, got 9, max 8"
    );
}

#[test]
fn accepts_declared_local_usage() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
        },
        Statement::ExprStatement(Expr::Assign {
            op_span: span(),
            target: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            value: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                op_span: span(),
                left: Box::new(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 2,
                    span: span(),
                }),
            }),
        }),
        Statement::Return(Expr::Variable {
            name: "x".to_string(),
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_declaration_without_initializer_assigned_before_use() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: None,
        },
        Statement::ExprStatement(Expr::Assign {
            op_span: span(),
            target: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            value: Box::new(Expr::IntLiteral {
                value: 3,
                span: span(),
            }),
        }),
        Statement::Return(Expr::Variable {
            name: "x".to_string(),
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_assignment_expression_in_return() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: None,
        },
        Statement::Return(Expr::Assign {
            op_span: span(),
            target: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            value: Box::new(Expr::IntLiteral {
                value: 3,
                span: span(),
            }),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_compound_assignment_to_int() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral {
                value: 3,
                span: span(),
            }),
        },
        Statement::ExprStatement(Expr::CompoundAssign {
            target: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            op: BinaryOp::Add,
            op_span: span(),
            value: Box::new(Expr::IntLiteral {
                value: 4,
                span: span(),
            }),
        }),
        Statement::Return(Expr::Variable {
            name: "x".to_string(),
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_compound_assignment_to_char_from_int_expression() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Char,
            name_span: span(),
            name: "c".to_string(),
            init: Some(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
        },
        Statement::ExprStatement(Expr::CompoundAssign {
            target: Box::new(Expr::Variable {
                name: "c".to_string(),
                span: span(),
            }),
            op: BinaryOp::Add,
            op_span: span(),
            value: Box::new(Expr::IntLiteral {
                value: 2,
                span: span(),
            }),
        }),
        Statement::Return(Expr::Variable {
            name: "c".to_string(),
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_compound_assignment_expression_in_return() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral {
                value: 3,
                span: span(),
            }),
        },
        Statement::Return(Expr::CompoundAssign {
            target: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            op: BinaryOp::Add,
            op_span: span(),
            value: Box::new(Expr::IntLiteral {
                value: 4,
                span: span(),
            }),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn typed_binary_expression_has_result_type() {
    let program = main_program(vec![Statement::Return(Expr::Binary {
        op: BinaryOp::Add,
        op_span: span(),
        left: Box::new(Expr::IntLiteral {
            value: 1,
            span: span(),
        }),
        right: Box::new(Expr::IntLiteral {
            value: 2,
            span: span(),
        }),
    })]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &typed_program.functions[0].body[0] else {
        panic!("expected return statement");
    };

    assert_eq!(expr.ty, Type::Int);
    assert!(matches!(expr.kind, TypedExprKind::Binary { .. }));
}

#[test]
fn typed_char_compound_assignment_has_target_type() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Char,
            name_span: span(),
            name: "c".to_string(),
            init: Some(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
        },
        Statement::Return(Expr::CompoundAssign {
            target: Box::new(Expr::Variable {
                name: "c".to_string(),
                span: span(),
            }),
            op: BinaryOp::Add,
            op_span: span(),
            value: Box::new(Expr::IntLiteral {
                value: 2,
                span: span(),
            }),
        }),
    ]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &typed_program.functions[0].body[1] else {
        panic!("expected return statement");
    };

    assert_eq!(expr.ty, Type::Char);
    assert!(matches!(expr.kind, TypedExprKind::CompoundAssign { .. }));
}

#[test]
fn typed_postfix_increment_preserves_postfix_kind() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
        },
        Statement::Return(Expr::PostfixInc {
            expr: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            op_span: span(),
        }),
    ]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &typed_program.functions[0].body[1] else {
        panic!("expected return statement");
    };

    assert_eq!(expr.ty, Type::Int);
    assert!(matches!(expr.kind, TypedExprKind::PostfixInc { .. }));
}

#[test]
fn typed_call_preserves_typed_arguments() {
    let program = Program {
        functions: vec![
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "id".to_string(),
                params: vec![param_with_span(Type::Char, "c", span())],
                body: vec![Statement::Return(Expr::Variable {
                    name: "c".to_string(),
                    span: span(),
                })],
            },
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Call {
                    name_span: span(),
                    name: "id".to_string(),
                    args: vec![Expr::IntLiteral {
                        value: 65,
                        span: span(),
                    }],
                })],
            },
        ],
        eof_span: span(),
    };

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &typed_program.functions[1].body[0] else {
        panic!("expected return statement");
    };

    let TypedExprKind::Call { args, .. } = &expr.kind else {
        panic!("expected call expression");
    };

    assert_eq!(expr.ty, Type::Int);
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].ty, Type::Int);
    assert!(matches!(args[0].kind, TypedExprKind::IntLiteral { .. }));
}

#[test]
fn rejects_assignment_expression_to_undeclared_local() {
    let program = main_program(vec![Statement::Return(Expr::Assign {
        op_span: span(),
        target: Box::new(Expr::Variable {
            name: "x".to_string(),
            span: span(),
        }),
        value: Box::new(Expr::IntLiteral {
            value: 3,
            span: span(),
        }),
    })]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared local variable 'x'");
}

#[test]
fn rejects_assignment_to_non_lvalue_expression() {
    let op_span = span_from(10, 11);
    let program = main_program(vec![Statement::Return(Expr::Assign {
        target: Box::new(Expr::Binary {
            op: BinaryOp::Add,
            op_span: span(),
            left: Box::new(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
            right: Box::new(Expr::IntLiteral {
                value: 2,
                span: span(),
            }),
        }),
        op_span,
        value: Box::new(Expr::IntLiteral {
            value: 3,
            span: span(),
        }),
    })]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.span, op_span);
    assert_eq!(err.message, "cannot assign to non-variable expression");
}

#[test]
fn rejects_compound_assignment_to_non_lvalue_expression() {
    let op_span = span_from(10, 12);
    let program = main_program(vec![Statement::Return(Expr::CompoundAssign {
        target: Box::new(Expr::Binary {
            op: BinaryOp::Add,
            op_span: span(),
            left: Box::new(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
            right: Box::new(Expr::IntLiteral {
                value: 2,
                span: span(),
            }),
        }),
        op: BinaryOp::Add,
        op_span,
        value: Box::new(Expr::IntLiteral {
            value: 3,
            span: span(),
        }),
    })]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.span, op_span);
    assert_eq!(err.message, "cannot assign to non-variable expression");
}

#[test]
fn accepts_prefix_and_postfix_increment_decrement() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
        },
        Statement::ExprStatement(Expr::PrefixInc {
            expr: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            op_span: span(),
        }),
        Statement::ExprStatement(Expr::PostfixInc {
            expr: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            op_span: span(),
        }),
        Statement::ExprStatement(Expr::PrefixDec {
            expr: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            op_span: span(),
        }),
        Statement::Return(Expr::PostfixDec {
            expr: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            op_span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_increment_of_non_lvalue_expression() {
    let op_span = span_from(10, 12);
    let program = main_program(vec![Statement::Return(Expr::PostfixInc {
        expr: Box::new(Expr::Binary {
            op: BinaryOp::Add,
            op_span: span(),
            left: Box::new(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
            right: Box::new(Expr::IntLiteral {
                value: 2,
                span: span(),
            }),
        }),
        op_span,
    })]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.span, op_span);
    assert_eq!(err.message, "cannot assign to non-variable expression");
}

#[test]
fn accepts_initializer_using_earlier_local() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
        },
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "y".to_string(),
            init: Some(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
        },
        Statement::Return(Expr::Variable {
            name: "y".to_string(),
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_initializer_using_declared_name_itself() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
        },
        Statement::Return(Expr::Variable {
            name: "x".to_string(),
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_duplicate_local_declaration() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
        },
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral {
                value: 2,
                span: span(),
            }),
        },
        Statement::Return(Expr::Variable {
            name: "x".to_string(),
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "duplicate local variable 'x'");
}

#[test]
fn rejects_assignment_to_undeclared_local() {
    let program = main_program(vec![
        Statement::ExprStatement(Expr::Assign {
            op_span: span(),
            target: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            value: Box::new(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
        }),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared local variable 'x'");
}

#[test]
fn rejects_returning_undeclared_local() {
    let program = main_program(vec![Statement::Return(Expr::Variable {
        name: "x".to_string(),
        span: span(),
    })]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared local variable 'x'");
}

#[test]
fn rejects_initializer_using_later_local() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "y".to_string(),
            init: Some(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
        },
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
        },
        Statement::Return(Expr::Variable {
            name: "y".to_string(),
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared local variable 'x'");
}

#[test]
fn rejects_undeclared_local_inside_nested_expression() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
        },
        Statement::Return(Expr::Binary {
            op: BinaryOp::Multiply,
            op_span: span(),
            left: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            right: Box::new(Expr::Variable {
                name: "y".to_string(),
                span: span(),
            }),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared local variable 'y'");
}

#[test]
fn accepts_char_function_return_used_as_char_initializer() {
    let program = Program {
        functions: vec![
            Function {
                return_type: Type::Char,
                name_span: span(),
                name: "id".to_string(),
                params: vec![param_with_span(Type::Char, "x", span())],
                body: vec![Statement::Return(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                })],
            },
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![param_with_span(Type::Char, "c", span())],
                body: vec![
                    Statement::VarDecl {
                        ty: Type::Char,
                        name_span: span(),
                        name: "result".to_string(),
                        init: Some(Expr::Call {
                            name_span: span(),
                            name: "id".to_string(),
                            args: vec![Expr::Variable {
                                name: "c".to_string(),
                                span: span(),
                            }],
                        }),
                    },
                    Statement::Return(Expr::IntLiteral {
                        value: 0,
                        span: span(),
                    }),
                ],
            },
        ],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_char_initializer_from_int_literal() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Char,
            name_span: span(),
            name: "c".to_string(),
            init: Some(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
        },
        Statement::Return(Expr::IntLiteral {
            value: 0,
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_char_initializer_from_int_expression() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Char,
            name_span: span(),
            name: "c".to_string(),
            init: Some(Expr::Binary {
                op: BinaryOp::Add,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 1,
                    span: span(),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 2,
                    span: span(),
                }),
            }),
        },
        Statement::Return(Expr::IntLiteral {
            value: 0,
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_char_assignment_from_int_expression() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Char,
            name_span: span(),
            name: "c".to_string(),
            init: None,
        },
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "i".to_string(),
            init: None,
        },
        Statement::ExprStatement(Expr::Assign {
            op_span: span(),
            target: Box::new(Expr::Variable {
                name: "c".to_string(),
                span: span(),
            }),
            value: Box::new(Expr::Variable {
                name: "i".to_string(),
                span: span(),
            }),
        }),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_int_initializer_from_char_expression() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Char,
            name_span: span(),
            name: "c".to_string(),
            init: Some(Expr::IntLiteral {
                value: 65,
                span: span(),
            }),
        },
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "i".to_string(),
            init: Some(Expr::Variable {
                name: "c".to_string(),
                span: span(),
            }),
        },
        Statement::Return(Expr::Variable {
            name: "i".to_string(),
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_char_argument_from_int_expression() {
    let program = Program {
        functions: vec![
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "takes_char".to_string(),
                params: vec![param_with_span(Type::Char, "x", span())],
                body: vec![Statement::Return(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                })],
            },
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![
                    Statement::VarDecl {
                        ty: Type::Int,
                        name_span: span(),
                        name: "i".to_string(),
                        init: None,
                    },
                    Statement::Return(Expr::Call {
                        name_span: span(),
                        name: "takes_char".to_string(),
                        args: vec![Expr::Variable {
                            name: "i".to_string(),
                            span: span(),
                        }],
                    }),
                ],
            },
        ],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_int_argument_from_char_expression() {
    let program = Program {
        functions: vec![
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "takes_int".to_string(),
                params: vec![param_with_span(Type::Int, "x", span())],
                body: vec![Statement::Return(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                })],
            },
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![
                    Statement::VarDecl {
                        ty: Type::Char,
                        name_span: span(),
                        name: "c".to_string(),
                        init: Some(Expr::IntLiteral {
                            value: 65,
                            span: span(),
                        }),
                    },
                    Statement::Return(Expr::Call {
                        name_span: span(),
                        name: "takes_int".to_string(),
                        args: vec![Expr::Variable {
                            name: "c".to_string(),
                            span: span(),
                        }],
                    }),
                ],
            },
        ],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_char_return_from_int_expression() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Char,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::Add,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 1,
                    span: span(),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 2,
                    span: span(),
                }),
            })],
        }],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_int_return_from_char_expression() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![param_with_span(Type::Char, "c", span())],
            body: vec![Statement::Return(Expr::Variable {
                name: "c".to_string(),
                span: span(),
            })],
        }],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_binary_expression_between_char_and_int() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Char,
            name_span: span(),
            name: "c".to_string(),
            init: None,
        },
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "i".to_string(),
            init: None,
        },
        Statement::Return(Expr::Binary {
            op: BinaryOp::Add,
            op_span: span(),
            left: Box::new(Expr::Variable {
                name: "c".to_string(),
                span: span(),
            }),
            right: Box::new(Expr::Variable {
                name: "i".to_string(),
                span: span(),
            }),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_block_using_outer_local() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
        },
        Statement::Block(vec![Statement::Return(Expr::Variable {
            name: "x".to_string(),
            span: span(),
        })]),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_use_of_local_after_block_scope_ends() {
    let program = main_program(vec![
        Statement::Block(vec![Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
        }]),
        Statement::Return(Expr::Variable {
            name: "x".to_string(),
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared local variable 'x'");
}

#[test]
fn accepts_shadowing_in_inner_block() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
        },
        Statement::Block(vec![
            Statement::VarDecl {
                ty: Type::Int,
                name_span: span(),
                name: "x".to_string(),
                init: Some(Expr::IntLiteral {
                    value: 2,
                    span: span(),
                }),
            },
            Statement::Return(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
        ]),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_duplicate_local_in_same_block() {
    let program = main_program(vec![
        Statement::Block(vec![
            Statement::VarDecl {
                ty: Type::Int,
                name_span: span(),
                name: "x".to_string(),
                init: Some(Expr::IntLiteral {
                    value: 1,
                    span: span(),
                }),
            },
            Statement::VarDecl {
                ty: Type::Int,
                name_span: span(),
                name: "x".to_string(),
                init: Some(Expr::IntLiteral {
                    value: 2,
                    span: span(),
                }),
            },
        ]),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "duplicate local variable 'x'");
}

#[test]
fn accepts_if_else_with_locals_in_branches() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral {
                value: 1,
                span: span(),
            }),
        },
        Statement::If {
            cond: Expr::Binary {
                op: BinaryOp::Less,
                op_span: span(),
                left: Box::new(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 2,
                    span: span(),
                }),
            },
            then_branch: Box::new(Statement::Block(vec![
                Statement::VarDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "y".to_string(),
                    init: Some(Expr::Variable {
                        name: "x".to_string(),
                        span: span(),
                    }),
                },
                Statement::Return(Expr::Variable {
                    name: "y".to_string(),
                    span: span(),
                }),
            ])),
            else_branch: Some(Box::new(Statement::Block(vec![
                Statement::VarDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "z".to_string(),
                    init: Some(Expr::Variable {
                        name: "x".to_string(),
                        span: span(),
                    }),
                },
                Statement::Return(Expr::Variable {
                    name: "z".to_string(),
                    span: span(),
                }),
            ]))),
        },
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_while_with_local_condition_and_body() {
    let program = main_program(vec![
        Statement::VarDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral {
                value: 3,
                span: span(),
            }),
        },
        Statement::While {
            cond: Expr::Variable {
                name: "x".to_string(),
                span: span(),
            },
            body: Box::new(Statement::Block(vec![Statement::ExprStatement(
                Expr::Assign {
                    op_span: span(),
                    target: Box::new(Expr::Variable {
                        name: "x".to_string(),
                        span: span(),
                    }),
                    value: Box::new(Expr::Binary {
                        op: BinaryOp::Subtract,
                        op_span: span(),
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                            span: span(),
                        }),
                        right: Box::new(Expr::IntLiteral {
                            value: 1,
                            span: span(),
                        }),
                    }),
                },
            )])),
        },
        Statement::Return(Expr::Variable {
            name: "x".to_string(),
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_break_and_continue_inside_loop() {
    let program = main_program(vec![
        Statement::While {
            cond: Expr::IntLiteral {
                value: 1,
                span: span(),
            },
            body: Box::new(Statement::Block(vec![
                Statement::Continue { span: span() },
                Statement::Break { span: span() },
            ])),
        },
        Statement::Return(Expr::IntLiteral {
            value: 0,
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_break_and_continue_inside_do_while_loop() {
    let program = main_program(vec![
        Statement::DoWhile {
            body: Box::new(Statement::Block(vec![
                Statement::Continue { span: span() },
                Statement::Break { span: span() },
            ])),
            cond: Expr::IntLiteral {
                value: 1,
                span: span(),
            },
        },
        Statement::Return(Expr::IntLiteral {
            value: 0,
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_break_inside_nested_if_in_loop() {
    let program = main_program(vec![
        Statement::While {
            cond: Expr::IntLiteral {
                value: 1,
                span: span(),
            },
            body: Box::new(Statement::If {
                cond: Expr::IntLiteral {
                    value: 1,
                    span: span(),
                },
                then_branch: Box::new(Statement::Break { span: span() }),
                else_branch: None,
            }),
        },
        Statement::Return(Expr::IntLiteral {
            value: 0,
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_break_outside_loop() {
    let program = main_program(vec![Statement::Break { span: span() }]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "cannot use 'break' outside of a loop");
}

#[test]
fn rejects_continue_outside_loop() {
    let program = main_program(vec![Statement::Continue { span: span() }]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "cannot use 'continue' outside of a loop");
}

#[test]
fn rejects_while_condition_using_undeclared_local() {
    let program = main_program(vec![
        Statement::While {
            cond: Expr::Variable {
                name: "x".to_string(),
                span: span(),
            },
            body: Box::new(Statement::Return(Expr::IntLiteral {
                value: 0,
                span: span(),
            })),
        },
        Statement::Return(Expr::IntLiteral {
            value: 0,
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared local variable 'x'");
}

#[test]
fn rejects_do_while_condition_using_undeclared_local() {
    let program = main_program(vec![
        Statement::DoWhile {
            body: Box::new(Statement::Empty),
            cond: Expr::Variable {
                name: "x".to_string(),
                span: span(),
            },
        },
        Statement::Return(Expr::IntLiteral {
            value: 0,
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared local variable 'x'");
}
