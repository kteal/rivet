use crate::source::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub declarations: Vec<ExternalDecl>,
    pub eof_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalDecl {
    FunctionDecl(FunctionDecl),
    FunctionDef(FunctionDef),
    Typedef(Typedef),
    Global(GlobalDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Typedef {
    pub name: String,
    pub name_span: Span,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionType {
    pub return_type: Box<Type>,
    pub params: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDef {
    pub return_type: Type,
    pub name: String,
    pub name_span: Span,
    pub params: Vec<Param>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDecl {
    pub return_type: Type,
    pub name: String,
    pub name_span: Span,
    pub params: Vec<ParamDecl>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalDecl {
    pub ty: Type,
    pub name: String,
    pub name_span: Span,
    pub init: Option<Initializer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParamDecl {
    pub ty: Type,
    pub name: Option<String>,
    pub name_span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub ty: Type,
    pub name: String,
    pub name_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalDecl {
    pub ty: Type,
    pub name: String,
    pub name_span: Span,
    pub init: Option<Initializer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Decl(Vec<LocalDecl>),
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
        value: u64,
        suffix: IntLiteralSuffix,
        base: IntLiteralBase,
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
        callee: Box<Self>,
        args: Vec<Self>,
        span: Span,
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
    Cast {
        ty: Type,
        expr: Box<Self>,
        span: Span,
    },
    SizeOfType {
        ty: Type,
        span: Span,
    },
    SizeOfExpr {
        expr: Box<Self>,
        span: Span,
    },
    StringLiteral {
        bytes: Vec<u8>,
        span: Span,
    },
}

impl Expr {
    #[must_use]
    pub const fn diagnostic_span(&self) -> Span {
        match self {
            Self::IntLiteral { span, .. }
            | Self::Variable { span, .. }
            | Self::Index { span, .. }
            | Self::Cast { span, .. }
            | Self::Call { span, .. }
            | Self::SizeOfType { span, .. }
            | Self::SizeOfExpr { span, .. }
            | Self::StringLiteral { span, .. } => *span,
            Self::Unary { op_span, .. }
            | Self::Binary { op_span, .. }
            | Self::Assign { op_span, .. }
            | Self::CompoundAssign { op_span, .. }
            | Self::PrefixInc { op_span, .. }
            | Self::PrefixDec { op_span, .. }
            | Self::PostfixInc { op_span, .. }
            | Self::PostfixDec { op_span, .. } => *op_span,
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
    AddressOf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntLiteralSuffix {
    None,
    Unsigned,
    Long,
    UnsignedLong,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntLiteralBase {
    Decimal,
    Hex,
}

impl IntLiteralBase {
    #[must_use]
    pub const fn radix(&self) -> u32 {
        match self {
            Self::Decimal => 10,
            Self::Hex => 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    UnsignedInt,
    Char,
    SignedChar,
    UnsignedChar,
    Long,
    UnsignedLong,
    Pointer(Box<Self>),
    Array { element: Box<Self>, len: usize },
    Function(Box<FunctionType>),
}

impl Type {
    #[must_use]
    pub const fn is_integer(&self) -> bool {
        matches!(
            self,
            Self::Int
                | Self::Char
                | Self::UnsignedChar
                | Self::SignedChar
                | Self::UnsignedInt
                | Self::Long
                | Self::UnsignedLong
        )
    }

    #[must_use]
    pub const fn is_pointer(&self) -> bool {
        matches!(self, Self::Pointer(_))
    }

    #[must_use]
    /// Returns the size of values of this type in bytes.
    ///
    /// # Panics
    ///
    /// Panics for function types because functions are not object values.
    pub fn size(&self) -> usize {
        match self {
            Self::Char | Self::SignedChar | Self::UnsignedChar => 1,
            Self::Int | Self::UnsignedInt | Self::Pointer(_) | Self::Long | Self::UnsignedLong => 4,
            Self::Array { element, len } => element.size() * len,
            Self::Function(_) => panic!("cannot calculate size of function type"),
        }
    }

    #[must_use]
    /// Returns the required alignment of objects of this type in bytes.
    ///
    /// # Panics
    ///
    /// Panics for function types because functions are not object values.
    pub fn align(&self) -> usize {
        match self {
            Self::Char
            | Self::UnsignedChar
            | Self::SignedChar
            | Self::Int
            | Self::UnsignedInt
            | Self::Pointer(_)
            | Self::Long
            | Self::UnsignedLong => self.size(),
            Self::Array { element, .. } => element.align(),
            Self::Function(_) => panic!("cannot calculate alignment of function type"),
        }
    }

    #[must_use]
    pub fn is_assignable_from(&self, value: &Self) -> bool {
        self == value || (self.is_integer() && value.is_integer())
    }

    #[must_use]
    pub const fn promoted(&self) -> Option<Self> {
        match self {
            Self::Char | Self::UnsignedChar | Self::SignedChar | Self::Int => Some(Self::Int),
            Self::UnsignedInt => Some(Self::UnsignedInt),
            Self::Long => Some(Self::Long),
            Self::UnsignedLong => Some(Self::UnsignedLong),
            Self::Pointer(_) | Self::Array { .. } | Self::Function(_) => None,
        }
    }

    /// Returns the common type after integer promotions and arithmetic conversions.
    ///
    /// # Panics
    ///
    /// Panics if either argument is not an integer type.
    #[must_use]
    pub fn usual_arithmetic_type(left: &Self, right: &Self) -> Self {
        let left = left.promoted().expect("left must be integer");
        let right = right.promoted().expect("right must be integer");

        if (left == Self::UnsignedLong || right == Self::UnsignedLong)
            || ((left == Self::Long || right == Self::Long)
                && (left == Self::UnsignedInt || right == Self::UnsignedInt))
        {
            Self::UnsignedLong
        } else if left == Self::Long || right == Self::Long {
            Self::Long
        } else if left == Self::UnsignedInt || right == Self::UnsignedInt {
            Self::UnsignedInt
        } else {
            Self::Int
        }
    }
}
