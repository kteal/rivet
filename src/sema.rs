use std::collections::HashSet;

use crate::ast::{Expr, Program, Statement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    pub message: String,
}

struct Checker {
    locals: HashSet<String>,
}

impl Checker {
    fn new() -> Self {
        Self {
            locals: HashSet::new(),
        }
    }

    fn check_local_declared(&mut self, name: &str) -> Result<(), SemanticError> {
        if !self.locals.contains(name) {
            return Err(SemanticError {
                message: format!("undeclared local variable '{name}'"),
            });
        }
        Ok(())
    }

    fn check_local_not_declared(&mut self, name: &str) -> Result<(), SemanticError> {
        if self.locals.contains(name) {
            return Err(SemanticError {
                message: format!("duplicate local variable '{name}'"),
            });
        }
        Ok(())
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<(), SemanticError> {
        match expr {
            Expr::IntLiteral(_) => Ok(()),
            Expr::Variable(name) => self.check_local_declared(name),
            Expr::Unary { op: _, expr } => self.check_expr(expr),
            Expr::Binary { op: _, left, right } => {
                self.check_expr(left)?;
                self.check_expr(right)
            }
        }
    }

    fn check_program(&mut self, program: &Program) -> Result<(), SemanticError> {
        let function = &program.function;

        for statement in &function.body {
            match statement {
                Statement::Return(expr) => self.check_expr(expr)?,
                Statement::VarDecl { name, init } => {
                    self.check_local_not_declared(name)?;
                    self.locals.insert(name.clone());
                    self.check_expr(init)?;
                }
                Statement::Assign { name, value } => {
                    self.check_local_declared(name)?;
                    self.check_expr(value)?;
                }
            }
        }

        Ok(())
    }
}

pub fn check(program: &Program) -> Result<(), SemanticError> {
    let mut checker = Checker::new();
    checker.check_program(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOp, Expr, Function, Program, Statement};

    fn main_program(body: Vec<Statement>) -> Program {
        Program {
            function: Function {
                name: "main".to_string(),
                body,
            },
        }
    }

    #[test]
    fn accepts_declared_local_usage() {
        let program = main_program(vec![
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
        ]);

        check(&program).expect("semantic check should succeed");
    }

    #[test]
    fn accepts_initializer_using_earlier_local() {
        let program = main_program(vec![
            Statement::VarDecl {
                name: "x".to_string(),
                init: Expr::IntLiteral(1),
            },
            Statement::VarDecl {
                name: "y".to_string(),
                init: Expr::Variable("x".to_string()),
            },
            Statement::Return(Expr::Variable("y".to_string())),
        ]);

        check(&program).expect("semantic check should succeed");
    }

    #[test]
    fn accepts_initializer_using_declared_name_itself() {
        let program = main_program(vec![
            Statement::VarDecl {
                name: "x".to_string(),
                init: Expr::Variable("x".to_string()),
            },
            Statement::Return(Expr::Variable("x".to_string())),
        ]);

        check(&program).expect("semantic check should succeed");
    }

    #[test]
    fn rejects_duplicate_local_declaration() {
        let program = main_program(vec![
            Statement::VarDecl {
                name: "x".to_string(),
                init: Expr::IntLiteral(1),
            },
            Statement::VarDecl {
                name: "x".to_string(),
                init: Expr::IntLiteral(2),
            },
            Statement::Return(Expr::Variable("x".to_string())),
        ]);

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(err.message, "duplicate local variable 'x'");
    }

    #[test]
    fn rejects_assignment_to_undeclared_local() {
        let program = main_program(vec![
            Statement::Assign {
                name: "x".to_string(),
                value: Expr::IntLiteral(1),
            },
            Statement::Return(Expr::IntLiteral(0)),
        ]);

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(err.message, "undeclared local variable 'x'");
    }

    #[test]
    fn rejects_returning_undeclared_local() {
        let program = main_program(vec![Statement::Return(Expr::Variable("x".to_string()))]);

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(err.message, "undeclared local variable 'x'");
    }

    #[test]
    fn rejects_initializer_using_later_local() {
        let program = main_program(vec![
            Statement::VarDecl {
                name: "y".to_string(),
                init: Expr::Variable("x".to_string()),
            },
            Statement::VarDecl {
                name: "x".to_string(),
                init: Expr::IntLiteral(1),
            },
            Statement::Return(Expr::Variable("y".to_string())),
        ]);

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(err.message, "undeclared local variable 'x'");
    }

    #[test]
    fn rejects_undeclared_local_inside_nested_expression() {
        let program = main_program(vec![
            Statement::VarDecl {
                name: "x".to_string(),
                init: Expr::IntLiteral(1),
            },
            Statement::Return(Expr::Binary {
                op: BinaryOp::Multiply,
                left: Box::new(Expr::Variable("x".to_string())),
                right: Box::new(Expr::Variable("y".to_string())),
            }),
        ]);

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(err.message, "undeclared local variable 'y'");
    }
}
