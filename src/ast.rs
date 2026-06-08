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
        init: Option<Initializer>,
    },
    Return(Expr),
    Block(Vec<Self>),
    If {
        cond: Expr,
        then_branch: Box<Self>,
        else_branch: Option<Box<Self>>,
    },
    While {
        cond: Expr,
        body: Box<Self>,
    },
    DoWhile {
        body: Box<Self>,
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
        init: Option<Box<Self>>, // VarDecl, Assign, ExprStatement, Empty
        cond: Option<Expr>,
        post: Option<Expr>,
        body: Box<Self>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Initializer {
    Expr(Expr),
    List(Vec<Expr>),
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
        left: Box<Self>,
        right: Box<Self>,
    },
    Unary {
        op: UnaryOp,
        op_span: Span,
        expr: Box<Self>,
    },
    Call {
        name: String,
        name_span: Span,
        args: Vec<Self>,
    },
    Assign {
        target: Box<Self>,
        op_span: Span,
        value: Box<Self>,
    },
    CompoundAssign {
        target: Box<Self>,
        op: BinaryOp,
        op_span: Span,
        value: Box<Self>,
    },
    PrefixInc {
        expr: Box<Self>,
        op_span: Span,
    },
    PrefixDec {
        expr: Box<Self>,
        op_span: Span,
    },
    PostfixInc {
        expr: Box<Self>,
        op_span: Span,
    },
    PostfixDec {
        expr: Box<Self>,
        op_span: Span,
    },
    Index {
        base: Box<Self>,
        index: Box<Self>,
        span: Span,
    },
}

impl Expr {
    #[must_use]
    pub const fn diagnostic_span(&self) -> Span {
        match self {
            Self::IntLiteral { span, .. }
            | Self::Variable { span, .. }
            | Self::Index { span, .. } => *span,
            Self::Unary { op_span, .. }
            | Self::Binary { op_span, .. }
            | Self::Assign { op_span, .. }
            | Self::CompoundAssign { op_span, .. }
            | Self::PrefixInc { op_span, .. }
            | Self::PrefixDec { op_span, .. }
            | Self::PostfixInc { op_span, .. }
            | Self::PostfixDec { op_span, .. } => *op_span,
            Self::Call { name_span, .. } => *name_span,
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
    Dereference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Char,
    UnsignedInt,
    Pointer(Box<Self>),
    Array { element: Box<Self>, len: usize },
}

impl Type {
    #[must_use]
    pub const fn is_integer(&self) -> bool {
        matches!(self, Self::Int | Self::Char | Self::UnsignedInt)
    }

    #[must_use]
    pub const fn is_pointer(&self) -> bool {
        matches!(self, Self::Pointer(_))
    }

    #[must_use]
    pub fn size(&self) -> usize {
        match self {
            Self::Char => 1,
            Self::Int | Self::UnsignedInt | Self::Pointer(_) => 4,
            Self::Array { element, len } => element.size() * len,
        }
    }

    #[must_use]
    pub fn align(&self) -> usize {
        match self {
            Self::Char | Self::Int | Self::UnsignedInt | Self::Pointer(_) => self.size(),
            Self::Array { element, .. } => element.align(),
        }
    }

    #[must_use]
    pub fn is_assignable_from(&self, value: &Self) -> bool {
        self == value || (self.is_integer() && value.is_integer())
    }

    #[must_use]
    pub const fn promoted(&self) -> Option<Self> {
        match self {
            Self::Char | Self::Int => Some(Self::Int),
            Self::UnsignedInt => Some(Self::UnsignedInt),
            Self::Pointer(_) | Self::Array { .. } => None,
        }
    }

    #[must_use]
    pub fn usual_arithmetic_type(left: &Self, right: &Self) -> Self {
        if *left == Self::UnsignedInt || *right == Self::UnsignedInt {
            Self::UnsignedInt
        } else {
            Self::Int
        }
    }
}
