mod common;

use common::{call_expr, param, param_with_span, program_with_functions, span};
use rivet::ast::{
    BinaryOp, Expr, ExternalDecl, FunctionDef, GlobalDecl, Initializer, IntLiteralBase,
    IntLiteralSuffix, Program, Statement, Type, UnaryOp,
};
use rivet::codegen::{CodegenTarget, generate};
use rivet::lexer::lex;
use rivet::parser::parse;
use rivet::preprocess::preprocess;
use rivet::sema::check;
use rivet::source::DUMMY_FILE_ID;

fn generate_checked(program: &Program) -> String {
    let typed_program = check(program).expect("semantic check should succeed");
    generate(&typed_program, CodegenTarget::Rv32)
}

fn generate_source(source: &str) -> String {
    let tokens = lex(source, DUMMY_FILE_ID).expect("lexing should succeed");
    let tokens = preprocess(tokens).expect("preprocessing should succeed");
    let program = parse(tokens).expect("parsing should succeed");
    generate_checked(&program)
}

fn generate_raw_with_codegen(program: &Program) -> String {
    generate_checked(program)
}

fn generate_with_codegen(program: &Program) -> String {
    generate_raw_with_codegen(program).replace("    j main_end\nmain_end:\n", "")
}

fn empty_main_function() -> FunctionDef {
    FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![Statement::Return(Expr::IntLiteral {
            value: 0,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        })],
    }
}

fn global_decl(name: &str, ty: Type, init: Option<Initializer>) -> GlobalDecl {
    GlobalDecl {
        ty,
        name: name.to_string(),
        name_span: span(),
        init,
    }
}

const fn int_literal(value: u64) -> Expr {
    Expr::IntLiteral {
        value,
        suffix: IntLiteralSuffix::None,
        base: IntLiteralBase::Decimal,
        span: span(),
    }
}

fn add_function() -> FunctionDef {
    FunctionDef {
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

fn right_function() -> FunctionDef {
    FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "right".to_string(),
        params: vec![],
        body: vec![Statement::Return(Expr::IntLiteral {
            value: 1,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        })],
    }
}

#[test]
fn generates_multiple_functions() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "helper".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::IntLiteral {
                value: 3,
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
            body: vec![Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })],
        },
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(".globl helper\nhelper:\n"));
    assert!(asm.contains(".globl main\nmain:\n"));
    assert!(asm.contains("    j helper_end\nhelper_end:\n"));
    assert!(asm.contains("    j main_end\nmain_end:\n"));
}

#[test]
fn resets_local_offsets_between_functions() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "first".to_string(),
            params: vec![],
            body: vec![
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
        },
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert_eq!(asm.matches("sw a0, -12(s0)").count(), 2);
    assert_eq!(asm.matches("addi a0, s0, -12").count(), 2);
    assert_eq!(asm.matches("lw a0, 0(a0)").count(), 2);
}

#[test]
fn computes_frame_layout_per_function() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "helper".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::IntLiteral {
                value: 1,
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
                    name: "a".to_string(),
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
                    name: "b".to_string(),
                    init: Some(Initializer::Expr(Expr::IntLiteral {
                        value: 2,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    })),
                }]),
                Statement::Decl(vec![rivet::ast::LocalDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "c".to_string(),
                    init: Some(Initializer::Expr(Expr::IntLiteral {
                        value: 3,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    })),
                }]),
                Statement::Return(Expr::Variable {
                    name: "c".to_string(),
                    span: span(),
                }),
            ],
        },
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("helper:\n    addi sp, sp, -16\n"));
    assert!(asm.contains("main:\n    addi sp, sp, -32\n"));
}

#[test]
fn generates_zero_argument_function_call() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "helper".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::IntLiteral {
                value: 3,
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
            body: vec![Statement::Return(call_expr("helper", vec![]))],
        },
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("main:\n"));
    assert!(asm.contains("    call helper\n    j main_end\n"));
}

#[test]
fn uses_call_result_as_expression_operand() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "helper".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::IntLiteral {
                value: 3,
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
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::Add,
                op_span: span(),
                left: Box::new(call_expr("helper", vec![])),
                right: Box::new(Expr::IntLiteral {
                    value: 2,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
            })],
        },
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    call helper\n"));
    assert!(asm.contains("    add a0, t0, a0\n"));
}

#[test]
fn stores_single_parameter_in_function_frame() {
    let program = program_with_functions(vec![
        FunctionDef {
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
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "id:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    sw a0, -12(s0)\n    addi a0, s0, -12\n    lw a0, 0(a0)\n"
    ));
}

#[test]
fn stores_multiple_parameters_in_function_frame() {
    let program = program_with_functions(vec![add_function(), empty_main_function()]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    sw a0, -12(s0)\n"));
    assert!(asm.contains("    sw a1, -16(s0)\n"));
    assert!(asm.contains("    addi a0, s0, -12\n    lw a0, 0(a0)\n"));
    assert!(asm.contains("    addi a0, s0, -16\n    lw a0, 0(a0)\n"));
}

#[test]
fn generates_function_call_with_arguments() {
    let program = program_with_functions(vec![
        add_function(),
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(call_expr(
                "add",
                vec![
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
            ))],
        },
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    li a0, 1\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 2\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    lw a1, 0(sp)\n    addi sp, sp, 4\n    lw a0, 0(sp)\n    addi sp, sp, 4\n    call add\n"
    ));
}

#[test]
fn generates_function_call_with_expression_arguments() {
    let program = program_with_functions(vec![
        add_function(),
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(call_expr(
                "add",
                vec![
                    Expr::Binary {
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
                    },
                    Expr::Binary {
                        op: BinaryOp::Add,
                        op_span: span(),
                        left: Box::new(Expr::IntLiteral {
                            value: 3,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span(),
                        }),
                        right: Box::new(Expr::IntLiteral {
                            value: 4,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span(),
                        }),
                    },
                ],
            ))],
        },
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    lw a1, 0(sp)\n    addi sp, sp, 4\n    lw a0, 0(sp)\n"));
    assert!(asm.contains("    call add\n"));
}

#[test]
fn generates_direct_call_for_function_designator_call() {
    let asm = generate_source("int id(int x) { return x; } int main() { return id(3); }");

    assert!(asm.contains("    call id\n"));
    assert!(!asm.contains("    jalr ra, 0(t0)\n"));
}

#[test]
fn generates_indirect_call_for_function_pointer_call() {
    let asm = generate_source(
        "int id(int x) { return x; } int main() { int (*fp)(int) = id; return fp(3); }",
    );

    assert!(asm.contains("    la a0, id\n"));
    assert!(asm.contains("    jalr ra, 0(t0)\n"));
    assert!(!asm.contains("    call fp\n"));
}

#[test]
fn generates_indirect_call_for_explicitly_dereferenced_function_pointer_call() {
    let asm = generate_source(
        "int id(int x) { return x; } int main() { int (*fp)(int) = id; return (*fp)(3); }",
    );

    assert!(asm.contains("    la a0, id\n"));
    assert!(asm.contains("    jalr ra, 0(t0)\n"));
    assert!(!asm.contains("    call fp\n"));
}

#[test]
fn emits_nothing_for_empty_statement() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::Empty,
            Statement::Return(Expr::IntLiteral {
                value: 7,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        ],
    }]);

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 7\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn emits_expression_statement_and_discards_result() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "helper".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::IntLiteral {
                value: 3,
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
                Statement::ExprStatement(call_expr("helper", vec![])),
                Statement::Return(Expr::IntLiteral {
                    value: 7,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
            ],
        },
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("main:\n"));
    assert!(asm.contains("    call helper\n    li a0, 7\n"));
}

#[test]
fn generates_return_jump_to_shared_epilogue() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![Statement::Return(Expr::IntLiteral {
            value: 42,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        })],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    j main_end\nmain_end:\n"));
}

#[test]
fn generates_logical_and_with_short_circuit_branch() {
    let program = program_with_functions(vec![
        right_function(),
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::LogicalAnd,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 0,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
                right: Box::new(call_expr("right", vec![])),
            })],
        },
    ]);

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
    let program = program_with_functions(vec![
        right_function(),
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "main".to_string(),
            params: vec![],
            body: vec![Statement::Return(Expr::Binary {
                op: BinaryOp::LogicalOr,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
                right: Box::new(call_expr("right", vec![])),
            })],
        },
    ]);

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
fn generates_single_local_variable() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "x".to_string(),
                init: Some(Initializer::Expr(Expr::IntLiteral {
                    value: 5,
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
    }]);

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 5\n    sw a0, -12(s0)\n    addi a0, s0, -12\n    lw a0, 0(a0)\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_local_variable_without_initializer() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
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
        ],
    }]);

    let asm = generate_with_codegen(&program);

    assert_eq!(
        asm,
        ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    addi a0, s0, -12\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 3\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    sw a0, 0(t0)\n    addi a0, s0, -12\n    lw a0, 0(a0)\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
    );
}

#[test]
fn generates_initialized_int_global() {
    let program = Program {
        declarations: vec![
            ExternalDecl::Global(global_decl(
                "g",
                Type::Int,
                Some(Initializer::Expr(int_literal(3))),
            )),
            ExternalDecl::FunctionDef(empty_main_function()),
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.starts_with(".data\n"));
    assert!(asm.contains(".globl g\n"));
    assert!(asm.contains("g:\n"));
    assert!(asm.contains(".word 3\n"));
    assert!(asm.contains(".text\n"));
    assert!(asm.find(".word 3\n").unwrap() < asm.find(".text\n").unwrap());
    assert!(asm.find(".text\n").unwrap() < asm.find(".globl main\n").unwrap());
}

#[test]
fn generates_zero_initialized_int_global() {
    let program = Program {
        declarations: vec![
            ExternalDecl::Global(global_decl("g", Type::Int, None)),
            ExternalDecl::FunctionDef(empty_main_function()),
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(".globl g\n"));
    assert!(asm.contains("g:\n"));
    assert!(asm.contains(".word 0\n"));
}

#[test]
fn generates_char_global() {
    let program = Program {
        declarations: vec![
            ExternalDecl::Global(global_decl(
                "c",
                Type::Char,
                Some(Initializer::Expr(int_literal(7))),
            )),
            ExternalDecl::FunctionDef(empty_main_function()),
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(".globl c\n"));
    assert!(asm.contains("c:\n"));
    assert!(asm.contains(".byte 7\n"));
}

#[test]
fn generates_load_from_int_global() {
    let program = Program {
        declarations: vec![
            ExternalDecl::Global(global_decl(
                "g",
                Type::Int,
                Some(Initializer::Expr(int_literal(3))),
            )),
            ExternalDecl::FunctionDef(FunctionDef {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Variable {
                    name: "g".to_string(),
                    span: span(),
                })],
            }),
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    la a0, g\n    lw a0, 0(a0)\n"));
}

#[test]
fn generates_store_to_int_global() {
    let program = Program {
        declarations: vec![
            ExternalDecl::Global(global_decl("g", Type::Int, None)),
            ExternalDecl::FunctionDef(FunctionDef {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![
                    Statement::ExprStatement(Expr::Assign {
                        op_span: span(),
                        target: Box::new(Expr::Variable {
                            name: "g".to_string(),
                            span: span(),
                        }),
                        value: Box::new(int_literal(9)),
                    }),
                    Statement::Return(Expr::Variable {
                        name: "g".to_string(),
                        span: span(),
                    }),
                ],
            }),
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    la a0, g\n    addi sp, sp, -4\n    sw a0, 0(sp)\n"));
    assert!(asm.contains("    li a0, 9\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    sw a0, 0(t0)\n"));
    assert_eq!(asm.matches("    la a0, g\n").count(), 2);
}

#[test]
fn generates_zero_initialized_int_global_array() {
    let program = Program {
        declarations: vec![
            ExternalDecl::Global(global_decl(
                "nums",
                Type::Array {
                    element: Box::new(Type::Int),
                    len: 3,
                },
                None,
            )),
            ExternalDecl::FunctionDef(empty_main_function()),
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(".globl nums\n"));
    assert!(asm.contains("nums:\n"));
    assert_eq!(asm.matches("  .word 0\n").count(), 3);
}

#[test]
fn generates_partially_initialized_int_global_array() {
    let program = Program {
        declarations: vec![
            ExternalDecl::Global(global_decl(
                "nums",
                Type::Array {
                    element: Box::new(Type::Int),
                    len: 3,
                },
                Some(Initializer::List(vec![int_literal(1), int_literal(2)])),
            )),
            ExternalDecl::FunctionDef(empty_main_function()),
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("nums:\n      .word 1\n      .word 2\n      .word 0\n"));
}

#[test]
fn generates_char_global_array() {
    let program = Program {
        declarations: vec![
            ExternalDecl::Global(global_decl(
                "buf",
                Type::Array {
                    element: Box::new(Type::Char),
                    len: 3,
                },
                Some(Initializer::List(vec![
                    int_literal(65),
                    int_literal(66),
                    int_literal(67),
                ])),
            )),
            ExternalDecl::FunctionDef(empty_main_function()),
        ],
        eof_span: span(),
    };

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("buf:\n      .byte 65\n      .byte 66\n      .byte 67\n"));
}

#[test]
fn array_local_reserves_full_frame_slot_and_aligns_next_local() {
    let program = program_with_functions(vec![FunctionDef {
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
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "x".to_string(),
                init: Some(Initializer::Expr(Expr::IntLiteral {
                    value: 7,
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
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("main:\n    addi sp, sp, -16\n"));
    assert!(asm.contains("    li a0, 7\n    sw a0, -16(s0)\n"));
    assert!(asm.contains("    addi a0, s0, -16\n    lw a0, 0(a0)\n"));
}

#[test]
fn generates_array_initializer_element_stores() {
    let program = program_with_functions(vec![FunctionDef {
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
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Array {
                    element: Box::new(Type::Int),
                    len: 2,
                },
                name_span: span(),
                name: "nums".to_string(),
                init: Some(Initializer::List(vec![
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
                ])),
            }]),
            Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    sb a0, -11(s0)\n"));
    assert!(asm.contains("    sb a0, -10(s0)\n"));
    assert!(asm.contains("    sb a0, -9(s0)\n"));
    assert!(asm.contains("    li a0, 4\n    sw a0, -20(s0)\n"));
    assert!(asm.contains("    li a0, 5\n    sw a0, -16(s0)\n"));
}

#[test]
fn generates_zero_stores_for_empty_array_initializer_list() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Array {
                    element: Box::new(Type::Char),
                    len: 2,
                },
                name_span: span(),
                name: "buf".to_string(),
                init: Some(Initializer::List(vec![])),
            }]),
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
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    li a0, 0\n    andi a0, a0, 255\n    sb a0, -10(s0)\n"));
    assert!(asm.contains("    li a0, 0\n    andi a0, a0, 255\n    sb a0, -9(s0)\n"));
    assert!(asm.contains("    li a0, 0\n    sw a0, -20(s0)\n"));
    assert!(asm.contains("    li a0, 0\n    sw a0, -16(s0)\n"));
}

#[test]
fn narrows_char_local_initializer() {
    let program = program_with_functions(vec![FunctionDef {
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
                    value: 300,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })),
            }]),
            Statement::Return(Expr::Variable {
                name: "c".to_string(),
                span: span(),
            }),
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    li a0, 300\n    andi a0, a0, 255\n    sb a0, -9(s0)\n"));
}

#[test]
fn loads_char_local_with_unsigned_byte_load() {
    let program = program_with_functions(vec![FunctionDef {
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
                    value: 255,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })),
            }]),
            Statement::Return(Expr::Variable {
                name: "c".to_string(),
                span: span(),
            }),
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    sb a0, -9(s0)\n    addi a0, s0, -9\n    lbu a0, 0(a0)\n"));
}

#[test]
fn narrows_char_assignment_through_address() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Char,
                name_span: span(),
                name: "c".to_string(),
                init: None,
            }]),
            Statement::ExprStatement(Expr::Assign {
                op_span: span(),
                target: Box::new(Expr::Variable {
                    name: "c".to_string(),
                    span: span(),
                }),
                value: Box::new(Expr::IntLiteral {
                    value: 300,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
            }),
            Statement::Return(Expr::Variable {
                name: "c".to_string(),
                span: span(),
            }),
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    addi a0, s0, -9\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 300\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    andi a0, a0, 255\n    sb a0, 0(t0)\n"
    ));
}

#[test]
fn generates_compound_assignment() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
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
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    addi a0, s0, -12\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    lw a0, 0(a0)\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 4\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    add a0, t0, a0\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    sw a0, 0(t0)\n"
    ));
}

#[test]
fn generates_compound_assignment_expression_result() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
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
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    addi a0, s0, -12\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    lw a0, 0(a0)\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 4\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    add a0, t0, a0\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    sw a0, 0(t0)\n    j main_end\n"
    ));
}

#[test]
fn narrows_char_compound_assignment() {
    let program = program_with_functions(vec![FunctionDef {
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
                    value: 250,
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
                    value: 10,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
            }),
            Statement::Return(Expr::Variable {
                name: "c".to_string(),
                span: span(),
            }),
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    addi a0, s0, -9\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    lbu a0, 0(a0)\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 10\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    add a0, t0, a0\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    andi a0, a0, 255\n    sb a0, 0(t0)\n"
    ));
}

#[test]
fn narrows_char_return_value() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Char,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![Statement::Return(Expr::IntLiteral {
            value: 300,
            suffix: IntLiteralSuffix::None,
            base: IntLiteralBase::Decimal,
            span: span(),
        })],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    li a0, 300\n    andi a0, a0, 255\n    j main_end\n"));
}

#[test]
fn narrows_char_parameter_on_function_entry() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
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
            params: vec![],
            body: vec![Statement::Return(call_expr(
                "id",
                vec![Expr::IntLiteral {
                    value: 300,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }],
            ))],
        },
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("id:\n"));
    assert!(asm.contains("    andi a0, a0, 255\n    sb a0, -9(s0)\n"));
}

#[test]
fn narrows_char_increment_store() {
    let program = program_with_functions(vec![FunctionDef {
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
                    value: 255,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })),
            }]),
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
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    addi a0, s0, -9\n    mv t0, a0\n    lbu a0, 0(a0)\n    addi a0, a0, 1\n    andi a0, a0, 255\n    sb a0, 0(t0)\n"
    ));
}

#[test]
fn generates_chained_assignment_expression_right_associative() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "x".to_string(),
                init: None,
            }]),
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "y".to_string(),
                init: None,
            }]),
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
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
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
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    li a0, 4\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    sw a0, 0(t0)\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    sw a0, 0(t0)\n"
    ));
}

#[test]
fn generates_if_without_else() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::If {
                cond: Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                },
                then_branch: Box::new(Statement::Return(Expr::IntLiteral {
                    value: 2,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })),
                else_branch: None,
            },
            Statement::Return(Expr::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        ],
    }]);

    let asm = generate_with_codegen(&program);

    assert!(asm.contains("li a0, 1"));
    assert!(asm.contains("li a0, 2"));
    assert!(asm.contains("li a0, 3"));
    assert!(asm.contains("beqz a0,"));
}

#[test]
fn generates_if_else() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![Statement::If {
            cond: Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            },
            then_branch: Box::new(Statement::Return(Expr::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            })),
            else_branch: Some(Box::new(Statement::Return(Expr::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }))),
        }],
    }]);

    let asm = generate_with_codegen(&program);

    assert!(asm.contains("li a0, 0"));
    assert!(asm.contains("li a0, 2"));
    assert!(asm.contains("li a0, 3"));
    assert!(asm.contains("beqz a0,"));
    assert!(asm.contains("j "));
}

#[test]
fn generates_while_loop() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
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
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("while_start_"));
    assert!(asm.contains("while_end_"));
    assert!(asm.contains("beqz a0, while_end_"));
    assert!(asm.contains("j while_start_"));
}

#[test]
fn generates_break_jump_to_loop_end() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::While {
                cond: Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                },
                body: Box::new(Statement::Block(vec![Statement::Break { span: span() }])),
            },
            Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("while_end_"));
    assert!(asm.contains("    j while_end_"));
}

#[test]
fn generates_continue_jump_to_loop_start() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::While {
                cond: Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                },
                body: Box::new(Statement::Block(vec![Statement::Continue { span: span() }])),
            },
            Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("while_start_"));
    assert!(
        asm.matches("    j while_start_").count() >= 2,
        "continue should add a jump to the loop start in addition to the loop backedge"
    );
}

#[test]
fn generates_do_while_loop_with_body_before_condition() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
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
                                suffix: IntLiteralSuffix::None,
                                base: IntLiteralBase::Decimal,
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
    }]);

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
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::DoWhile {
                body: Box::new(Statement::Block(vec![Statement::Continue { span: span() }])),
                cond: Expr::IntLiteral {
                    value: 0,
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
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("do_while_continue_"));
    assert!(asm.contains("    j do_while_continue_"));
}

#[test]
fn counts_locals_inside_do_while_body_for_frame_size() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::DoWhile {
                body: Box::new(Statement::Block(vec![
                    Statement::Decl(vec![rivet::ast::LocalDecl {
                        ty: Type::Int,
                        name_span: span(),
                        name: "a".to_string(),
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
                        name: "b".to_string(),
                        init: Some(Initializer::Expr(Expr::IntLiteral {
                            value: 2,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span(),
                        })),
                    }]),
                    Statement::Decl(vec![rivet::ast::LocalDecl {
                        ty: Type::Int,
                        name_span: span(),
                        name: "c".to_string(),
                        init: Some(Initializer::Expr(Expr::IntLiteral {
                            value: 3,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span(),
                        })),
                    }]),
                ])),
                cond: Expr::IntLiteral {
                    value: 0,
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
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("addi sp, sp, -32"));
}

#[test]
fn nested_loop_break_uses_inner_loop_end() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::While {
                cond: Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                },
                body: Box::new(Statement::Block(vec![Statement::While {
                    cond: Expr::IntLiteral {
                        value: 1,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    },
                    body: Box::new(Statement::Block(vec![Statement::Break { span: span() }])),
                }])),
            },
            Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    j while_end_3"));
}

#[test]
fn generates_for_loop_with_init_condition_and_post() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "i".to_string(),
                init: Some(Initializer::Expr(Expr::IntLiteral {
                    value: 0,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })),
            }]),
            Statement::For {
                init: Some(Box::new(Statement::ExprStatement(Expr::Assign {
                    op_span: span(),
                    target: Box::new(Expr::Variable {
                        name: "i".to_string(),
                        span: span(),
                    }),
                    value: Box::new(Expr::IntLiteral {
                        value: 0,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
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
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
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
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
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
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("for_start_"));
    assert!(asm.contains("for_continue_"));
    assert!(asm.contains("for_break_"));
    assert!(asm.contains("beqz a0, for_break_"));
    assert!(asm.contains("j for_start_"));
}

#[test]
fn generates_for_loop_without_condition_as_unconditional_loop() {
    let program = program_with_functions(vec![FunctionDef {
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
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("for_start_"));
    assert!(asm.contains("for_break_"));
    assert!(!asm.contains("beqz a0, for_break_"));
    assert!(asm.contains("    j for_break_"));
}

#[test]
fn generates_continue_in_for_loop_to_post_clause() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "i".to_string(),
                init: Some(Initializer::Expr(Expr::IntLiteral {
                    value: 0,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })),
            }]),
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
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
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
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
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
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("for_continue_"));
    assert!(asm.contains("    j for_continue_"));
}

#[test]
fn counts_locals_inside_for_init_and_body_for_frame_size() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::For {
                init: Some(Box::new(Statement::Decl(vec![rivet::ast::LocalDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "i".to_string(),
                    init: Some(Initializer::Expr(Expr::IntLiteral {
                        value: 0,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    })),
                }]))),
                cond: Some(Expr::Binary {
                    op: BinaryOp::Less,
                    op_span: span(),
                    left: Box::new(Expr::Variable {
                        name: "i".to_string(),
                        span: span(),
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 1,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    }),
                }),
                post: None,
                body: Box::new(Statement::Block(vec![
                    Statement::Decl(vec![rivet::ast::LocalDecl {
                        ty: Type::Int,
                        name_span: span(),
                        name: "a".to_string(),
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
                        name: "b".to_string(),
                        init: Some(Initializer::Expr(Expr::IntLiteral {
                            value: 2,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span(),
                        })),
                    }]),
                    Statement::Decl(vec![rivet::ast::LocalDecl {
                        ty: Type::Int,
                        name_span: span(),
                        name: "c".to_string(),
                        init: Some(Initializer::Expr(Expr::IntLiteral {
                            value: 3,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span(),
                        })),
                    }]),
                    Statement::Break { span: span() },
                ])),
            },
            Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("addi sp, sp, -32"));
}

#[test]
fn for_init_scope_can_shadow_outer_local_without_replacing_it() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Int,
                name_span: span(),
                name: "i".to_string(),
                init: Some(Initializer::Expr(Expr::IntLiteral {
                    value: 5,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                })),
            }]),
            Statement::For {
                init: Some(Box::new(Statement::Decl(vec![rivet::ast::LocalDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "i".to_string(),
                    init: Some(Initializer::Expr(Expr::IntLiteral {
                        value: 0,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span(),
                    })),
                }]))),
                cond: Some(Expr::Binary {
                    op: BinaryOp::Less,
                    op_span: span(),
                    left: Box::new(Expr::Variable {
                        name: "i".to_string(),
                        span: span(),
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 1,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
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
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
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
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(
        asm.contains("    li a0, 5\n    sw a0, -12(s0)")
            && asm.contains("    addi a0, s0, -12\n    lw a0, 0(a0)\n    j main_end"),
        "return after the for loop should load the outer local"
    );
}

#[test]
fn counts_locals_inside_while_body_for_frame_size() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
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
            Statement::While {
                cond: Expr::Variable {
                    name: "x".to_string(),
                    span: span(),
                },
                body: Box::new(Statement::Block(vec![
                    Statement::Decl(vec![rivet::ast::LocalDecl {
                        ty: Type::Int,
                        name_span: span(),
                        name: "a".to_string(),
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
                        name: "b".to_string(),
                        init: Some(Initializer::Expr(Expr::IntLiteral {
                            value: 2,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span(),
                        })),
                    }]),
                    Statement::Decl(vec![rivet::ast::LocalDecl {
                        ty: Type::Int,
                        name_span: span(),
                        name: "c".to_string(),
                        init: Some(Initializer::Expr(Expr::IntLiteral {
                            value: 3,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span(),
                        })),
                    }]),
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
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("addi sp, sp, -32"));
}

#[test]
fn loads_char_pointer_dereference_with_byte_load() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "first".to_string(),
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
        empty_main_function(),
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    sw a0, -12(s0)\n"));
    assert!(asm.contains("    addi a0, s0, -12\n    lw a0, 0(a0)\n    lbu a0, 0(a0)\n"));
}

#[test]
fn loads_int_pointer_dereference_with_word_load() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "first".to_string(),
            params: vec![param_with_span(
                Type::Pointer(Box::new(Type::Int)),
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
        empty_main_function(),
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains("    addi a0, s0, -12\n    lw a0, 0(a0)\n    lw a0, 0(a0)\n"));
}

#[test]
fn stores_through_int_pointer_dereference() {
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
            body: vec![
                Statement::ExprStatement(Expr::Assign {
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
                }),
                Statement::Return(Expr::IntLiteral {
                    value: 0,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
            ],
        },
        empty_main_function(),
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    addi a0, s0, -12\n    lw a0, 0(a0)\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 3\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    sw a0, 0(t0)\n"
    ));
}

#[test]
fn stores_through_char_pointer_dereference_with_byte_store() {
    let program = program_with_functions(vec![
        FunctionDef {
            return_type: Type::Int,
            name_span: span(),
            name: "store_char".to_string(),
            params: vec![param_with_span(
                Type::Pointer(Box::new(Type::Char)),
                "p",
                span(),
            )],
            body: vec![
                Statement::ExprStatement(Expr::Assign {
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
                        value: 300,
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
            ],
        },
        empty_main_function(),
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    addi a0, s0, -12\n    lw a0, 0(a0)\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 300\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    andi a0, a0, 255\n    sb a0, 0(t0)\n"
    ));
}

#[test]
fn generates_compound_assignment_through_pointer_dereference() {
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
            body: vec![
                Statement::ExprStatement(Expr::CompoundAssign {
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
                }),
                Statement::Return(Expr::IntLiteral {
                    value: 0,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span(),
                }),
            ],
        },
        empty_main_function(),
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    addi a0, s0, -12\n    lw a0, 0(a0)\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    lw a0, 0(a0)\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 3\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    add a0, t0, a0\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    sw a0, 0(t0)\n"
    ));
}

#[test]
fn generates_postfix_increment_through_pointer_dereference() {
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
        empty_main_function(),
    ]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    addi a0, s0, -12\n    lw a0, 0(a0)\n    mv t0, a0\n    lw a0, 0(a0)\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    addi a0, a0, 1\n    sw a0, 0(t0)\n    lw a0, 0(sp)\n    addi sp, sp, 4\n"
    ));
}

#[test]
fn scales_int_pointer_compound_assignment_by_pointee_size() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Pointer(Box::new(Type::Int)),
                name_span: span(),
                name: "p".to_string(),
                init: None,
            }]),
            Statement::ExprStatement(Expr::CompoundAssign {
                target: Box::new(Expr::Variable {
                    name: "p".to_string(),
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
            Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    addi a0, s0, -12\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    lw a0, 0(a0)\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 2\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    slli a0, a0, 2\n    add a0, t0, a0\n    lw t0, 0(sp)\n    addi sp, sp, 4\n    sw a0, 0(t0)\n"
    ));
}

#[test]
fn scales_int_pointer_increment_by_pointee_size() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Pointer(Box::new(Type::Int)),
                name_span: span(),
                name: "p".to_string(),
                init: None,
            }]),
            Statement::ExprStatement(Expr::PrefixInc {
                expr: Box::new(Expr::Variable {
                    name: "p".to_string(),
                    span: span(),
                }),
                op_span: span(),
            }),
            Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    addi a0, s0, -12\n    mv t0, a0\n    lw a0, 0(a0)\n    addi a0, a0, 4\n    sw a0, 0(t0)\n"
    ));
}

#[test]
fn leaves_char_pointer_increment_unscaled() {
    let program = program_with_functions(vec![FunctionDef {
        return_type: Type::Int,
        name_span: span(),
        name: "main".to_string(),
        params: vec![],
        body: vec![
            Statement::Decl(vec![rivet::ast::LocalDecl {
                ty: Type::Pointer(Box::new(Type::Char)),
                name_span: span(),
                name: "p".to_string(),
                init: None,
            }]),
            Statement::ExprStatement(Expr::PrefixInc {
                expr: Box::new(Expr::Variable {
                    name: "p".to_string(),
                    span: span(),
                }),
                op_span: span(),
            }),
            Statement::Return(Expr::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: span(),
            }),
        ],
    }]);

    let asm = generate_raw_with_codegen(&program);

    assert!(asm.contains(
        "    addi a0, s0, -12\n    mv t0, a0\n    lw a0, 0(a0)\n    addi a0, a0, 1\n    sw a0, 0(t0)\n"
    ));
}
