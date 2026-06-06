use crate::ast::{BinaryOp, Expr, Function, Program, Statement, UnaryOp};
use crate::lexer::{Span, Token, TokenKind};

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

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        token
    }

    fn expect(&mut self, expected: TokenKind) -> Result<Token, ParseError> {
        let token = self.advance();

        if token.kind == expected {
            Ok(token)
        } else {
            Err(ParseError {
                message: format!("expected {expected:?}, found {:?}", token.kind),
                span: token.span,
            })
        }
    }

    fn parse_left_assoc(
        &mut self,
        parse_operand: fn(&mut Self) -> Result<Expr, ParseError>,
        ops: &[(TokenKind, BinaryOp)],
    ) -> Result<Expr, ParseError> {
        let mut left = parse_operand(self)?;

        while let Some(op) = self.parse_binary_op_from(ops) {
            let right = parse_operand(self)?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }

        Ok(left)
    }

    fn parse_binary_op_from(&mut self, ops: &[(TokenKind, BinaryOp)]) -> Option<BinaryOp> {
        for (token_kind, op) in ops {
            if self.peek_kind() == token_kind {
                self.advance();
                return Some(*op);
            }
        }

        None
    }

    fn parse_comma_separated_until_rparen<T>(
        &mut self,
        parse_item: fn(&mut Self) -> Result<T, ParseError>,
    ) -> Result<Vec<T>, ParseError> {
        let mut items = vec![];

        while *self.peek_kind() != TokenKind::RParen {
            items.push(parse_item(self)?);

            if *self.peek_kind() == TokenKind::Comma {
                self.expect(TokenKind::Comma)?;

                if *self.peek_kind() == TokenKind::RParen {
                    return Err(ParseError {
                        message: "trailing comma".to_string(),
                        span: self.peek().span,
                    });
                }
            }
        }

        Ok(items)
    }

    fn parse_call_arg(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr()
    }

    fn parse_param(&mut self) -> Result<String, ParseError> {
        self.expect(TokenKind::KwInt)?;

        let token = self.advance();

        match token.kind {
            TokenKind::Ident(name) => Ok(name),
            found => Err(ParseError {
                message: format!("expected identifier for parameter, found {found:?}"),
                span: token.span,
            }),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let token = self.advance();

        match token.kind {
            TokenKind::IntLiteral(value) => Ok(Expr::IntLiteral(value)),
            TokenKind::Ident(name) => {
                if *self.peek_kind() == TokenKind::LParen {
                    self.expect(TokenKind::LParen)?;
                    let args = self.parse_comma_separated_until_rparen(Self::parse_call_arg)?;
                    self.expect(TokenKind::RParen)?;
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
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            found => {
                return Err(ParseError {
                    message: format!("expected expression, found {found:?}"),
                    span: token.span,
                });
            }
        }
    }

    fn parse_unary_op(&mut self) -> Option<UnaryOp> {
        match self.peek_kind() {
            TokenKind::Minus => {
                self.advance();
                Some(UnaryOp::Negate)
            }
            TokenKind::Bang => {
                self.advance();
                Some(UnaryOp::LogicalNot)
            }
            TokenKind::Tilde => {
                self.advance();
                Some(UnaryOp::BitwiseNot)
            }
            _ => None,
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if let Some(op) = self.parse_unary_op() {
            let right = self.parse_unary()?;
            return Ok(Expr::Unary {
                op,
                expr: Box::new(right),
            });
        }

        self.parse_primary()
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

    fn parse_assignment(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_logical_or()?;

        if *self.peek_kind() == TokenKind::Equal {
            if let Expr::Variable { name, span } = left {
                self.expect(TokenKind::Equal)?;
                let value = Box::new(self.parse_assignment()?);
                return Ok(Expr::Assign {
                    name,
                    name_span: span,
                    value,
                });
            } else {
                return Err(ParseError {
                    message: format!("cannot assign to non-variable expression"),
                    span: self.peek().span,
                });
            }
        }

        Ok(left)
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_assignment()
    }

    fn parse_var_decl(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::KwInt)?;
        let token = self.advance();
        let name = match token.kind {
            TokenKind::Ident(name) => name,
            found => {
                return Err(ParseError {
                    message: format!("expected variable name, found {found:?}"),
                    span: token.span,
                });
            }
        };
        if *self.peek_kind() == TokenKind::Semicolon {
            self.expect(TokenKind::Semicolon)?;
            return Ok(Statement::VarDecl {
                name,
                name_span: token.span,
                init: None,
            });
        }
        self.expect(TokenKind::Equal)?;
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Statement::VarDecl {
            name,
            name_span: token.span,
            init: Some(expr),
        })
    }

    fn parse_through_rbrace(&mut self, vec: &mut Vec<Statement>) -> Result<(), ParseError> {
        while *self.peek_kind() != TokenKind::RBrace {
            vec.push(self.parse_statement()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(())
    }

    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::KwIf)?;
        self.expect(TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        let then_statement = self.parse_statement()?;
        let else_statement = if *self.peek_kind() == TokenKind::KwElse {
            self.expect(TokenKind::KwElse)?;
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };

        Ok(Statement::If {
            cond,
            then_branch: Box::new(then_statement),
            else_branch: else_statement,
        })
    }

    fn parse_while_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::KwWhile)?;
        self.expect(TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        let body = Box::new(self.parse_statement()?);

        Ok(Statement::While { cond, body })
    }

    fn parse_expr_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Statement::ExprStatement(expr))
    }

    fn parse_for_statement_init(&mut self) -> Result<Statement, ParseError> {
        match self.peek_kind() {
            // Variable declaration
            TokenKind::KwInt => self.parse_var_decl(),
            // Empty
            TokenKind::Semicolon => {
                self.expect(TokenKind::Semicolon)?;
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
        self.expect(TokenKind::KwFor)?;
        self.expect(TokenKind::LParen)?;

        let mut init = None;
        let mut cond = None;
        let mut post = None;

        if *self.peek_kind() == TokenKind::Semicolon {
            self.expect(TokenKind::Semicolon)?;
        } else {
            init = Some(Box::new(self.parse_for_statement_init()?));
        }

        if *self.peek_kind() != TokenKind::Semicolon {
            cond = Some(self.parse_expr()?);
        }
        self.expect(TokenKind::Semicolon)?;

        if *self.peek_kind() != TokenKind::RParen {
            post = Some(self.parse_expr()?)
        }
        self.expect(TokenKind::RParen)?;

        let body = Box::new(self.parse_statement()?);

        Ok(Statement::For {
            init,
            cond,
            post,
            body,
        })
    }

    fn is_expr_start(token_kind: &TokenKind) -> bool {
        matches!(
            token_kind,
            TokenKind::Ident(_)
                | TokenKind::IntLiteral(_)
                | TokenKind::LParen
                | TokenKind::Minus
                | TokenKind::Bang
                | TokenKind::Tilde
        )
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek_kind() {
            // Control flow
            TokenKind::KwReturn => {
                self.expect(TokenKind::KwReturn)?;
                let expr = self.parse_expr()?;
                self.expect(TokenKind::Semicolon)?;
                Ok(Statement::Return(expr))
            }
            TokenKind::KwIf => self.parse_if_statement(),
            TokenKind::KwWhile => self.parse_while_statement(),
            TokenKind::KwBreak => {
                let token = self.expect(TokenKind::KwBreak)?;
                self.expect(TokenKind::Semicolon)?;
                Ok(Statement::Break { span: token.span })
            }
            TokenKind::KwContinue => {
                let token = self.expect(TokenKind::KwContinue)?;
                self.expect(TokenKind::Semicolon)?;
                Ok(Statement::Continue { span: token.span })
            }
            TokenKind::KwFor => self.parse_for_statement(),
            // Variable declaration
            TokenKind::KwInt => self.parse_var_decl(),
            // Block
            TokenKind::LBrace => {
                self.expect(TokenKind::LBrace)?;
                let mut body = vec![];
                self.parse_through_rbrace(&mut body)?;
                Ok(Statement::Block(body))
            }
            // Empty
            TokenKind::Semicolon => {
                self.expect(TokenKind::Semicolon)?;
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

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        self.expect(TokenKind::KwInt)?;

        let token = self.advance();

        let name = match token.kind {
            TokenKind::Ident(name) => name,
            found => {
                return Err(ParseError {
                    message: format!("expected function name, found {found:?}"),
                    span: token.span,
                });
            }
        };

        self.expect(TokenKind::LParen)?;

        let params = self.parse_comma_separated_until_rparen(Self::parse_param)?;
        self.expect(TokenKind::RParen)?;

        self.expect(TokenKind::LBrace)?;

        let mut body = vec![];
        self.parse_through_rbrace(&mut body)?;

        Ok(Function {
            name,
            name_span: token.span,
            params,
            body,
        })
    }

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut functions = vec![];

        while *self.peek_kind() != TokenKind::Eof {
            functions.push(self.parse_function()?);
        }
        let token = self.expect(TokenKind::Eof)?;

        Ok(Program {
            functions,
            eof_span: token.span,
        })
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span() -> Span {
        Span { start: 0, end: 0 }
    }

    fn token(kind: TokenKind) -> Token {
        Token {
            kind,
            span: Span { start: 0, end: 0 },
        }
    }

    fn token_with_span(kind: TokenKind, start: usize, end: usize) -> Token {
        Token {
            kind,
            span: Span { start, end },
        }
    }

    macro_rules! tokens {
        ($($kind:expr),* $(,)?) => {
            vec![$(token($kind)),*]
        };
    }

    #[test]
    fn parse_expect_errors_use_found_token_span() {
        let tokens = vec![
            token_with_span(TokenKind::IntLiteral(1), 0, 1),
            token_with_span(TokenKind::Eof, 1, 1),
        ];

        let mut parser = Parser::new(tokens);
        let err = parser
            .parse_statement()
            .expect_err("missing semicolon should fail");

        assert_eq!(err.span, Span { start: 1, end: 1 });
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

        assert_eq!(err.span, Span { start: 7, end: 8 });
        assert_eq!(err.message, "expected expression, found RParen");
    }

    #[test]
    fn trailing_comma_errors_point_at_right_paren() {
        let tokens = vec![
            token_with_span(TokenKind::KwReturn, 0, 6),
            token_with_span(TokenKind::Ident("add".to_string()), 7, 10),
            token_with_span(TokenKind::LParen, 10, 11),
            token_with_span(TokenKind::IntLiteral(1), 11, 12),
            token_with_span(TokenKind::Comma, 12, 13),
            token_with_span(TokenKind::RParen, 14, 15),
            token_with_span(TokenKind::Semicolon, 15, 16),
        ];

        let mut parser = Parser::new(tokens);
        let err = parser
            .parse_statement()
            .expect_err("trailing argument comma should fail");

        assert_eq!(err.span, Span { start: 14, end: 15 });
        assert_eq!(err.message, "trailing comma");
    }

    #[test]
    fn assignment_target_errors_point_at_equal_token() {
        let tokens = vec![
            token_with_span(TokenKind::KwReturn, 0, 6),
            token_with_span(TokenKind::LParen, 7, 8),
            token_with_span(TokenKind::IntLiteral(1), 8, 9),
            token_with_span(TokenKind::Plus, 10, 11),
            token_with_span(TokenKind::IntLiteral(2), 12, 13),
            token_with_span(TokenKind::RParen, 13, 14),
            token_with_span(TokenKind::Equal, 15, 16),
            token_with_span(TokenKind::IntLiteral(3), 17, 18),
            token_with_span(TokenKind::Semicolon, 18, 19),
        ];

        let mut parser = Parser::new(tokens);
        let err = parser
            .parse_statement()
            .expect_err("assignment to non-variable should fail");

        assert_eq!(err.span, Span { start: 15, end: 16 });
        assert_eq!(err.message, "cannot assign to non-variable expression");
    }

    #[test]
    fn basic_parse() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::IntLiteral(42),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                functions: vec![Function {
                    name_span: span(),
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![Statement::Return(Expr::IntLiteral(42))],
                }],
                eof_span: span(),
            }
        );
    }

    #[test]
    fn parse_binary_op() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral(1),
            TokenKind::Plus,
            TokenKind::IntLiteral(2),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::IntLiteral(1)),
                right: Box::new(Expr::IntLiteral(2)),
            })
        )
    }

    #[test]
    fn parses_function_returning_binary_op() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::IntLiteral(1),
            TokenKind::Plus,
            TokenKind::IntLiteral(2),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                functions: vec![Function {
                    name_span: span(),
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![Statement::Return(Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::IntLiteral(1)),
                        right: Box::new(Expr::IntLiteral(2)),
                    })],
                }],
                eof_span: span(),
            }
        )
    }

    #[test]
    fn parses_zero_argument_function_call() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::Ident("helper".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Call {
                name_span: span(),
                name: "helper".to_string(),
                args: vec![],
            })
        );
    }

    #[test]
    fn parses_function_call_as_binary_operand() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::Ident("helper".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::Plus,
            TokenKind::IntLiteral(2),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Call {
                    name_span: span(),
                    name: "helper".to_string(),
                    args: vec![],
                }),
                right: Box::new(Expr::IntLiteral(2)),
            })
        );
    }

    #[test]
    fn parses_empty_statement() {
        let tokens = tokens![TokenKind::Semicolon];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(statement, Statement::Empty);
        assert_eq!(parser.pos, 1, "empty statement should consume semicolon");
    }

    #[test]
    fn parses_function_call_expression_statement() {
        let tokens = tokens![
            TokenKind::Ident("helper".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::ExprStatement(Expr::Call {
                name_span: span(),
                name: "helper".to_string(),
                args: vec![],
            })
        );
        assert_eq!(
            parser.pos, 4,
            "expression statement should consume semicolon"
        );
    }

    #[test]
    fn parses_literal_expression_statement() {
        let tokens = tokens![
            TokenKind::IntLiteral(1),
            TokenKind::Plus,
            TokenKind::IntLiteral(2),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::ExprStatement(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::IntLiteral(1)),
                right: Box::new(Expr::IntLiteral(2)),
            })
        );
        assert_eq!(
            parser.pos, 4,
            "expression statement should consume semicolon"
        );
    }

    #[test]
    fn parses_unary_expression_statement() {
        let tokens = tokens![
            TokenKind::Bang,
            TokenKind::IntLiteral(0),
            TokenKind::Semicolon
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::ExprStatement(Expr::Unary {
                op: UnaryOp::LogicalNot,
                expr: Box::new(Expr::IntLiteral(0)),
            })
        );
        assert_eq!(
            parser.pos, 3,
            "expression statement should consume semicolon"
        );
    }

    #[test]
    fn rejects_expression_statement_without_semicolon() {
        let tokens = tokens![TokenKind::IntLiteral(1), TokenKind::Eof];

        let mut parser = Parser::new(tokens);

        assert!(
            parser.parse_statement().is_err(),
            "expression statements should require semicolons"
        );
    }

    #[test]
    fn parses_function_parameters() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("add".to_string()),
            TokenKind::LParen,
            TokenKind::KwInt,
            TokenKind::Ident("x".to_string()),
            TokenKind::Comma,
            TokenKind::KwInt,
            TokenKind::Ident("y".to_string()),
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::Ident("x".to_string()),
            TokenKind::Plus,
            TokenKind::Ident("y".to_string()),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                functions: vec![Function {
                    name_span: span(),
                    name: "add".to_string(),
                    params: vec!["x".to_string(), "y".to_string()],
                    body: vec![Statement::Return(Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                            span: span()
                        }),
                        right: Box::new(Expr::Variable {
                            name: "y".to_string(),
                            span: span()
                        }),
                    })],
                }],
                eof_span: span(),
            }
        );
    }

    #[test]
    fn parses_function_call_arguments() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::Ident("add".to_string()),
            TokenKind::LParen,
            TokenKind::IntLiteral(1),
            TokenKind::Comma,
            TokenKind::IntLiteral(2),
            TokenKind::Plus,
            TokenKind::IntLiteral(3),
            TokenKind::RParen,
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Call {
                name_span: span(),
                name: "add".to_string(),
                args: vec![
                    Expr::IntLiteral(1),
                    Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::IntLiteral(2)),
                        right: Box::new(Expr::IntLiteral(3)),
                    },
                ],
            })
        );
    }

    #[test]
    fn rejects_trailing_comma_in_function_call() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::Ident("add".to_string()),
            TokenKind::LParen,
            TokenKind::IntLiteral(1),
            TokenKind::Comma,
            TokenKind::RParen,
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        assert!(
            parser.parse_statement().is_err(),
            "function calls should reject trailing commas"
        );
    }

    #[test]
    fn rejects_trailing_comma_in_function_parameters() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("add".to_string()),
            TokenKind::LParen,
            TokenKind::KwInt,
            TokenKind::Ident("x".to_string()),
            TokenKind::Comma,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::Ident("x".to_string()),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        assert!(
            parse(tokens).is_err(),
            "function parameter lists should reject trailing commas"
        );
    }

    #[test]
    fn parses_chained_addition_left_associative() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral(1),
            TokenKind::Plus,
            TokenKind::IntLiteral(2),
            TokenKind::Plus,
            TokenKind::IntLiteral(3),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::IntLiteral(1)),
                    right: Box::new(Expr::IntLiteral(2)),
                }),
                right: Box::new(Expr::IntLiteral(3)),
            })
        )
    }

    #[test]
    fn parses_subtraction() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral(5),
            TokenKind::Minus,
            TokenKind::IntLiteral(2),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::Subtract,
                left: Box::new(Expr::IntLiteral(5)),
                right: Box::new(Expr::IntLiteral(2)),
            })
        )
    }

    #[test]
    fn parses_chained_subtraction_left_associative() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral(5),
            TokenKind::Minus,
            TokenKind::IntLiteral(2),
            TokenKind::Minus,
            TokenKind::IntLiteral(1),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::Subtract,
                left: Box::new(Expr::Binary {
                    op: BinaryOp::Subtract,
                    left: Box::new(Expr::IntLiteral(5)),
                    right: Box::new(Expr::IntLiteral(2)),
                }),
                right: Box::new(Expr::IntLiteral(1)),
            })
        )
    }

    #[test]
    fn parses_multiplication() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral(2),
            TokenKind::Star,
            TokenKind::IntLiteral(3),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(Expr::IntLiteral(2)),
                right: Box::new(Expr::IntLiteral(3)),
            })
        )
    }

    #[test]
    fn parses_unary_negation() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::Minus,
            TokenKind::IntLiteral(5),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Unary {
                op: UnaryOp::Negate,
                expr: Box::new(Expr::IntLiteral(5)),
            })
        )
    }

    #[test]
    fn parses_logical_not() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::Bang,
            TokenKind::IntLiteral(0),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Unary {
                op: UnaryOp::LogicalNot,
                expr: Box::new(Expr::IntLiteral(0)),
            })
        )
    }

    #[test]
    fn parses_bitwise_not() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::Tilde,
            TokenKind::IntLiteral(1),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Unary {
                op: UnaryOp::BitwiseNot,
                expr: Box::new(Expr::IntLiteral(1)),
            })
        )
    }

    #[test]
    fn parses_unary_before_multiplication() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::Minus,
            TokenKind::Ident("x".to_string()),
            TokenKind::Star,
            TokenKind::IntLiteral(2),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(Expr::Unary {
                    op: UnaryOp::Negate,
                    expr: Box::new(Expr::Variable {
                        name: "x".to_string(),
                        span: span()
                    }),
                }),
                right: Box::new(Expr::IntLiteral(2)),
            })
        )
    }

    #[test]
    fn parses_variable_declaration() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("x".to_string()),
            TokenKind::Equal,
            TokenKind::IntLiteral(5),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::VarDecl {
                name_span: span(),
                name: "x".to_string(),
                init: Some(Expr::IntLiteral(5)),
            }
        )
    }

    #[test]
    fn parses_variable_declaration_without_initializer() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("x".to_string()),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::VarDecl {
                name_span: span(),
                name: "x".to_string(),
                init: None,
            }
        );
        assert_eq!(
            parser.pos, 3,
            "declaration without initializer should consume semicolon"
        );
    }

    #[test]
    fn parses_function_with_assignment() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::KwInt,
            TokenKind::Ident("x".to_string()),
            TokenKind::Equal,
            TokenKind::IntLiteral(1),
            TokenKind::Semicolon,
            TokenKind::Ident("x".to_string()),
            TokenKind::Equal,
            TokenKind::Ident("x".to_string()),
            TokenKind::Plus,
            TokenKind::IntLiteral(2),
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
            Program {
                functions: vec![Function {
                    name_span: span(),
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![
                        Statement::VarDecl {
                            name_span: span(),
                            name: "x".to_string(),
                            init: Some(Expr::IntLiteral(1)),
                        },
                        Statement::ExprStatement(Expr::Assign {
                            name_span: span(),
                            name: "x".to_string(),
                            value: Box::new(Expr::Binary {
                                op: BinaryOp::Add,
                                left: Box::new(Expr::Variable {
                                    name: "x".to_string(),
                                    span: span()
                                }),
                                right: Box::new(Expr::IntLiteral(2)),
                            }),
                        }),
                        Statement::Return(Expr::Variable {
                            name: "x".to_string(),
                            span: span()
                        }),
                    ],
                }],
                eof_span: span(),
            }
        )
    }

    #[test]
    fn parses_assignment_as_expression_statement() {
        let tokens = tokens![
            TokenKind::Ident("x".to_string()),
            TokenKind::Equal,
            TokenKind::IntLiteral(3),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::ExprStatement(Expr::Assign {
                name_span: span(),
                name: "x".to_string(),
                value: Box::new(Expr::IntLiteral(3)),
            })
        );
    }

    #[test]
    fn parses_assignment_as_return_expression() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::Ident("x".to_string()),
            TokenKind::Equal,
            TokenKind::IntLiteral(3),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Assign {
                name_span: span(),
                name: "x".to_string(),
                value: Box::new(Expr::IntLiteral(3)),
            })
        );
    }

    #[test]
    fn parses_assignment_right_associative() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::Ident("x".to_string()),
            TokenKind::Equal,
            TokenKind::Ident("y".to_string()),
            TokenKind::Equal,
            TokenKind::IntLiteral(4),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Assign {
                name_span: span(),
                name: "x".to_string(),
                value: Box::new(Expr::Assign {
                    name_span: span(),
                    name: "y".to_string(),
                    value: Box::new(Expr::IntLiteral(4)),
                }),
            })
        );
    }

    #[test]
    fn rejects_assignment_to_non_variable_expression() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::LParen,
            TokenKind::IntLiteral(1),
            TokenKind::Plus,
            TokenKind::IntLiteral(2),
            TokenKind::RParen,
            TokenKind::Equal,
            TokenKind::IntLiteral(3),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let err = parser
            .parse_statement()
            .expect_err("assignment target should be rejected");

        assert_eq!(err.message, "cannot assign to non-variable expression");
    }

    #[test]
    fn parses_block_statement() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::IntLiteral(1),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                functions: vec![Function {
                    name_span: span(),
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![Statement::Block(vec![Statement::Return(Expr::IntLiteral(
                        1
                    ))])],
                }],
                eof_span: span(),
            }
        )
    }

    #[test]
    fn parses_nested_block_statements() {
        let tokens = tokens![
            TokenKind::KwInt,
            TokenKind::Ident("main".to_string()),
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::LBrace,
            TokenKind::LBrace,
            TokenKind::KwReturn,
            TokenKind::IntLiteral(1),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::RBrace,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                functions: vec![Function {
                    name_span: span(),
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![Statement::Block(vec![Statement::Block(vec![
                        Statement::Return(Expr::IntLiteral(1))
                    ])])],
                }],
                eof_span: span(),
            }
        )
    }

    #[test]
    fn rejects_if_without_parentheses() {
        let tokens = tokens![
            TokenKind::KwIf,
            TokenKind::Ident("x".to_string()),
            TokenKind::KwReturn,
            TokenKind::IntLiteral(1),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        assert!(
            parser.parse_statement().is_err(),
            "if statements should require parentheses around the condition"
        );
    }

    #[test]
    fn parses_while_statement() {
        let tokens = tokens![
            TokenKind::KwWhile,
            TokenKind::LParen,
            TokenKind::Ident("x".to_string()),
            TokenKind::RParen,
            TokenKind::Ident("x".to_string()),
            TokenKind::Equal,
            TokenKind::Ident("x".to_string()),
            TokenKind::Minus,
            TokenKind::IntLiteral(1),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::While {
                cond: Expr::Variable {
                    name: "x".to_string(),
                    span: span()
                },
                body: Box::new(Statement::ExprStatement(Expr::Assign {
                    name_span: span(),
                    name: "x".to_string(),
                    value: Box::new(Expr::Binary {
                        op: BinaryOp::Subtract,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                            span: span()
                        }),
                        right: Box::new(Expr::IntLiteral(1)),
                    }),
                })),
            }
        );
    }

    #[test]
    fn parses_break_statement() {
        let tokens = tokens![TokenKind::KwBreak, TokenKind::Semicolon];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(statement, Statement::Break { span: span() });
        assert_eq!(parser.pos, 2, "break statement should consume semicolon");
    }

    #[test]
    fn parses_continue_statement() {
        let tokens = tokens![TokenKind::KwContinue, TokenKind::Semicolon];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(statement, Statement::Continue { span: span() });
        assert_eq!(parser.pos, 2, "continue statement should consume semicolon");
    }

    #[test]
    fn rejects_break_without_semicolon() {
        let tokens = tokens![TokenKind::KwBreak, TokenKind::Eof];

        let mut parser = Parser::new(tokens);

        assert!(
            parser.parse_statement().is_err(),
            "break statements should require semicolons"
        );
    }

    #[test]
    fn rejects_continue_without_semicolon() {
        let tokens = tokens![TokenKind::KwContinue, TokenKind::Eof];

        let mut parser = Parser::new(tokens);

        assert!(
            parser.parse_statement().is_err(),
            "continue statements should require semicolons"
        );
    }

    #[test]
    fn rejects_while_without_parentheses() {
        let tokens = tokens![
            TokenKind::KwWhile,
            TokenKind::Ident("x".to_string()),
            TokenKind::Ident("x".to_string()),
            TokenKind::Equal,
            TokenKind::IntLiteral(0),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        assert!(
            parser.parse_statement().is_err(),
            "while statements should require parentheses around the condition"
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
            TokenKind::IntLiteral(1),
            TokenKind::Plus,
            TokenKind::IntLiteral(2),
            TokenKind::RParen,
            TokenKind::Star,
            TokenKind::IntLiteral(3),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                functions: vec![Function {
                    name_span: span(),
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![Statement::Return(Expr::Binary {
                        op: BinaryOp::Multiply,
                        left: Box::new(Expr::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::IntLiteral(1)),
                            right: Box::new(Expr::IntLiteral(2)),
                        }),
                        right: Box::new(Expr::IntLiteral(3)),
                    })],
                }],
                eof_span: span(),
            }
        )
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
            TokenKind::IntLiteral(1),
            TokenKind::Plus,
            TokenKind::IntLiteral(2),
            TokenKind::Less,
            TokenKind::IntLiteral(4),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                functions: vec![Function {
                    name_span: span(),
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![Statement::Return(Expr::Binary {
                        op: BinaryOp::Less,
                        left: Box::new(Expr::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::IntLiteral(1)),
                            right: Box::new(Expr::IntLiteral(2)),
                        }),
                        right: Box::new(Expr::IntLiteral(4)),
                    })],
                }],
                eof_span: span(),
            }
        )
    }

    #[test]
    fn parses_shift_after_additive() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral(1),
            TokenKind::Plus,
            TokenKind::IntLiteral(2),
            TokenKind::LessLess,
            TokenKind::IntLiteral(3),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::ShiftLeft,
                left: Box::new(Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::IntLiteral(1)),
                    right: Box::new(Expr::IntLiteral(2)),
                }),
                right: Box::new(Expr::IntLiteral(3)),
            })
        )
    }

    #[test]
    fn parses_relational_after_shift() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral(1),
            TokenKind::LessLess,
            TokenKind::IntLiteral(2),
            TokenKind::Less,
            TokenKind::IntLiteral(8),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::Less,
                left: Box::new(Expr::Binary {
                    op: BinaryOp::ShiftLeft,
                    left: Box::new(Expr::IntLiteral(1)),
                    right: Box::new(Expr::IntLiteral(2)),
                }),
                right: Box::new(Expr::IntLiteral(8)),
            })
        )
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
                left: Box::new(Expr::Variable {
                    name: "a".to_string(),
                    span: span()
                }),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::Equal,
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
        )
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
                left: Box::new(Expr::Variable {
                    name: "a".to_string(),
                    span: span()
                }),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::BitAnd,
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
        )
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
                left: Box::new(Expr::Variable {
                    name: "a".to_string(),
                    span: span()
                }),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::BitXor,
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
        )
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
            TokenKind::IntLiteral(1),
            TokenKind::Plus,
            TokenKind::IntLiteral(2),
            TokenKind::RParen,
            TokenKind::EqualEqual,
            TokenKind::IntLiteral(3),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                functions: vec![Function {
                    name_span: span(),
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![Statement::Return(Expr::Binary {
                        op: BinaryOp::Equal,
                        left: Box::new(Expr::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::IntLiteral(1)),
                            right: Box::new(Expr::IntLiteral(2)),
                        }),
                        right: Box::new(Expr::IntLiteral(3)),
                    })],
                }],
                eof_span: span(),
            }
        )
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
            TokenKind::IntLiteral(10),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                functions: vec![Function {
                    name_span: span(),
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![Statement::Return(Expr::Binary {
                        op: BinaryOp::GreaterEqual,
                        left: Box::new(Expr::Variable {
                            name: "x".to_string(),
                            span: span()
                        }),
                        right: Box::new(Expr::IntLiteral(10)),
                    })],
                }],
                eof_span: span(),
            }
        )
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
            TokenKind::IntLiteral(1),
            TokenKind::Less,
            TokenKind::IntLiteral(2),
            TokenKind::Less,
            TokenKind::IntLiteral(3),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                functions: vec![Function {
                    name_span: span(),
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![Statement::Return(Expr::Binary {
                        op: BinaryOp::Less,
                        left: Box::new(Expr::Binary {
                            op: BinaryOp::Less,
                            left: Box::new(Expr::IntLiteral(1)),
                            right: Box::new(Expr::IntLiteral(2)),
                        }),
                        right: Box::new(Expr::IntLiteral(3)),
                    })],
                }],
                eof_span: span(),
            }
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
            TokenKind::IntLiteral(5),
            TokenKind::Semicolon,
            TokenKind::KwReturn,
            TokenKind::IntLiteral(42),
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                functions: vec![Function {
                    name_span: span(),
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![
                        Statement::VarDecl {
                            name_span: span(),
                            name: "x".to_string(),
                            init: Some(Expr::IntLiteral(5)),
                        },
                        Statement::Return(Expr::IntLiteral(42)),
                    ],
                }],
                eof_span: span(),
            }
        )
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
            TokenKind::IntLiteral(5),
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
            Program {
                functions: vec![Function {
                    name_span: span(),
                    name: "main".to_string(),
                    params: vec![],
                    body: vec![
                        Statement::VarDecl {
                            name_span: span(),
                            name: "x".to_string(),
                            init: Some(Expr::IntLiteral(5)),
                        },
                        Statement::Return(Expr::Variable {
                            name: "x".to_string(),
                            span: span()
                        }),
                    ],
                }],
                eof_span: span(),
            }
        )
    }

    #[test]
    fn multiplication_has_higher_precedence_than_addition() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral(1),
            TokenKind::Plus,
            TokenKind::IntLiteral(2),
            TokenKind::Star,
            TokenKind::IntLiteral(3),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::Add,
                left: Box::new(Expr::IntLiteral(1)),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::Multiply,
                    left: Box::new(Expr::IntLiteral(2)),
                    right: Box::new(Expr::IntLiteral(3)),
                }),
            })
        )
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
            TokenKind::IntLiteral(1),
            TokenKind::PipePipe,
            TokenKind::IntLiteral(0),
            TokenKind::AmpersandAmpersand,
            TokenKind::IntLiteral(2),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::LogicalOr,
                left: Box::new(Expr::IntLiteral(1)),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::LogicalAnd,
                    left: Box::new(Expr::IntLiteral(0)),
                    right: Box::new(Expr::IntLiteral(2)),
                }),
            })
        );
    }

    #[test]
    fn parses_bitwise_or_before_logical_and() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral(1),
            TokenKind::Pipe,
            TokenKind::IntLiteral(2),
            TokenKind::AmpersandAmpersand,
            TokenKind::IntLiteral(3),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::LogicalAnd,
                left: Box::new(Expr::Binary {
                    op: BinaryOp::BitOr,
                    left: Box::new(Expr::IntLiteral(1)),
                    right: Box::new(Expr::IntLiteral(2)),
                }),
                right: Box::new(Expr::IntLiteral(3)),
            })
        );
    }

    #[test]
    fn parses_chained_logical_and_left_associative() {
        let tokens = tokens![
            TokenKind::KwReturn,
            TokenKind::IntLiteral(1),
            TokenKind::AmpersandAmpersand,
            TokenKind::IntLiteral(2),
            TokenKind::AmpersandAmpersand,
            TokenKind::IntLiteral(3),
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::LogicalAnd,
                left: Box::new(Expr::Binary {
                    op: BinaryOp::LogicalAnd,
                    left: Box::new(Expr::IntLiteral(1)),
                    right: Box::new(Expr::IntLiteral(2)),
                }),
                right: Box::new(Expr::IntLiteral(3)),
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
            TokenKind::IntLiteral(0),
            TokenKind::Semicolon,
            TokenKind::Ident("i".to_string()),
            TokenKind::Less,
            TokenKind::IntLiteral(10),
            TokenKind::Semicolon,
            TokenKind::Ident("i".to_string()),
            TokenKind::Equal,
            TokenKind::Ident("i".to_string()),
            TokenKind::Plus,
            TokenKind::IntLiteral(1),
            TokenKind::RParen,
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::For {
                init: Some(Box::new(Statement::ExprStatement(Expr::Assign {
                    name_span: span(),
                    name: "i".to_string(),
                    value: Box::new(Expr::IntLiteral(0)),
                }))),
                cond: Some(Expr::Binary {
                    op: BinaryOp::Less,
                    left: Box::new(Expr::Variable {
                        name: "i".to_string(),
                        span: span()
                    }),
                    right: Box::new(Expr::IntLiteral(10)),
                }),
                post: Some(Expr::Assign {
                    name_span: span(),
                    name: "i".to_string(),
                    value: Box::new(Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Variable {
                            name: "i".to_string(),
                            span: span()
                        }),
                        right: Box::new(Expr::IntLiteral(1)),
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
            TokenKind::IntLiteral(0),
            TokenKind::Semicolon,
            TokenKind::Ident("i".to_string()),
            TokenKind::Less,
            TokenKind::IntLiteral(10),
            TokenKind::Semicolon,
            TokenKind::Ident("i".to_string()),
            TokenKind::Equal,
            TokenKind::Ident("i".to_string()),
            TokenKind::Plus,
            TokenKind::IntLiteral(1),
            TokenKind::RParen,
            TokenKind::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::For {
                init: Some(Box::new(Statement::VarDecl {
                    name_span: span(),
                    name: "i".to_string(),
                    init: Some(Expr::IntLiteral(0)),
                })),
                cond: Some(Expr::Binary {
                    op: BinaryOp::Less,
                    left: Box::new(Expr::Variable {
                        name: "i".to_string(),
                        span: span()
                    }),
                    right: Box::new(Expr::IntLiteral(10)),
                }),
                post: Some(Expr::Assign {
                    name_span: span(),
                    name: "i".to_string(),
                    value: Box::new(Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Variable {
                            name: "i".to_string(),
                            span: span()
                        }),
                        right: Box::new(Expr::IntLiteral(1)),
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
            TokenKind::IntLiteral(10),
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
                    left: Box::new(Expr::Variable {
                        name: "i".to_string(),
                        span: span()
                    }),
                    right: Box::new(Expr::IntLiteral(10)),
                }),
                post: None,
                body: Box::new(Statement::Empty),
            }
        );
    }
}
