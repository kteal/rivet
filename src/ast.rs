use crate::lexer::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<Function>,
    pub eof_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub return_type: Type,
    pub name: String,
    pub name_span: Span,
    pub params: Vec<Param>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub ty: Type,
    pub name: String,
    pub name_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    VarDecl {
        ty: Type,
        name: String,
        name_span: Span,
        init: Option<Expr>,
    },
    Return(Expr),
    Block(Vec<Statement>),
    If {
        cond: Expr,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    While {
        cond: Expr,
        body: Box<Statement>,
    },
    DoWhile {
        body: Box<Statement>,
        cond: Expr,
    },
    ExprStatement(Expr),
    Empty,
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    For {
        init: Option<Box<Statement>>, // VarDecl, Assign, ExprStatement, Empty
        cond: Option<Expr>,
        post: Option<Expr>,
        body: Box<Statement>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    IntLiteral {
        value: i32,
        span: Span,
    },
    Variable {
        name: String,
        span: Span,
    },
    Binary {
        op: BinaryOp,
        op_span: Span,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        op_span: Span,
        expr: Box<Expr>,
    },
    Call {
        name: String,
        name_span: Span,
        args: Vec<Expr>,
    },
    Assign {
        target: Box<Expr>,
        op_span: Span,
        value: Box<Expr>,
    },
    CompoundAssign {
        target: Box<Expr>,
        op: BinaryOp,
        op_span: Span,
        value: Box<Expr>,
    },
    PrefixInc {
        expr: Box<Expr>,
        op_span: Span,
    },
    PrefixDec {
        expr: Box<Expr>,
        op_span: Span,
    },
    PostfixInc {
        expr: Box<Expr>,
        op_span: Span,
    },
    PostfixDec {
        expr: Box<Expr>,
        op_span: Span,
    },
}

impl Expr {
    pub fn diagnostic_span(&self) -> Span {
        match self {
            Expr::IntLiteral { span, .. } => *span,
            Expr::Variable { span, .. } => *span,
            Expr::Unary { op_span, .. } => *op_span,
            Expr::Binary { op_span, .. } => *op_span,
            Expr::Call { name_span, .. } => *name_span,
            Expr::Assign { op_span, .. } => *op_span,
            Expr::CompoundAssign { op_span, .. } => *op_span,
            Expr::PrefixInc { op_span, .. } => *op_span,
            Expr::PrefixDec { op_span, .. } => *op_span,
            Expr::PostfixInc { op_span, .. } => *op_span,
            Expr::PostfixDec { op_span, .. } => *op_span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    BitAnd,
    BitXor,
    BitOr,
    ShiftLeft,
    ShiftRight,
    LogicalAnd,
    LogicalOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    LogicalNot,
    BitwiseNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Int,
    Char,
    UnsignedInt,
}
