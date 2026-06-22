use std::collections::HashMap;

use crate::ast::{
    BinaryOp, Expr, ExternalDecl, FunctionDecl, FunctionDef, GlobalDecl, Initializer,
    IntLiteralBase, IntLiteralSuffix, LocalDecl, Param, ParamDecl, Program, Statement, Type,
    Typedef, UnaryOp,
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

#[derive(Default, Clone, Copy, PartialEq, Eq)]
struct TypeSpec {
    unsigned: bool,
    signed: bool,
    int_count: usize,
    char_count: usize,
    long_count: usize,
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
struct RawParam {
    pub ty: Type,
    pub ty_span: Span,
    pub name: Option<String>,
    pub name_span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DeclaratorKind {
    Name {
        name: String,
        name_span: Span,
    },
    Pointer(Box<Declarator>),
    Array {
        inner: Box<Declarator>,
        len: usize,
    },
    Function {
        inner: Box<Declarator>,
        params: Vec<RawParam>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LoweredDeclarator {
    Object {
        ty: Type,
        name: String,
        name_span: Span,
    },
    Function {
        return_type: Type,
        name: String,
        name_span: Span,
        params: Vec<RawParam>,
    },
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

    // Top level parsing

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut declarations = vec![];

        while self.peek_kind() != &TokenKind::Eof {
            declarations.extend(self.parse_external_decl()?);
        }

        let eof_span = self.expect(&TokenKind::Eof)?.span;

        Ok(Program {
            declarations,
            eof_span,
        })
    }

    fn parse_external_decl(&mut self) -> Result<Vec<ExternalDecl>, ParseError> {
        if self.peek_kind() == &TokenKind::KwTypedef {
            return Ok(self
                .parse_typedefs()?
                .into_iter()
                .map(ExternalDecl::Typedef)
                .collect());
        }

        let (spec, spec_span) = self.parse_decl_spec()?;
        let base_ty = self.lower_decl_spec(&spec, spec_span)?;
        let declarator = self.parse_declarator()?;
        let lowered = Self::lower_declarator(&base_ty, &declarator)?;

        match lowered {
            LoweredDeclarator::Function {
                return_type,
                name,
                name_span,
                params,
            } => match self.peek_kind() {
                TokenKind::Semicolon => {
                    self.expect(&TokenKind::Semicolon)?;
                    Ok(vec![ExternalDecl::FunctionDecl(FunctionDecl {
                        return_type,
                        name,
                        name_span,
                        params: params.into_iter().map(raw_param_to_decl).collect(),
                    })])
                }
                TokenKind::LBrace => {
                    self.expect(&TokenKind::LBrace)?;
                    let mut body = vec![];
                    self.parse_through_rbrace(&mut body)?;
                    Ok(vec![ExternalDecl::FunctionDef(FunctionDef {
                        return_type,
                        name,
                        name_span,
                        params: params
                            .into_iter()
                            .map(raw_param_to_def)
                            .collect::<Result<Vec<_>, _>>()?,
                        body,
                    })])
                }
                _ => {
                    let token = self.peek();
                    Err(ParseError {
                        message: format!(
                            "expected ';' or '{{' after function declarator, got '{:?}'",
                            token.kind
                        ),
                        span: token.span,
                    })
                }
            },
            LoweredDeclarator::Object {
                ty,
                name,
                name_span,
            } => match self.peek() {
                Token {
                    kind: TokenKind::Semicolon,
                    ..
                } => {
                    self.expect(&TokenKind::Semicolon)?;
                    Ok(vec![ExternalDecl::Global(GlobalDecl {
                        ty,
                        name,
                        name_span,
                        init: None,
                    })])
                }
                Token {
                    kind: TokenKind::Equal,
                    ..
                } => {
                    self.expect(&TokenKind::Equal)?;
                    let init = Some(self.parse_initializer()?);
                    self.expect(&TokenKind::Semicolon)?;
                    Ok(vec![ExternalDecl::Global(GlobalDecl {
                        ty,
                        name,
                        name_span,
                        init,
                    })])
                }
                token => Err(ParseError {
                    message: format!("unexpected token in object declaration, '{:?}'", token.kind),
                    span: token.span,
                }),
            },
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
            match Self::lower_declarator(&base_ty, &init_declarator.declarator)? {
                LoweredDeclarator::Object {
                    ty,
                    name,
                    name_span,
                } => {
                    if self.typedefs.contains_key(&name) {
                        return Err(ParseError {
                            message: format!("duplicate typedef with name '{name}'"),
                            span: name_span,
                        });
                    }
                    self.typedefs.insert(name.clone(), ty.clone());
                    typedefs.push(Typedef {
                        name,
                        name_span,
                        ty,
                    });
                }
                LoweredDeclarator::Function { name_span, .. } => {
                    return Err(ParseError {
                        message: "function typedefs are not supported".to_string(),
                        span: name_span,
                    });
                }
            }
        }
        Ok(typedefs)
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
        let mut declarator = Declarator {
            kind: DeclaratorKind::Name { name, name_span },
        };

        loop {
            match self.peek_kind() {
                TokenKind::LBracket => {
                    // Array declaration
                    self.expect(&TokenKind::LBracket)?;
                    let (len, _, _, len_span) = self.expect_int_literal()?;

                    // Don't allow array length <1
                    if len < 1 {
                        return Err(ParseError {
                            message: format!("array size must be greater than 0, got '{len}'"),
                            span: len_span,
                        });
                    }

                    self.expect(&TokenKind::RBracket)?;
                    declarator = Declarator {
                        kind: DeclaratorKind::Array {
                            inner: Box::new(declarator),
                            len: usize::try_from(len).expect("u64 cannot be converted to usize"),
                        },
                    };
                }
                TokenKind::LParen => {
                    // Function declaration
                    self.expect(&TokenKind::LParen)?;
                    let params = self.parse_comma_separated_until_terminator(
                        Self::parse_raw_param,
                        &TokenKind::RParen,
                        false,
                    )?;
                    self.expect(&TokenKind::RParen)?;
                    declarator = Declarator {
                        kind: DeclaratorKind::Function {
                            inner: Box::new(declarator),
                            params,
                        },
                    };
                }
                _ => break,
            }
        }

        Ok(declarator)
    }

    fn lower_declarator(
        base_type: &Type,
        declarator: &Declarator,
    ) -> Result<LoweredDeclarator, ParseError> {
        match &declarator.kind {
            DeclaratorKind::Name { name, name_span } => Ok(LoweredDeclarator::Object {
                ty: base_type.clone(),
                name: name.clone(),
                name_span: *name_span,
            }),
            DeclaratorKind::Pointer(inner) => {
                Self::lower_declarator(&Type::Pointer(Box::new(base_type.clone())), inner)
            }
            DeclaratorKind::Array { inner, len } => {
                match Self::lower_declarator(base_type, inner)? {
                    LoweredDeclarator::Object {
                        ty,
                        name,
                        name_span,
                    } => Ok(LoweredDeclarator::Object {
                        ty: Type::Array {
                            element: Box::new(ty),
                            len: *len,
                        },
                        name,
                        name_span,
                    }),
                    LoweredDeclarator::Function { name_span, .. } => Err(ParseError {
                        message: "function returning array is unsupported".to_string(),
                        span: name_span,
                    }),
                }
            }
            DeclaratorKind::Function { inner, params } => {
                match Self::lower_declarator(base_type, inner)? {
                    LoweredDeclarator::Object {
                        ty,
                        name,
                        name_span,
                    } => Ok(LoweredDeclarator::Function {
                        return_type: ty,
                        name,
                        name_span,
                        params: params.clone(),
                    }),
                    LoweredDeclarator::Function { name_span, .. } => Err(ParseError {
                        message: "function returning function is unsupported".to_string(),
                        span: name_span,
                    }),
                }
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
        } else if matches!(self.peek_kind(), TokenKind::Star) {
            let ty = self.parse_abstract_pointer_type(&ty)?;
            match self.peek_kind() {
                TokenKind::Comma | TokenKind::RParen => Ok(RawParam {
                    ty,
                    ty_span,
                    name: None,
                    name_span: None,
                }),
                TokenKind::Ident(_) => {
                    let (name, name_span) = self.expect_ident()?;
                    Ok(RawParam {
                        ty,
                        ty_span,
                        name: Some(name),
                        name_span: Some(name_span),
                    })
                }
                _ => {
                    let token = self.peek();
                    Err(ParseError {
                        message: format!(
                            "expected pointer type, got unexpected token '{:?}'",
                            token.kind
                        ),
                        span: token.span,
                    })
                }
            }
        } else {
            let declarator = self.parse_declarator()?;
            match Self::lower_declarator(&ty, &declarator)? {
                LoweredDeclarator::Object {
                    ty,
                    name,
                    name_span,
                } => Ok(RawParam {
                    ty,
                    ty_span,
                    name: Some(name),
                    name_span: Some(name_span),
                }),
                LoweredDeclarator::Function { name_span, .. } => Err(ParseError {
                    message: "function parameter declarators are not supported".to_string(),
                    span: name_span,
                }),
            }
        }
    }

    fn parse_abstract_pointer_type(&mut self, base: &Type) -> Result<Type, ParseError> {
        let mut ty = base.clone();

        while self.peek_kind() == &TokenKind::Star {
            self.expect(&TokenKind::Star)?;
            ty = Type::Pointer(Box::new(ty));
        }

        Ok(ty)
    }

    // Expression parsing

    fn parse_binary_op_from(&mut self, ops: &[(TokenKind, BinaryOp)]) -> Option<(BinaryOp, Span)> {
        for (token_kind, op) in ops {
            if self.peek_kind() == token_kind {
                let token = self.advance();
                return Some((*op, token.span));
            }
        }
        None
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
            TokenKind::Ampersand => {
                let token = self.advance();
                Some((UnaryOp::AddressOf, token.span))
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
            match Self::lower_declarator(&base_ty, &init_declarator.declarator)? {
                LoweredDeclarator::Object {
                    ty,
                    name,
                    name_span,
                } => {
                    declarators.push(LocalDecl {
                        ty,
                        name,
                        name_span,
                        init: init_declarator.initializer,
                    });
                }
                LoweredDeclarator::Function { name_span, .. } => {
                    return Err(ParseError {
                        message: "function declarations are not supported inside blocks"
                            .to_string(),
                        span: name_span,
                    });
                }
            }
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
