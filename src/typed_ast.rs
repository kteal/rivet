use crate::ast::{BinaryOp, Type, UnaryOp};
use crate::source::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedProgram {
    pub declarations: Vec<TypedExternalDecl>,
    pub eof_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedExternalDecl {
    Function(TypedFunction),
    Typedef,
    Global(TypedGlobalDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedFunction {
    pub return_type: Type,
    pub name: String,
    pub name_span: Span,
    pub params: Vec<TypedParam>,
    pub body: Vec<TypedStatement>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedGlobalDecl {
    pub id: GlobalId,
    pub ty: Type,
    pub name: String,
    pub name_span: Span,
    pub init: Option<TypedInitializer>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedParam {
    pub id: LocalId,
    pub ty: Type,
    pub name: String,
    pub name_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedLocalDecl {
    pub id: LocalId,
    pub ty: Type,
    pub name: String,
    pub name_span: Span,
    pub init: Option<TypedInitializer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedStatement {
    Decl(Vec<TypedLocalDecl>),
    Return(TypedExpr),
    Block(Vec<Self>),
    If {
        cond: TypedExpr,
        then_branch: Box<Self>,
        else_branch: Option<Box<Self>>,
    },
    While {
        cond: TypedExpr,
        body: Box<Self>,
    },
    DoWhile {
        body: Box<Self>,
        cond: TypedExpr,
    },
    ExprStatement(TypedExpr),
    Empty,
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    For {
        init: Option<Box<Self>>, // VarDecl, Assign, ExprStatement, Empty
        cond: Option<TypedExpr>,
        post: Option<TypedExpr>,
        body: Box<Self>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedInitializer {
    Expr(TypedExpr),
    List(Vec<TypedExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedExpr {
    pub kind: TypedExprKind,
    pub ty: Type,
}

impl TypedExpr {
    #[must_use]
    pub const fn diagnostic_span(&self) -> Span {
        match &self.kind {
            TypedExprKind::IntLiteral { span, .. }
            | TypedExprKind::Variable { span, .. }
            | TypedExprKind::Index { span, .. }
            | TypedExprKind::Cast { span, .. }
            | TypedExprKind::Call { span, .. }
            | TypedExprKind::FunctionToPointer { span, .. }
            | TypedExprKind::ArrayToPointer { span, .. }
            | TypedExprKind::LvalueToRvalue { span, .. } => *span,
            TypedExprKind::Unary { op_span, .. }
            | TypedExprKind::Binary { op_span, .. }
            | TypedExprKind::Assign { op_span, .. }
            | TypedExprKind::CompoundAssign { op_span, .. }
            | TypedExprKind::PrefixInc { op_span, .. }
            | TypedExprKind::PrefixDec { op_span, .. }
            | TypedExprKind::PostfixInc { op_span, .. }
            | TypedExprKind::PostfixDec { op_span, .. } => *op_span,
            TypedExprKind::FunctionDesignator { name_span, .. } => *name_span,
        }
    }

    #[must_use]
    pub fn is_null_pointer_constant(&self) -> bool {
        self.ty.is_integer() && self.eval_int_constant_expr() == Some(0)
    }

    #[must_use]
    pub fn eval_int_constant_expr(&self) -> Option<u64> {
        match &self.kind {
            TypedExprKind::IntLiteral { value, .. } => Some(*value),
            TypedExprKind::Binary {
                op, left, right, ..
            } => {
                if let Some(l) = left.eval_int_constant_expr()
                    && let Some(r) = right.eval_int_constant_expr()
                {
                    match op {
                        BinaryOp::Add => Some(l + r),
                        BinaryOp::Subtract => Some(l - r),
                        BinaryOp::Multiply => Some(l * r),
                        BinaryOp::Divide if r != 0 => Some(l / r),
                        BinaryOp::Remainder if r != 0 => Some(l % r),
                        BinaryOp::BitAnd => Some(l & r),
                        BinaryOp::BitOr => Some(l | r),
                        BinaryOp::BitXor => Some(l ^ r),
                        BinaryOp::Equal => Some(u64::from(l == r)),
                        BinaryOp::NotEqual => Some(u64::from(l != r)),
                        BinaryOp::Greater => Some(u64::from(l > r)),
                        BinaryOp::GreaterEqual => Some(u64::from(l >= r)),
                        BinaryOp::Less => Some(u64::from(l < r)),
                        BinaryOp::LessEqual => Some(u64::from(l <= r)),
                        BinaryOp::LogicalAnd => Some(u64::from(l != 0 && r != 0)),
                        BinaryOp::LogicalOr => Some(u64::from(l != 0 || r != 0)),
                        BinaryOp::ShiftLeft if (0..32).contains(&r) => Some(l << r),
                        BinaryOp::ShiftRight if (0..32).contains(&r) => Some(l >> r),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedExprKind {
    IntLiteral {
        value: u64,
        span: Span,
    },
    Variable {
        id: ObjectId,
        name: String,
        span: Span,
    },
    Binary {
        op: BinaryOp,
        op_span: Span,
        operand_ty: Type,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
    },
    Unary {
        op: UnaryOp,
        op_span: Span,
        expr: Box<TypedExpr>,
    },
    Call {
        callee: Box<TypedExpr>,
        args: Vec<TypedExpr>,
        span: Span,
    },
    Assign {
        target: Box<TypedExpr>,
        op_span: Span,
        value: Box<TypedExpr>,
    },
    CompoundAssign {
        target: Box<TypedExpr>,
        op: BinaryOp,
        op_span: Span,
        operand_ty: Type,
        value: Box<TypedExpr>,
    },
    PrefixInc {
        expr: Box<TypedExpr>,
        op_span: Span,
    },
    PrefixDec {
        expr: Box<TypedExpr>,
        op_span: Span,
    },
    PostfixInc {
        expr: Box<TypedExpr>,
        op_span: Span,
    },
    PostfixDec {
        expr: Box<TypedExpr>,
        op_span: Span,
    },
    Index {
        base: Box<TypedExpr>,
        index: Box<TypedExpr>,
        span: Span,
    },
    Cast {
        target_ty: Type,
        expr: Box<TypedExpr>,
        span: Span,
    },
    FunctionDesignator {
        name: String,
        name_span: Span,
    },
    FunctionToPointer {
        expr: Box<TypedExpr>,
        span: Span,
    },
    ArrayToPointer {
        expr: Box<TypedExpr>,
        span: Span,
    },
    LvalueToRvalue {
        expr: Box<TypedExpr>,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectId {
    Local(LocalId),
    Global(GlobalId),
}
