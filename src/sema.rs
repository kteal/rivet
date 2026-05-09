use std::collections::HashSet;

use crate::ast::{Expr, Program, Statement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    pub message: String,
}

struct Checker {
    scopes: Vec<HashSet<String>>,
}

impl Checker {
    fn new() -> Self {
        Self {
            scopes: vec![HashSet::new()],
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn current_scope(&self) -> &HashSet<String> {
        self.scopes
            .last()
            .expect("semantic checker should have an active scope")
    }

    fn current_scope_mut(&mut self) -> &mut HashSet<String> {
        self.scopes
            .last_mut()
            .expect("semantic checker should have an active scope")
    }

    fn declare_local(&mut self, name: &str) -> Result<(), SemanticError> {
        if self.current_scope().contains(name) {
            return Err(SemanticError {
                message: format!("duplicate local variable '{name}'"),
            });
        }
        self.current_scope_mut().insert(name.to_string());
        Ok(())
    }

    fn check_local_declared(&mut self, name: &str) -> Result<(), SemanticError> {
        for scope in self.scopes.iter().rev() {
            if scope.contains(name) {
                return Ok(());
            }
        }
        return Err(SemanticError {
            message: format!("undeclared local variable '{name}'"),
        });
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

    fn check_statement(&mut self, statement: &Statement) -> Result<(), SemanticError> {
        match statement {
            Statement::Return(expr) => self.check_expr(expr)?,
            Statement::VarDecl { name, init } => {
                self.declare_local(name)?;
                self.check_expr(init)?;
            }
            Statement::Assign { name, value } => {
                self.check_local_declared(name)?;
                self.check_expr(value)?;
            }
            Statement::Block(body) => self.check_block(body)?,
        }
        Ok(())
    }

    fn check_statements(&mut self, statements: &[Statement]) -> Result<(), SemanticError> {
        for statement in statements {
            self.check_statement(statement)?
        }
        Ok(())
    }

    fn check_block(&mut self, body: &[Statement]) -> Result<(), SemanticError> {
        self.enter_scope();
        let res = self.check_statements(body);
        self.exit_scope();
        res
    }

    fn check_program(&mut self, program: &Program) -> Result<(), SemanticError> {
        let function = &program.function;

        self.check_statements(&function.body)?;

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

    #[test]
    fn accepts_block_using_outer_local() {
        let program = main_program(vec![
            Statement::VarDecl {
                name: "x".to_string(),
                init: Expr::IntLiteral(1),
            },
            Statement::Block(vec![Statement::Return(Expr::Variable("x".to_string()))]),
        ]);

        check(&program).expect("semantic check should succeed");
    }

    #[test]
    fn rejects_use_of_local_after_block_scope_ends() {
        let program = main_program(vec![
            Statement::Block(vec![Statement::VarDecl {
                name: "x".to_string(),
                init: Expr::IntLiteral(1),
            }]),
            Statement::Return(Expr::Variable("x".to_string())),
        ]);

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(err.message, "undeclared local variable 'x'");
    }

    #[test]
    fn accepts_shadowing_in_inner_block() {
        let program = main_program(vec![
            Statement::VarDecl {
                name: "x".to_string(),
                init: Expr::IntLiteral(1),
            },
            Statement::Block(vec![
                Statement::VarDecl {
                    name: "x".to_string(),
                    init: Expr::IntLiteral(2),
                },
                Statement::Return(Expr::Variable("x".to_string())),
            ]),
        ]);

        check(&program).expect("semantic check should succeed");
    }

    #[test]
    fn rejects_duplicate_local_in_same_block() {
        let program = main_program(vec![
            Statement::Block(vec![
                Statement::VarDecl {
                    name: "x".to_string(),
                    init: Expr::IntLiteral(1),
                },
                Statement::VarDecl {
                    name: "x".to_string(),
                    init: Expr::IntLiteral(2),
                },
            ]),
            Statement::Return(Expr::IntLiteral(0)),
        ]);

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(err.message, "duplicate local variable 'x'");
    }
}
