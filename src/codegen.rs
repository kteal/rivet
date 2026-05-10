use crate::ast::{BinaryOp, Expr, Function, Program, Statement, UnaryOp};
use std::collections::HashMap;
use std::fmt::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodegenTarget {
    Rv32,
}

#[derive(Debug, Default, Clone)]
struct FrameLayout {
    size: i32,
}

impl FrameLayout {
    fn count_locals_in_statement(statement: &Statement) -> i32 {
        match statement {
            Statement::VarDecl { .. } => 1,
            Statement::Block(body) => FrameLayout::count_locals_in_statements(body),
            Statement::If {
                cond: _,
                then_branch,
                else_branch,
            } => {
                let mut sum = FrameLayout::count_locals_in_statement(then_branch);
                if let Some(else_statement) = else_branch {
                    sum += FrameLayout::count_locals_in_statement(else_statement);
                }
                sum
            }
            Statement::While { cond: _, body } => FrameLayout::count_locals_in_statement(body),
            _ => 0,
        }
    }

    fn count_locals_in_statements(statements: &[Statement]) -> i32 {
        let mut sum = 0;
        for statement in statements {
            sum += FrameLayout::count_locals_in_statement(statement);
        }
        sum
    }

    fn for_function(function: &Function) -> Self {
        let num_locals = FrameLayout::count_locals_in_statements(&function.body);
        let mut size = (num_locals * 4) + 8;
        // Add space for parameters
        size += 4 * i32::try_from(function.params.len()).expect("too many arguments");
        // Round up to nearest 16 for stack alignment
        size = (size + 15) / 16 * 16;

        Self { size }
    }
}

struct Codegen {
    out: String,
    frame: FrameLayout,
    scopes: Vec<HashMap<String, i32>>,
    next_local_offset: i32,
    label_counter: usize,
    return_label: Option<String>,
    #[allow(dead_code)]
    target: CodegenTarget,
}

impl Codegen {
    fn new(target: CodegenTarget) -> Self {
        Self {
            out: String::new(),
            frame: FrameLayout::default(),
            scopes: vec![HashMap::new()],
            next_local_offset: -12,
            label_counter: 0,
            return_label: None,
            target,
        }
    }

    fn reset_for_function(&mut self, function: &Function) {
        self.frame = FrameLayout::for_function(function);
        self.scopes = vec![HashMap::new()];
        self.next_local_offset = -12;
        self.return_label = Some(format!("{}_end", function.name));
    }

    fn new_label(&mut self, prefix: &str) -> String {
        let label = format!("{}_{}", prefix, self.label_counter);
        self.label_counter += 1;
        label
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<String, i32> {
        self.scopes
            .last_mut()
            .expect("codegen should have an active scope")
    }

    fn resolve_local(&mut self, name: &str) -> Option<i32> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn declare_local(&mut self, name: &str) -> i32 {
        let offset = self.next_local_offset;
        self.current_scope_mut().insert(name.to_string(), offset);
        self.next_local_offset -= 4;
        offset
    }

    fn emit(&mut self, args: fmt::Arguments) {
        self.out.write_fmt(args).unwrap();
    }

    fn emit_line(&mut self, args: fmt::Arguments) {
        self.emit(format_args!("    {args}\n"));
    }

    fn emit_label(&mut self, args: fmt::Arguments) {
        self.emit(format_args!("{args}:\n"));
    }

    fn emit_binary(&mut self, op: &BinaryOp, left: &Expr, right: &Expr) {
        self.emit_expr(left);
        self.emit_line(format_args!("addi sp, sp, -4"));
        self.emit_line(format_args!("sw a0, 0(sp)"));
        self.emit_expr(right);
        self.emit_line(format_args!("lw t0, 0(sp)"));
        self.emit_line(format_args!("addi sp, sp, 4"));

        match op {
            BinaryOp::Add => self.emit_line(format_args!("add a0, t0, a0")),
            BinaryOp::Subtract => self.emit_line(format_args!("sub a0, t0, a0")),
            BinaryOp::Multiply => self.emit_line(format_args!("mul a0, t0, a0")),
            BinaryOp::Divide => self.emit_line(format_args!("div a0, t0, a0")),
            BinaryOp::Remainder => self.emit_line(format_args!("rem a0, t0, a0")),
            BinaryOp::Equal => {
                self.emit_line(format_args!("xor a0, t0, a0"));
                self.emit_line(format_args!("seqz a0, a0"));
            }
            BinaryOp::NotEqual => {
                self.emit_line(format_args!("xor a0, t0, a0"));
                self.emit_line(format_args!("snez a0, a0"));
            }
            BinaryOp::Less => self.emit_line(format_args!("slt a0, t0, a0")),
            BinaryOp::LessEqual => {
                self.emit_line(format_args!("slt a0, a0, t0"));
                self.emit_line(format_args!("xori a0, a0, 1"));
            }
            BinaryOp::Greater => self.emit_line(format_args!("slt a0, a0, t0")),
            BinaryOp::GreaterEqual => {
                self.emit_line(format_args!("slt a0, t0, a0"));
                self.emit_line(format_args!("xori a0, a0, 1"));
            }
            BinaryOp::BitAnd => self.emit_line(format_args!("and a0, a0, t0")),
            BinaryOp::BitXor => self.emit_line(format_args!("xor a0, a0, t0")),
            BinaryOp::BitOr => self.emit_line(format_args!("or a0, a0, t0")),
            BinaryOp::ShiftLeft => self.emit_line(format_args!("sll a0, t0, a0")),
            BinaryOp::ShiftRight => self.emit_line(format_args!("sra a0, t0, a0")),
        };
    }

    fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::IntLiteral(value) => self.emit_line(format_args!("li a0, {value}")),
            Expr::Binary { op, left, right } => self.emit_binary(op, left, right),
            Expr::Variable(name) => {
                let offset = self
                    .resolve_local(name)
                    .expect("usage of undefined variable");
                self.emit_line(format_args!("lw a0, {offset}(s0)"));
            }
            Expr::Unary { op, expr } => {
                self.emit_expr(expr);
                match op {
                    UnaryOp::Negate => self.emit_line(format_args!("neg a0, a0")),
                    UnaryOp::LogicalNot => self.emit_line(format_args!("seqz a0, a0")),
                    UnaryOp::BitwiseNot => self.emit_line(format_args!("not a0, a0")),
                }
            }
            Expr::Call { name, args } => {
                for arg in args {
                    self.emit_expr(arg);
                    // Push a0 onto the stack
                    self.emit_line(format_args!("addi sp, sp, -4"));
                    self.emit_line(format_args!("sw a0, 0(sp)"));
                }
                // Pop off arguments in reverse order
                for arg_num in (0..args.len()).rev() {
                    self.emit_line(format_args!("lw a{arg_num}, 0(sp)"));
                    self.emit_line(format_args!("addi sp, sp, 4"));
                }
                self.emit_line(format_args!("call {name}"));
            }
        }
    }

    fn emit_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Return(expr) => {
                self.emit_expr(expr);
                let return_label = self
                    .return_label
                    .clone()
                    .expect("codegen should have an active return label");
                self.emit_line(format_args!("j {return_label}"));
            }
            Statement::VarDecl { name, init: value } => {
                // After this, value is in a0
                self.emit_expr(value);

                let offset = self.declare_local(name);
                self.emit_line(format_args!("sw a0, {offset}(s0)"));
            }
            Statement::Assign { name, value } => {
                self.emit_expr(value);

                let offset = self
                    .resolve_local(name)
                    .expect("assignment to undefined variable");
                self.emit_line(format_args!("sw a0, {offset}(s0)"));
            }
            Statement::Block(body) => {
                self.emit_block(body);
            }
            Statement::If {
                cond,
                then_branch,
                else_branch,
            } => self.emit_if_statement(cond, then_branch, else_branch.as_deref()),
            Statement::While { cond, body } => self.emit_while_statement(cond, body),
        }
    }

    fn emit_while_statement(&mut self, cond: &Expr, body: &Statement) {
        let start_label = self.new_label("while_start");
        let end_label = self.new_label("while_end");

        self.emit_label(format_args!("{start_label}"));
        self.emit_expr(cond);
        self.emit_line(format_args!("beqz a0, {end_label}"));
        self.emit_statement(body);
        self.emit_line(format_args!("j {start_label}"));
        self.emit_label(format_args!("{end_label}"));
    }

    fn emit_if_statement(
        &mut self,
        cond: &Expr,
        then_branch: &Statement,
        else_branch: Option<&Statement>,
    ) {
        self.emit_expr(cond);
        let end_label = self.new_label("if_end");
        if let Some(else_statement) = else_branch {
            let else_label = self.new_label("if_else");
            self.emit_line(format_args!("beqz a0, {else_label}"));

            self.emit_statement(then_branch);

            self.emit_line(format_args!("j {end_label}"));
            self.emit_label(format_args!("{else_label}"));
            self.emit_statement(else_statement);
        } else {
            self.emit_line(format_args!("beqz a0, {end_label}"));

            self.emit_statement(then_branch);
        };
        self.emit_label(format_args!("{end_label}"));
    }

    fn emit_statements(&mut self, statements: &[Statement]) {
        for statement in statements {
            self.emit_statement(statement);
        }
    }

    fn emit_block(&mut self, body: &[Statement]) {
        self.enter_scope();
        self.emit_statements(body);
        self.exit_scope();
    }

    fn emit_prologue(&mut self, name: &str) {
        let frame_size = self.frame.size;
        self.emit(format_args!(".globl {name}\n"));
        self.emit_label(format_args!("{name}"));
        self.emit_line(format_args!("addi sp, sp, -{frame_size}"));
        self.emit_line(format_args!("sw ra, {}(sp)", frame_size - 4));
        self.emit_line(format_args!("sw s0, {}(sp)", frame_size - 8));
        self.emit_line(format_args!("addi s0, sp, {frame_size}"));
    }

    fn emit_epilogue(&mut self) {
        let frame_size = self.frame.size;
        let return_label = self
            .return_label
            .clone()
            .expect("codegen should have an active return label");
        self.emit_label(format_args!("{return_label}"));
        self.emit_line(format_args!("lw ra, {}(sp)", frame_size - 4));
        self.emit_line(format_args!("lw s0, {}(sp)", frame_size - 8));
        self.emit_line(format_args!("addi sp, sp, {frame_size}"));
        self.emit_line(format_args!("ret"));
    }

    fn emit_function(&mut self, function: &Function) {
        self.reset_for_function(function);
        self.emit_prologue(&function.name);

        // Store the argument registers a0-a7 onto the stack, declaring as local vars
        for (i, param) in function.params.iter().enumerate() {
            let offset = self.declare_local(param);
            self.emit_line(format_args!("sw a{i}, {offset}(s0)"));
        }

        self.emit_statements(&function.body);
        self.emit_epilogue();
    }

    fn emit_program(&mut self, program: &Program) -> String {
        for function in &program.functions {
            self.emit_function(function);
        }

        std::mem::take(&mut self.out)
    }
}

pub fn generate(program: &Program, target: CodegenTarget) -> String {
    let mut codegen = Codegen::new(target);
    codegen.emit_program(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Function;

    fn generate_raw_with_codegen(program: &Program) -> String {
        let mut codegen = Codegen::new(CodegenTarget::Rv32);
        codegen.emit_program(program)
    }

    fn generate_with_codegen(program: &Program) -> String {
        generate_raw_with_codegen(program).replace("    j main_end\nmain_end:\n", "")
    }

    #[test]
    fn generates_multiple_functions() {
        let program = Program {
            functions: vec![
                Function {
                    name: "helper".to_string(),
                    params: vec![],
                    body: vec![Statement::Return(Expr::IntLiteral(3))],
                },
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![Statement::Return(Expr::IntLiteral(0))],
                },
            ],
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
                    name: "first".to_string(),
                    params: vec![],
                    body: vec![
                        Statement::VarDecl {
                            name: "x".to_string(),
                            init: Expr::IntLiteral(1),
                        },
                        Statement::Return(Expr::Variable("x".to_string())),
                    ],
                },
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![
                        Statement::VarDecl {
                            name: "x".to_string(),
                            init: Expr::IntLiteral(2),
                        },
                        Statement::Return(Expr::Variable("x".to_string())),
                    ],
                },
            ],
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
                    name: "helper".to_string(),
                    params: vec![],
                    body: vec![Statement::Return(Expr::IntLiteral(1))],
                },
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![
                        Statement::VarDecl {
                            name: "a".to_string(),
                            init: Expr::IntLiteral(1),
                        },
                        Statement::VarDecl {
                            name: "b".to_string(),
                            init: Expr::IntLiteral(2),
                        },
                        Statement::VarDecl {
                            name: "c".to_string(),
                            init: Expr::IntLiteral(3),
                        },
                        Statement::Return(Expr::Variable("c".to_string())),
                    ],
                },
            ],
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
                    name: "helper".to_string(),
                    params: vec![],
                    body: vec![Statement::Return(Expr::IntLiteral(3))],
                },
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![Statement::Return(Expr::Call {
                        name: "helper".to_string(),
                        args: vec![],
                    })],
                },
            ],
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
                    name: "helper".to_string(),
                    params: vec![],
                    body: vec![Statement::Return(Expr::IntLiteral(3))],
                },
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![Statement::Return(Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Call {
                            name: "helper".to_string(),
                            args: vec![],
                        }),
                        right: Box::new(Expr::IntLiteral(2)),
                    })],
                },
            ],
        };

        let asm = generate_raw_with_codegen(&program);

        assert!(asm.contains("    call helper\n"));
        assert!(asm.contains("    add a0, t0, a0\n"));
    }

    #[test]
    fn stores_single_parameter_in_function_frame() {
        let program = Program {
            functions: vec![Function {
                name: "id".to_string(),
                params: vec!["x".to_string()],
                body: vec![Statement::Return(Expr::Variable("x".to_string()))],
            }],
        };

        let asm = generate_raw_with_codegen(&program);

        assert!(asm.contains(
            "id:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    sw a0, -12(s0)\n    lw a0, -12(s0)\n"
        ));
    }

    #[test]
    fn stores_multiple_parameters_in_function_frame() {
        let program = Program {
            functions: vec![Function {
                name: "add".to_string(),
                params: vec!["x".to_string(), "y".to_string()],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Variable("x".to_string())),
                    right: Box::new(Expr::Variable("y".to_string())),
                })],
            }],
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
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Call {
                    name: "add".to_string(),
                    args: vec![Expr::IntLiteral(1), Expr::IntLiteral(2)],
                })],
            }],
        };

        let asm = generate_raw_with_codegen(&program);

        assert!(asm.contains(
            "    li a0, 1\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    li a0, 2\n    addi sp, sp, -4\n    sw a0, 0(sp)\n    lw a1, 0(sp)\n    addi sp, sp, 4\n    lw a0, 0(sp)\n    addi sp, sp, 4\n    call add\n"
        ));
    }

    #[test]
    fn generates_function_call_with_expression_arguments() {
        let program = Program {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Call {
                    name: "add".to_string(),
                    args: vec![
                        Expr::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::IntLiteral(1)),
                            right: Box::new(Expr::IntLiteral(2)),
                        },
                        Expr::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::IntLiteral(3)),
                            right: Box::new(Expr::IntLiteral(4)),
                        },
                    ],
                })],
            }],
        };

        let asm = generate_raw_with_codegen(&program);

        assert!(asm.contains("    lw a1, 0(sp)\n    addi sp, sp, 4\n    lw a0, 0(sp)\n"));
        assert!(asm.contains("    call add\n"));
    }

    #[test]
    fn generates_return_jump_to_shared_epilogue() {
        let program = Program {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::IntLiteral(42))],
            }],
        };

        let asm = generate_raw_with_codegen(&program);

        assert!(asm.contains("    j main_end\nmain_end:\n"));
    }

    #[test]
    fn basic_codegen() {
        let program = Program {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::IntLiteral(42))],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::IntLiteral(1)),
                    right: Box::new(Expr::IntLiteral(2)),
                })],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Subtract,
                    left: Box::new(Expr::IntLiteral(5)),
                    right: Box::new(Expr::IntLiteral(2)),
                })],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Multiply,
                    left: Box::new(Expr::IntLiteral(2)),
                    right: Box::new(Expr::IntLiteral(3)),
                })],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Divide,
                    left: Box::new(Expr::IntLiteral(8)),
                    right: Box::new(Expr::IntLiteral(2)),
                })],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Remainder,
                    left: Box::new(Expr::IntLiteral(8)),
                    right: Box::new(Expr::IntLiteral(3)),
                })],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Equal,
                    left: Box::new(Expr::IntLiteral(5)),
                    right: Box::new(Expr::IntLiteral(5)),
                })],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::NotEqual,
                    left: Box::new(Expr::IntLiteral(5)),
                    right: Box::new(Expr::IntLiteral(3)),
                })],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Less,
                    left: Box::new(Expr::IntLiteral(2)),
                    right: Box::new(Expr::IntLiteral(5)),
                })],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::LessEqual,
                    left: Box::new(Expr::IntLiteral(5)),
                    right: Box::new(Expr::IntLiteral(5)),
                })],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Greater,
                    left: Box::new(Expr::IntLiteral(5)),
                    right: Box::new(Expr::IntLiteral(2)),
                })],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::GreaterEqual,
                    left: Box::new(Expr::IntLiteral(5)),
                    right: Box::new(Expr::IntLiteral(5)),
                })],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Unary {
                    op: UnaryOp::Negate,
                    expr: Box::new(Expr::IntLiteral(5)),
                })],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Unary {
                    op: UnaryOp::LogicalNot,
                    expr: Box::new(Expr::IntLiteral(0)),
                })],
            }],
        };

        let asm = generate_with_codegen(&program);

        assert_eq!(
            asm,
            ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 0\n    seqz a0, a0\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
        );
    }

    #[test]
    fn generates_bitwise_not() {
        let program = Program {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Unary {
                    op: UnaryOp::BitwiseNot,
                    expr: Box::new(Expr::IntLiteral(0)),
                })],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::IntLiteral(1)),
                    right: Box::new(Expr::Binary {
                        op: BinaryOp::Multiply,
                        left: Box::new(Expr::IntLiteral(2)),
                        right: Box::new(Expr::IntLiteral(3)),
                    }),
                })],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![
                    Statement::VarDecl {
                        name: "x".to_string(),
                        init: Expr::IntLiteral(5),
                    },
                    Statement::Return(Expr::Variable("x".to_string())),
                ],
            }],
        };

        let asm = generate_with_codegen(&program);

        assert_eq!(
            asm,
            ".globl main\nmain:\n    addi sp, sp, -16\n    sw ra, 12(sp)\n    sw s0, 8(sp)\n    addi s0, sp, 16\n    li a0, 5\n    sw a0, -12(s0)\n    lw a0, -12(s0)\n    lw ra, 12(sp)\n    lw s0, 8(sp)\n    addi sp, sp, 16\n    ret\n"
        );
    }

    #[test]
    fn generates_multiple_local_variables() {
        let program = Program {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                body: vec![
                    Statement::VarDecl {
                        name: "x".to_string(),
                        init: Expr::IntLiteral(1),
                    },
                    Statement::VarDecl {
                        name: "y".to_string(),
                        init: Expr::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::Variable("x".to_string())),
                            right: Box::new(Expr::IntLiteral(2)),
                        },
                    },
                    Statement::VarDecl {
                        name: "z".to_string(),
                        init: Expr::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::Variable("x".to_string())),
                            right: Box::new(Expr::Variable("y".to_string())),
                        },
                    },
                    Statement::Return(Expr::Variable("z".to_string())),
                ],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![
                    Statement::VarDecl {
                        name: "x".to_string(),
                        init: Expr::IntLiteral(1),
                    },
                    Statement::Block(vec![
                        Statement::VarDecl {
                            name: "x".to_string(),
                            init: Expr::IntLiteral(2),
                        },
                        Statement::Return(Expr::Variable("x".to_string())),
                    ]),
                ],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![
                    Statement::If {
                        cond: Expr::IntLiteral(1),
                        then_branch: Box::new(Statement::Return(Expr::IntLiteral(2))),
                        else_branch: None,
                    },
                    Statement::Return(Expr::IntLiteral(3)),
                ],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::If {
                    cond: Expr::IntLiteral(0),
                    then_branch: Box::new(Statement::Return(Expr::IntLiteral(2))),
                    else_branch: Some(Box::new(Statement::Return(Expr::IntLiteral(3)))),
                }],
            }],
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
                name: "main".to_string(),
                params: vec![],
                body: vec![
                    Statement::VarDecl {
                        name: "x".to_string(),
                        init: Expr::IntLiteral(3),
                    },
                    Statement::While {
                        cond: Expr::Variable("x".to_string()),
                        body: Box::new(Statement::Block(vec![Statement::Assign {
                            name: "x".to_string(),
                            value: Expr::Binary {
                                op: BinaryOp::Subtract,
                                left: Box::new(Expr::Variable("x".to_string())),
                                right: Box::new(Expr::IntLiteral(1)),
                            },
                        }])),
                    },
                    Statement::Return(Expr::Variable("x".to_string())),
                ],
            }],
        };

        let asm = generate_raw_with_codegen(&program);

        assert!(asm.contains("while_start_"));
        assert!(asm.contains("while_end_"));
        assert!(asm.contains("beqz a0, while_end_"));
        assert!(asm.contains("j while_start_"));
    }

    #[test]
    fn counts_locals_inside_while_body_for_frame_size() {
        let program = Program {
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                body: vec![
                    Statement::VarDecl {
                        name: "x".to_string(),
                        init: Expr::IntLiteral(1),
                    },
                    Statement::While {
                        cond: Expr::Variable("x".to_string()),
                        body: Box::new(Statement::Block(vec![
                            Statement::VarDecl {
                                name: "a".to_string(),
                                init: Expr::IntLiteral(1),
                            },
                            Statement::VarDecl {
                                name: "b".to_string(),
                                init: Expr::IntLiteral(2),
                            },
                            Statement::VarDecl {
                                name: "c".to_string(),
                                init: Expr::IntLiteral(3),
                            },
                            Statement::Return(Expr::Binary {
                                op: BinaryOp::Add,
                                left: Box::new(Expr::Binary {
                                    op: BinaryOp::Add,
                                    left: Box::new(Expr::Variable("a".to_string())),
                                    right: Box::new(Expr::Variable("b".to_string())),
                                }),
                                right: Box::new(Expr::Variable("c".to_string())),
                            }),
                        ])),
                    },
                    Statement::Return(Expr::IntLiteral(0)),
                ],
            }],
        };

        let asm = generate_raw_with_codegen(&program);

        assert!(asm.contains("addi sp, sp, -32"));
    }
}
