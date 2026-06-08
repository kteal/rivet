use crate::ast::{BinaryOp, Type, UnaryOp};
use crate::lexer::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedProgram {
    pub functions: Vec<TypedFunction>,
    pub eof_span: Span,
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
pub struct TypedParam {
    pub id: LocalId,
    pub ty: Type,
    pub name: String,
    pub name_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedStatement {
    VarDecl {
        id: LocalId,
        ty: Type,
        name: String,
        name_span: Span,
        init: Option<TypedExpr>,
    },
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
pub struct TypedExpr {
    pub kind: TypedExprKind,
    pub ty: Type,
}

impl TypedExpr {
    #[must_use]
    pub const fn diagnostic_span(&self) -> Span {
        match &self.kind {
            TypedExprKind::IntLiteral { span, .. } | TypedExprKind::Variable { span, .. } => *span,
            TypedExprKind::Unary { op_span, .. }
            | TypedExprKind::Binary { op_span, .. }
            | TypedExprKind::Assign { op_span, .. }
            | TypedExprKind::CompoundAssign { op_span, .. }
            | TypedExprKind::PrefixInc { op_span, .. }
            | TypedExprKind::PrefixDec { op_span, .. }
            | TypedExprKind::PostfixInc { op_span, .. }
            | TypedExprKind::PostfixDec { op_span, .. } => *op_span,
            TypedExprKind::Call { name_span, .. } => *name_span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedExprKind {
    IntLiteral {
        value: i32,
        span: Span,
    },
    Variable {
        id: LocalId,
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
        name: String,
        name_span: Span,
        args: Vec<TypedExpr>,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub usize);
