use std::collections::HashMap;

use crate::ast::{
    BinaryOp, Expr, ExternalDecl, FunctionDecl, FunctionDef, Initializer, IntLiteralBase,
    IntLiteralSuffix, LocalDecl, Param, ParamDecl, Program, Statement, Type, Typedef, UnaryOp,
};
use crate::lexer::{Token, TokenKind};
use crate::source::Span;

const MULTIPLICATIVE_OPS: &[(TokenKind, BinaryOp)] = &[
    (TokenKind::Star, BinaryOp::Multiply),
    (TokenKind::Slash, BinaryOp::Divide),
    (TokenKind::Percent, BinaryOp::Remainder),
];

const ADDITIVE_OPS: &[(TokenKind, BinaryOp)] = &[
    (TokenKind::Plus, BinaryOp::Add),
    (TokenKind::Minus, BinaryOp::Subtract),
];

const SHIFT_OPS: &[(TokenKind, BinaryOp)] = &[
    (TokenKind::LessLess, BinaryOp::ShiftLeft),
    (TokenKind::GreaterGreater, BinaryOp::ShiftRight),
];

const RELATIONAL_OPS: &[(TokenKind, BinaryOp)] = &[
    (TokenKind::Less, BinaryOp::Less),
    (TokenKind::LessEqual, BinaryOp::LessEqual),
    (TokenKind::Greater, BinaryOp::Greater),
    (TokenKind::GreaterEqual, BinaryOp::GreaterEqual),
];

const EQUALITY_OPS: &[(TokenKind, BinaryOp)] = &[
    (TokenKind::EqualEqual, BinaryOp::Equal),
    (TokenKind::BangEqual, BinaryOp::NotEqual),
];

const BITWISE_AND_OPS: &[(TokenKind, BinaryOp)] = &[(TokenKind::Ampersand, BinaryOp::BitAnd)];
const BITWISE_XOR_OPS: &[(TokenKind, BinaryOp)] = &[(TokenKind::Caret, BinaryOp::BitXor)];
const BITWISE_OR_OPS: &[(TokenKind, BinaryOp)] = &[(TokenKind::Pipe, BinaryOp::BitOr)];

const LOGICAL_AND_OPS: &[(TokenKind, BinaryOp)] =
    &[(TokenKind::AmpersandAmpersand, BinaryOp::LogicalAnd)];
const LOGICAL_OR_OPS: &[(TokenKind, BinaryOp)] = &[(TokenKind::PipePipe, BinaryOp::LogicalOr)];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

// Parser representation of one C declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Declaration {
    spec: DeclSpec,
    spec_span: Span,
    declarators: Vec<InitDeclarator>,
}

// The declaration-specifier sequence: storage class, type words, and qualifiers
// that appear before the declarator list.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DeclSpec {
    storage_class: Option<StorageClass>,
    type_specifiers: Vec<TypeSpecifier>,
    qualifiers: Vec<TypeQualifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StorageClass {
    Typedef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TypeSpecifier {
    Char,
    Int,
    Signed,
    Unsigned,
    Long,
    TypedefName { name: String, span: Span },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeQualifier {
    Const,
}

// One declarator inside a declaration, plus its optional initializer.
#[derive(Debug, Clone, PartialEq, Eq)]
struct InitDeclarator {
    declarator: Declarator,
    initializer: Option<Initializer>,
}

// The part of a declaration that contains the name and pointer/array shape.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Declarator {
    kind: DeclaratorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DeclaratorKind {
    Name { name: String, name_span: Span },
    Pointer(Box<Declarator>),
    Array { inner: Box<Declarator>, len: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoweredDeclarator {
    ty: Type,
    name: String,
    name_span: Span,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
struct TypeSpec {
    unsigned: bool,
    signed: bool,
    int_count: usize,
    char_count: usize,
    long_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedFunctionSignature {
    return_type: Type,
    name: String,
    name_span: Span,
    params: Vec<RawParam>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawParam {
    pub ty: Type,
    pub ty_span: Span,
    pub name: Option<String>,
    pub name_span: Option<Span>,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    typedefs: HashMap<String, Type>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            typedefs: HashMap::new(),
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_nth(&self, n: usize) -> &Token {
        self.tokens.get(self.pos + n).unwrap_or_else(|| {
            self.tokens
                .last()
                .expect("parser token stream should end with EOF")
        })
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn peek_nth_kind(&self, n: usize) -> &TokenKind {
        &self.peek_nth(n).kind
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        token
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<Token, ParseError> {
        let token = self.advance();

        if &token.kind == expected {
            Ok(token)
        } else {
            Err(ParseError {
                message: format!("expected {expected:?}, found {:?}", token.kind),
                span: token.span,
            })
        }
    }

    fn expect_ident(&mut self) -> Result<(String, Span), ParseError> {
        let token = self.advance();

        match token {
            Token {
                kind: TokenKind::Ident(name),
                span,
            } => Ok((name, span)),

            token => Err(ParseError {
                message: format!("expected identifier token, got '{:?}'", token.kind),
                span: token.span,
            }),
        }
    }

    fn expect_int_literal(
        &mut self,
    ) -> Result<(u64, IntLiteralSuffix, IntLiteralBase, Span), ParseError> {
        let token = self.advance();

        match token {
            Token {
                kind:
                    TokenKind::IntLiteral {
                        value,
                        suffix,
                        base,
                    },
                span,
            } => Ok((value, suffix, base, span)),

            token => Err(ParseError {
                message: format!("expected integer literal token, got '{:?}'", token.kind),
                span: token.span,
            }),
        }
    }

    fn parse_left_assoc(
        &mut self,
        parse_operand: fn(&mut Self) -> Result<Expr, ParseError>,
        ops: &[(TokenKind, BinaryOp)],
    ) -> Result<Expr, ParseError> {
        let mut left = parse_operand(self)?;

        while let Some((op, op_span)) = self.parse_binary_op_from(ops) {
            let right = parse_operand(self)?;
            left = Expr::Binary {
                op,
                op_span,
                left: Box::new(left),
                right: Box::new(right),
            }
        }

        Ok(left)
    }

    fn parse_binary_op_from(&mut self, ops: &[(TokenKind, BinaryOp)]) -> Option<(BinaryOp, Span)> {
        for (token_kind, op) in ops {
            if self.peek_kind() == token_kind {
                let token = self.advance();
                return Some((*op, token.span));
            }
        }

        None
    }

    fn parse_comma_separated_until_terminator<T>(
        &mut self,
        parse_item: fn(&mut Self) -> Result<T, ParseError>,
        terminator: &TokenKind,
        allow_trailing_comma: bool,
    ) -> Result<Vec<T>, ParseError> {
        let mut items = vec![];

        while self.peek_kind() != terminator {
            items.push(parse_item(self)?);

            if self.peek_kind() == &TokenKind::Comma {
                self.expect(&TokenKind::Comma)?;

                if self.peek_kind() == terminator && !allow_trailing_comma {
                    return Err(ParseError {
                        message: "trailing comma".to_string(),
                        span: self.peek().span,
                    });
                }
            }
        }

        Ok(items)
    }

    // Declaration / Type parsing

    fn is_type_decl(&self, token_kind: &TokenKind) -> bool {
        match token_kind {
            TokenKind::KwInt
            | TokenKind::KwChar
            | TokenKind::KwUnsigned
            | TokenKind::KwLong
            | TokenKind::KwSigned
            | TokenKind::KwConst => true,
            TokenKind::Ident(name) => self.typedefs.contains_key(name),
            _ => false,
        }
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let (spec, span) = self.parse_decl_spec()?;
        self.lower_decl_spec(&spec, span)
    }

    fn parse_decl_spec(&mut self) -> Result<(DeclSpec, Span), ParseError> {
        let start = self.peek().span;
        let mut spec = DeclSpec {
            storage_class: None,
            type_specifiers: vec![],
            qualifiers: vec![],
        };

        let mut saw_any = false;
        let mut saw_type = false;

        loop {
            match self.peek_kind() {
                // storage class
                TokenKind::KwTypedef => {
                    self.advance();
                    spec.storage_class = Some(StorageClass::Typedef);
                    saw_any = true;
                }

                // qualifiers
                TokenKind::KwConst => {
                    self.advance();
                    spec.qualifiers.push(TypeQualifier::Const);
                    saw_any = true;
                }

                // type specifiers
                TokenKind::KwUnsigned => {
                    self.advance();
                    spec.type_specifiers.push(TypeSpecifier::Unsigned);
                    saw_any = true;
                    saw_type = true;
                }
                TokenKind::KwSigned => {
                    self.advance();
                    spec.type_specifiers.push(TypeSpecifier::Signed);
                    saw_any = true;
                    saw_type = true;
                }
                TokenKind::KwInt => {
                    self.advance();
                    spec.type_specifiers.push(TypeSpecifier::Int);
                    saw_any = true;
                    saw_type = true;
                }
                TokenKind::KwChar => {
                    self.advance();
                    spec.type_specifiers.push(TypeSpecifier::Char);
                    saw_any = true;
                    saw_type = true;
                }
                TokenKind::KwLong => {
                    self.advance();
                    spec.type_specifiers.push(TypeSpecifier::Long);
                    saw_any = true;
                    saw_type = true;
                }

                TokenKind::Ident(name) if self.typedefs.contains_key(name) => {
                    let name = name.clone();
                    let span = self.advance().span;
                    spec.type_specifiers
                        .push(TypeSpecifier::TypedefName { name, span });
                    saw_any = true;
                    saw_type = true;
                }

                _ => break,
            }
        }

        if !saw_any || !saw_type {
            let token = self.peek();
            return Err(ParseError {
                message: format!("expected declaration specifier, found {:?}", token.kind),
                span: token.span,
            });
        }

        Ok((spec, start))
    }

    fn lower_decl_spec(&self, spec: &DeclSpec, span: Span) -> Result<Type, ParseError> {
        let mut typedef_name = None;
        let mut type_spec = TypeSpec::default();

        for type_specifier in &spec.type_specifiers {
            match type_specifier {
                TypeSpecifier::TypedefName { name, span } => {
                    if typedef_name.is_some() {
                        return Err(ParseError {
                            message: "multiple typedef names in declaration specifier".to_string(),
                            span: *span,
                        });
                    }
                    typedef_name = Some((name, *span));
                }

                TypeSpecifier::Unsigned => type_spec.unsigned = true,
                TypeSpecifier::Signed => type_spec.signed = true,
                TypeSpecifier::Int => type_spec.int_count += 1,
                TypeSpecifier::Char => type_spec.char_count += 1,
                TypeSpecifier::Long => type_spec.long_count += 1,
            }
        }

        let saw_builtin = type_spec.unsigned
            || type_spec.signed
            || type_spec.int_count > 0
            || type_spec.char_count > 0
            || type_spec.long_count > 0;

        if let Some((name, name_span)) = typedef_name {
            if saw_builtin {
                return Err(ParseError {
                    message: "cannot combine typedef name with other type specifiers".to_string(),
                    span: name_span,
                });
            }

            return self.typedefs.get(name).cloned().ok_or_else(|| ParseError {
                message: format!("unknown typedef name '{name}'"),
                span: name_span,
            });
        }

        Self::lower_type_spec(type_spec, span)
    }

    fn lower_type_spec(spec: TypeSpec, span: Span) -> Result<Type, ParseError> {
        match (
            spec.unsigned,
            spec.signed,
            spec.int_count,
            spec.char_count,
            spec.long_count,
        ) {
            // signed, signed int
            (false, _, 1, 0, 0) | (false, true, 0, 0, 0) => Ok(Type::Int),

            // unsigned, unsigned int
            (true, false, 0 | 1, 0, 0) => Ok(Type::UnsignedInt),

            // char
            (false, false, 0, 1, 0) => Ok(Type::Char),

            // unsigned char
            (true, false, 0, 1, 0) => Ok(Type::UnsignedChar),

            // signed char
            (false, true, 0, 1, 0) => Ok(Type::SignedChar),

            // long, long int, signed long, signed long int
            (false, _, 0 | 1, 0, 1) => Ok(Type::Long),

            // unsigned long, unsigned long int
            (true, false, 0 | 1, 0, 1) => Ok(Type::UnsignedLong),

            _ => Err(ParseError {
                message: "unsupported or invalid type specifier combination".to_string(),
                span,
            }),
        }
    }

    fn parse_declaration(&mut self) -> Result<Declaration, ParseError> {
        let (spec, spec_span) = self.parse_decl_spec()?;
        let mut declarators = vec![self.parse_init_declarator()?];

        while self.peek_kind() == &TokenKind::Comma {
            self.expect(&TokenKind::Comma)?;
            declarators.push(self.parse_init_declarator()?);
        }

        self.expect(&TokenKind::Semicolon)?;

        Ok(Declaration {
            spec,
            spec_span,
            declarators,
        })
    }

    fn parse_init_declarator(&mut self) -> Result<InitDeclarator, ParseError> {
        let declarator = self.parse_declarator()?;

        let initializer = if self.peek_kind() == &TokenKind::Equal {
            self.expect(&TokenKind::Equal)?;
            Some(self.parse_initializer()?)
        } else {
            None
        };

        Ok(InitDeclarator {
            declarator,
            initializer,
        })
    }

    fn parse_initializer(&mut self) -> Result<Initializer, ParseError> {
        if self.peek_kind() == &TokenKind::LBrace {
            self.expect(&TokenKind::LBrace)?;
            let elements = self.parse_comma_separated_until_terminator(
                Self::parse_expr,
                &TokenKind::RBrace,
                true,
            )?;
            self.expect(&TokenKind::RBrace)?;
            Ok(Initializer::List(elements))
        } else {
            let expr = self.parse_expr()?;
            Ok(Initializer::Expr(expr))
        }
    }

    fn parse_declarator(&mut self) -> Result<Declarator, ParseError> {
        if self.peek_kind() == &TokenKind::Star {
            self.advance();
            let inner = self.parse_declarator()?;
            return Ok(Declarator {
                kind: DeclaratorKind::Pointer(Box::new(inner)),
            });
        }

        self.parse_direct_declarator()
    }

    fn parse_direct_declarator(&mut self) -> Result<Declarator, ParseError> {
        let (name, name_span) = self.expect_ident()?;

        // Array declaration
        if self.peek_kind() == &TokenKind::LBracket {
            self.advance();
            let (len, _, _, len_span) = self.expect_int_literal()?;

            // Don't allow array length <1
            if len < 1 {
                return Err(ParseError {
                    message: format!("array size must be greater than 0, got '{len}'"),
                    span: len_span,
                });
            }

            self.expect(&TokenKind::RBracket)?;
            return Ok(Declarator {
                kind: DeclaratorKind::Array {
                    inner: Box::new(Declarator {
                        kind: DeclaratorKind::Name { name, name_span },
                    }),
                    len: usize::try_from(len).expect("u64 cannot be converted to usize"),
                },
            });
        }

        Ok(Declarator {
            kind: DeclaratorKind::Name { name, name_span },
        })
    }

    fn lower_declarator(
        base_type: &Type,
        declarator: &Declarator,
    ) -> Result<LoweredDeclarator, ParseError> {
        match &declarator.kind {
            DeclaratorKind::Name { name, name_span } => Ok(LoweredDeclarator {
                ty: base_type.clone(),
                name: name.clone(),
                name_span: *name_span,
            }),
            DeclaratorKind::Pointer(inner) => {
                Self::lower_declarator(&Type::Pointer(Box::new(base_type.clone())), inner)
            }
            DeclaratorKind::Array { inner, len } => {
                let LoweredDeclarator {
                    ty,
                    name,
                    name_span,
                } = Self::lower_declarator(base_type, inner)?;
                Ok(LoweredDeclarator {
                    ty: Type::Array {
                        element: Box::new(ty),
                        len: *len,
                    },
                    name,
                    name_span,
                })
            }
        }
    }

    fn parse_raw_param(&mut self) -> Result<RawParam, ParseError> {
        let (spec, ty_span) = self.parse_decl_spec()?;
        let ty = self.lower_decl_spec(&spec, ty_span)?;
        if matches!(self.peek_kind(), TokenKind::Comma | TokenKind::RParen) {
            Ok(RawParam {
                ty,
                ty_span,
                name: None,
                name_span: None,
            })
        } else {
            let declarator = self.parse_declarator()?;
            let LoweredDeclarator {
                ty,
                name,
                name_span,
            } = Self::lower_declarator(&ty, &declarator)?;
            Ok(RawParam {
                ty,
                ty_span,
                name: Some(name),
                name_span: Some(name_span),
            })
        }
    }

    // Expression parsing

    fn parse_call_arg(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr()
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.advance();

        match token.kind {
            TokenKind::IntLiteral {
                value,
                suffix,
                base,
            } => Ok(Expr::IntLiteral {
                value,
                suffix,
                base,
                span: token.span,
            }),
            TokenKind::CharLiteral(value) => Ok(Expr::IntLiteral {
                value: value.try_into().unwrap(),
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
                span: token.span,
            }),
            TokenKind::Ident(name) => {
                if self.peek_kind() == &TokenKind::LParen {
                    self.expect(&TokenKind::LParen)?;
                    let args = self.parse_comma_separated_until_terminator(
                        Self::parse_call_arg,
                        &TokenKind::RParen,
                        false,
                    )?;
                    self.expect(&TokenKind::RParen)?;
                    Ok(Expr::Call {
                        name,
                        name_span: token.span,
                        args,
                    })
                } else {
                    Ok(Expr::Variable {
                        name,
                        span: token.span,
                    })
                }
            }
            TokenKind::LParen => {
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                Ok(expr)
            }
            found => Err(ParseError {
                message: format!("expected expression, found {found:?}"),
                span: token.span,
            }),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek_kind() {
                TokenKind::LBracket => {
                    let token = self.expect(&TokenKind::LBracket)?;
                    let index = self.parse_expr()?;
                    self.expect(&TokenKind::RBracket)?;

                    expr = Expr::Index {
                        base: Box::new(expr),
                        index: Box::new(index),
                        span: token.span,
                    }
                }
                TokenKind::PlusPlus => {
                    let op = self.advance();
                    expr = Expr::PostfixInc {
                        expr: Box::new(expr),
                        op_span: op.span,
                    }
                }
                TokenKind::MinusMinus => {
                    let op = self.advance();
                    expr = Expr::PostfixDec {
                        expr: Box::new(expr),
                        op_span: op.span,
                    }
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_unary_op(&mut self) -> Option<(UnaryOp, Span)> {
        match self.peek_kind() {
            TokenKind::Minus => {
                let token = self.advance();
                Some((UnaryOp::Negate, token.span))
            }
            TokenKind::Bang => {
                let token = self.advance();
                Some((UnaryOp::LogicalNot, token.span))
            }
            TokenKind::Tilde => {
                let token = self.advance();
                Some((UnaryOp::BitwiseNot, token.span))
            }
            TokenKind::Star => {
                let token = self.advance();
                Some((UnaryOp::Dereference, token.span))
            }
            _ => None,
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.peek_kind() == &TokenKind::LParen && self.is_type_decl(self.peek_nth_kind(1)) {
            let span = self.advance().span;
            let ty = self.parse_type()?;
            self.expect(&TokenKind::RParen)?;
            let expr = self.parse_unary()?;
            return Ok(Expr::Cast {
                ty,
                expr: Box::new(expr),
                span,
            });
        }
        if self.peek_kind() == &TokenKind::PlusPlus {
            let op = self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::PrefixInc {
                expr: Box::new(expr),
                op_span: op.span,
            });
        }
        if self.peek_kind() == &TokenKind::MinusMinus {
            let op = self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::PrefixDec {
                expr: Box::new(expr),
                op_span: op.span,
            });
        }
        if let Some((op, op_span)) = self.parse_unary_op() {
            let right = self.parse_unary()?;
            return Ok(Expr::Unary {
                op,
                op_span,
                expr: Box::new(right),
            });
        }

        self.parse_postfix()
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        self.parse_left_assoc(Self::parse_unary, MULTIPLICATIVE_OPS)
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        self.parse_left_assoc(Self::parse_multiplicative, ADDITIVE_OPS)
    }

    fn parse_shift(&mut self) -> Result<Expr, ParseError> {
        self.parse_left_assoc(Self::parse_additive, SHIFT_OPS)
    }

    fn parse_relational(&mut self) -> Result<Expr, ParseError> {
        self.parse_left_assoc(Self::parse_shift, RELATIONAL_OPS)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        self.parse_left_assoc(Self::parse_relational, EQUALITY_OPS)
    }

    fn parse_bitwise_and(&mut self) -> Result<Expr, ParseError> {
        self.parse_left_assoc(Self::parse_equality, BITWISE_AND_OPS)
    }

    fn parse_bitwise_xor(&mut self) -> Result<Expr, ParseError> {
        self.parse_left_assoc(Self::parse_bitwise_and, BITWISE_XOR_OPS)
    }

    fn parse_bitwise_or(&mut self) -> Result<Expr, ParseError> {
        self.parse_left_assoc(Self::parse_bitwise_xor, BITWISE_OR_OPS)
    }

    fn parse_logical_and(&mut self) -> Result<Expr, ParseError> {
        self.parse_left_assoc(Self::parse_bitwise_or, LOGICAL_AND_OPS)
    }

    fn parse_logical_or(&mut self) -> Result<Expr, ParseError> {
        self.parse_left_assoc(Self::parse_logical_and, LOGICAL_OR_OPS)
    }

    fn parse_assignment_op(&mut self) -> Option<(Option<BinaryOp>, Span)> {
        let op = match self.peek_kind() {
            TokenKind::Equal => None,
            TokenKind::PlusEqual => Some(BinaryOp::Add),
            TokenKind::MinusEqual => Some(BinaryOp::Subtract),
            TokenKind::StarEqual => Some(BinaryOp::Multiply),
            TokenKind::SlashEqual => Some(BinaryOp::Divide),
            TokenKind::PercentEqual => Some(BinaryOp::Remainder),
            TokenKind::AmpersandEqual => Some(BinaryOp::BitAnd),
            TokenKind::CaretEqual => Some(BinaryOp::BitXor),
            TokenKind::PipeEqual => Some(BinaryOp::BitOr),
            TokenKind::LessLessEqual => Some(BinaryOp::ShiftLeft),
            TokenKind::GreaterGreaterEqual => Some(BinaryOp::ShiftRight),
            _ => return None,
        };

        let token = self.advance();
        Some((op, token.span))
    }

    fn parse_assignment(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_logical_or()?;

        if let Some((compound_op, op_span)) = self.parse_assignment_op() {
            let value = self.parse_assignment()?;

            if let Some(op) = compound_op {
                return Ok(Expr::CompoundAssign {
                    target: Box::new(left),
                    op,
                    op_span,
                    value: Box::new(value),
                });
            }

            return Ok(Expr::Assign {
                target: Box::new(left),
                op_span,
                value: Box::new(value),
            });
        }
        Ok(left)
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_assignment()
    }

    // Statement parsing

    fn parse_decl_statement(&mut self) -> Result<Statement, ParseError> {
        let mut declarators = vec![];
        let declaration = self.parse_declaration()?;
        if declaration.spec.storage_class.is_some() {
            return Err(ParseError {
                message: "storage class is not supported in local declarations".to_string(),
                span: declaration.spec_span,
            });
        }
        let base_ty = self.lower_decl_spec(&declaration.spec, declaration.spec_span)?;
        for init_declarator in declaration.declarators {
            let LoweredDeclarator {
                ty,
                name,
                name_span,
            } = Self::lower_declarator(&base_ty, &init_declarator.declarator)?;
            declarators.push(LocalDecl {
                ty,
                name,
                name_span,
                init: init_declarator.initializer,
            });
        }

        Ok(Statement::Decl(declarators))
    }

    fn parse_through_rbrace(&mut self, vec: &mut Vec<Statement>) -> Result<(), ParseError> {
        while self.peek_kind() != &TokenKind::RBrace {
            vec.push(self.parse_statement()?);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(())
    }

    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::KwIf)?;
        self.expect(&TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(&TokenKind::RParen)?;
        let then_statement = self.parse_statement()?;
        let else_statement = if self.peek_kind() == &TokenKind::KwElse {
            self.expect(&TokenKind::KwElse)?;
            Some(self.parse_statement()?)
        } else {
            None
        };

        Ok(Statement::If {
            cond,
            then_branch: Box::new(then_statement),
            else_branch: else_statement.map(Box::new),
        })
    }

    fn parse_while_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::KwWhile)?;
        self.expect(&TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(&TokenKind::RParen)?;
        let body = self.parse_statement()?;

        Ok(Statement::While {
            cond,
            body: Box::new(body),
        })
    }

    fn parse_do_while_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::KwDo)?;
        let body = self.parse_statement()?;
        self.expect(&TokenKind::KwWhile)?;
        self.expect(&TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Semicolon)?;

        Ok(Statement::DoWhile {
            body: Box::new(body),
            cond,
        })
    }

    fn parse_expr_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.parse_expr()?;
        self.expect(&TokenKind::Semicolon)?;
        Ok(Statement::ExprStatement(expr))
    }

    fn parse_for_statement_init(&mut self) -> Result<Statement, ParseError> {
        match self.peek_kind() {
            // Variable declaration
            token_kind if self.is_type_decl(token_kind) => self.parse_decl_statement(),
            // Empty
            TokenKind::Semicolon => {
                self.expect(&TokenKind::Semicolon)?;
                Ok(Statement::Empty)
            }
            // Expression-start tokens
            token_kind if Self::is_expr_start(token_kind) => self.parse_expr_statement(),
            found => Err(ParseError {
                message: format!("got unexpected keyword {found:?}"),
                span: self.peek().span,
            }),
        }
    }

    fn parse_for_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(&TokenKind::KwFor)?;
        self.expect(&TokenKind::LParen)?;

        let mut init = None;
        let mut cond = None;
        let mut post = None;

        if self.peek_kind() == &TokenKind::Semicolon {
            self.expect(&TokenKind::Semicolon)?;
        } else {
            init = Some(self.parse_for_statement_init()?);
        }

        if self.peek_kind() != &TokenKind::Semicolon {
            cond = Some(self.parse_expr()?);
        }
        self.expect(&TokenKind::Semicolon)?;

        if self.peek_kind() != &TokenKind::RParen {
            post = Some(self.parse_expr()?);
        }
        self.expect(&TokenKind::RParen)?;

        let body = self.parse_statement()?;

        Ok(Statement::For {
            init: init.map(Box::new),
            cond,
            post,
            body: Box::new(body),
        })
    }

    const fn is_expr_start(token_kind: &TokenKind) -> bool {
        matches!(
            token_kind,
            TokenKind::Ident(_)
                | TokenKind::IntLiteral { .. }
                | TokenKind::CharLiteral(_)
                | TokenKind::LParen
                | TokenKind::Minus
                | TokenKind::Bang
                | TokenKind::Tilde
                | TokenKind::PlusPlus
                | TokenKind::MinusMinus
                | TokenKind::Star
        )
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek_kind() {
            // Control flow
            TokenKind::KwReturn => {
                self.expect(&TokenKind::KwReturn)?;
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::Semicolon)?;
                Ok(Statement::Return(expr))
            }
            TokenKind::KwIf => self.parse_if_statement(),
            TokenKind::KwWhile => self.parse_while_statement(),
            TokenKind::KwDo => self.parse_do_while_statement(),
            TokenKind::KwBreak => {
                let token = self.expect(&TokenKind::KwBreak)?;
                self.expect(&TokenKind::Semicolon)?;
                Ok(Statement::Break { span: token.span })
            }
            TokenKind::KwContinue => {
                let token = self.expect(&TokenKind::KwContinue)?;
                self.expect(&TokenKind::Semicolon)?;
                Ok(Statement::Continue { span: token.span })
            }
            TokenKind::KwFor => self.parse_for_statement(),
            // Variable declaration
            token_kind if self.is_type_decl(token_kind) => self.parse_decl_statement(),
            // Block
            TokenKind::LBrace => {
                self.expect(&TokenKind::LBrace)?;
                let mut body = vec![];
                self.parse_through_rbrace(&mut body)?;
                Ok(Statement::Block(body))
            }
            // Empty
            TokenKind::Semicolon => {
                self.expect(&TokenKind::Semicolon)?;
                Ok(Statement::Empty)
            }
            // Expression-start tokens
            token_kind if Self::is_expr_start(token_kind) => self.parse_expr_statement(),
            found => Err(ParseError {
                message: format!("got unexpected keyword {found:?}"),
                span: self.peek().span,
            }),
        }
    }

    fn parse_function_signature(&mut self) -> Result<ParsedFunctionSignature, ParseError> {
        let base_ty = self.parse_type()?;
        let declarator = self.parse_declarator()?;
        let LoweredDeclarator {
            ty,
            name,
            name_span,
        } = Self::lower_declarator(&base_ty, &declarator)?;

        self.expect(&TokenKind::LParen)?;
        let params = self.parse_comma_separated_until_terminator(
            Self::parse_raw_param,
            &TokenKind::RParen,
            false,
        )?;
        self.expect(&TokenKind::RParen)?;

        Ok(ParsedFunctionSignature {
            return_type: ty,
            name,
            name_span,
            params,
        })
    }

    fn parse_function_external_decl(&mut self) -> Result<ExternalDecl, ParseError> {
        let sig = self.parse_function_signature()?;

        match self.peek_kind() {
            TokenKind::Semicolon => {
                self.advance();
                Ok(ExternalDecl::FunctionDecl(FunctionDecl {
                    return_type: sig.return_type,
                    name: sig.name,
                    name_span: sig.name_span,
                    params: sig.params.into_iter().map(raw_param_to_decl).collect(),
                }))
            }
            TokenKind::LBrace => {
                self.advance();
                let mut body = vec![];
                self.parse_through_rbrace(&mut body)?;
                Ok(ExternalDecl::FunctionDef(FunctionDef {
                    return_type: sig.return_type,
                    name: sig.name,
                    name_span: sig.name_span,
                    params: sig
                        .params
                        .into_iter()
                        .map(raw_param_to_def)
                        .collect::<Result<Vec<_>, _>>()?,
                    body,
                }))
            }
            _ => {
                let token = self.peek();
                Err(ParseError {
                    message: format!("expected ';' or '{{', got '{:?}'", token.kind),
                    span: token.span,
                })
            }
        }
    }

    fn parse_typedefs(&mut self) -> Result<Vec<Typedef>, ParseError> {
        let mut typedefs = vec![];
        let declaration = self.parse_declaration()?;
        if declaration.spec.storage_class != Some(StorageClass::Typedef) {
            return Err(ParseError {
                message: "expected typedef declaration".to_string(),
                span: declaration.spec_span,
            });
        }
        let base_ty = self.lower_decl_spec(&declaration.spec, declaration.spec_span)?;
        for init_declarator in declaration.declarators {
            if init_declarator.initializer.is_some() {
                return Err(ParseError {
                    message: "typedef declarator cannot have an initializer".to_string(),
                    span: declaration.spec_span,
                });
            }
            let lowered = Self::lower_declarator(&base_ty, &init_declarator.declarator)?;
            if self.typedefs.contains_key(&lowered.name) {
                return Err(ParseError {
                    message: format!("duplicate typedef with name '{}'", lowered.name),
                    span: lowered.name_span,
                });
            }
            self.typedefs
                .insert(lowered.name.clone(), lowered.ty.clone());
            typedefs.push(Typedef {
                name: lowered.name,
                name_span: lowered.name_span,
                ty: lowered.ty,
            });
        }
        Ok(typedefs)
    }

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut declarations = vec![];

        loop {
            match self.peek_kind() {
                TokenKind::Eof => break,
                TokenKind::KwTypedef => {
                    for typedef in self.parse_typedefs()? {
                        declarations.push(ExternalDecl::Typedef(typedef));
                    }
                }
                _ => declarations.push(self.parse_function_external_decl()?),
            }
        }
        let token = self.expect(&TokenKind::Eof)?;

        Ok(Program {
            declarations,
            eof_span: token.span,
        })
    }
}

/// Parses a token stream into a program AST.
///
/// # Errors
///
/// Returns a [`ParseError`] when the tokens do not match the supported C grammar.
pub fn parse(tokens: Vec<Token>) -> Result<Program, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

fn raw_param_to_decl(raw: RawParam) -> ParamDecl {
    ParamDecl {
        ty: raw.ty,
        name: raw.name,
        name_span: raw.name_span,
    }
}

fn raw_param_to_def(raw: RawParam) -> Result<Param, ParseError> {
    let Some(name) = raw.name else {
        return Err(ParseError {
            message: "expected parameter name in function definition".to_string(),
            span: raw.ty_span,
        });
    };

    let Some(name_span) = raw.name_span else {
        unreachable!("parameter name and span should be present together");
    };

    Ok(Param {
        ty: raw.ty,
        name,
        name_span,
    })
}

#[cfg(test)]
mod tests {
    use crate::source::DUMMY_FILE_ID;

    use super::*;

    fn span() -> Span {
        Span {
            file_id: DUMMY_FILE_ID,
            start: 0,
            end: 0,
        }
    }

    fn token(kind: TokenKind) -> Token {
        Token {
            kind,
            span: Span {
                file_id: DUMMY_FILE_ID,
                start: 0,
                end: 0,
            },
        }
    }

    fn token_with_span(kind: TokenKind, start: usize, end: usize) -> Token {
        Token {
            kind,
            span: Span {
                file_id: DUMMY_FILE_ID,
                start,
                end,
            },
        }
    }

    macro_rules! tokens {
        ($($kind:expr),* $(,)?) => {
            vec![$(token($kind)),*]
        };
    }

    fn program_with_functions(functions: Vec<FunctionDef>) -> Program {
        Program {
            declarations: functions
                .into_iter()
                .map(ExternalDecl::FunctionDef)
                .collect(),
            eof_span: span(),
        }
    }

    fn first_function(program: &Program) -> &FunctionDef {
        program
            .declarations
            .iter()
            .find_map(|decl| match decl {
                ExternalDecl::FunctionDef(function) => Some(function),
                ExternalDecl::Typedef(_) | ExternalDecl::FunctionDecl(_) => None,
            })
            .expect("expected function definition")
    }

    #[test]
    fn parse_expect_errors_use_found_token_span() {
        let tokens = vec![
            token_with_span(
                TokenKind::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                },
                0,
                1,
            ),
            token_with_span(TokenKind::Eof, 1, 1),
        ];

        let mut parser = Parser::new(tokens);
        let err = parser
            .parse_statement()
            .expect_err("missing semicolon should fail");

        assert_eq!(
            err.span,
            Span {
                file_id: DUMMY_FILE_ID,
                start: 1,
                end: 1
            }
        );
        assert!(err.message.contains("expected Semicolon"));
    }

    #[test]
    fn parse_expression_errors_use_unexpected_token_span() {
        let tokens = vec![
            token_with_span(TokenKind::KwReturn, 0, 6),
            token_with_span(TokenKind::RParen, 7, 8),
            token_with_span(TokenKind::Semicolon, 8, 9),
        ];

        let mut parser = Parser::new(tokens);
        let err = parser
            .parse_statement()
            .expect_err("return without expression should fail");

        assert_eq!(
            err.span,
            Span {
                file_id: DUMMY_FILE_ID,
                start: 7,
                end: 8
            }
        );
        assert_eq!(err.message, "expected expression, found RParen");
    }

    #[test]
    fn trailing_comma_errors_point_at_right_paren() {
        let tokens = vec![
            token_with_span(TokenKind::KwReturn, 0, 6),
            token_with_span(TokenKind::Ident("add".to_string()), 7, 10),
            token_with_span(TokenKind::LParen, 10, 11),
            token_with_span(
                TokenKind::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                },
                11,
                12,
            ),
            token_with_span(TokenKind::Comma, 12, 13),
            token_with_span(TokenKind::RParen, 14, 15),
            token_with_span(TokenKind::Semicolon, 15, 16),
        ];

        let mut parser = Parser::new(tokens);
        let err = parser
            .parse_statement()
            .expect_err("trailing argument comma should fail");

        assert_eq!(
            err.span,
            Span {
                file_id: DUMMY_FILE_ID,
                start: 14,
                end: 15
            }
        );
        assert_eq!(err.message, "trailing comma");
    }

    #[test]
    fn parses_assignment_to_non_variable_expression() {
        let tokens = vec![
            token_with_span(TokenKind::KwReturn, 0, 6),
            token_with_span(TokenKind::LParen, 7, 8),
            token_with_span(
                TokenKind::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                },
                8,
                9,
            ),
            token_with_span(TokenKind::Plus, 10, 11),
            token_with_span(
                TokenKind::IntLiteral {
                    value: 2,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                },
                12,
                13,
            ),
            token_with_span(TokenKind::RParen, 13, 14),
            token_with_span(TokenKind::Equal, 15, 16),
            token_with_span(
                TokenKind::IntLiteral {
                    value: 3,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                },
                17,
                18,
            ),
            token_with_span(TokenKind::Semicolon, 18, 19),
        ];

        let mut parser = Parser::new(tokens);
        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Assign {
                target: Box::new(Expr::Binary {
                    op: BinaryOp::Add,
                    op_span: Span {
                        file_id: DUMMY_FILE_ID,
                        start: 10,
                        end: 11
                    },
                    left: Box::new(Expr::IntLiteral {
                        value: 1,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: Span {
                            file_id: DUMMY_FILE_ID,
                            start: 8,
                            end: 9
                        },
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 2,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: Span {
                            file_id: DUMMY_FILE_ID,
                            start: 12,
                            end: 13
                        },
                    }),
                }),
                op_span: Span {
                    file_id: DUMMY_FILE_ID,
                    start: 15,
                    end: 16
                },
                value: Box::new(Expr::IntLiteral {
                    value: 3,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: Span {
                        file_id: DUMMY_FILE_ID,
                        start: 17,
                        end: 18
                    },
                }),
            })
        );
    }

    #[test]
    fn parses_compound_assignment_to_non_variable_expression() {
        let tokens = vec![
            token_with_span(TokenKind::KwReturn, 0, 6),
            token_with_span(TokenKind::LParen, 7, 8),
            token_with_span(
                TokenKind::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                },
                8,
                9,
            ),
            token_with_span(TokenKind::Plus, 10, 11),
            token_with_span(
                TokenKind::IntLiteral {
                    value: 2,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                },
                12,
                13,
            ),
            token_with_span(TokenKind::RParen, 13, 14),
            token_with_span(TokenKind::PlusEqual, 15, 17),
            token_with_span(
                TokenKind::IntLiteral {
                    value: 3,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                },
                18,
                19,
            ),
            token_with_span(TokenKind::Semicolon, 19, 20),
        ];

        let mut parser = Parser::new(tokens);
        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::CompoundAssign {
                target: Box::new(Expr::Binary {
                    op: BinaryOp::Add,
                    op_span: Span {
                        file_id: DUMMY_FILE_ID,
                        start: 10,
                        end: 11
                    },
                    left: Box::new(Expr::IntLiteral {
                        value: 1,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: Span {
                            file_id: DUMMY_FILE_ID,
                            start: 8,
                            end: 9
                        },
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 2,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: Span {
                            file_id: DUMMY_FILE_ID,
                            start: 12,
                            end: 13
                        },
                    }),
                }),
                op: BinaryOp::Add,
                op_span: Span {
                    file_id: DUMMY_FILE_ID,
                    start: 15,
                    end: 17
                },
                value: Box::new(Expr::IntLiteral {
                    value: 3,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: Span {
                        file_id: DUMMY_FILE_ID,
                        start: 18,
                        end: 19
                    },
                }),
            })
        );
    }

    #[test]
    fn binary_expression_preserves_operator_span() {
        let tokens = vec![
            token_with_span(TokenKind::Ident("x".to_string()), 0, 1),
            token_with_span(TokenKind::Plus, 2, 3),
            token_with_span(TokenKind::Ident("y".to_string()), 4, 5),
            token_with_span(TokenKind::Semicolon, 5, 6),
        ];

        let mut parser = Parser::new(tokens);
        let statement = parser.parse_statement().expect("parsing should succeed");

        let Statement::ExprStatement(Expr::Binary { op, op_span, .. }) = statement else {
            panic!("expected binary expression statement");
        };

        assert_eq!(op, BinaryOp::Add);
        assert_eq!(
            op_span,
            Span {
                file_id: DUMMY_FILE_ID,
                start: 2,
                end: 3
            }
        );
    }

    #[test]
    fn parse_binary_op() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Plus,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::Add,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span()
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 2,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span()
                }),
            })
        );
    }

    #[test]
    fn parses_unary_expression_statement() {
        let tokens = tokens![
            TokenKind::Bang,
            TokenKind::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::ExprStatement(Expr::Unary {
                op: UnaryOp::LogicalNot,
                op_span: span(),
                expr: Box::new(Expr::IntLiteral {
                    value: 0,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span()
                }),
            })
        );
        assert_eq!(
            parser.pos, 3,
            "expression statement should consume semicolon"
        );
    }

    #[test]
    fn rejects_expression_statement_without_semicolon() {
        let tokens = tokens![
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Eof
        ];

        let mut parser = Parser::new(tokens);

        assert!(
            parser.parse_statement().is_err(),
            "expression statements should require semicolons"
        );
    }

    #[test]
    fn parses_parameter_name_spans() {
        let tokens = vec![
            token_with_span(TokenKind::KwInt, 0, 3),
            token_with_span(TokenKind::Ident("add".to_string()), 4, 7),
            token_with_span(TokenKind::LParen, 7, 8),
            token_with_span(TokenKind::KwInt, 8, 11),
            token_with_span(TokenKind::Ident("x".to_string()), 12, 13),
            token_with_span(TokenKind::Comma, 13, 14),
            token_with_span(TokenKind::KwInt, 15, 18),
            token_with_span(TokenKind::Ident("y".to_string()), 19, 20),
            token_with_span(TokenKind::RParen, 20, 21),
            token_with_span(TokenKind::LBrace, 22, 23),
            token_with_span(TokenKind::KwReturn, 28, 34),
            token_with_span(TokenKind::Ident("x".to_string()), 35, 36),
            token_with_span(TokenKind::Semicolon, 36, 37),
            token_with_span(TokenKind::RBrace, 38, 39),
            token_with_span(TokenKind::Eof, 39, 39),
        ];

        let program = parse(tokens).expect("parsing should succeed");
        let params = &first_function(&program).params;

        assert_eq!(params[0].name, "x");
        assert_eq!(
            params[0].name_span,
            Span {
                file_id: DUMMY_FILE_ID,
                start: 12,
                end: 13
            }
        );
        assert_eq!(params[1].name, "y");
        assert_eq!(
            params[1].name_span,
            Span {
                file_id: DUMMY_FILE_ID,
                start: 19,
                end: 20
            }
        );
    }

    #[test]
    fn parses_parenthesized_expression_precedence() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::LParen,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Plus,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::RParen,
            TokenKind::Star,
            TokenKind::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            program_with_functions(vec![FunctionDef {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Multiply,
                    op_span: span(),
                    left: Box::new(Expr::Binary {
                        op: BinaryOp::Add,
                        op_span: span(),
                        left: Box::new(Expr::IntLiteral {
                            value: 1,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span()
                        }),
                        right: Box::new(Expr::IntLiteral {
                            value: 2,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span()
                        }),
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 3,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                })],
            }])
        );
    }

    #[test]
    fn parses_less_than_with_additive_operands() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Plus,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Less,
            TokenKind::IntLiteral {
                value: 4,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            program_with_functions(vec![FunctionDef {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Less,
                    op_span: span(),
                    left: Box::new(Expr::Binary {
                        op: BinaryOp::Add,
                        op_span: span(),
                        left: Box::new(Expr::IntLiteral {
                            value: 1,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span()
                        }),
                        right: Box::new(Expr::IntLiteral {
                            value: 2,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span()
                        }),
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 4,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                })],
            }])
        );
    }

    #[test]
    fn parses_shift_after_additive() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Plus,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::LessLess,
            TokenKind::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::ShiftLeft,
                op_span: span(),
                left: Box::new(Expr::Binary {
                    op: BinaryOp::Add,
                    op_span: span(),
                    left: Box::new(Expr::IntLiteral {
                        value: 1,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 2,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 3,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span()
                }),
            })
        );
    }

    #[test]
    fn parses_relational_after_shift() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::LessLess,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Less,
            TokenKind::IntLiteral {
                value: 8,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::Less,
                op_span: span(),
                left: Box::new(Expr::Binary {
                    op: BinaryOp::ShiftLeft,
                    op_span: span(),
                    left: Box::new(Expr::IntLiteral {
                        value: 1,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 2,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 8,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span()
                }),
            })
        );
    }

    #[test]
    fn parses_equality_before_bitwise_and() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::Ident("a".to_string()),
            TokenKind::Ampersand,
            TokenKind::Ident("b".to_string()),
            TokenKind::EqualEqual,
            TokenKind::Ident("c".to_string()),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::BitAnd,
                op_span: span(),
                left: Box::new(Expr::Variable {
                    name: "a".to_string(),
                    span: span()
                }),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::Equal,
                    op_span: span(),
                    left: Box::new(Expr::Variable {
                        name: "b".to_string(),
                        span: span()
                    }),
                    right: Box::new(Expr::Variable {
                        name: "c".to_string(),
                        span: span()
                    }),
                }),
            })
        );
    }

    #[test]
    fn parses_bitwise_and_before_xor() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::Ident("a".to_string()),
            TokenKind::Caret,
            TokenKind::Ident("b".to_string()),
            TokenKind::Ampersand,
            TokenKind::Ident("c".to_string()),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::BitXor,
                op_span: span(),
                left: Box::new(Expr::Variable {
                    name: "a".to_string(),
                    span: span()
                }),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::BitAnd,
                    op_span: span(),
                    left: Box::new(Expr::Variable {
                        name: "b".to_string(),
                        span: span()
                    }),
                    right: Box::new(Expr::Variable {
                        name: "c".to_string(),
                        span: span()
                    }),
                }),
            })
        );
    }

    #[test]
    fn parses_bitwise_xor_before_or() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::Ident("a".to_string()),
            TokenKind::Pipe,
            TokenKind::Ident("b".to_string()),
            TokenKind::Caret,
            TokenKind::Ident("c".to_string()),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::BitOr,
                op_span: span(),
                left: Box::new(Expr::Variable {
                    name: "a".to_string(),
                    span: span()
                }),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::BitXor,
                    op_span: span(),
                    left: Box::new(Expr::Variable {
                        name: "b".to_string(),
                        span: span()
                    }),
                    right: Box::new(Expr::Variable {
                        name: "c".to_string(),
                        span: span()
                    }),
                }),
            })
        );
    }

    #[test]
    fn parses_equality_with_parenthesized_expression() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::LParen,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Plus,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::RParen,
            TokenKind::EqualEqual,
            TokenKind::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            program_with_functions(vec![FunctionDef {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Equal,
                    op_span: span(),
                    left: Box::new(Expr::Binary {
                        op: BinaryOp::Add,
                        op_span: span(),
                        left: Box::new(Expr::IntLiteral {
                            value: 1,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span()
                        }),
                        right: Box::new(Expr::IntLiteral {
                            value: 2,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span()
                        }),
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 3,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                })],
            }])
        );
    }

    #[test]
    fn parses_greater_equal() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::Ident("x".to_string()),
            TokenKind::GreaterEqual,
            TokenKind::IntLiteral {
                value: 10,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            program_with_functions(vec![FunctionDef {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::GreaterEqual,
                    op_span: span(),
                    left: Box::new(Expr::Variable {
                        name: "x".to_string(),
                        span: span()
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 10,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                })],
            }])
        );
    }

    #[test]
    fn parses_chained_comparisons_left_associative() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Less,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Less,
            TokenKind::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            program_with_functions(vec![FunctionDef {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Less,
                    op_span: span(),
                    left: Box::new(Expr::Binary {
                        op: BinaryOp::Less,
                        op_span: span(),
                        left: Box::new(Expr::IntLiteral {
                            value: 1,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span()
                        }),
                        right: Box::new(Expr::IntLiteral {
                            value: 2,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span()
                        }),
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 3,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                })],
            }])
        );
    }

    #[test]
    fn parses_function_with_multiple_statements() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwInt,
            TokenKind::Ident("x".to_string()),
            TokenKind::Equal,
            TokenKind::IntLiteral {
                value: 5,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 42,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            program_with_functions(vec![FunctionDef {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![
                    Statement::Decl(vec![LocalDecl {
                        ty: Type::Int,
                        name_span: span(),
                        name: "x".to_string(),
                        init: Some(Initializer::Expr(Expr::IntLiteral {
                            value: 5,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span()
                        })),
                    }]),
                    Statement::Return(Expr::IntLiteral {
                        value: 42,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                ],
            }])
        );
    }

    #[test]
    fn parses_function_returning_variable() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwInt,
            TokenKind::Ident("x".to_string()),
            TokenKind::Equal,
            TokenKind::IntLiteral {
                value: 5,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::KwReturn,
            TokenKind::Ident("x".to_string()),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            program_with_functions(vec![FunctionDef {
                return_type: Type::Int,
                name_span: span(),
                name: "main".to_string(),
                params: vec![],
                body: vec![
                    Statement::Decl(vec![LocalDecl {
                        ty: Type::Int,
                        name_span: span(),
                        name: "x".to_string(),
                        init: Some(Initializer::Expr(Expr::IntLiteral {
                            value: 5,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span()
                        })),
                    }]),
                    Statement::Return(Expr::Variable {
                        name: "x".to_string(),
                        span: span()
                    }),
                ],
            }])
        );
    }

    #[test]
    fn multiplication_has_higher_precedence_than_addition() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Plus,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Star,
            TokenKind::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::Add,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span()
                }),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::Multiply,
                    op_span: span(),
                    left: Box::new(Expr::IntLiteral {
                        value: 2,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 3,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                }),
            })
        );
    }

    #[test]
    fn parses_for_loop_with_all_clauses_empty() {
        let tokens = tokens![
            TokenKind::KwFor,
            TokenKind::LParen,
            TokenKind::Semicolon,
            TokenKind::Semicolon,
            TokenKind::RParen,
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::For {
                init: None,
                cond: None,
                post: None,
                body: Box::new(Statement::Empty),
            }
        );
    }

    #[test]
    fn parses_logical_and_before_logical_or() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::PipePipe,
            TokenKind::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::AmpersandAmpersand,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::LogicalOr,
                op_span: span(),
                left: Box::new(Expr::IntLiteral {
                    value: 1,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span()
                }),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::LogicalAnd,
                    op_span: span(),
                    left: Box::new(Expr::IntLiteral {
                        value: 0,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 2,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                }),
            })
        );
    }

    #[test]
    fn parses_bitwise_or_before_logical_and() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Pipe,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::AmpersandAmpersand,
            TokenKind::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::LogicalAnd,
                op_span: span(),
                left: Box::new(Expr::Binary {
                    op: BinaryOp::BitOr,
                    op_span: span(),
                    left: Box::new(Expr::IntLiteral {
                        value: 1,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 2,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 3,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span()
                }),
            })
        );
    }

    #[test]
    fn parses_chained_logical_and_left_associative() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::AmpersandAmpersand,
            TokenKind::IntLiteral {
                value: 2,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::AmpersandAmpersand,
            TokenKind::IntLiteral {
                value: 3,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::LogicalAnd,
                op_span: span(),
                left: Box::new(Expr::Binary {
                    op: BinaryOp::LogicalAnd,
                    op_span: span(),
                    left: Box::new(Expr::IntLiteral {
                        value: 1,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 2,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                }),
                right: Box::new(Expr::IntLiteral {
                    value: 3,
                    suffix: IntLiteralSuffix::None,
                    base: IntLiteralBase::Decimal,
                    span: span()
                }),
            })
        );
    }

    #[test]
    fn parses_for_loop_with_assignment_init_condition_and_assignment_post() {
        let tokens = tokens![
            TokenKind::KwFor,
            TokenKind::LParen,
            TokenKind::Ident("i".to_string()),
            TokenKind::Equal,
            TokenKind::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::Ident("i".to_string()),
            TokenKind::Less,
            TokenKind::IntLiteral {
                value: 10,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::Ident("i".to_string()),
            TokenKind::Equal,
            TokenKind::Ident("i".to_string()),
            TokenKind::Plus,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::RParen,
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::For {
                init: Some(Box::new(Statement::ExprStatement(Expr::Assign {
                    op_span: span(),
                    target: Box::new(Expr::Variable {
                        name: "i".to_string(),
                        span: span()
                    }),
                    value: Box::new(Expr::IntLiteral {
                        value: 0,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                }))),
                cond: Some(Expr::Binary {
                    op: BinaryOp::Less,
                    op_span: span(),
                    left: Box::new(Expr::Variable {
                        name: "i".to_string(),
                        span: span()
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 10,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                }),
                post: Some(Expr::Assign {
                    op_span: span(),
                    target: Box::new(Expr::Variable {
                        name: "i".to_string(),
                        span: span()
                    }),
                    value: Box::new(Expr::Binary {
                        op: BinaryOp::Add,
                        op_span: span(),
                        left: Box::new(Expr::Variable {
                            name: "i".to_string(),
                            span: span()
                        }),
                        right: Box::new(Expr::IntLiteral {
                            value: 1,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span()
                        }),
                    }),
                }),
                body: Box::new(Statement::Empty),
            }
        );
    }

    #[test]
    fn parses_for_loop_with_variable_declaration_init() {
        let tokens = tokens![
            TokenKind::KwFor,
            TokenKind::LParen,
            TokenKind::KwInt,
            TokenKind::Ident("i".to_string()),
            TokenKind::Equal,
            TokenKind::IntLiteral {
                value: 0,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::Ident("i".to_string()),
            TokenKind::Less,
            TokenKind::IntLiteral {
                value: 10,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::Ident("i".to_string()),
            TokenKind::Equal,
            TokenKind::Ident("i".to_string()),
            TokenKind::Plus,
            TokenKind::IntLiteral {
                value: 1,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::RParen,
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::For {
                init: Some(Box::new(Statement::Decl(vec![LocalDecl {
                    ty: Type::Int,
                    name_span: span(),
                    name: "i".to_string(),
                    init: Some(Initializer::Expr(Expr::IntLiteral {
                        value: 0,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    })),
                }]))),
                cond: Some(Expr::Binary {
                    op: BinaryOp::Less,
                    op_span: span(),
                    left: Box::new(Expr::Variable {
                        name: "i".to_string(),
                        span: span()
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 10,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                }),
                post: Some(Expr::Assign {
                    op_span: span(),
                    target: Box::new(Expr::Variable {
                        name: "i".to_string(),
                        span: span()
                    }),
                    value: Box::new(Expr::Binary {
                        op: BinaryOp::Add,
                        op_span: span(),
                        left: Box::new(Expr::Variable {
                            name: "i".to_string(),
                            span: span()
                        }),
                        right: Box::new(Expr::IntLiteral {
                            value: 1,
                            suffix: IntLiteralSuffix::None,
                            base: IntLiteralBase::Decimal,
                            span: span()
                        }),
                    }),
                }),
                body: Box::new(Statement::Empty),
            }
        );
    }

    #[test]
    fn parses_for_loop_with_empty_init_and_post() {
        let tokens = tokens![
            TokenKind::KwFor,
            TokenKind::LParen,
            TokenKind::Semicolon,
            TokenKind::Ident("i".to_string()),
            TokenKind::Less,
            TokenKind::IntLiteral {
                value: 10,
                suffix: IntLiteralSuffix::None,
                base: IntLiteralBase::Decimal,
            },
            TokenKind::Semicolon,
            TokenKind::RParen,
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::For {
                init: None,
                cond: Some(Expr::Binary {
                    op: BinaryOp::Less,
                    op_span: span(),
                    left: Box::new(Expr::Variable {
                        name: "i".to_string(),
                        span: span()
                    }),
                    right: Box::new(Expr::IntLiteral {
                        value: 10,
                        suffix: IntLiteralSuffix::None,
                        base: IntLiteralBase::Decimal,
                        span: span()
                    }),
                }),
                post: None,
                body: Box::new(Statement::Empty),
            }
        );
    }
}
