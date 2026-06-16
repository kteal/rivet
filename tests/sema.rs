mod common;

use common::{
    first_typed_function, param, param_with_span, program_with_functions, span, span_from,
    typed_function_at,
};
use rivet::ast::{
    BinaryOp, Expr, ExternalDecl, FunctionDecl, FunctionDef, Initializer, IntLiteralBase,
    IntLiteralSuffix, Program, Statement, Type, UnaryOp,
};
use rivet::sema::check;
use rivet::typed_ast::{TypedExprKind, TypedStatement};

use crate::common::param_decl;

fn main_program(body: Vec<Statement>) -> Program {
    program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body,
    }])
}

fn function(name: &str, body: Vec<Statement>) -> FunctionDef {
    FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: name.to_string(),
        params: vec![],
        body,
    }
}

fn function_with_params(name: &str, params: &[&str], body: Vec<Statement>) -> FunctionDef {
    FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: name.to_string(),
        params: params.iter().map(|name| param(name)).collect(),
        body,
    }
}

fn function_decl(name: &str, params: &[&str]) -> FunctionDecl {
    FunctionDecl {
        return_type: Type::Int,
        name_span: span(),
        name: name.to_string(),
        params: params.iter().map(|name| param_decl(name)).collect(),
    }
}

#[test]
fn accepts_multiple_functions() {
    let program = program_with_functions(vec![
        function(
            "helper",
            vec![Statement::Return(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })],
        ),
        function(
            "main",
            vec![Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })],
        ),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_same_local_name_in_different_functions() {
    let program = program_with_functions(vec![
        function(
            "first",
            vec![
                Statement::Decl(vec![rivet::ast::LocalDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "x".to_string(),
                    init: Some(Initializer::Expr(Expr::IntLiteral {
                        value: 1,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    })),
                }]),
                Statement::Return(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                }),
            ],
        ),
        function(
            "second",
            vec![
                Statement::Decl(vec![rivet::ast::LocalDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "x".to_string(),
                    init: Some(Initializer::Expr(Expr::IntLiteral {
                        value: 2,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    })),
                }]),
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
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })],
        ),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_duplicate_function_names() {
    let program = program_with_functions(vec![
        function(
            "main",
            vec![Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })],
        ),
        function(
            "main",
            vec![Statement::Return(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })],
        ),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "duplicate function definition 'main'");
}

#[test]
fn rejects_program_without_main_function() {
    let program = program_with_functions(vec![function(
        "helper",
        vec![Statement::Return(Expr::IntLiteral {
            value: 1,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        })],
    )]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "no 'main' function found");
}

#[test]
fn accepts_call_to_declared_function() {
    let program = program_with_functions(vec![
        function(
            "helper",
            vec![Statement::Return(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
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
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_forward_call_to_later_function() {
    let program = program_with_functions(vec![
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
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })],
        ),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_function_prototype_before_definition() {
    let program = Program {
        declarations: vec![
            ExternalDecl::FunctionDecl(function_decl("helper", &["x"])),
            ExternalDecl::FunctionDef(function_with_params(
                "helper",
                &["value"],
                vec![Statement::Return(Expr::Variable {
                    name: "value".to_string(),
                    span: span(),
                })],
            )),
            ExternalDecl::FunctionDef(function(
                "main",
                vec![Statement::Return(Expr::Call {
                    name_span: span(),
                    name: "helper".to_string(),
                    args: vec![Expr::IntLiteral {
                        value: 3,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    }],
                })],
            )),
        ],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_call_to_prototyped_function_without_definition() {
    let program = Program {
        declarations: vec![
            ExternalDecl::FunctionDecl(function_decl("helper", &["x"])),
            ExternalDecl::FunctionDef(function(
                "main",
                vec![Statement::Return(Expr::Call {
                    name_span: span(),
                    name: "helper".to_string(),
                    args: vec![Expr::IntLiteral {
                        value: 3,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    }],
                })],
            )),
        ],
        eof_span: span(),
    };

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_function_definition_with_conflicting_prototype_return_type() {
    let program = Program {
        declarations: vec![
            ExternalDecl::FunctionDecl(function_decl("helper", &["x"])),
            ExternalDecl::FunctionDef(FunctionDef {
                return_type: Type::Char,
                name_span: span(),
                name: "helper".to_string(),
                params: vec![param("x")],
                body: vec![Statement::Return(Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })],
            }),
            ExternalDecl::FunctionDef(function(
                "main",
                vec![Statement::Return(Expr::IntLiteral {
                    value: 0,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })],
            )),
        ],
        eof_span: span(),
    };

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "function declaration and definition must have same return type, got 'Int' and 'Char'"
    );
}

#[test]
fn rejects_function_definition_with_conflicting_prototype_parameter_type() {
    let program = Program {
        declarations: vec![
            ExternalDecl::FunctionDecl(function_decl("helper", &["x"])),
            ExternalDecl::FunctionDef(FunctionDef {
                return_type: Type::Int,
                name_span: span(),
                name: "helper".to_string(),
                params: vec![param_with_span(Type::Char, "x", span())],
                body: vec![Statement::Return(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                })],
            }),
            ExternalDecl::FunctionDef(function(
                "main",
                vec![Statement::Return(Expr::IntLiteral {
                    value: 0,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })],
            )),
        ],
        eof_span: span(),
    };

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "function declaration and definition must have same parameter types, got '[Int]' and '[Char]'"
    );
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
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_expression_statement() {
    let program = program_with_functions(vec![
        function(
            "helper",
            vec![Statement::Return(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
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
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
            ],
        ),
    ]);

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
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared local variable 'x'");
}

#[test]
fn accepts_parameter_usage_as_local() {
    let program = program_with_functions(vec![function_with_params(
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
    )]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_duplicate_parameter_names() {
    let program = program_with_functions(vec![function_with_params(
        "main",
        &["x", "x"],
        vec![Statement::Return(Expr::Variable {
            name: "x".to_string(),
            span: span(),
        })],
    )]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "duplicate local variable 'x'");
}

#[test]
fn duplicate_parameter_errors_point_at_duplicate_parameter() {
    let duplicate_span = span_from(20, 21);
    let program = program_with_functions(vec![FunctionDef {
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
    }]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "duplicate local variable 'x'");
    assert_eq!(err.span, duplicate_span);
}

#[test]
fn rejects_local_redeclaring_parameter_in_function_scope() {
    let program = program_with_functions(vec![function_with_params(
        "main",
        &["x"],
        vec![
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "x".to_string(),
                init: Some(Initializer::Expr(Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })),
            }]),
            Statement::Return(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
        ],
    )]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "duplicate local variable 'x'");
}

#[test]
fn accepts_inner_block_shadowing_parameter() {
    let program = program_with_functions(vec![function_with_params(
        "main",
        &["x"],
        vec![Statement::Block(vec![
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "x".to_string(),
                init: Some(Initializer::Expr(Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })),
            }]),
            Statement::Return(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
        ])],
    )]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_call_with_matching_argument_count() {
    let program = program_with_functions(vec![
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
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    },
                    Expr::IntLiteral {
                        value: 2,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    },
                ],
            })],
        ),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_call_with_too_few_arguments() {
    let program = program_with_functions(vec![
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
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }],
            })],
        ),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "function call of 'add' has 1 arguments, declaration has 2"
    );
}

#[test]
fn rejects_call_with_too_many_arguments_for_signature() {
    let program = program_with_functions(vec![
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
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    },
                    Expr::IntLiteral {
                        value: 2,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    },
                ],
            })],
        ),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "function call of 'id' has 2 arguments, declaration has 1"
    );
}

#[test]
fn rejects_function_with_more_than_eight_parameters() {
    let program = program_with_functions(vec![function_with_params(
        "main",
        &["a", "b", "c", "d", "e", "f", "g", "h", "i"],
        vec![Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        })],
    )]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "too many parameters in function main, got 9, max 8"
    );
}

#[test]
fn too_many_parameter_errors_point_at_ninth_parameter() {
    let ninth_span = span_from(50, 51);
    let program = program_with_functions(vec![FunctionDef {
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
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        })],
    }]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "too many parameters in function main, got 9, max 8"
    );
    assert_eq!(err.span, span_from(4, 8));
}

#[test]
fn rejects_call_with_more_than_eight_arguments() {
    let program = program_with_functions(vec![
        function(
            "helper",
            vec![Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
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
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    },
                    Expr::IntLiteral {
                        value: 2,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    },
                    Expr::IntLiteral {
                        value: 3,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    },
                    Expr::IntLiteral {
                        value: 4,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    },
                    Expr::IntLiteral {
                        value: 5,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    },
                    Expr::IntLiteral {
                        value: 6,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    },
                    Expr::IntLiteral {
                        value: 7,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    },
                    Expr::IntLiteral {
                        value: 8,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    },
                    Expr::IntLiteral {
                        value: 9,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    },
                ],
            })],
        ),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "too many arguments in call to function helper, got 9, max 8"
    );
}

#[test]
fn accepts_declared_local_usage() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
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
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
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
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: None,
        }]),
        Statement::ExprStatement(Expr::Assign {
            op_span: span(),
            target: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            value: Box::new(Expr::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
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
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: None,
        }]),
        Statement::Return(Expr::Assign {
            op_span: span(),
            target: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            value: Box::new(Expr::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_compound_assignment_to_int() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::ExprStatement(Expr::CompoundAssign {
            target: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            op: BinaryOp::Add,
            op_span: span(),
            value: Box::new(Expr::IntLiteral {
                value: 4,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
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
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Char,
            name_span: span(),
            name: "c".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::ExprStatement(Expr::CompoundAssign {
            target: Box::new(Expr::Variable {
                name: "c".to_string(),
                span: span(),
            }),
            op: BinaryOp::Add,
            op_span: span(),
            value: Box::new(Expr::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
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
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Return(Expr::CompoundAssign {
            target: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            op: BinaryOp::Add,
            op_span: span(),
            value: Box::new(Expr::IntLiteral {
                value: 4,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
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
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
        right: Box::new(Expr::IntLiteral {
            value: 2,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    })]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &first_typed_function(&typed_program).body[0] else {
        panic!("expected return statement");
    };

    assert_eq!(expr.ty, Type::Int);
    assert!(matches!(expr.kind, TypedExprKind::Binary { .. }));
}

#[test]
fn typed_shift_uses_promoted_left_operand_type() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::UnsignedInt,
            name_span: span(),
            name: "shift".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Return(Expr::Binary {
            op: BinaryOp::ShiftRight,
            op_span: span(),
            left: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            right: Box::new(Expr::Variable {
                name: "shift".to_string(),
                span: span(),
            }),
        }),
    ]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &first_typed_function(&typed_program).body[2] else {
        panic!("expected return statement");
    };

    let TypedExprKind::Binary { operand_ty, .. } = &expr.kind else {
        panic!("expected binary expression");
    };

    assert_eq!(expr.ty, Type::Int);
    assert_eq!(*operand_ty, Type::Int);
}

#[test]
fn typed_unary_negate_preserves_unsigned_operand_type() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::UnsignedInt,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Return(Expr::Unary {
            op: UnaryOp::Negate,
            op_span: span(),
            expr: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
        }),
    ]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &first_typed_function(&typed_program).body[1] else {
        panic!("expected return statement");
    };

    assert_eq!(expr.ty, Type::UnsignedInt);
    assert!(matches!(expr.kind, TypedExprKind::Unary { .. }));
}

#[test]
fn typed_unary_bitwise_not_preserves_unsigned_operand_type() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::UnsignedInt,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Return(Expr::Unary {
            op: UnaryOp::BitwiseNot,
            op_span: span(),
            expr: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
        }),
    ]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &first_typed_function(&typed_program).body[1] else {
        panic!("expected return statement");
    };

    assert_eq!(expr.ty, Type::UnsignedInt);
    assert!(matches!(expr.kind, TypedExprKind::Unary { .. }));
}

#[test]
fn typed_unary_logical_not_returns_int_for_unsigned_operand() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::UnsignedInt,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Return(Expr::Unary {
            op: UnaryOp::LogicalNot,
            op_span: span(),
            expr: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
        }),
    ]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &first_typed_function(&typed_program).body[1] else {
        panic!("expected return statement");
    };

    assert_eq!(expr.ty, Type::Int);
    assert!(matches!(expr.kind, TypedExprKind::Unary { .. }));
}

#[test]
fn typed_pointer_dereference_has_pointee_type() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "first".to_string(),
            params: vec![param_with_span(
                Type::Pointer(Box::new(Type::Char)),
                "buf",
                span(),
            )],
            body: vec![Statement::Return(Expr::Unary {
                op: UnaryOp::Dereference,
                op_span: span(),
                expr: Box::new(Expr::Variable {
                    name: "buf".to_string(),
                    span: span(),
                }),
            })],
        },
        function(
            "main",
            vec![Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })],
        ),
    ]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &first_typed_function(&typed_program).body[0] else {
        panic!("expected return statement");
    };

    assert_eq!(expr.ty, Type::Char);
    assert!(matches!(
        expr.kind,
        TypedExprKind::Unary {
            op: UnaryOp::Dereference,
            ..
        }
    ));
}

#[test]
fn accepts_assignment_through_pointer_dereference() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "store".to_string(),
            params: vec![param_with_span(
                Type::Pointer(Box::new(Type::Int)),
                "p",
                span(),
            )],
            body: vec![Statement::Return(Expr::Assign {
                target: Box::new(Expr::Unary {
                    op: UnaryOp::Dereference,
                    op_span: span(),
                    expr: Box::new(Expr::Variable {
                        name: "p".to_string(),
                        span: span(),
                    }),
                }),
                op_span: span(),
                value: Box::new(Expr::IntLiteral {
                    value: 3,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
            })],
        },
        function(
            "main",
            vec![Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })],
        ),
    ]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &first_typed_function(&typed_program).body[0] else {
        panic!("expected return statement");
    };
    let TypedExprKind::Assign { target, .. } = &expr.kind else {
        panic!("expected assignment expression");
    };

    assert_eq!(expr.ty, Type::Int);
    assert_eq!(target.ty, Type::Int);
    assert!(matches!(
        target.kind,
        TypedExprKind::Unary {
            op: UnaryOp::Dereference,
            ..
        }
    ));
}

#[test]
fn accepts_compound_assignment_through_pointer_dereference() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "add_to_pointed_value".to_string(),
            params: vec![param_with_span(
                Type::Pointer(Box::new(Type::Int)),
                "p",
                span(),
            )],
            body: vec![Statement::Return(Expr::CompoundAssign {
                target: Box::new(Expr::Unary {
                    op: UnaryOp::Dereference,
                    op_span: span(),
                    expr: Box::new(Expr::Variable {
                        name: "p".to_string(),
                        span: span(),
                    }),
                }),
                op: BinaryOp::Add,
                op_span: span(),
                value: Box::new(Expr::IntLiteral {
                    value: 3,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
            })],
        },
        function(
            "main",
            vec![Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })],
        ),
    ]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &first_typed_function(&typed_program).body[0] else {
        panic!("expected return statement");
    };
    let TypedExprKind::CompoundAssign {
        target, operand_ty, ..
    } = &expr.kind
    else {
        panic!("expected compound assignment expression");
    };

    assert_eq!(expr.ty, Type::Int);
    assert_eq!(target.ty, Type::Int);
    assert_eq!(*operand_ty, Type::Int);
}

#[test]
fn accepts_increment_through_pointer_dereference() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "increment_pointed_value".to_string(),
            params: vec![param_with_span(
                Type::Pointer(Box::new(Type::Int)),
                "p",
                span(),
            )],
            body: vec![Statement::Return(Expr::PostfixInc {
                expr: Box::new(Expr::Unary {
                    op: UnaryOp::Dereference,
                    op_span: span(),
                    expr: Box::new(Expr::Variable {
                        name: "p".to_string(),
                        span: span(),
                    }),
                }),
                op_span: span(),
            })],
        },
        function(
            "main",
            vec![Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })],
        ),
    ]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &first_typed_function(&typed_program).body[0] else {
        panic!("expected return statement");
    };
    let TypedExprKind::PostfixInc { expr: target, .. } = &expr.kind else {
        panic!("expected postfix increment expression");
    };

    assert_eq!(expr.ty, Type::Int);
    assert_eq!(target.ty, Type::Int);
}

#[test]
fn typed_pointer_arithmetic_has_pointer_operand_and_result_type() {
    let pointer_to_int = Type::Pointer(Box::new(Type::Int));
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: pointer_to_int.clone(),
            name_span: span(),
            name: "advance".to_string(),
            params: vec![param_with_span(pointer_to_int.clone(), "p", span())],
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::Add,
                op_span: span(),
                left: Box::new(Expr::Variable {
                    name: "p".to_string(),
                    span: span(),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 2,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
            })],
        },
        function(
            "main",
            vec![Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })],
        ),
    ]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &first_typed_function(&typed_program).body[0] else {
        panic!("expected return statement");
    };

    let TypedExprKind::Binary { operand_ty, .. } = &expr.kind else {
        panic!("expected binary expression");
    };

    assert_eq!(expr.ty, pointer_to_int);
    assert_eq!(*operand_ty, pointer_to_int);
}

#[test]
fn accepts_pointer_compound_assignment_by_integer() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Pointer(Box::new(Type::Char)),
            name_span: span(),
            name: "buf".to_string(),
            init: None,
        }]),
        Statement::ExprStatement(Expr::CompoundAssign {
            target: Box::new(Expr::Variable {
                name: "buf".to_string(),
                span: span(),
            }),
            op: BinaryOp::Add,
            op_span: span(),
            value: Box::new(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        }),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_dereference_of_non_pointer_expression() {
    let program = main_program(vec![Statement::Return(Expr::Unary {
        op: UnaryOp::Dereference,
        op_span: span(),
        expr: Box::new(Expr::IntLiteral {
            value: 1,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    })]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "cannot dereference non-pointer type 'Int'");
}

#[test]
fn rejects_assignment_through_non_pointer_dereference() {
    let op_span = span_from(10, 11);
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Return(Expr::Assign {
            target: Box::new(Expr::Unary {
                op: UnaryOp::Dereference,
                op_span,
                expr: Box::new(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                }),
            }),
            op_span: span(),
            value: Box::new(Expr::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.span, op_span);
    assert_eq!(err.message, "cannot dereference non-pointer type 'Int'");
}

#[test]
fn rejects_pointer_plus_pointer() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Pointer(Box::new(Type::Char)),
            name_span: span(),
            name: "left".to_string(),
            init: None,
        }]),
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Pointer(Box::new(Type::Char)),
            name_span: span(),
            name: "right".to_string(),
            init: None,
        }]),
        Statement::Return(Expr::Binary {
            op: BinaryOp::Add,
            op_span: span(),
            left: Box::new(Expr::Variable {
                name: "left".to_string(),
                span: span(),
            }),
            right: Box::new(Expr::Variable {
                name: "right".to_string(),
                span: span(),
            }),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert!(err.message.contains("invalid operands to binary operator"));
}

#[test]
fn rejects_assignment_between_different_pointer_types() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Pointer(Box::new(Type::Char)),
            name_span: span(),
            name: "chars".to_string(),
            init: None,
        }]),
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Pointer(Box::new(Type::Int)),
            name_span: span(),
            name: "ints".to_string(),
            init: None,
        }]),
        Statement::ExprStatement(Expr::Assign {
            target: Box::new(Expr::Variable {
                name: "chars".to_string(),
                span: span(),
            }),
            op_span: span(),
            value: Box::new(Expr::Variable {
                name: "ints".to_string(),
                span: span(),
            }),
        }),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "cannot assign value of type 'Pointer(Int)' to variable of type 'Pointer(Char)'"
    );
}

#[test]
fn typed_char_compound_assignment_has_target_type() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Char,
            name_span: span(),
            name: "c".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Return(Expr::CompoundAssign {
            target: Box::new(Expr::Variable {
                name: "c".to_string(),
                span: span(),
            }),
            op: BinaryOp::Add,
            op_span: span(),
            value: Box::new(Expr::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        }),
    ]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &first_typed_function(&typed_program).body[1] else {
        panic!("expected return statement");
    };

    assert_eq!(expr.ty, Type::Char);
    assert!(matches!(expr.kind, TypedExprKind::CompoundAssign { .. }));
}

#[test]
fn typed_postfix_increment_preserves_postfix_kind() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Return(Expr::PostfixInc {
            expr: Box::new(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            }),
            op_span: span(),
        }),
    ]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &first_typed_function(&typed_program).body[1] else {
        panic!("expected return statement");
    };

    assert_eq!(expr.ty, Type::Int);
    assert!(matches!(expr.kind, TypedExprKind::PostfixInc { .. }));
}

#[test]
fn typed_call_preserves_typed_arguments() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "id".to_string(),
            params: vec![param_with_span(Type::Char, "c", span())],
            body: vec![Statement::Return(Expr::Variable {
                name: "c".to_string(),
                span: span(),
            })],
        },
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Call {
                name_span: span(),
                name: "id".to_string(),
                args: vec![Expr::IntLiteral {
                    value: 65,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }],
            })],
        },
    ]);

    let typed_program = check(&program).expect("semantic check should succeed");

    let TypedStatement::Return(expr) = &typed_function_at(&typed_program, 1).body[0] else {
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
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
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
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
            right: Box::new(Expr::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        }),
        op_span,
        value: Box::new(Expr::IntLiteral {
            value: 3,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    })]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.span, op_span);
    assert_eq!(err.message, "cannot assign to non-lvalue expression");
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
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
            right: Box::new(Expr::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        }),
        op: BinaryOp::Add,
        op_span,
        value: Box::new(Expr::IntLiteral {
            value: 3,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    })]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.span, op_span);
    assert_eq!(err.message, "cannot assign to non-lvalue expression");
}

#[test]
fn accepts_prefix_and_postfix_increment_decrement() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
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
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
            right: Box::new(Expr::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        }),
        op_span,
    })]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.span, op_span);
    assert_eq!(err.message, "cannot assign to non-lvalue expression");
}

#[test]
fn accepts_initializer_using_earlier_local() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "y".to_string(),
            init: Some(Initializer::Expr(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            })),
        }]),
        Statement::Return(Expr::Variable {
            name: "y".to_string(),
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_multiple_local_declarators_left_to_right() {
    let program = main_program(vec![
        Statement::Decl(vec![
            rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "a".to_string(),
                init: Some(Initializer::Expr(Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })),
            },
            rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "b".to_string(),
                init: Some(Initializer::Expr(Expr::Binary {
                    op: BinaryOp::Add,
                    op_span: span(),
                    left: Box::new(Expr::Variable {
                        name: "a".to_string(),
                        span: span(),
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 1,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    }),
                })),
            },
        ]),
        Statement::Return(Expr::Variable {
            name: "b".to_string(),
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_initializer_using_declared_name_itself() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            })),
        }]),
        Statement::Return(Expr::Variable {
            name: "x".to_string(),
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_local_array_declaration_without_value_use() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Array {
                element: Box::new(Type::Char),
                len: 3,
            },
            name_span: span(),
            name: "buf".to_string(),
            init: None,
        }]),
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Return(Expr::Variable {
            name: "x".to_string(),
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn typed_array_variable_expression_decays_to_pointer_argument() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "takes_char_pointer".to_string(),
            params: vec![param_with_span(
                Type::Pointer(Box::new(Type::Char)),
                "p",
                span(),
            )],
            body: vec![Statement::Return(Expr::Unary {
                op: UnaryOp::Dereference,
                op_span: span(),
                expr: Box::new(Expr::Variable {
                    name: "p".to_string(),
                    span: span(),
                }),
            })],
        },
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::Decl(vec![rivet::ast::LocalDecl {
                    ty: Type::Array {
                        element: Box::new(Type::Char),
                        len: 3,
                    },
                    name_span: span(),
                    name: "buf".to_string(),
                    init: None,
                }]),
                Statement::Return(Expr::Call {
                    name_span: span(),
                    name: "takes_char_pointer".to_string(),
                    args: vec![Expr::Variable {
                        name: "buf".to_string(),
                        span: span(),
                    }],
                }),
            ],
        },
    ]);

    let typed_program = check(&program).expect("semantic check should succeed");
    let TypedStatement::Return(expr) = &typed_function_at(&typed_program, 1).body[1] else {
        panic!("expected return statement");
    };
    let TypedExprKind::Call { args, .. } = &expr.kind else {
        panic!("expected call expression");
    };

    assert_eq!(args[0].ty, Type::Pointer(Box::new(Type::Char)));
}

#[test]
fn accepts_local_array_initializer_list() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Array {
                element: Box::new(Type::Char),
                len: 3,
            },
            name_span: span(),
            name: "buf".to_string(),
            init: Some(Initializer::List(vec![
                Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                },
                Expr::IntLiteral {
                    value: 2,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                },
                Expr::IntLiteral {
                    value: 3,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                },
            ])),
        }]),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_local_array_initializer_list_with_shorter_length() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Array {
                element: Box::new(Type::Char),
                len: 3,
            },
            name_span: span(),
            name: "buf".to_string(),
            init: Some(Initializer::List(vec![
                Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                },
                Expr::IntLiteral {
                    value: 2,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                },
            ])),
        }]),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_empty_local_array_initializer_list() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Array {
                element: Box::new(Type::Int),
                len: 2,
            },
            name_span: span(),
            name: "nums".to_string(),
            init: Some(Initializer::List(vec![])),
        }]),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_array_initializer_list_with_larger_length() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Array {
                element: Box::new(Type::Char),
                len: 3,
            },
            name_span: span(),
            name: "buf".to_string(),
            init: Some(Initializer::List(vec![
                Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                },
                Expr::IntLiteral {
                    value: 2,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                },
                Expr::IntLiteral {
                    value: 3,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                },
                Expr::IntLiteral {
                    value: 4,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                },
            ])),
        }]),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "array initializer list must have <= '3' elements, has '4' elements"
    );
}

#[test]
fn rejects_array_initializer_expression() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Array {
                element: Box::new(Type::Char),
                len: 3,
            },
            name_span: span(),
            name: "buf".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "array must be initialized with list");
}

#[test]
fn rejects_scalar_initializer_list() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::List(vec![Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }])),
        }]),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "cannot initialize scalar type 'Int' with list");
}

#[test]
fn rejects_duplicate_local_declaration() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
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
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        }),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
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
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "y".to_string(),
            init: Some(Initializer::Expr(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            })),
        }]),
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Return(Expr::Variable {
            name: "y".to_string(),
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared local variable 'x'");
}

#[test]
fn rejects_multiple_local_declarator_initializer_using_later_name() {
    let program = main_program(vec![
        Statement::Decl(vec![
            rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "a".to_string(),
                init: Some(Initializer::Expr(Expr::Variable {
                    name: "b".to_string(),
                    span: span(),
                })),
            },
            rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "b".to_string(),
                init: Some(Initializer::Expr(Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })),
            },
        ]),
        Statement::Return(Expr::Variable {
            name: "a".to_string(),
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared local variable 'b'");
}

#[test]
fn rejects_undeclared_local_inside_nested_expression() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
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
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Char,
            name_span: span(),
            name: "id".to_string(),
            params: vec![param_with_span(Type::Char, "x", span())],
            body: vec![Statement::Return(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            })],
        },
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![param_with_span(Type::Char, "c", span())],
            body: vec![
                Statement::Decl(vec![rivet::ast::LocalDecl {
                    ty: Type::Char,
                    name_span: span(),
                    name: "result".to_string(),
                    init: Some(Initializer::Expr(Expr::Call {
                        name_span: span(),
                        name: "id".to_string(),
                        args: vec![Expr::Variable {
                            name: "c".to_string(),
                            span: span(),
                        }],
                    })),
                }]),
                Statement::Return(Expr::IntLiteral {
                    value: 0,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
            ],
        },
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_char_initializer_from_int_literal() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Char,
            name_span: span(),
            name: "c".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_char_initializer_from_int_expression() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Char,
            name_span: span(),
            name: "c".to_string(),
            init: Some(Initializer::Expr(Expr::Binary {
                op: BinaryOp::Add,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 2,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
            })),
        }]),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_char_assignment_from_int_expression() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Char,
            name_span: span(),
            name: "c".to_string(),
            init: None,
        }]),
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "i".to_string(),
            init: None,
        }]),
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
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_int_initializer_from_char_expression() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Char,
            name_span: span(),
            name: "c".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 65,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "i".to_string(),
            init: Some(Initializer::Expr(Expr::Variable {
                name: "c".to_string(),
                span: span(),
            })),
        }]),
        Statement::Return(Expr::Variable {
            name: "i".to_string(),
            span: span(),
        }),
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_char_argument_from_int_expression() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "takes_char".to_string(),
            params: vec![param_with_span(Type::Char, "x", span())],
            body: vec![Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })],
        },
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::Decl(vec![rivet::ast::LocalDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "i".to_string(),
                    init: None,
                }]),
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
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_int_argument_from_char_expression() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "takes_int".to_string(),
            params: vec![param_with_span(Type::Int, "x", span())],
            body: vec![Statement::Return(Expr::Variable {
                name: "x".to_string(),
                span: span(),
            })],
        },
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::Decl(vec![rivet::ast::LocalDecl {
                    ty: Type::Char,
                    name_span: span(),
                    name: "c".to_string(),
                    init: Some(Initializer::Expr(Expr::IntLiteral {
                        value: 65,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    })),
                }]),
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
    ]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_char_return_from_int_expression() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Char,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![Statement::Return(Expr::Binary {
            op: BinaryOp::Add,
            op_span: span(),
            left: Box::new(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
            right: Box::new(Expr::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        })],
    }]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_int_return_from_char_expression() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![param_with_span(Type::Char, "c", span())],
        body: vec![Statement::Return(Expr::Variable {
            name: "c".to_string(),
            span: span(),
        })],
    }]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn accepts_binary_expression_between_char_and_int() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Char,
            name_span: span(),
            name: "c".to_string(),
            init: None,
        }]),
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "i".to_string(),
            init: None,
        }]),
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
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
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
        Statement::Block(vec![Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }])]),
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
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
        Statement::Block(vec![
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "x".to_string(),
                init: Some(Initializer::Expr(Expr::IntLiteral {
                    value: 2,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })),
            }]),
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
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "x".to_string(),
                init: Some(Initializer::Expr(Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })),
            }]),
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "x".to_string(),
                init: Some(Initializer::Expr(Expr::IntLiteral {
                    value: 2,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })),
            }]),
        ]),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "duplicate local variable 'x'");
}

#[test]
fn accepts_if_else_with_locals_in_branches() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
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
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
            },
            then_branch: Box::new(Statement::Block(vec![
                Statement::Decl(vec![rivet::ast::LocalDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "y".to_string(),
                    init: Some(Initializer::Expr(Expr::Variable {
                        name: "x".to_string(),
                        span: span(),
                    })),
                }]),
                Statement::Return(Expr::Variable {
                    name: "y".to_string(),
                    span: span(),
                }),
            ])),
            else_branch: Some(Box::new(Statement::Block(vec![
                Statement::Decl(vec![rivet::ast::LocalDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "z".to_string(),
                    init: Some(Initializer::Expr(Expr::Variable {
                        name: "x".to_string(),
                        span: span(),
                    })),
                }]),
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
        Statement::Decl(vec![rivet::ast::LocalDecl {
            ty: Type::Int,
            name_span: span(),
            name: "x".to_string(),
            init: Some(Initializer::Expr(Expr::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        }]),
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
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
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
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            },
            body: Box::new(Statement::Block(vec![
                Statement::Continue { span: span() },
                Statement::Break { span: span() },
            ])),
        },
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
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
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            },
        },
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
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
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            },
            body: Box::new(Statement::If {
                cond: Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                },
                then_branch: Box::new(Statement::Break { span: span() }),
                else_branch: None,
            }),
        },
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
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
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
        },
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
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
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "undeclared local variable 'x'");
}

#[test]
fn rejects_indexing_non_array_non_pointer() {
    let program = main_program(vec![Statement::Return(Expr::Index {
        base: Box::new(Expr::IntLiteral {
            value: 42,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
        index: Box::new(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
        span: span(),
    })]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "cannot index expression of type 'Int'");
}

#[test]
fn rejects_non_integer_array_index() {
    let program = program_with_functions(vec![FunctionDef {
        name: "main".to_string(),
        name_span: span(),
        return_type: Type::Int,
        params: vec![],
        body: vec![
            Statement::Decl(vec![rivet::ast::LocalDecl {
                name: "buf".to_string(),
                name_span: span(),
                ty: Type::Array {
                    element: Box::new(Type::Char),
                    len: 3,
                },
                init: None,
            }]),
            Statement::Decl(vec![rivet::ast::LocalDecl {
                name: "p".to_string(),
                name_span: span(),
                ty: Type::Pointer(Box::new(Type::Int)),
                init: None,
            }]),
            Statement::Return(Expr::Index {
                base: Box::new(Expr::Variable {
                    name: "buf".to_string(),
                    span: span(),
                }),
                index: Box::new(Expr::Variable {
                    name: "p".to_string(),
                    span: span(),
                }),
                span: span(),
            }),
        ],
    }]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "array index must be integer type, found 'Pointer(Int)'"
    );
}

#[test]
fn rejects_assigning_to_array_variable() {
    let program = main_program(vec![
        Statement::Decl(vec![rivet::ast::LocalDecl {
            name: "buf".to_string(),
            ty: Type::Array {
                element: Box::new(Type::Char),
                len: 3,
            },
            init: None,
            name_span: span(),
        }]),
        Statement::ExprStatement(Expr::Assign {
            target: Box::new(Expr::Variable {
                name: "buf".to_string(),
                span: span(),
            }),
            op_span: span(),
            value: Box::new(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        }),
        Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    ]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "cannot assign to array expression");
}

#[test]
fn typed_integer_literal_suffixes_choose_literal_type() {
    let cases = [
        (IntLiteralSuffix::None, Type::Int),
        (IntLiteralSuffix::Unsigned, Type::UnsignedInt),
        (IntLiteralSuffix::Long, Type::Long),
        (IntLiteralSuffix::UnsignedLong, Type::UnsignedLong),
    ];

    for (suffix, expected_ty) in cases {
        let program = main_program(vec![Statement::Return(Expr::IntLiteral {
            value: 1,
            suffix,
            base: IntLiteralBase::Decimal,
            span: span(),
        })]);

        let typed_program = check(&program).expect("semantic check should succeed");
        let TypedStatement::Return(expr) = &first_typed_function(&typed_program).body[0] else {
            panic!("expected typed return statement");
        };

        assert_eq!(expr.ty, expected_ty);
    }
}

#[test]
fn accepts_integer_literal_suffix_boundary_values() {
    let cases = [
        (i32::MAX as u64, IntLiteralSuffix::None),
        (u64::from(u32::MAX), IntLiteralSuffix::Unsigned),
        (i32::MAX as u64, IntLiteralSuffix::Long),
        (u64::from(u32::MAX), IntLiteralSuffix::UnsignedLong),
    ];

    for (value, suffix) in cases {
        let program = main_program(vec![Statement::Return(Expr::IntLiteral {
            value,
            suffix,
            base: IntLiteralBase::Decimal,
            span: span(),
        })]);

        check(&program).expect("semantic check should succeed");
    }
}

#[test]
fn rejects_integer_literals_that_do_not_fit_suffix_type() {
    let cases = [
        (
            (i32::MAX as u64) + 1,
            IntLiteralSuffix::None,
            "integer literal '2147483648' is too large for type 'Int'",
        ),
        (
            u64::from(u32::MAX) + 1,
            IntLiteralSuffix::Unsigned,
            "integer literal '4294967296' is too large for type 'UnsignedInt'",
        ),
        (
            (i32::MAX as u64) + 1,
            IntLiteralSuffix::Long,
            "integer literal '2147483648' is too large for type 'Long'",
        ),
        (
            u64::from(u32::MAX) + 1,
            IntLiteralSuffix::UnsignedLong,
            "integer literal '4294967296' is too large for type 'UnsignedLong'",
        ),
    ];

    for (value, suffix, expected_message) in cases {
        let program = main_program(vec![Statement::Return(Expr::IntLiteral {
            value,
            suffix,
            base: IntLiteralBase::Decimal,
            span: span(),
        })]);

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(err.message, expected_message);
    }
}

#[test]
fn typed_unsuffixed_hex_literals_use_hex_candidate_types() {
    let cases = [
        (0x7fff_ffff, Type::Int),
        (0x8000_0000, Type::UnsignedInt),
        (0xffff_ffff, Type::UnsignedInt),
    ];

    for (value, expected_ty) in cases {
        let program = main_program(vec![Statement::Return(Expr::IntLiteral {
            value,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Hex,
            span: span(),
        })]);

        let typed_program = check(&program).expect("semantic check should succeed");
        let TypedStatement::Return(expr) = &first_typed_function(&typed_program).body[0] else {
            panic!("expected typed return statement");
        };

        assert_eq!(expr.ty, expected_ty);
    }
}

#[test]
fn rejects_unsuffixed_hex_literals_that_do_not_fit_current_integer_types() {
    let program = main_program(vec![Statement::Return(Expr::IntLiteral {
        value: u64::from(u32::MAX) + 1,
        suffix: IntLiteralSuffix::None,
        base: IntLiteralBase::Hex,
        span: span(),
    })]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(
        err.message,
        "integer literal '4294967296' is too large for type 'Int'"
    );
}

#[test]
fn typed_scalar_cast_has_target_type() {
    let program = main_program(vec![Statement::Return(Expr::Cast {
        ty: Type::UnsignedLong,
        span: span(),
        expr: Box::new(Expr::IntLiteral {
            value: 1,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        }),
    })]);

    let typed_program = check(&program).expect("semantic check should succeed");
    let TypedStatement::Return(expr) = &first_typed_function(&typed_program).body[0] else {
        panic!("expected typed return statement");
    };

    assert_eq!(expr.ty, Type::UnsignedLong);

    let TypedExprKind::Cast {
        target_ty,
        expr: inner,
        ..
    } = &expr.kind
    else {
        panic!("expected typed cast expression");
    };

    assert_eq!(*target_ty, Type::UnsignedLong);
    assert_eq!(inner.ty, Type::Int);
}

#[test]
fn accepts_casts_between_integer_types() {
    let program = main_program(vec![Statement::Return(Expr::Cast {
        ty: Type::UnsignedChar,
        span: span(),
        expr: Box::new(Expr::Cast {
            ty: Type::SignedChar,
            span: span(),
            expr: Box::new(Expr::IntLiteral {
                value: 255,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        }),
    })]);

    check(&program).expect("semantic check should succeed");
}

#[test]
fn rejects_cast_from_pointer_to_integer() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Pointer(Box::new(Type::Char)),
                name: "p".to_string(),
                name_span: span(),
                init: Some(Initializer::Expr(Expr::IntLiteral {
                    value: 0,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })),
            }]),
            Statement::Return(Expr::Cast {
                ty: Type::Int,
                span: span(),
                expr: Box::new(Expr::Variable {
                    name: "p".to_string(),
                    span: span(),
                }),
            }),
        ],
    }]);

    let err = check(&program).expect_err("semantic check should fail");

    assert_eq!(err.message, "cannot cast type 'Pointer(Char)' to 'Int'");
}
