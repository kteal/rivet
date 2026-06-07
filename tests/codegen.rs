mod common;

use common::{param, param_with_span, span};
use rivet::ast::{BinaryOp, Expr, Function, Program, Statement, Type, UnaryOp};
use rivet::codegen::{CodegenTarget, generate};
use rivet::sema::check;

fn generate_checked(program: &Program) -> String {
    let typed_program = check(program).expect("semantic check should succeed");
    generate(&typed_program, CodegenTarget::Rv32)
}

fn generate_raw_with_codegen(program: &Program) -> String {
    generate_checked(program)
}

fn generate_with_codegen(program: &Program) -> String {
    generate_raw_with_codegen(program).replace("    j main_end\nmain_end:\n", "")
}

fn empty_main_function() -> Function {
    Function {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![Statement::Return(Expr::IntLiteral {
            value: 0,
            span: span(),
        })],
    }
}

fn add_function() -> Function {
    Function {
        return_type: Type::Int,
        name_span: span(),
        name: "add".to_string(),
        params: vec![param("x"), param("y")],
        body: vec![Statement::Return(Expr::Binary {
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
    }
}

fn right_function() -> Function {
    Function {
        return_type: Type::Int,
        name_span: span(),
        name: "right".to_string(),
        params: vec![],
        body: vec![Statement::Return(Expr::IntLiteral {
            value: 1,
            span: span(),
        })],
    }
}

#[test]
fn generates_multiple_functions() {
    let program = Program {
        functions: vec![
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "helper".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::IntLiteral {
                    value: 3,
                    span: span(),
                })],
            },
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                })],
            },
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(".globl helper\nhelper:\n"));
    assert!(asm.contains(".globl main\nmain:\n"));
    assert!(asm.contains("    j helper_end\nhelper_end:\n"));
    assert!(asm.contains("    j main_end\nmain_end:\n"));
}

#[test]
fn resets_local_offsets_between_functions() {
    let program = Program {
        functions: vec![
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "first".to_string(),
                params: vec![],
                body: vec![
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
            },
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert_eq!(asm.matches("sw a0, -12(s0)").count(), 2);
    assert_eq!(asm.matches("lw a0, -12(s0)").count(), 2);
}

#[test]
fn computes_frame_layout_per_function() {
    let program = Program {
        functions: vec![
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "helper".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::IntLiteral {
                    value: 1,
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
                        name: "a".to_string(),
                        init: Some(Expr::IntLiteral {
                            value: 1,
                            span: span(),
                        }),
                    },
                    Statement::VarDecl {
                        ty: Type::Int,
                        name_span: span(),
                        name: "b".to_string(),
                        init: Some(Expr::IntLiteral {
                            value: 2,
                            span: span(),
                        }),
                    },
                    Statement::VarDecl {
                        ty: Type::Int,
                        name_span: span(),
                        name: "c".to_string(),
                        init: Some(Expr::IntLiteral {
                            value: 3,
                            span: span(),
                        }),
                    },
                    Statement::Return(Expr::Variable {
                        name: "c".to_string(),
                        span: span(),
                    }),
                ],
            },
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("helper:\n    addi sp, sp, -16\n"));
    assert!(asm.contains("main:\n    addi sp, sp, -32\n"));
}

#[test]
fn generates_zero_argument_function_call() {
    let program = Program {
        functions: vec![
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "helper".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::IntLiteral {
                    value: 3,
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
                    name: "helper".to_string(),
                    args: vec![],
                })],
            },
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("main:\n"));
    assert!(asm.contains("    call helper\n    j main_end\n"));
}

#[test]
fn uses_call_result_as_expression_operand() {
    let program = Program {
        functions: vec![
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "helper".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::IntLiteral {
                    value: 3,
                    span: span(),
                })],
            },
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Add,
                    op_span: span(),
                    left: Box::new(Expr::Call {
                        name_span: span(),
                        name: "helper".to_string(),
                        args: vec![],
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 2,
                        span: span(),
                    }),
                })],
            },
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    call helper\n"));
    assert!(asm.contains("    add a0, t0, a0\n"));
}

#[test]
fn stores_single_parameter_in_function_frame() {
    let program = Program {
        functions: vec![
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "id".to_string(),
                params: vec![param("x")],
                body: vec![Statement::Return(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                })],
            },
            empty_main_function(),
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "id:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    sw a0, -12(s0)\n    lw a0, -12(s0)\n"
    ));
}

#[test]
fn stores_multiple_parameters_in_function_frame() {
    let program = Program {
        functions: vec![add_function(), empty_main_function()],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    sw a0, -12(s0)\n"));
    assert!(asm.contains("    sw a1, -16(s0)\n"));
    assert!(asm.contains("    lw a0, -12(s0)\n"));
    assert!(asm.contains("    lw a0, -16(s0)\n"));
}

#[test]
fn generates_function_call_with_arguments() {
    let program = Program {
        functions: vec![
            add_function(),
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Call {
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
            },
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    li a0, 1\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 2\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    lw a1, 0(sp)\n    addi sp, sp, 4\n    lw a0, 0(sp)\n    addi sp, sp, 4\n    call add\n"
    ));
}

#[test]
fn generates_function_call_with_expression_arguments() {
    let program = Program {
        functions: vec![
            add_function(),
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Call {
                    name_span: span(),
                    name: "add".to_string(),
                    args: vec![
                        Expr::Binary {
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
                        },
                        Expr::Binary {
                            op: BinaryOp::Add,
                            op_span: span(),
                            left: Box::new(Expr::IntLiteral {
                                value: 3,
                                span: span(),
                            }),
                            right: Box::new(Expr::IntLiteral {
                                value: 4,
                                span: span(),
                            }),
                        },
                    ],
                })],
            },
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    lw a1, 0(sp)\n    addi sp, sp, 4\n    lw a0, 0(sp)\n"));
    assert!(asm.contains("    call add\n"));
}

#[test]
fn emits_nothing_for_empty_statement() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::Empty,
                Statement::Return(Expr::IntLiteral {
                    value: 7,
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 7\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn emits_expression_statement_and_discards_result() {
    let program = Program {
        functions: vec![
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "helper".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::IntLiteral {
                    value: 3,
                    span: span(),
                })],
            },
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![
                    Statement::ExprStatement(Expr::Call {
                        name_span: span(),
                        name: "helper".to_string(),
                        args: vec![],
                    }),
                    Statement::Return(Expr::IntLiteral {
                        value: 7,
                        span: span(),
                    }),
                ],
            },
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("main:\n"));
    assert!(asm.contains("    call helper\n    li a0, 7\n"));
}

#[test]
fn generates_return_jump_to_shared_epilogue() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::IntLiteral {
                value: 42,
                span: span(),
            })],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    j main_end\nmain_end:\n"));
}

#[test]
fn basic_codegen() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::IntLiteral {
                value: 42,
                span: span(),
            })],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 42\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_binary_add() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
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

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 1\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 2\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    add a0, t0, a0\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_binary_subtract() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::Subtract,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 5,
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

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 5\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 2\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    sub a0, t0, a0\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_binary_multiply() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::Multiply,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 2,
                    span: span(),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 3,
                    span: span(),
                }),
            })],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 2\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 3\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    mul a0, t0, a0\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_binary_divide() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::Divide,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 8,
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

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 8\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 2\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    div a0, t0, a0\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_binary_remainder() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::Remainder,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 8,
                    span: span(),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 3,
                    span: span(),
                }),
            })],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 8\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 3\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    rem a0, t0, a0\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_binary_equal() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::Equal,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 5,
                    span: span(),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 5,
                    span: span(),
                }),
            })],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 5\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 5\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    xor a0, t0, a0\n    seqz a0, a0\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_binary_not_equal() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::NotEqual,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 5,
                    span: span(),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 3,
                    span: span(),
                }),
            })],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 5\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 3\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    xor a0, t0, a0\n    snez a0, a0\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_binary_less() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::Less,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 2,
                    span: span(),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 5,
                    span: span(),
                }),
            })],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 2\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 5\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    slt a0, t0, a0\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_binary_less_equal() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::LessEqual,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 5,
                    span: span(),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 5,
                    span: span(),
                }),
            })],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 5\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 5\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    slt a0, a0, t0\n    xori a0, a0, 1\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_binary_greater() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::Greater,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 5,
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

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 5\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 2\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    slt a0, a0, t0\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_binary_greater_equal() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::GreaterEqual,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 5,
                    span: span(),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 5,
                    span: span(),
                }),
            })],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 5\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 5\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    slt a0, t0, a0\n    xori a0, a0, 1\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_unary_negation() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Unary {
                op: UnaryOp::Negate,
                op_span: span(),
                expr: Box::new(Expr::IntLiteral {
                    value: 5,
                    span: span(),
                }),
            })],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 5\n    neg a0, a0\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_logical_not() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Unary {
                op: UnaryOp::LogicalNot,
                op_span: span(),
                expr: Box::new(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                }),
            })],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 0\n    seqz a0, a0\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_logical_and_with_short_circuit_branch() {
    let program = Program {
        functions: vec![
            right_function(),
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::LogicalAnd,
                    op_span: span(),
                    left: Box::new(Expr::IntLiteral {
                        value: 0,
                        span: span(),
                    }),
                    right: Box::new(Expr::Call {
                        name_span: span(),
                        name: "right".to_string(),
                        args: vec![],
                    }),
                })],
            },
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("logical_and_false_"));
    assert!(asm.contains("logical_and_end_"));
    assert!(asm.contains("    beqz a0, logical_and_false_"));
    assert!(asm.contains("    snez a0, a0"));
    assert!(
        asm.find("    beqz a0, logical_and_false_").unwrap() < asm.find("    call right").unwrap(),
        "logical && should branch before emitting the right operand"
    );
}

#[test]
fn generates_logical_or_with_short_circuit_branch() {
    let program = Program {
        functions: vec![
            right_function(),
            Function {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::LogicalOr,
                    op_span: span(),
                    left: Box::new(Expr::IntLiteral {
                        value: 1,
                        span: span(),
                    }),
                    right: Box::new(Expr::Call {
                        name_span: span(),
                        name: "right".to_string(),
                        args: vec![],
                    }),
                })],
            },
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("logical_or_true_"));
    assert!(asm.contains("logical_or_end_"));
    assert!(asm.contains("    bnez a0, logical_or_true_"));
    assert!(asm.contains("    snez a0, a0"));
    assert!(
        asm.find("    bnez a0, logical_or_true_").unwrap() < asm.find("    call right").unwrap(),
        "logical || should branch before emitting the right operand"
    );
}

#[test]
fn generates_bitwise_not() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Unary {
                op: UnaryOp::BitwiseNot,
                op_span: span(),
                expr: Box::new(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                }),
            })],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 0\n    not a0, a0\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_nested_expression_with_stack_temporaries() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
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
                right: Box::new(Expr::Binary {
                    op: BinaryOp::Multiply,
                    op_span: span(),
                    left: Box::new(Expr::IntLiteral {
                        value: 2,
                        span: span(),
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 3,
                        span: span(),
                    }),
                }),
            })],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 1\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 2\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 3\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    mul a0, t0, a0\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    add a0, t0, a0\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_single_local_variable() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::VarDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "x".to_string(),
                    init: Some(Expr::IntLiteral {
                        value: 5,
                        span: span(),
                    }),
                },
                Statement::Return(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 5\n    sw a0, -12(s0)\n    lw a0, -12(s0)\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_local_variable_without_initializer() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
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
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 3\n    sw a0, -12(s0)\n    lw a0, -12(s0)\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn narrows_char_local_initializer() {
    let program = Program {
        functions: vec![Function {
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
                        value: 300,
                        span: span(),
                    }),
                },
                Statement::Return(Expr::Variable {
                    name: "c".to_string(),
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    li a0, 300\n    andi a0, a0, 255\n    sb a0, -12(s0)\n"));
}

#[test]
fn loads_char_local_with_unsigned_byte_load() {
    let program = Program {
        functions: vec![Function {
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
                        value: 255,
                        span: span(),
                    }),
                },
                Statement::Return(Expr::Variable {
                    name: "c".to_string(),
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    sb a0, -12(s0)\n    lbu a0, -12(s0)\n"));
}

#[test]
fn narrows_char_assignment() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::VarDecl {
                    ty: Type::Char,
                    name_span: span(),
                    name: "c".to_string(),
                    init: None,
                },
                Statement::ExprStatement(Expr::Assign {
                    op_span: span(),
                    target: Box::new(Expr::Variable {
                        name: "c".to_string(),
                        span: span(),
                    }),
                    value: Box::new(Expr::IntLiteral {
                        value: 300,
                        span: span(),
                    }),
                }),
                Statement::Return(Expr::Variable {
                    name: "c".to_string(),
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    li a0, 300\n    andi a0, a0, 255\n    sb a0, -12(s0)\n"));
}

#[test]
fn generates_compound_assignment() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
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
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    lw a0, -12(s0)\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 4\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    add a0, t0, a0\n    sw a0, -12(s0)\n"
    ));
}

#[test]
fn generates_compound_assignment_expression_result() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
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
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    lw a0, -12(s0)\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 4\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    add a0, t0, a0\n    sw a0, -12(s0)\n    j main_end\n"
    ));
}

#[test]
fn narrows_char_compound_assignment() {
    let program = Program {
        functions: vec![Function {
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
                        value: 250,
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
                        value: 10,
                        span: span(),
                    }),
                }),
                Statement::Return(Expr::Variable {
                    name: "c".to_string(),
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    lbu a0, -12(s0)\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 10\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    add a0, t0, a0\n    andi a0, a0, 255\n    sb a0, -12(s0)\n"
    ));
}

#[test]
fn narrows_char_return_value() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Char,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::IntLiteral {
                value: 300,
                span: span(),
            })],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    li a0, 300\n    andi a0, a0, 255\n    j main_end\n"));
}

#[test]
fn narrows_char_parameter_on_function_entry() {
    let program = Program {
        functions: vec![
            Function {
                return_type: Type::Int,
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
                params: vec![],
                body: vec![Statement::Return(Expr::Call {
                    name_span: span(),
                    name: "id".to_string(),
                    args: vec![Expr::IntLiteral {
                        value: 300,
                        span: span(),
                    }],
                })],
            },
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("id:\n"));
    assert!(asm.contains("    andi a0, a0, 255\n    sb a0, -12(s0)\n"));
}

#[test]
fn generates_assignment_expression_result() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
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
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 3\n    sw a0, -12(s0)\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_prefix_increment_expression_result() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::VarDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "x".to_string(),
                    init: Some(Expr::IntLiteral {
                        value: 1,
                        span: span(),
                    }),
                },
                Statement::Return(Expr::PrefixInc {
                    expr: Box::new(Expr::Variable {
                        name: "x".to_string(),
                        span: span(),
                    }),
                    op_span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 1\n    sw a0, -12(s0)\n    lw a0, -12(s0)\n    addi a0, a0, 1\n    sw a0, -12(s0)\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_postfix_increment_expression_result() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
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
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 1\n    sw a0, -12(s0)\n    lw a0, -12(s0)\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    addi a0, a0, 1\n    sw a0, -12(s0)\n    lw a0, 0(sp)\n    addi sp, sp, 4\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn narrows_char_increment_store() {
    let program = Program {
        functions: vec![Function {
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
                        value: 255,
                        span: span(),
                    }),
                },
                Statement::ExprStatement(Expr::PrefixInc {
                    expr: Box::new(Expr::Variable {
                        name: "c".to_string(),
                        span: span(),
                    }),
                    op_span: span(),
                }),
                Statement::Return(Expr::Variable {
                    name: "c".to_string(),
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    lbu a0, -12(s0)\n    addi a0, a0, 1\n    andi a0, a0, 255\n    sb a0, -12(s0)\n"
    ));
}

#[test]
fn generates_chained_assignment_expression_right_associative() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::VarDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "x".to_string(),
                    init: None,
                },
                Statement::VarDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "y".to_string(),
                    init: None,
                },
                Statement::ExprStatement(Expr::Assign {
                    op_span: span(),
                    target: Box::new(Expr::Variable {
                        name: "x".to_string(),
                        span: span(),
                    }),
                    value: Box::new(Expr::Assign {
                        op_span: span(),
                        target: Box::new(Expr::Variable {
                            name: "y".to_string(),
                            span: span(),
                        }),
                        value: Box::new(Expr::IntLiteral {
                            value: 4,
                            span: span(),
                        }),
                    }),
                }),
                Statement::Return(Expr::Binary {
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
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    li a0, 4\n    sw a0, -16(s0)\n    sw a0, -12(s0)\n"));
}

#[test]
fn generates_multiple_local_variables() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
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
                    init: Some(Expr::Binary {
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
                },
                Statement::VarDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "z".to_string(),
                    init: Some(Expr::Binary {
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
                    }),
                },
                Statement::Return(Expr::Variable {
                    name: "z".to_string(),
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -32\n    sw ra, 28(sp)\n    sw s0, 24(sp)\n    addi s0, sp, 32\n    li a0, 1\n    sw a0, -12(s0)\n    lw a0, -12(s0)\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 2\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    add a0, t0, a0\n    sw a0, -16(s0)\n    lw a0, -12(s0)\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    lw a0, -16(s0)\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    add a0, t0, a0\n    sw a0, -20(s0)\n    lw a0, -20(s0)\n    lw ra, 28(sp)\n    lw s0, 24(sp)\n    addi sp, sp, 32\n    ret\n"
    );
}

#[test]
fn generates_shadowed_local_in_nested_block() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
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
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 1\n    sw a0, -12(s0)\n    li a0, 2\n    sw a0, -16(s0)\n    lw a0, -16(s0)\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_if_without_else() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::If {
                    cond: Expr::IntLiteral {
                        value: 1,
                        span: span(),
                    },
                    then_branch: Box::new(Statement::Return(Expr::IntLiteral {
                        value: 2,
                        span: span(),
                    })),
                    else_branch: None,
                },
                Statement::Return(Expr::IntLiteral {
                    value: 3,
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert!(asm.contains("li a0, 1"));
    assert!(asm.contains("li a0, 2"));
    assert!(asm.contains("li a0, 3"));
    assert!(asm.contains("beqz a0,"));
}

#[test]
fn generates_if_else() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::If {
                cond: Expr::IntLiteral {
                    value: 0,
                    span: span(),
                },
                then_branch: Box::new(Statement::Return(Expr::IntLiteral {
                    value: 2,
                    span: span(),
                })),
                else_branch: Some(Box::new(Statement::Return(Expr::IntLiteral {
                    value: 3,
                    span: span(),
                }))),
            }],
        }],
        eof_span: span(),
    };

    let asm = generate_with_codegen(&program);

    assert!(asm.contains("li a0, 0"));
    assert!(asm.contains("li a0, 2"));
    assert!(asm.contains("li a0, 3"));
    assert!(asm.contains("beqz a0,"));
    assert!(asm.contains("j "));
}

#[test]
fn generates_while_loop() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
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
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("while_start_"));
    assert!(asm.contains("while_end_"));
    assert!(asm.contains("beqz a0, while_end_"));
    assert!(asm.contains("j while_start_"));
}

#[test]
fn generates_break_jump_to_loop_end() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::While {
                    cond: Expr::IntLiteral {
                        value: 1,
                        span: span(),
                    },
                    body: Box::new(Statement::Block(vec![Statement::Break { span: span() }])),
                },
                Statement::Return(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("while_end_"));
    assert!(asm.contains("    j while_end_"));
}

#[test]
fn generates_continue_jump_to_loop_start() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::While {
                    cond: Expr::IntLiteral {
                        value: 1,
                        span: span(),
                    },
                    body: Box::new(Statement::Block(vec![Statement::Continue { span: span() }])),
                },
                Statement::Return(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("while_start_"));
    assert!(
        asm.matches("    j while_start_").count() >= 2,
        "continue should add a jump to the loop start in addition to the loop backedge"
    );
}

#[test]
fn generates_do_while_loop_with_body_before_condition() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::VarDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "x".to_string(),
                    init: Some(Expr::IntLiteral {
                        value: 1,
                        span: span(),
                    }),
                },
                Statement::DoWhile {
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
                    cond: Expr::Variable {
                        name: "x".to_string(),
                        span: span(),
                    },
                },
                Statement::Return(Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("do_while_start_"));
    assert!(asm.contains("do_while_continue_"));
    assert!(asm.contains("do_while_end_"));
    assert!(asm.contains("bnez a0, do_while_start_"));
    assert!(
        asm.find("do_while_start_").unwrap() < asm.find("do_while_continue_").unwrap(),
        "do while should emit the body label before the condition label"
    );
}

#[test]
fn generates_continue_in_do_while_to_condition() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::DoWhile {
                    body: Box::new(Statement::Block(vec![Statement::Continue { span: span() }])),
                    cond: Expr::IntLiteral {
                        value: 0,
                        span: span(),
                    },
                },
                Statement::Return(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("do_while_continue_"));
    assert!(asm.contains("    j do_while_continue_"));
}

#[test]
fn counts_locals_inside_do_while_body_for_frame_size() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::DoWhile {
                    body: Box::new(Statement::Block(vec![
                        Statement::VarDecl {
                            ty: Type::Int,
                            name_span: span(),
                            name: "a".to_string(),
                            init: Some(Expr::IntLiteral {
                                value: 1,
                                span: span(),
                            }),
                        },
                        Statement::VarDecl {
                            ty: Type::Int,
                            name_span: span(),
                            name: "b".to_string(),
                            init: Some(Expr::IntLiteral {
                                value: 2,
                                span: span(),
                            }),
                        },
                        Statement::VarDecl {
                            ty: Type::Int,
                            name_span: span(),
                            name: "c".to_string(),
                            init: Some(Expr::IntLiteral {
                                value: 3,
                                span: span(),
                            }),
                        },
                    ])),
                    cond: Expr::IntLiteral {
                        value: 0,
                        span: span(),
                    },
                },
                Statement::Return(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("addi sp, sp, -32"));
}

#[test]
fn nested_loop_break_uses_inner_loop_end() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::While {
                    cond: Expr::IntLiteral {
                        value: 1,
                        span: span(),
                    },
                    body: Box::new(Statement::Block(vec![Statement::While {
                        cond: Expr::IntLiteral {
                            value: 1,
                            span: span(),
                        },
                        body: Box::new(Statement::Block(vec![Statement::Break { span: span() }])),
                    }])),
                },
                Statement::Return(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    j while_end_3"));
}

#[test]
fn generates_for_loop_with_init_condition_and_post() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::VarDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "i".to_string(),
                    init: Some(Expr::IntLiteral {
                        value: 0,
                        span: span(),
                    }),
                },
                Statement::For {
                    init: Some(Box::new(Statement::ExprStatement(Expr::Assign {
                        op_span: span(),
                        target: Box::new(Expr::Variable {
                            name: "i".to_string(),
                            span: span(),
                        }),
                        value: Box::new(Expr::IntLiteral {
                            value: 0,
                            span: span(),
                        }),
                    }))),
                    cond: Some(Expr::Binary {
                        op: BinaryOp::Less,
                        op_span: span(),
                        left: Box::new(Expr::Variable {
                            name: "i".to_string(),
                            span: span(),
                        }),
                        right: Box::new(Expr::IntLiteral {
                            value: 3,
                            span: span(),
                        }),
                    }),
                    post: Some(Expr::Assign {
                        op_span: span(),
                        target: Box::new(Expr::Variable {
                            name: "i".to_string(),
                            span: span(),
                        }),
                        value: Box::new(Expr::Binary {
                            op: BinaryOp::Add,
                            op_span: span(),
                            left: Box::new(Expr::Variable {
                                name: "i".to_string(),
                                span: span(),
                            }),
                            right: Box::new(Expr::IntLiteral {
                                value: 1,
                                span: span(),
                            }),
                        }),
                    }),
                    body: Box::new(Statement::Block(vec![Statement::Empty])),
                },
                Statement::Return(Expr::Variable {
                    name: "i".to_string(),
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("for_start_"));
    assert!(asm.contains("for_continue_"));
    assert!(asm.contains("for_break_"));
    assert!(asm.contains("beqz a0, for_break_"));
    assert!(asm.contains("j for_start_"));
}

#[test]
fn generates_for_loop_without_condition_as_unconditional_loop() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::For {
                    init: None,
                    cond: None,
                    post: None,
                    body: Box::new(Statement::Block(vec![Statement::Break { span: span() }])),
                },
                Statement::Return(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("for_start_"));
    assert!(asm.contains("for_break_"));
    assert!(!asm.contains("beqz a0, for_break_"));
    assert!(asm.contains("    j for_break_"));
}

#[test]
fn generates_continue_in_for_loop_to_post_clause() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::VarDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "i".to_string(),
                    init: Some(Expr::IntLiteral {
                        value: 0,
                        span: span(),
                    }),
                },
                Statement::For {
                    init: None,
                    cond: Some(Expr::Binary {
                        op: BinaryOp::Less,
                        op_span: span(),
                        left: Box::new(Expr::Variable {
                            name: "i".to_string(),
                            span: span(),
                        }),
                        right: Box::new(Expr::IntLiteral {
                            value: 3,
                            span: span(),
                        }),
                    }),
                    post: Some(Expr::Assign {
                        op_span: span(),
                        target: Box::new(Expr::Variable {
                            name: "i".to_string(),
                            span: span(),
                        }),
                        value: Box::new(Expr::Binary {
                            op: BinaryOp::Add,
                            op_span: span(),
                            left: Box::new(Expr::Variable {
                                name: "i".to_string(),
                                span: span(),
                            }),
                            right: Box::new(Expr::IntLiteral {
                                value: 1,
                                span: span(),
                            }),
                        }),
                    }),
                    body: Box::new(Statement::Block(vec![Statement::Continue { span: span() }])),
                },
                Statement::Return(Expr::Variable {
                    name: "i".to_string(),
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("for_continue_"));
    assert!(asm.contains("    j for_continue_"));
}

#[test]
fn counts_locals_inside_for_init_and_body_for_frame_size() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::For {
                    init: Some(Box::new(Statement::VarDecl {
                        ty: Type::Int,
                        name_span: span(),
                        name: "i".to_string(),
                        init: Some(Expr::IntLiteral {
                            value: 0,
                            span: span(),
                        }),
                    })),
                    cond: Some(Expr::Binary {
                        op: BinaryOp::Less,
                        op_span: span(),
                        left: Box::new(Expr::Variable {
                            name: "i".to_string(),
                            span: span(),
                        }),
                        right: Box::new(Expr::IntLiteral {
                            value: 1,
                            span: span(),
                        }),
                    }),
                    post: None,
                    body: Box::new(Statement::Block(vec![
                        Statement::VarDecl {
                            ty: Type::Int,
                            name_span: span(),
                            name: "a".to_string(),
                            init: Some(Expr::IntLiteral {
                                value: 1,
                                span: span(),
                            }),
                        },
                        Statement::VarDecl {
                            ty: Type::Int,
                            name_span: span(),
                            name: "b".to_string(),
                            init: Some(Expr::IntLiteral {
                                value: 2,
                                span: span(),
                            }),
                        },
                        Statement::VarDecl {
                            ty: Type::Int,
                            name_span: span(),
                            name: "c".to_string(),
                            init: Some(Expr::IntLiteral {
                                value: 3,
                                span: span(),
                            }),
                        },
                        Statement::Break { span: span() },
                    ])),
                },
                Statement::Return(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("addi sp, sp, -32"));
}

#[test]
fn for_init_scope_can_shadow_outer_local_without_replacing_it() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::VarDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "i".to_string(),
                    init: Some(Expr::IntLiteral {
                        value: 5,
                        span: span(),
                    }),
                },
                Statement::For {
                    init: Some(Box::new(Statement::VarDecl {
                        ty: Type::Int,
                        name_span: span(),
                        name: "i".to_string(),
                        init: Some(Expr::IntLiteral {
                            value: 0,
                            span: span(),
                        }),
                    })),
                    cond: Some(Expr::Binary {
                        op: BinaryOp::Less,
                        op_span: span(),
                        left: Box::new(Expr::Variable {
                            name: "i".to_string(),
                            span: span(),
                        }),
                        right: Box::new(Expr::IntLiteral {
                            value: 1,
                            span: span(),
                        }),
                    }),
                    post: Some(Expr::Assign {
                        op_span: span(),
                        target: Box::new(Expr::Variable {
                            name: "i".to_string(),
                            span: span(),
                        }),
                        value: Box::new(Expr::Binary {
                            op: BinaryOp::Add,
                            op_span: span(),
                            left: Box::new(Expr::Variable {
                                name: "i".to_string(),
                                span: span(),
                            }),
                            right: Box::new(Expr::IntLiteral {
                                value: 1,
                                span: span(),
                            }),
                        }),
                    }),
                    body: Box::new(Statement::Block(vec![Statement::Empty])),
                },
                Statement::Return(Expr::Variable {
                    name: "i".to_string(),
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(
        asm.contains("    li a0, 5\n    sw a0, -12(s0)")
            && asm.contains("    lw a0, -12(s0)\n    j main_end"),
        "return after the for loop should load the outer local"
    );
}

#[test]
fn counts_locals_inside_while_body_for_frame_size() {
    let program = Program {
        functions: vec![Function {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![
                Statement::VarDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "x".to_string(),
                    init: Some(Expr::IntLiteral {
                        value: 1,
                        span: span(),
                    }),
                },
                Statement::While {
                    cond: Expr::Variable {
                        name: "x".to_string(),
                        span: span(),
                    },
                    body: Box::new(Statement::Block(vec![
                        Statement::VarDecl {
                            ty: Type::Int,
                            name_span: span(),
                            name: "a".to_string(),
                            init: Some(Expr::IntLiteral {
                                value: 1,
                                span: span(),
                            }),
                        },
                        Statement::VarDecl {
                            ty: Type::Int,
                            name_span: span(),
                            name: "b".to_string(),
                            init: Some(Expr::IntLiteral {
                                value: 2,
                                span: span(),
                            }),
                        },
                        Statement::VarDecl {
                            ty: Type::Int,
                            name_span: span(),
                            name: "c".to_string(),
                            init: Some(Expr::IntLiteral {
                                value: 3,
                                span: span(),
                            }),
                        },
                        Statement::Return(Expr::Binary {
                            op: BinaryOp::Add,
                            op_span: span(),
                            left: Box::new(Expr::Binary {
                                op: BinaryOp::Add,
                                op_span: span(),
                                left: Box::new(Expr::Variable {
                                    name: "a".to_string(),
                                    span: span(),
                                }),
                                right: Box::new(Expr::Variable {
                                    name: "b".to_string(),
                                    span: span(),
                                }),
                            }),
                            right: Box::new(Expr::Variable {
                                name: "c".to_string(),
                                span: span(),
                            }),
                        }),
                    ])),
                },
                Statement::Return(Expr::IntLiteral {
                    value: 0,
                    span: span(),
                }),
            ],
        }],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("addi sp, sp, -32"));
}
