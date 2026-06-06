mod common;

use common::{param, param_with_span, span, span_from};
use rivet::ast::{BinaryOp, Expr, Function, Program, Statement};
use rivet::sema::check;

fn main_program(body: Vec<Statement>) -> Program {
    Program {
        functions: vec![Function {
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
        name_span: span(),
        name: name.to_string(),
        params: vec![],
        body,
    }
}

fn function_with_params(name: &str, params: &[&str], body: Vec<Statement>) -> Function {
    Function {
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
            function("helper", vec![Statement::Return(Expr::IntLiteral(1))]),
            function("main", vec![Statement::Return(Expr::IntLiteral(0))]),
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
                        name_span: span(),
                        name: "x".to_string(),
                        init: Some(Expr::IntLiteral(1)),
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
                        name_span: span(),
                        name: "x".to_string(),
                        init: Some(Expr::IntLiteral(2)),
                    },
                    Statement::Return(Expr::Variable {
                        name: "x".to_string(),
                        span: span(),
                    }),
                ],
            ),
            function("main", vec![Statement::Return(Expr::IntLiteral(0))]),
        ],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_duplicate_function_names() {
    let program = Program {
        functions: vec![
            function("main", vec![Statement::Return(Expr::IntLiteral(0))]),
            function("main", vec![Statement::Return(Expr::IntLiteral(1))]),
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
            vec![Statement::Return(Expr::IntLiteral(1))],
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
            function("helper", vec![Statement::Return(Expr::IntLiteral(1))]),
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
            function("helper", vec![Statement::Return(Expr::IntLiteral(1))]),
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
        Statement::Return(Expr::IntLiteral(0)),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_expression_statement() {
    let program = Program {
        functions: vec![
            function("helper", vec![Statement::Return(Expr::IntLiteral(1))]),
            function(
                "main",
                vec![
                    Statement::ExprStatement(Expr::Call {
                        name_span: span(),
                        name: "helper".to_string(),
                        args: vec![],
                    }),
                    Statement::Return(Expr::IntLiteral(0)),
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
        Statement::Return(Expr::IntLiteral(0)),
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
            name_span: span_from(4, 8),
            name: "main".to_string(),
            params: vec![
                param_with_span("x", span_from(13, 14)),
                param_with_span("x", duplicate_span),
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
                    name_span: span(),
                    name: "x".to_string(),
                    init: Some(Expr::IntLiteral(1)),
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
                    name_span: span(),
                    name: "x".to_string(),
                    init: Some(Expr::IntLiteral(1)),
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
                    args: vec![Expr::IntLiteral(1), Expr::IntLiteral(2)],
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
                    args: vec![Expr::IntLiteral(1)],
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
                    args: vec![Expr::IntLiteral(1), Expr::IntLiteral(2)],
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
            vec![Statement::Return(Expr::IntLiteral(0))],
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
            name_span: span_from(4, 8),
            name: "main".to_string(),
            params: vec![
                param_with_span("a", span_from(13, 14)),
                param_with_span("b", span_from(18, 19)),
                param_with_span("c", span_from(23, 24)),
                param_with_span("d", span_from(28, 29)),
                param_with_span("e", span_from(33, 34)),
                param_with_span("f", span_from(38, 39)),
                param_with_span("g", span_from(43, 44)),
                param_with_span("h", span_from(48, 49)),
                param_with_span("i", ninth_span),
            ],
            body: vec![Statement::Return(Expr::IntLiteral(0))],
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
            function("helper", vec![Statement::Return(Expr::IntLiteral(0))]),
            function(
                "main",
                vec![Statement::Return(Expr::Call {
                    name_span: span(),
                    name: "helper".to_string(),
                    args: vec![
                        Expr::IntLiteral(1),
                        Expr::IntLiteral(2),
                        Expr::IntLiteral(3),
                        Expr::IntLiteral(4),
                        Expr::IntLiteral(5),
                        Expr::IntLiteral(6),
                        Expr::IntLiteral(7),
                        Expr::IntLiteral(8),
                        Expr::IntLiteral(9),
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
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral(1)),
        },
        Statement::ExprStatement(Expr::Assign {
            name_span: span(),
            name: "x".to_string(),
            value: Box::new(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                }),
                right: Box::new(Expr::IntLiteral(2)),
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
            name_span: span(),
            name: "x".to_string(),
            init: None,
        },
        Statement::ExprStatement(Expr::Assign {
            name_span: span(),
            name: "x".to_string(),
            value: Box::new(Expr::IntLiteral(3)),
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
            name_span: span(),
            name: "x".to_string(),
            init: None,
        },
        Statement::Return(Expr::Assign {
            name_span: span(),
            name: "x".to_string(),
            value: Box::new(Expr::IntLiteral(3)),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_assignment_expression_to_undeclared_local() {
    let program = main_program(vec![Statement::Return(Expr::Assign {
        name_span: span(),
        name: "x".to_string(),
        value: Box::new(Expr::IntLiteral(3)),
    })]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared local variable 'x'");
}

#[test]
fn accepts_initializer_using_earlier_local() {
    let program = main_program(vec![
        Statement::VarDecl {
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral(1)),
        },
        Statement::VarDecl {
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
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral(1)),
        },
        Statement::VarDecl {
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral(2)),
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
            name_span: span(),
            name: "x".to_string(),
            value: Box::new(Expr::IntLiteral(1)),
        }),
        Statement::Return(Expr::IntLiteral(0)),
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
            name_span: span(),
            name: "y".to_string(),
            init: Some(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
        },
        Statement::VarDecl {
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral(1)),
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
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral(1)),
        },
        Statement::Return(Expr::Binary {
            op: BinaryOp::Multiply,
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
fn accepts_block_using_outer_local() {
    let program = main_program(vec![
        Statement::VarDecl {
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral(1)),
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
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral(1)),
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
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral(1)),
        },
        Statement::Block(vec![
            Statement::VarDecl {
                name_span: span(),
                name: "x".to_string(),
                init: Some(Expr::IntLiteral(2)),
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
                name_span: span(),
                name: "x".to_string(),
                init: Some(Expr::IntLiteral(1)),
            },
            Statement::VarDecl {
                name_span: span(),
                name: "x".to_string(),
                init: Some(Expr::IntLiteral(2)),
            },
        ]),
        Statement::Return(Expr::IntLiteral(0)),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "duplicate local variable 'x'");
}

#[test]
fn accepts_if_else_with_locals_in_branches() {
    let program = main_program(vec![
        Statement::VarDecl {
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral(1)),
        },
        Statement::If {
            cond: Expr::Binary {
                op: BinaryOp::Less,
                left: Box::new(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                }),
                right: Box::new(Expr::IntLiteral(2)),
            },
            then_branch: Box::new(Statement::Block(vec![
                Statement::VarDecl {
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
            name_span: span(),
            name: "x".to_string(),
            init: Some(Expr::IntLiteral(3)),
        },
        Statement::While {
            cond: Expr::Variable {
                name: "x".to_string(),
                span: span(),
            },
            body: Box::new(Statement::Block(vec![Statement::ExprStatement(
                Expr::Assign {
                    name_span: span(),
                    name: "x".to_string(),
                    value: Box::new(Expr::Binary {
                        op: BinaryOp::Subtract,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                            span: span(),
                        }),
                        right: Box::new(Expr::IntLiteral(1)),
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
            cond: Expr::IntLiteral(1),
            body: Box::new(Statement::Block(vec![
                Statement::Continue { span: span() },
                Statement::Break { span: span() },
            ])),
        },
        Statement::Return(Expr::IntLiteral(0)),
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
            cond: Expr::IntLiteral(1),
        },
        Statement::Return(Expr::IntLiteral(0)),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_break_inside_nested_if_in_loop() {
    let program = main_program(vec![
        Statement::While {
            cond: Expr::IntLiteral(1),
            body: Box::new(Statement::If {
                cond: Expr::IntLiteral(1),
                then_branch: Box::new(Statement::Break { span: span() }),
                else_branch: None,
            }),
        },
        Statement::Return(Expr::IntLiteral(0)),
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
            body: Box::new(Statement::Return(Expr::IntLiteral(0))),
        },
        Statement::Return(Expr::IntLiteral(0)),
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
        Statement::Return(Expr::IntLiteral(0)),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared local variable 'x'");
}
