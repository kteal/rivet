use crate::ast::{BinaryOp, Expr, Function, Program, Statement, Type, UnaryOp};
use std::collections::HashMap;
use std::fmt::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodegenTarget {
    Rv32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoopLabels {
    continue_label: String,
    break_label: String,
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
            Statement::DoWhile { body, cond: _ } => FrameLayout::count_locals_in_statement(body),
            Statement::For {
                init,
                cond: _,
                post: _,
                body,
            } => {
                let mut sum = FrameLayout::count_locals_in_statement(body);
                if let Some(init_statement) = init {
                    sum += FrameLayout::count_locals_in_statement(init_statement);
                }
                // C does not allow VarDecl in post
                sum
            }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LocalInfo {
    offset: i32,
    ty: Type,
}

struct Codegen {
    out: String,
    frame: FrameLayout,
    scopes: Vec<HashMap<String, LocalInfo>>,
    next_local_offset: i32,
    label_counter: usize,
    return_label: Option<String>,
    loop_stack: Vec<LoopLabels>,
    current_function_return_type: Option<Type>,
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
            loop_stack: vec![],
            current_function_return_type: None,
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

    fn push_loop_labels(&mut self, continue_label: &str, break_label: &str) {
        self.loop_stack.push(LoopLabels {
            continue_label: continue_label.to_string(),
            break_label: break_label.to_string(),
        });
    }

    fn pop_loop_labels(&mut self) {
        self.loop_stack.pop();
    }

    fn current_break_label(&self) -> Option<String> {
        self.loop_stack
            .last()
            .map(|labels| labels.break_label.clone())
    }

    fn current_continue_label(&self) -> Option<String> {
        self.loop_stack
            .last()
            .map(|labels| labels.continue_label.clone())
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<String, LocalInfo> {
        self.scopes
            .last_mut()
            .expect("codegen should have an active scope")
    }

    fn resolve_local(&mut self, name: &str) -> Option<LocalInfo> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn declare_local(&mut self, ty: &Type, name: &str) -> i32 {
        let offset = self.next_local_offset;
        self.current_scope_mut()
            .insert(name.to_string(), LocalInfo { offset, ty: *ty });
        self.next_local_offset -= 4;
        offset
    }

    fn emit(&mut self, args: fmt::Arguments) {
        self.out.write_fmt(args).unwrap();
    }

    fn emit_line(&mut self, args: fmt::Arguments) {
        self.emit(format_args!("    {args}\n"));
    }

    fn emit_label(&mut self, label: &str) {
        self.emit(format_args!("{label}:\n"));
    }

    fn emit_narrow_to_type(&mut self, ty: Type) {
        match ty {
            Type::Int => (),
            Type::Char => self.emit_line(format_args!("andi a0, a0, 255")),
        };
    }

    fn emit_load_local(&mut self, local: LocalInfo) {
        match local.ty {
            Type::Int => self.emit_line(format_args!("lw a0, {}(s0)", local.offset)),
            Type::Char => self.emit_line(format_args!("lbu a0, {}(s0)", local.offset)),
        }
    }

    fn emit_store_local(&mut self, local: LocalInfo) {
        match local.ty {
            Type::Int => self.emit_line(format_args!("sw a0, {}(s0)", local.offset)),
            Type::Char => {
                self.emit_narrow_to_type(Type::Char);
                self.emit_line(format_args!("sb a0, {}(s0)", local.offset));
            }
        }
    }

    fn emit_store_param(&mut self, reg: usize, local: LocalInfo) {
        match local.ty {
            Type::Int => self.emit_line(format_args!("sw a{reg}, {}(s0)", local.offset)),
            Type::Char => {
                self.emit_line(format_args!("andi a{reg}, a{reg}, 255"));
                self.emit_line(format_args!("sb a{reg}, {}(s0)", local.offset));
            }
        }
    }

    fn emit_logical_and(&mut self, left: &Expr, right: &Expr) {
        let false_label = self.new_label("logical_and_false");
        let end_label = self.new_label("logical_and_end");

        self.emit_expr(left);
        self.emit_line(format_args!("beqz a0, {false_label}"));

        self.emit_expr(right);
        self.emit_line(format_args!("snez a0, a0"));
        self.emit_line(format_args!("j {end_label}"));

        self.emit_label(&false_label);
        self.emit_line(format_args!("li a0, 0"));

        self.emit_label(&end_label);
    }

    fn emit_logical_or(&mut self, left: &Expr, right: &Expr) {
        let true_label = self.new_label("logical_or_true");
        let end_label = self.new_label("logical_or_end");

        self.emit_expr(left);
        self.emit_line(format_args!("bnez a0, {true_label}"));

        self.emit_expr(right);
        self.emit_line(format_args!("snez a0, a0"));
        self.emit_line(format_args!("j {end_label}"));

        self.emit_label(&true_label);
        self.emit_line(format_args!("li a0, 1"));

        self.emit_label(&end_label);
    }

    fn emit_binary(&mut self, op: &BinaryOp, left: &Expr, right: &Expr) {
        // Short-circuit binary operations
        match op {
            BinaryOp::LogicalAnd => {
                self.emit_logical_and(left, right);
                return;
            }
            BinaryOp::LogicalOr => {
                self.emit_logical_or(left, right);
                return;
            }
            _ => {}
        }

        // Fully-evaluated binary operations
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
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => unreachable!(),
        };
    }

    fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::IntLiteral { value, .. } => self.emit_line(format_args!("li a0, {value}")),
            Expr::Binary {
                op,
                op_span: _,
                left,
                right,
            } => self.emit_binary(op, left, right),
            Expr::Variable { name, .. } => {
                let local = self
                    .resolve_local(name)
                    .expect("usage of undefined variable");
                self.emit_load_local(local);
            }
            Expr::Unary {
                op,
                op_span: _,
                expr,
            } => {
                self.emit_expr(expr);
                match op {
                    UnaryOp::Negate => self.emit_line(format_args!("neg a0, a0")),
                    UnaryOp::LogicalNot => self.emit_line(format_args!("seqz a0, a0")),
                    UnaryOp::BitwiseNot => self.emit_line(format_args!("not a0, a0")),
                }
            }
            Expr::Call {
                name,
                name_span: _,
                args,
            } => {
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
            Expr::Assign {
                name,
                name_span: _,
                value,
            } => {
                self.emit_expr(value);

                let local = self
                    .resolve_local(name)
                    .expect("assignment to undefined variable");
                self.emit_store_local(local);
            }
        }
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
            self.emit_label(&else_label);
            self.emit_statement(else_statement);
        } else {
            self.emit_line(format_args!("beqz a0, {end_label}"));

            self.emit_statement(then_branch);
        };
        self.emit_label(&end_label);
    }

    fn emit_while_statement(&mut self, cond: &Expr, body: &Statement) {
        let start_label = self.new_label("while_start");
        let end_label = self.new_label("while_end");

        self.push_loop_labels(&start_label, &end_label);

        self.emit_label(&start_label);
        self.emit_expr(cond);
        self.emit_line(format_args!("beqz a0, {end_label}"));
        self.emit_statement(body);
        self.emit_line(format_args!("j {start_label}"));
        self.emit_label(&end_label);

        self.pop_loop_labels();
    }

    fn emit_do_while_statement(&mut self, body: &Statement, cond: &Expr) {
        let start_label = self.new_label("do_while_start");
        let continue_label = self.new_label("do_while_continue");
        let end_label = self.new_label("do_while_end");

        self.push_loop_labels(&continue_label, &end_label);

        self.emit_label(&start_label);
        self.emit_statement(body);
        self.emit_label(&continue_label);
        self.emit_expr(cond);
        self.emit_line(format_args!("bnez a0, {start_label}"));
        self.emit_label(&end_label);

        self.pop_loop_labels();
    }

    fn emit_for_statement(
        &mut self,
        init: Option<&Statement>,
        cond: Option<&Expr>,
        post: Option<&Expr>,
        body: &Statement,
    ) {
        let start_label = self.new_label("for_start");
        let continue_label = self.new_label("for_continue");
        let break_label = self.new_label("for_break");

        self.push_loop_labels(&continue_label, &break_label);
        self.enter_scope();

        // Init
        if let Some(init_statement) = init {
            self.emit_statement(init_statement);
        }

        // Cond + Branch + Body
        self.emit_label(&start_label);
        if let Some(cond_expr) = cond {
            self.emit_expr(cond_expr);
            self.emit_line(format_args!("beqz a0, {break_label}"));
        }
        self.emit_statement(body);

        // Post + Jump back
        self.emit_label(&continue_label);
        if let Some(post_expr) = post {
            self.emit_expr(post_expr);
        }
        self.emit_line(format_args!("j {start_label}"));

        // Break
        self.emit_label(&break_label);

        self.exit_scope();
        self.pop_loop_labels();
    }

    fn emit_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::Return(expr) => {
                self.emit_expr(expr);
                match self
                    .current_function_return_type
                    .expect("codegen should have a function return type")
                {
                    Type::Int => (),
                    Type::Char => self.emit_line(format_args!("andi a0, a0, 255")),
                };
                let return_label = self
                    .return_label
                    .clone()
                    .expect("codegen should have an active return label");
                self.emit_line(format_args!("j {return_label}"));
            }
            Statement::VarDecl {
                ty,
                name,
                name_span: _,
                init,
            } => {
                let offset = self.declare_local(ty, name);
                if let Some(init_expr) = init {
                    self.emit_expr(init_expr);
                    let local = LocalInfo { offset, ty: *ty };
                    self.emit_store_local(local);
                }
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
            Statement::DoWhile { body, cond } => self.emit_do_while_statement(body, cond),
            Statement::Empty => (),
            Statement::ExprStatement(expr) => self.emit_expr(expr),
            Statement::Break { .. } => {
                if let Some(label) = self.current_break_label() {
                    self.emit_line(format_args!("j {label}"));
                }
            }
            Statement::Continue { .. } => {
                if let Some(label) = self.current_continue_label() {
                    self.emit_line(format_args!("j {label}"));
                }
            }
            Statement::For {
                init,
                cond,
                post,
                body,
            } => self.emit_for_statement(init.as_deref(), cond.as_ref(), post.as_ref(), body),
        }
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
        self.emit_label(&name);
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
        self.emit_label(&return_label);
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
            let offset = self.declare_local(&param.ty, &param.name);
            let local = LocalInfo {
                offset,
                ty: param.ty,
            };
            self.emit_store_param(i, local);
        }

        self.current_function_return_type = Some(function.return_type);
        self.emit_statements(&function.body);
        self.current_function_return_type = None;
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
