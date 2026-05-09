use crate::ast::{BinaryOp, Expr, Function, Program, Statement};
use crate::lexer::Token;

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

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.advance() {
            Token::IntLiteral(value) => Ok(Expr::IntLiteral(value)),
            Token::Ident(name) => Ok(Expr::Variable(name)),
            found => {
                return Err(ParseError {
                    message: format!("expected expression, found {found:?}"),
                });
            }
        }
    }

    fn parse_multiplicative_op(&mut self) -> Option<BinaryOp> {
        match self.peek() {
            Token::Star => {
                self.advance();
                Some(BinaryOp::Multiply)
            }
            Token::Slash => {
                self.advance();
                Some(BinaryOp::Divide)
            }
            Token::Percent => {
                self.advance();
                Some(BinaryOp::Remainder)
            }
            _ => None,
        }
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_primary()?;

        while let Some(op) = self.parse_multiplicative_op() {
            let right = self.parse_primary()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }

        Ok(left)
    }

    fn parse_additive_op(&mut self) -> Option<BinaryOp> {
        match self.peek() {
            Token::Plus => {
                self.advance();
                Some(BinaryOp::Add)
            }
            Token::Minus => {
                self.advance();
                Some(BinaryOp::Subtract)
            }
            _ => None,
        }
    }

    fn parse_additive(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplicative()?;

        while let Some(op) = self.parse_additive_op() {
            let right = self.parse_multiplicative()?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        }

        Ok(left)
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_additive()
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

        while self.peek() != &Token::RBrace {
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
