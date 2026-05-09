use crate::ast::{BinaryOp, Expr, Function, Program, Statement, UnaryOp};
use crate::lexer::Token;

const MULTIPLICATIVE_OPS: &[(Token, BinaryOp)] = &[
    (Token::Star, BinaryOp::Multiply),
    (Token::Slash, BinaryOp::Divide),
    (Token::Percent, BinaryOp::Remainder),
];

const ADDITIVE_OPS: &[(Token, BinaryOp)] = &[
    (Token::Plus, BinaryOp::Add),
    (Token::Minus, BinaryOp::Subtract),
];

const SHIFT_OPS: &[(Token, BinaryOp)] = &[
    (Token::LessLess, BinaryOp::ShiftLeft),
    (Token::GreaterGreater, BinaryOp::ShiftRight),
];

const RELATIONAL_OPS: &[(Token, BinaryOp)] = &[
    (Token::Less, BinaryOp::Less),
    (Token::LessEqual, BinaryOp::LessEqual),
    (Token::Greater, BinaryOp::Greater),
    (Token::GreaterEqual, BinaryOp::GreaterEqual),
];

const EQUALITY_OPS: &[(Token, BinaryOp)] = &[
    (Token::EqualEqual, BinaryOp::Equal),
    (Token::BangEqual, BinaryOp::NotEqual),
];

const BITWISE_AND_OPS: &[(Token, BinaryOp)] = &[(Token::Ampersand, BinaryOp::BitAnd)];
const BITWISE_XOR_OPS: &[(Token, BinaryOp)] = &[(Token::Caret, BinaryOp::BitXor)];
const BITWISE_OR_OPS: &[(Token, BinaryOp)] = &[(Token::Pipe, BinaryOp::BitOr)];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
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

    fn lookahead(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.pos + n)
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        token
    }

    fn expect(&mut self, expected: Token) -> Result<(), ParseError> {
        let found = self.advance();

        if found == expected {
            Ok(())
        } else {
            Err(ParseError {
                message: format!("expected {expected:?}, found {found:?}"),
            })
        }
    }

    fn parse_left_assoc(
        &mut self,
        parse_operand: fn(&mut Self) -> Result<Expr, ParseError>,
        ops: &[(Token, BinaryOp)],
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

    fn parse_binary_op_from(&mut self, ops: &[(Token, BinaryOp)]) -> Option<BinaryOp> {
        for (token, op) in ops {
            if self.peek() == token {
                self.advance();
                return Some(*op);
            }
        }

        None
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.advance() {
            Token::IntLiteral(value) => Ok(Expr::IntLiteral(value)),
            Token::Ident(name) => Ok(Expr::Variable(name)),
            Token::LParen => {
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            found => {
                return Err(ParseError {
                    message: format!("expected expression, found {found:?}"),
                });
            }
        }
    }

    fn parse_unary_op(&mut self) -> Option<UnaryOp> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                Some(UnaryOp::Negate)
            }
            Token::Bang => {
                self.advance();
                Some(UnaryOp::LogicalNot)
            }
            Token::Tilde => {
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

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_bitwise_or()
    }

    fn parse_var_decl(&mut self) -> Result<Statement, ParseError> {
        self.expect(Token::KwInt)?;
        let name = match self.advance() {
            Token::Ident(name) => name,
            found => {
                return Err(ParseError {
                    message: format!("expected function name, found {found:?}"),
                });
            }
        };
        self.expect(Token::Equal)?;
        let expr = self.parse_expr()?;
        self.expect(Token::Semicolon)?;
        return Ok(Statement::VarDecl { name, init: expr });
    }

    fn parse_assignment(&mut self) -> Result<Statement, ParseError> {
        let name = match self.advance() {
            Token::Ident(name) => name,
            found => {
                return Err(ParseError {
                    message: format!("got unexpected token {found:?}"),
                });
            }
        };
        self.expect(Token::Equal)?;
        let value = self.parse_expr()?;
        self.expect(Token::Semicolon)?;

        Ok(Statement::Assign { name, value })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek() {
            Token::KwReturn => {
                self.expect(Token::KwReturn)?;
                let expr = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Ok(Statement::Return(expr))
            }
            Token::KwInt => self.parse_var_decl(),
            Token::Ident(_) => match self.lookahead(1) {
                Some(Token::Equal) => self.parse_assignment(),
                Some(found) => Err(ParseError {
                    message: format!("got unexpected token {found:?}"),
                }),
                None => Err(ParseError {
                    message: "reached end of tokens".to_string(),
                }),
            },
            found => Err(ParseError {
                message: format!("got unexpected keyword {found:?}"),
            }),
        }
    }

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        self.expect(Token::KwInt)?;

        let name = match self.advance() {
            Token::Ident(name) => name,
            found => {
                return Err(ParseError {
                    message: format!("expected function name, found {found:?}"),
                });
            }
        };

        self.expect(Token::LParen)?;
        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;

        let mut body = vec![];

        while *self.peek() != Token::RBrace {
            body.push(self.parse_statement()?);
        }
        self.expect(Token::RBrace)?;

        Ok(Function { name, body })
    }

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let function = self.parse_function()?;
        self.expect(Token::Eof)?;

        Ok(Program { function })
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_parse() {
        let tokens = vec![
            Token::KwInt,
            Token::Ident("main".to_string()),
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::KwReturn,
            Token::IntLiteral(42),
            Token::Semicolon,
            Token::RBrace,
            Token::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                function: Function {
                    name: "main".to_string(),
                    body: vec![Statement::Return(Expr::IntLiteral(42))],
                },
            }
        );
    }

    #[test]
    fn parse_binary_op() {
        let tokens = vec![
            Token::KwReturn,
            Token::IntLiteral(1),
            Token::Plus,
            Token::IntLiteral(2),
            Token::Semicolon,
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
        let tokens = vec![
            Token::KwInt,
            Token::Ident("main".to_string()),
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::KwReturn,
            Token::IntLiteral(1),
            Token::Plus,
            Token::IntLiteral(2),
            Token::Semicolon,
            Token::RBrace,
            Token::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                function: Function {
                    name: "main".to_string(),
                    body: vec![Statement::Return(Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::IntLiteral(1)),
                        right: Box::new(Expr::IntLiteral(2)),
                    })],
                },
            }
        )
    }

    #[test]
    fn parses_chained_addition_left_associative() {
        let tokens = vec![
            Token::KwReturn,
            Token::IntLiteral(1),
            Token::Plus,
            Token::IntLiteral(2),
            Token::Plus,
            Token::IntLiteral(3),
            Token::Semicolon,
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
        let tokens = vec![
            Token::KwReturn,
            Token::IntLiteral(5),
            Token::Minus,
            Token::IntLiteral(2),
            Token::Semicolon,
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
        let tokens = vec![
            Token::KwReturn,
            Token::IntLiteral(5),
            Token::Minus,
            Token::IntLiteral(2),
            Token::Minus,
            Token::IntLiteral(1),
            Token::Semicolon,
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
        let tokens = vec![
            Token::KwReturn,
            Token::IntLiteral(2),
            Token::Star,
            Token::IntLiteral(3),
            Token::Semicolon,
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
        let tokens = vec![
            Token::KwReturn,
            Token::Minus,
            Token::IntLiteral(5),
            Token::Semicolon,
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
        let tokens = vec![
            Token::KwReturn,
            Token::Bang,
            Token::IntLiteral(0),
            Token::Semicolon,
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
        let tokens = vec![
            Token::KwReturn,
            Token::Tilde,
            Token::IntLiteral(1),
            Token::Semicolon,
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
        let tokens = vec![
            Token::KwReturn,
            Token::Minus,
            Token::Ident("x".to_string()),
            Token::Star,
            Token::IntLiteral(2),
            Token::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(Expr::Unary {
                    op: UnaryOp::Negate,
                    expr: Box::new(Expr::Variable("x".to_string())),
                }),
                right: Box::new(Expr::IntLiteral(2)),
            })
        )
    }

    #[test]
    fn parses_variable_declaration() {
        let tokens = vec![
            Token::KwInt,
            Token::Ident("x".to_string()),
            Token::Equal,
            Token::IntLiteral(5),
            Token::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::VarDecl {
                name: "x".to_string(),
                init: Expr::IntLiteral(5),
            }
        )
    }

    #[test]
    fn parses_function_with_assignment() {
        let tokens = vec![
            Token::KwInt,
            Token::Ident("main".to_string()),
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::KwInt,
            Token::Ident("x".to_string()),
            Token::Equal,
            Token::IntLiteral(1),
            Token::Semicolon,
            Token::Ident("x".to_string()),
            Token::Equal,
            Token::Ident("x".to_string()),
            Token::Plus,
            Token::IntLiteral(2),
            Token::Semicolon,
            Token::KwReturn,
            Token::Ident("x".to_string()),
            Token::Semicolon,
            Token::RBrace,
            Token::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                function: Function {
                    name: "main".to_string(),
                    body: vec![
                        Statement::VarDecl {
                            name: "x".to_string(),
                            init: Expr::IntLiteral(1),
                        },
                        Statement::Assign {
                            name: "x".to_string(),
                            value: Expr::Binary {
                                op: BinaryOp::Add,
                                left: Box::new(Expr::Variable("x".to_string())),
                                right: Box::new(Expr::IntLiteral(2)),
                            },
                        },
                        Statement::Return(Expr::Variable("x".to_string())),
                    ],
                },
            }
        )
    }

    #[test]
    fn parses_parenthesized_expression_precedence() {
        let tokens = vec![
            Token::KwInt,
            Token::Ident("main".to_string()),
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::KwReturn,
            Token::LParen,
            Token::IntLiteral(1),
            Token::Plus,
            Token::IntLiteral(2),
            Token::RParen,
            Token::Star,
            Token::IntLiteral(3),
            Token::Semicolon,
            Token::RBrace,
            Token::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                function: Function {
                    name: "main".to_string(),
                    body: vec![Statement::Return(Expr::Binary {
                        op: BinaryOp::Multiply,
                        left: Box::new(Expr::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::IntLiteral(1)),
                            right: Box::new(Expr::IntLiteral(2)),
                        }),
                        right: Box::new(Expr::IntLiteral(3)),
                    })],
                },
            }
        )
    }

    #[test]
    fn parses_less_than_with_additive_operands() {
        let tokens = vec![
            Token::KwInt,
            Token::Ident("main".to_string()),
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::KwReturn,
            Token::IntLiteral(1),
            Token::Plus,
            Token::IntLiteral(2),
            Token::Less,
            Token::IntLiteral(4),
            Token::Semicolon,
            Token::RBrace,
            Token::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                function: Function {
                    name: "main".to_string(),
                    body: vec![Statement::Return(Expr::Binary {
                        op: BinaryOp::Less,
                        left: Box::new(Expr::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::IntLiteral(1)),
                            right: Box::new(Expr::IntLiteral(2)),
                        }),
                        right: Box::new(Expr::IntLiteral(4)),
                    })],
                },
            }
        )
    }

    #[test]
    fn parses_shift_after_additive() {
        let tokens = vec![
            Token::KwReturn,
            Token::IntLiteral(1),
            Token::Plus,
            Token::IntLiteral(2),
            Token::LessLess,
            Token::IntLiteral(3),
            Token::Semicolon,
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
        let tokens = vec![
            Token::KwReturn,
            Token::IntLiteral(1),
            Token::LessLess,
            Token::IntLiteral(2),
            Token::Less,
            Token::IntLiteral(8),
            Token::Semicolon,
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
        let tokens = vec![
            Token::KwReturn,
            Token::Ident("a".to_string()),
            Token::Ampersand,
            Token::Ident("b".to_string()),
            Token::EqualEqual,
            Token::Ident("c".to_string()),
            Token::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::BitAnd,
                left: Box::new(Expr::Variable("a".to_string())),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::Equal,
                    left: Box::new(Expr::Variable("b".to_string())),
                    right: Box::new(Expr::Variable("c".to_string())),
                }),
            })
        )
    }

    #[test]
    fn parses_bitwise_and_before_xor() {
        let tokens = vec![
            Token::KwReturn,
            Token::Ident("a".to_string()),
            Token::Caret,
            Token::Ident("b".to_string()),
            Token::Ampersand,
            Token::Ident("c".to_string()),
            Token::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::BitXor,
                left: Box::new(Expr::Variable("a".to_string())),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::BitAnd,
                    left: Box::new(Expr::Variable("b".to_string())),
                    right: Box::new(Expr::Variable("c".to_string())),
                }),
            })
        )
    }

    #[test]
    fn parses_bitwise_xor_before_or() {
        let tokens = vec![
            Token::KwReturn,
            Token::Ident("a".to_string()),
            Token::Pipe,
            Token::Ident("b".to_string()),
            Token::Caret,
            Token::Ident("c".to_string()),
            Token::Semicolon,
        ];

        let mut parser = Parser::new(tokens);

        let statement = parser.parse_statement().expect("parsing should succeed");

        assert_eq!(
            statement,
            Statement::Return(Expr::Binary {
                op: BinaryOp::BitOr,
                left: Box::new(Expr::Variable("a".to_string())),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::BitXor,
                    left: Box::new(Expr::Variable("b".to_string())),
                    right: Box::new(Expr::Variable("c".to_string())),
                }),
            })
        )
    }

    #[test]
    fn parses_equality_with_parenthesized_expression() {
        let tokens = vec![
            Token::KwInt,
            Token::Ident("main".to_string()),
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::KwReturn,
            Token::LParen,
            Token::IntLiteral(1),
            Token::Plus,
            Token::IntLiteral(2),
            Token::RParen,
            Token::EqualEqual,
            Token::IntLiteral(3),
            Token::Semicolon,
            Token::RBrace,
            Token::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                function: Function {
                    name: "main".to_string(),
                    body: vec![Statement::Return(Expr::Binary {
                        op: BinaryOp::Equal,
                        left: Box::new(Expr::Binary {
                            op: BinaryOp::Add,
                            left: Box::new(Expr::IntLiteral(1)),
                            right: Box::new(Expr::IntLiteral(2)),
                        }),
                        right: Box::new(Expr::IntLiteral(3)),
                    })],
                },
            }
        )
    }

    #[test]
    fn parses_greater_equal() {
        let tokens = vec![
            Token::KwInt,
            Token::Ident("main".to_string()),
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::KwReturn,
            Token::Ident("x".to_string()),
            Token::GreaterEqual,
            Token::IntLiteral(10),
            Token::Semicolon,
            Token::RBrace,
            Token::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                function: Function {
                    name: "main".to_string(),
                    body: vec![Statement::Return(Expr::Binary {
                        op: BinaryOp::GreaterEqual,
                        left: Box::new(Expr::Variable("x".to_string())),
                        right: Box::new(Expr::IntLiteral(10)),
                    })],
                },
            }
        )
    }

    #[test]
    fn parses_chained_comparisons_left_associative() {
        let tokens = vec![
            Token::KwInt,
            Token::Ident("main".to_string()),
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::KwReturn,
            Token::IntLiteral(1),
            Token::Less,
            Token::IntLiteral(2),
            Token::Less,
            Token::IntLiteral(3),
            Token::Semicolon,
            Token::RBrace,
            Token::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                function: Function {
                    name: "main".to_string(),
                    body: vec![Statement::Return(Expr::Binary {
                        op: BinaryOp::Less,
                        left: Box::new(Expr::Binary {
                            op: BinaryOp::Less,
                            left: Box::new(Expr::IntLiteral(1)),
                            right: Box::new(Expr::IntLiteral(2)),
                        }),
                        right: Box::new(Expr::IntLiteral(3)),
                    })],
                },
            }
        );
    }

    #[test]
    fn parses_function_with_multiple_statements() {
        let tokens = vec![
            Token::KwInt,
            Token::Ident("main".to_string()),
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::KwInt,
            Token::Ident("x".to_string()),
            Token::Equal,
            Token::IntLiteral(5),
            Token::Semicolon,
            Token::KwReturn,
            Token::IntLiteral(42),
            Token::Semicolon,
            Token::RBrace,
            Token::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                function: Function {
                    name: "main".to_string(),
                    body: vec![
                        Statement::VarDecl {
                            name: "x".to_string(),
                            init: Expr::IntLiteral(5),
                        },
                        Statement::Return(Expr::IntLiteral(42)),
                    ],
                },
            }
        )
    }

    #[test]
    fn parses_function_returning_variable() {
        let tokens = vec![
            Token::KwInt,
            Token::Ident("main".to_string()),
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::KwInt,
            Token::Ident("x".to_string()),
            Token::Equal,
            Token::IntLiteral(5),
            Token::Semicolon,
            Token::KwReturn,
            Token::Ident("x".to_string()),
            Token::Semicolon,
            Token::RBrace,
            Token::Eof,
        ];

        let program = parse(tokens).expect("parsing should succeed");

        assert_eq!(
            program,
            Program {
                function: Function {
                    name: "main".to_string(),
                    body: vec![
                        Statement::VarDecl {
                            name: "x".to_string(),
                            init: Expr::IntLiteral(5),
                        },
                        Statement::Return(Expr::Variable("x".to_string())),
                    ],
                },
            }
        )
    }

    #[test]
    fn multiplication_has_higher_precedence_than_addition() {
        let tokens = vec![
            Token::KwReturn,
            Token::IntLiteral(1),
            Token::Plus,
            Token::IntLiteral(2),
            Token::Star,
            Token::IntLiteral(3),
            Token::Semicolon,
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
}
