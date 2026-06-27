use crate::ast::{BinaryOp, Linkage, Type, UnaryOp};
use crate::typed_ast::{
    GlobalId, LocalId, ObjectId, TypedExpr, TypedExprKind, TypedExternalDecl, TypedFunction,
    TypedGlobalDecl, TypedInitializer, TypedProgram, TypedStatement,
};
use std::collections::HashMap;
use std::fmt::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodegenTarget {
    Rv32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StringData {
    label: String,
    bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoopLabels {
    continue_label: String,
    break_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FrameSlot {
    offset: i32,
    size: i32,
    align: i32,
    ty: Type,
}

#[derive(Debug, Default, Clone)]
struct FrameLayout {
    size: i32,
    locals: HashMap<LocalId, FrameSlot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GlobalSlot {
    label: String,
    ty: Type,
}

impl FrameLayout {
    fn for_function(function: &TypedFunction) -> Self {
        let mut layout = Self::default();
        let mut local_bytes = 0;

        for param in &function.params {
            layout.add_slot(param.id, &param.ty, &mut local_bytes);
        }

        layout.add_statement_slots(&function.body, &mut local_bytes);

        // Round up to nearest 16 for stack alignment
        let raw_size = local_bytes + 8;
        layout.size = (raw_size + 15) / 16 * 16;
        layout
    }

    fn add_slot(&mut self, id: LocalId, ty: &Type, local_bytes: &mut i32) {
        let slot_size = i32::try_from(ty.size()).expect("type size exceeds i32");
        let slot_align = i32::try_from(ty.align()).expect("type alignment exceeds i32");

        *local_bytes = Self::align_to(*local_bytes, slot_align);
        *local_bytes += slot_size;

        let offset = -8 - *local_bytes;

        self.locals.insert(
            id,
            FrameSlot {
                offset,
                size: slot_size,
                align: slot_align,
                ty: ty.clone(),
            },
        );
    }

    fn add_statement_slots(&mut self, statements: &[TypedStatement], local_bytes: &mut i32) {
        for statement in statements {
            self.add_statement_slot(statement, local_bytes);
        }
    }

    fn add_statement_slot(&mut self, statement: &TypedStatement, local_bytes: &mut i32) {
        match statement {
            TypedStatement::Decl(declarations) => {
                for declaration in declarations {
                    self.add_slot(declaration.id, &declaration.ty, local_bytes);
                }
            }
            TypedStatement::Block(body) => {
                self.add_statement_slots(body, local_bytes);
            }
            TypedStatement::If {
                then_branch,
                else_branch,
                ..
            } => {
                self.add_statement_slot(then_branch, local_bytes);
                if let Some(else_branch) = else_branch {
                    self.add_statement_slot(else_branch, local_bytes);
                }
            }
            TypedStatement::While { body, .. } | TypedStatement::DoWhile { body, .. } => {
                self.add_statement_slot(body, local_bytes);
            }
            TypedStatement::For { init, body, .. } => {
                if let Some(init) = init {
                    self.add_statement_slot(init, local_bytes);
                }
                self.add_statement_slot(body, local_bytes);
            }
            _ => (),
        }
    }

    const fn align_to(value: i32, align: i32) -> i32 {
        (value + align - 1) / align * align
    }

    fn local(&self, id: LocalId) -> &FrameSlot {
        self.locals
            .get(&id)
            .expect("semantic analysis should only emit declared locals")
    }
}

struct Codegen {
    out: String,
    frame: FrameLayout,
    label_counter: usize,
    return_label: Option<String>,
    loop_stack: Vec<LoopLabels>,
    current_function_return_type: Option<Type>,
    globals: HashMap<GlobalId, GlobalSlot>,
    strings: Vec<StringData>,
    #[allow(dead_code)]
    target: CodegenTarget,
}

impl Codegen {
    fn new(target: CodegenTarget) -> Self {
        Self {
            out: String::new(),
            frame: FrameLayout::default(),
            label_counter: 0,
            return_label: None,
            loop_stack: vec![],
            current_function_return_type: None,
            globals: HashMap::new(),
            strings: vec![],
            target,
        }
    }

    fn reset_for_function(&mut self, function: &TypedFunction) {
        self.frame = FrameLayout::for_function(function);
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

    fn resolve_local(&self, id: LocalId) -> &FrameSlot {
        self.frame.local(id)
    }

    fn resolve_global(&self, id: GlobalId) -> &GlobalSlot {
        self.globals.get(&id).expect("global id should have slot")
    }

    fn add_string_literal(&mut self, bytes: &[u8]) -> String {
        let label = format!(".Lstr{}", self.strings.len());
        self.strings.push(StringData {
            label: label.clone(),
            bytes: bytes.to_vec(),
        });
        label
    }

    fn emit_string_data(&mut self) {
        if self.strings.is_empty() {
            return;
        }

        self.emit_line(format_args!(".section .rodata"));

        let strings = self.strings.clone();
        for string in strings {
            self.emit_line(format_args!("{}:", string.label));

            for byte in &string.bytes {
                self.emit_line(format_args!("    .byte {byte}"));
            }

            self.emit_line(format_args!("    .byte 0"));
        }
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

    fn emit_global_scalar_value(&mut self, ty: &Type, value: u64) {
        match ty {
            Type::Int | Type::UnsignedInt => {
                self.emit_line(format_args!("  .word {value}"));
            }
            Type::Char | Type::UnsignedChar => {
                self.emit_line(format_args!("  .byte {value}"));
            }
            _ => unimplemented!("global scalar codegen for {:?}", ty),
        }
    }

    fn emit_global_array(&mut self, element: &Type, len: usize, init: Option<&TypedInitializer>) {
        let values = match init {
            None => vec![],
            Some(TypedInitializer::List(values)) => values.clone(),
            Some(TypedInitializer::Expr(_)) => {
                unreachable!("array global initializer cannot be scalar after sema")
            }
        };

        for expr in &values {
            let value = expr
                .eval_int_constant_expr()
                .expect("global array initializer should be integer constant");
            self.emit_global_scalar_value(element, value);
        }

        for _ in values.len()..len {
            self.emit_global_scalar_value(element, 0);
        }
    }

    fn emit_global(&mut self, global: &TypedGlobalDecl) {
        if global.linkage == Linkage::External {
            self.emit_line(format_args!(".globl {}", global.name));
        }
        self.emit_line(format_args!("{}:", global.name));

        match &global.ty {
            Type::Int | Type::UnsignedInt | Type::Char | Type::UnsignedChar => {
                let value = global_init_int_value(global.init.as_ref());
                self.emit_global_scalar_value(&global.ty, value);
            }
            Type::Array { element, len } => {
                self.emit_global_array(element, *len, global.init.as_ref());
            }
            _ => unimplemented!("global codegen for {:?}", global.ty),
        }
    }

    fn push_a0(&mut self) {
        self.emit_line(format_args!("addi sp, sp, -4"));
        self.emit_line(format_args!("sw a0, 0(sp)"));
    }

    fn pop_a0(&mut self) {
        self.emit_line(format_args!("lw a0, 0(sp)"));
        self.emit_line(format_args!("addi sp, sp, 4"));
    }

    fn pop_t0(&mut self) {
        self.emit_line(format_args!("lw t0, 0(sp)"));
        self.emit_line(format_args!("addi sp, sp, 4"));
    }

    fn scale_reg(&mut self, scale: i32, reg: &str) {
        match scale {
            1 => (),
            2 => self.emit_line(format_args!("slli {reg}, {reg}, 1")),
            4 => self.emit_line(format_args!("slli {reg}, {reg}, 2")),
            8 => self.emit_line(format_args!("slli {reg}, {reg}, 3")),
            x => {
                self.emit_line(format_args!("li t1, {x}"));
                self.emit_line(format_args!("mul {reg}, {reg}, t1"));
            }
        }
    }

    fn emit_narrow_to_type(&mut self, ty: &Type) {
        match ty {
            Type::Int | Type::UnsignedInt | Type::Pointer(_) | Type::Long | Type::UnsignedLong => {
                #[allow(clippy::no_effect)]
                ();
            }
            Type::Char | Type::UnsignedChar | Type::SignedChar => {
                self.emit_line(format_args!("andi a0, a0, 255"));
            }
            Type::Array { .. } => unreachable!("array rvalues should have decayed before codegen"),
            Type::IncompleteArray { .. } => {
                unreachable!("incomplete arrays should have been handled before codegen")
            }
            Type::Function(_) => unreachable!("function values should have decayed before codegen"),
            Type::Void => unreachable!("void types should be rejected before codegen"),
        }
    }

    fn emit_store_local(&mut self, local: &FrameSlot) {
        self.emit_store_to_base_offset(&local.ty, "s0", local.offset);
    }

    fn emit_store_param(&mut self, reg: usize, local: &FrameSlot) {
        match local.ty {
            Type::Int | Type::UnsignedInt | Type::Pointer(_) | Type::Long | Type::UnsignedLong => {
                self.emit_line(format_args!("sw a{reg}, {}(s0)", local.offset));
            }
            Type::Char | Type::UnsignedChar | Type::SignedChar => {
                self.emit_line(format_args!("andi a{reg}, a{reg}, 255"));
                self.emit_line(format_args!("sb a{reg}, {}(s0)", local.offset));
            }
            Type::Array { .. } => unreachable!("array rvalues should have decayed before codegen"),
            Type::IncompleteArray { .. } => {
                unreachable!("incomplete arrays should have been handled before codegen")
            }
            Type::Function(_) => unreachable!("function values should have decayed before codegen"),
            Type::Void => unreachable!("void types should be rejected before codegen"),
        }
    }

    fn emit_call_args(&mut self, args: &[TypedExpr]) {
        for arg in args {
            self.emit_expr(arg);
            self.push_a0();
        }
        // Pop off arguments in reverse order
        for arg_num in (0..args.len()).rev() {
            self.emit_line(format_args!("lw a{arg_num}, 0(sp)"));
            self.emit_line(format_args!("addi sp, sp, 4"));
        }
    }

    fn emit_addr(&mut self, expr: &TypedExpr) {
        match &expr.kind {
            TypedExprKind::Variable { id, .. } => match id {
                ObjectId::Global(id) => {
                    let label = self.resolve_global(*id).label.clone();
                    self.emit_line(format_args!("la a0, {label}"));
                }
                ObjectId::Local(id) => {
                    let slot = self.resolve_local(*id).clone();
                    self.emit_line(format_args!("addi a0, s0, {}", slot.offset));
                }
            },
            TypedExprKind::Unary {
                op: UnaryOp::Dereference,
                expr,
                ..
            } => {
                self.emit_expr(expr);
            }
            TypedExprKind::Index { base, index, .. } => {
                let element = match &base.ty {
                    Type::Array { element, .. } => {
                        self.emit_addr(base);
                        element
                    }
                    Type::Pointer(element) => {
                        self.emit_expr(base);
                        element
                    }
                    _ => panic!("sema guarantees that only arrays or pointers are indexed"),
                };
                self.push_a0();

                self.emit_expr(index);
                self.scale_reg(
                    i32::try_from(element.size()).expect("type size too large for i32"),
                    "a0",
                );

                self.pop_t0();
                self.emit_line(format_args!("add a0, t0, a0"));
            }
            TypedExprKind::StringLiteral { bytes, .. } => {
                let label = self.add_string_literal(bytes);
                self.emit_line(format_args!("la a0, {label}"));
            }
            _ => unreachable!("semantic analysis should reject non-lvalue expression"),
        }
    }

    fn emit_load_from_addr(&mut self, ty: &Type) {
        match ty {
            Type::Char | Type::UnsignedChar => self.emit_line(format_args!("lbu a0, 0(a0)")),
            Type::SignedChar => self.emit_line(format_args!("lb a0, 0(a0)")),
            Type::Int | Type::UnsignedInt | Type::Pointer(_) | Type::Long | Type::UnsignedLong => {
                self.emit_line(format_args!("lw a0, 0(a0)"));
            }
            Type::Array { .. } => unreachable!("array rvalues should have decayed before codegen"),
            Type::IncompleteArray { .. } => {
                unreachable!("incomplete arrays should have been handled before codegen")
            }
            Type::Function(_) => unreachable!("function values should have decayed before codegen"),
            Type::Void => unreachable!("void types should be rejected before codegen"),
        }
    }

    fn emit_store_to_addr(&mut self, ty: &Type) {
        self.emit_store_to_base_offset(ty, "t0", 0);
    }

    fn emit_store_to_base_offset(&mut self, ty: &Type, base: &str, offset: i32) {
        match ty {
            Type::Char | Type::UnsignedChar | Type::SignedChar => {
                self.emit_narrow_to_type(ty);
                self.emit_line(format_args!("sb a0, {offset}({base})"));
            }
            Type::Int | Type::UnsignedInt | Type::Pointer(_) | Type::Long | Type::UnsignedLong => {
                self.emit_line(format_args!("sw a0, {offset}({base})"));
            }
            Type::Array { .. } => unreachable!("array rvalues should have decayed before codegen"),
            Type::IncompleteArray { .. } => {
                unreachable!("incomplete arrays should have been handled before codegen")
            }
            Type::Function(_) => unreachable!("function values should have decayed before codegen"),
            Type::Void => unreachable!("void types should be rejected before codegen"),
        }
    }

    fn emit_logical_and(&mut self, left: &TypedExpr, right: &TypedExpr) {
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

    fn emit_logical_or(&mut self, left: &TypedExpr, right: &TypedExpr) {
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

    #[allow(clippy::too_many_lines)]
    fn emit_binary_op(&mut self, op: BinaryOp, ty: &Type) {
        match op {
            BinaryOp::Add => self.emit_line(format_args!("add a0, t0, a0")),
            BinaryOp::Subtract => self.emit_line(format_args!("sub a0, t0, a0")),
            BinaryOp::Multiply => self.emit_line(format_args!("mul a0, t0, a0")),
            BinaryOp::Divide => match ty {
                Type::Int | Type::Char | Type::Long | Type::SignedChar => {
                    self.emit_line(format_args!("div a0, t0, a0"));
                }
                Type::UnsignedInt | Type::UnsignedLong | Type::UnsignedChar => {
                    self.emit_line(format_args!("divu a0, t0, a0"));
                }
                Type::Pointer(_) => {
                    unreachable!("pointer arithmetic should be handled before emit_binary_op")
                }
                Type::Array { .. } => {
                    unreachable!("array rvalues should have decayed before codegen")
                }
                Type::IncompleteArray { .. } => {
                    unreachable!("incomplete arrays should have been handled before codegen")
                }
                Type::Function(_) => {
                    unreachable!("function values should have decayed before codegen")
                }
                Type::Void => unreachable!("void types should be rejected before codegen"),
            },
            BinaryOp::Remainder => match ty {
                Type::Int | Type::Char | Type::Long | Type::SignedChar => {
                    self.emit_line(format_args!("rem a0, t0, a0"));
                }
                Type::UnsignedInt | Type::UnsignedLong | Type::UnsignedChar => {
                    self.emit_line(format_args!("remu a0, t0, a0"));
                }
                Type::Pointer(_) => {
                    unreachable!("pointer arithmetic should be handled before emit_binary_op")
                }
                Type::Array { .. } => {
                    unreachable!("array rvalues should have decayed before codegen")
                }
                Type::IncompleteArray { .. } => {
                    unreachable!("incomplete arrays should have been handled before codegen")
                }
                Type::Function(_) => {
                    unreachable!("function values should have decayed before codegen")
                }
                Type::Void => unreachable!("void types should be rejected before codegen"),
            },
            BinaryOp::Equal => {
                self.emit_line(format_args!("xor a0, t0, a0"));
                self.emit_line(format_args!("seqz a0, a0"));
            }
            BinaryOp::NotEqual => {
                self.emit_line(format_args!("xor a0, t0, a0"));
                self.emit_line(format_args!("snez a0, a0"));
            }
            BinaryOp::Less => match ty {
                Type::Int | Type::Char | Type::Long | Type::SignedChar => {
                    self.emit_line(format_args!("slt a0, t0, a0"));
                }
                Type::UnsignedInt | Type::UnsignedLong | Type::UnsignedChar => {
                    self.emit_line(format_args!("sltu a0, t0, a0"));
                }
                Type::Pointer(_) => {
                    unreachable!("pointer arithmetic should be handled before emit_binary_op")
                }
                Type::Array { .. } => {
                    unreachable!("array rvalues should have decayed before codegen")
                }
                Type::IncompleteArray { .. } => {
                    unreachable!("incomplete arrays should have been handled before codegen")
                }
                Type::Function(_) => {
                    unreachable!("function values should have decayed before codegen")
                }
                Type::Void => unreachable!("void types should be rejected before codegen"),
            },
            BinaryOp::LessEqual => match ty {
                Type::Int | Type::Char | Type::Long | Type::SignedChar => {
                    self.emit_line(format_args!("slt a0, a0, t0"));
                    self.emit_line(format_args!("xori a0, a0, 1"));
                }
                Type::UnsignedInt | Type::UnsignedLong | Type::UnsignedChar => {
                    self.emit_line(format_args!("sltu a0, a0, t0"));
                    self.emit_line(format_args!("xori a0, a0, 1"));
                }
                Type::Pointer(_) => {
                    unreachable!("pointer arithmetic should be handled before emit_binary_op")
                }
                Type::Array { .. } => {
                    unreachable!("array rvalues should have decayed before codegen")
                }
                Type::IncompleteArray { .. } => {
                    unreachable!("incomplete arrays should have been handled before codegen")
                }
                Type::Function(_) => {
                    unreachable!("function values should have decayed before codegen")
                }
                Type::Void => unreachable!("void types should be rejected before codegen"),
            },
            BinaryOp::Greater => match ty {
                Type::Int | Type::Char | Type::Long | Type::SignedChar => {
                    self.emit_line(format_args!("slt a0, a0, t0"));
                }
                Type::UnsignedInt | Type::UnsignedLong | Type::UnsignedChar => {
                    self.emit_line(format_args!("sltu a0, a0, t0"));
                }
                Type::Pointer(_) => {
                    unreachable!("pointer arithmetic should be handled before emit_binary_op")
                }
                Type::Array { .. } => {
                    unreachable!("array rvalues should have decayed before codegen")
                }
                Type::IncompleteArray { .. } => {
                    unreachable!("incomplete arrays should have been handled before codegen")
                }
                Type::Function(_) => {
                    unreachable!("function values should have decayed before codegen")
                }
                Type::Void => unreachable!("void types should be rejected before codegen"),
            },
            BinaryOp::GreaterEqual => match ty {
                Type::Int | Type::Char | Type::Long | Type::SignedChar => {
                    self.emit_line(format_args!("slt a0, t0, a0"));
                    self.emit_line(format_args!("xori a0, a0, 1"));
                }
                Type::UnsignedInt | Type::UnsignedLong | Type::UnsignedChar => {
                    self.emit_line(format_args!("sltu a0, t0, a0"));
                    self.emit_line(format_args!("xori a0, a0, 1"));
                }
                Type::Pointer(_) => {
                    unreachable!("pointer arithmetic should be handled before emit_binary_op")
                }
                Type::Array { .. } => {
                    unreachable!("array rvalues should have decayed before codegen")
                }
                Type::IncompleteArray { .. } => {
                    unreachable!("incomplete arrays should have been handled before codegen")
                }
                Type::Function(_) => {
                    unreachable!("function values should have decayed before codegen")
                }
                Type::Void => unreachable!("void types should be rejected before codegen"),
            },
            BinaryOp::BitAnd => self.emit_line(format_args!("and a0, a0, t0")),
            BinaryOp::BitXor => self.emit_line(format_args!("xor a0, a0, t0")),
            BinaryOp::BitOr => self.emit_line(format_args!("or a0, a0, t0")),
            BinaryOp::ShiftLeft => self.emit_line(format_args!("sll a0, t0, a0")),
            BinaryOp::ShiftRight => match ty {
                Type::Int | Type::Char | Type::Long | Type::SignedChar => {
                    self.emit_line(format_args!("sra a0, t0, a0"));
                }
                Type::UnsignedInt | Type::UnsignedLong | Type::UnsignedChar => {
                    self.emit_line(format_args!("srl a0, t0, a0"));
                }
                Type::Pointer(_) => {
                    unreachable!("pointer arithmetic should be handled before emit_binary_op")
                }
                Type::Array { .. } => {
                    unreachable!("array rvalues should have decayed before codegen")
                }
                Type::IncompleteArray { .. } => {
                    unreachable!("incomplete arrays should have been handled before codegen")
                }
                Type::Function(_) => {
                    unreachable!("function values should have decayed before codegen")
                }
                Type::Void => unreachable!("void types should be rejected before codegen"),
            },
            BinaryOp::LogicalAnd | BinaryOp::LogicalOr => unreachable!(),
        }
    }

    fn emit_pointer_binary_op(
        &mut self,
        op: BinaryOp,
        pointee_ty: &Type,
        left_type: &Type,
        right_type: &Type,
    ) {
        let scale = i32::try_from(pointee_ty.size()).expect("type size exceeds i32");

        match (op, left_type, right_type) {
            (BinaryOp::Add, Type::Pointer(_), integer) if integer.is_integer() => {
                self.scale_reg(scale, "a0");
                self.emit_line(format_args!("add a0, t0, a0"));
            }
            (BinaryOp::Add, integer, Type::Pointer(_)) if integer.is_integer() => {
                self.scale_reg(scale, "t0");
                self.emit_line(format_args!("add a0, t0, a0"));
            }
            (BinaryOp::Subtract, Type::Pointer(_), integer) if integer.is_integer() => {
                self.scale_reg(scale, "a0");
                self.emit_line(format_args!("sub a0, t0, a0"));
            }
            _ => unreachable!("sema should reject invalid pointer arithmetic"),
        }
    }

    fn emit_binary(
        &mut self,
        op: BinaryOp,
        operand_ty: &Type,
        left: &TypedExpr,
        right: &TypedExpr,
    ) {
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
        self.push_a0();
        self.emit_expr(right);
        self.pop_t0();

        if let Type::Pointer(pointee_ty) = operand_ty
            && matches!(op, BinaryOp::Add | BinaryOp::Subtract)
        {
            self.emit_pointer_binary_op(op, pointee_ty, &left.ty, &right.ty);
            return;
        }

        self.emit_binary_op(op, operand_ty);
    }

    fn emit_compound_assign(
        &mut self,
        target: &TypedExpr,
        op: BinaryOp,
        operand_ty: &Type,
        value: &TypedExpr,
    ) {
        self.emit_addr(target);
        self.push_a0();
        self.emit_load_from_addr(&target.ty);
        self.push_a0();

        self.emit_expr(value);
        self.pop_t0();

        if let Type::Pointer(pointee_ty) = operand_ty
            && matches!(op, BinaryOp::Add | BinaryOp::Subtract)
        {
            self.emit_pointer_binary_op(op, pointee_ty, &target.ty, &value.ty);
        } else {
            self.emit_binary_op(op, operand_ty);
        }
        self.pop_t0();
        self.emit_store_to_addr(&target.ty);
    }

    fn emit_inc_dec(&mut self, expr: &TypedExpr, delta: i32, postfix: bool) {
        self.emit_addr(expr);
        self.emit_line(format_args!("mv t0, a0"));
        self.emit_load_from_addr(&expr.ty);

        if postfix {
            self.push_a0();
        }

        let mut accumulator = delta;
        if let Type::Pointer(inner) = &expr.ty {
            let size = i32::try_from(inner.size()).expect("type size exceeds i32");
            accumulator *= size;
        }

        self.emit_line(format_args!("addi a0, a0, {accumulator}"));
        self.emit_store_to_addr(&expr.ty);

        if postfix {
            self.pop_a0();
        }
    }

    fn emit_cast_to(&mut self, target_ty: &Type) {
        match target_ty {
            Type::Char | Type::SignedChar => {
                // Shifting left 24 bits, then right 24 bits sign-extends low byte
                self.emit_line(format_args!("slli a0, a0, 24"));
                self.emit_line(format_args!("srai a0, a0, 24"));
            }
            Type::UnsignedChar => {
                // Zeros out top 3 bytes
                self.emit_line(format_args!("andi a0, a0, 255"));
            }
            _ => (),
        }
    }

    fn emit_call_target(&mut self, callee: &TypedExpr) {
        match &callee.kind {
            TypedExprKind::Unary {
                op: UnaryOp::Dereference,
                expr,
                ..
            } if matches!(callee.ty, Type::Function(_)) => {
                self.emit_expr(expr);
            }
            _ => self.emit_expr(callee),
        }
    }

    fn emit_expr(&mut self, expr: &TypedExpr) {
        match &expr.kind {
            TypedExprKind::IntLiteral { value, .. } => {
                self.emit_line(format_args!("li a0, {value}"));
            }
            TypedExprKind::Binary {
                op,
                operand_ty,
                left,
                right,
                ..
            } => self.emit_binary(*op, operand_ty, left, right),
            TypedExprKind::Variable { .. } => {
                unreachable!("lvalue variable should be converted before value codegen")
            }
            TypedExprKind::Unary {
                op: UnaryOp::AddressOf,
                expr: operand,
                ..
            } => {
                self.emit_addr(operand);
            }
            TypedExprKind::Unary {
                op, expr: operand, ..
            } => {
                self.emit_expr(operand);
                match op {
                    UnaryOp::Negate => self.emit_line(format_args!("neg a0, a0")),
                    UnaryOp::LogicalNot => self.emit_line(format_args!("seqz a0, a0")),
                    UnaryOp::BitwiseNot => self.emit_line(format_args!("not a0, a0")),
                    UnaryOp::Dereference => self.emit_load_from_addr(&expr.ty),
                    UnaryOp::AddressOf => unreachable!("handled above"),
                }
            }
            TypedExprKind::Call { callee, args, .. } => {
                if let TypedExprKind::FunctionDesignator { name, .. } = &callee.kind {
                    self.emit_call_args(args);
                    self.emit_line(format_args!("call {name}"));
                } else {
                    self.emit_call_target(callee);
                    self.push_a0();

                    self.emit_call_args(args);

                    self.pop_t0();
                    self.emit_line(format_args!("jalr ra, 0(t0)"));
                }
            }
            TypedExprKind::Assign { target, value, .. } => {
                self.emit_addr(target);
                self.push_a0();

                self.emit_expr(value);

                self.pop_t0();
                self.emit_store_to_addr(&target.ty);
            }
            TypedExprKind::CompoundAssign {
                target,
                op,
                operand_ty,
                value,
                ..
            } => {
                self.emit_compound_assign(target, *op, operand_ty, value);
            }
            TypedExprKind::PrefixInc { expr, .. } => self.emit_inc_dec(expr, 1, false),
            TypedExprKind::PrefixDec { expr, .. } => self.emit_inc_dec(expr, -1, false),
            TypedExprKind::PostfixInc { expr, .. } => self.emit_inc_dec(expr, 1, true),
            TypedExprKind::PostfixDec { expr, .. } => self.emit_inc_dec(expr, -1, true),
            TypedExprKind::Index { .. } => {
                self.emit_addr(expr);
                self.emit_load_from_addr(&expr.ty);
            }
            TypedExprKind::Cast {
                target_ty, expr, ..
            } => {
                self.emit_expr(expr);
                self.emit_cast_to(target_ty);
            }
            TypedExprKind::FunctionDesignator { name, .. } => {
                self.emit_line(format_args!("la a0, {name}"));
            }
            TypedExprKind::FunctionToPointer { expr, .. } => match &expr.kind {
                TypedExprKind::FunctionDesignator { name, .. } => {
                    self.emit_line(format_args!("la a0, {name}"));
                }
                _ => self.emit_expr(expr),
            },
            TypedExprKind::ArrayToPointer { expr, .. } => {
                self.emit_addr(expr);
            }
            TypedExprKind::LvalueToRvalue { expr, .. } => {
                self.emit_addr(expr);
                self.emit_load_from_addr(&expr.ty);
            }
            TypedExprKind::StringLiteral { .. } => {
                self.emit_addr(expr);
            }
        }
    }

    fn emit_if_statement(
        &mut self,
        cond: &TypedExpr,
        then_branch: &TypedStatement,
        else_branch: Option<&TypedStatement>,
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
        }
        self.emit_label(&end_label);
    }

    fn emit_while_statement(&mut self, cond: &TypedExpr, body: &TypedStatement) {
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

    fn emit_do_while_statement(&mut self, body: &TypedStatement, cond: &TypedExpr) {
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
        init: Option<&TypedStatement>,
        cond: Option<&TypedExpr>,
        post: Option<&TypedExpr>,
        body: &TypedStatement,
    ) {
        let start_label = self.new_label("for_start");
        let continue_label = self.new_label("for_continue");
        let break_label = self.new_label("for_break");

        self.push_loop_labels(&continue_label, &break_label);

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

        self.pop_loop_labels();
    }

    fn emit_statement(&mut self, statement: &TypedStatement) {
        match statement {
            TypedStatement::Return(expr) => {
                if let Some(expr) = expr {
                    self.emit_expr(expr);
                    let return_type = self
                        .current_function_return_type
                        .clone()
                        .expect("codegen should have a function return type");
                    self.emit_narrow_to_type(&return_type);
                }
                let return_label = self
                    .return_label
                    .clone()
                    .expect("codegen should have an active return label");
                self.emit_line(format_args!("j {return_label}"));
            }
            TypedStatement::Decl(declarations) => {
                for declaration in declarations {
                    if let Some(initializer) = &declaration.init {
                        match initializer {
                            TypedInitializer::Expr(init_expr) => {
                                self.emit_expr(init_expr);
                                let slot = self.resolve_local(declaration.id).clone();
                                self.emit_store_local(&slot);
                            }
                            TypedInitializer::List(values) => {
                                let (element_ty, array_len, offset) = {
                                    let slot = self.resolve_local(declaration.id);

                                    let Type::Array { element, len } = &slot.ty else {
                                        unreachable!("sema guarantees that this is an array");
                                    };

                                    ((*element).clone(), *len, slot.offset)
                                };

                                for (i, expr) in values.iter().enumerate() {
                                    self.emit_expr(expr);
                                    self.emit_store_to_base_offset(
                                        &element_ty,
                                        "s0",
                                        offset
                                            + i32::try_from(i * element_ty.size())
                                                .expect("type size too large for i32"),
                                    );
                                }

                                // Zero-initialize remaining elements
                                for i in values.len()..array_len {
                                    self.emit_line(format_args!("li a0, 0"));
                                    self.emit_store_to_base_offset(
                                        &element_ty,
                                        "s0",
                                        offset
                                            + i32::try_from(i * element_ty.size())
                                                .expect("type size too large for i32"),
                                    );
                                }
                            }
                        }
                    }
                }
            }
            TypedStatement::Block(body) => {
                for statement in body {
                    self.emit_statement(statement);
                }
            }
            TypedStatement::If {
                cond,
                then_branch,
                else_branch,
            } => self.emit_if_statement(cond, then_branch, else_branch.as_deref()),
            TypedStatement::While { cond, body } => self.emit_while_statement(cond, body),
            TypedStatement::DoWhile { body, cond } => self.emit_do_while_statement(body, cond),
            TypedStatement::Empty => (),
            TypedStatement::ExprStatement(expr) => self.emit_expr(expr),
            TypedStatement::Break { .. } => {
                if let Some(label) = self.current_break_label() {
                    self.emit_line(format_args!("j {label}"));
                }
            }
            TypedStatement::Continue { .. } => {
                if let Some(label) = self.current_continue_label() {
                    self.emit_line(format_args!("j {label}"));
                }
            }
            TypedStatement::For {
                init,
                cond,
                post,
                body,
            } => self.emit_for_statement(init.as_deref(), cond.as_ref(), post.as_ref(), body),
        }
    }

    fn emit_prologue(&mut self, name: &str, linkage: Linkage) {
        let frame_size = self.frame.size;
        if linkage == Linkage::External {
            self.emit(format_args!(".globl {name}\n"));
        }
        self.emit_label(name);
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

    fn emit_function(&mut self, function: &TypedFunction) {
        self.reset_for_function(function);
        self.emit_prologue(&function.name, function.linkage);

        for (i, param) in function.params.iter().enumerate() {
            let slot = self.resolve_local(param.id).clone();
            self.emit_store_param(i, &slot);
        }

        self.current_function_return_type = Some(function.return_type.clone());
        for statement in &function.body {
            self.emit_statement(statement);
        }
        self.current_function_return_type = None;
        self.emit_epilogue();
    }

    fn collect_globals(&mut self, program: &TypedProgram) {
        for decl in &program.declarations {
            if let TypedExternalDecl::Global(global) = decl {
                self.globals.insert(
                    global.id,
                    GlobalSlot {
                        label: global.name.clone(),
                        ty: global.ty.clone(),
                    },
                );
            }
        }
    }

    fn emit_globals(&mut self, program: &TypedProgram) {
        let mut emitted_section = false;

        for decl in &program.declarations {
            if let TypedExternalDecl::Global(global) = decl {
                if !emitted_section {
                    self.emit(format_args!(".data\n"));
                    emitted_section = true;
                }
                self.emit_global(global);
            }
        }

        if emitted_section {
            self.emit(format_args!(".text\n"));
        }
    }

    fn emit_program(&mut self, program: &TypedProgram) -> String {
        self.collect_globals(program);
        self.emit_globals(program);

        for decl in &program.declarations {
            if let TypedExternalDecl::Function(function) = decl {
                self.emit_function(function);
            }
        }

        self.emit_string_data();

        std::mem::take(&mut self.out)
    }
}

#[must_use]
pub fn generate(program: &TypedProgram, target: CodegenTarget) -> String {
    let mut codegen = Codegen::new(target);
    codegen.emit_program(program)
}

fn global_init_int_value(init: Option<&TypedInitializer>) -> u64 {
    match init {
        None => 0,
        Some(TypedInitializer::Expr(expr)) => expr
            .eval_int_constant_expr()
            .expect("global initializer should be integer constant"),
        Some(TypedInitializer::List(_)) => {
            unreachable!("scalar global initializer cannot be list after sema")
        }
    }
}
