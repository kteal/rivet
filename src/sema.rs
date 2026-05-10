use std::collections::{HashMap, HashSet};

use crate::ast::{Expr, Function, Program, Statement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    pub message: String,
}

pub struct FunctionInfo {
    param_count: usize,
}

struct Checker {
    scopes: Vec<HashSet<String>>,
    functions: HashMap<String, FunctionInfo>,
}

impl Checker {
    fn new() -> Self {
        Self {
            scopes: vec![HashSet::new()],
            functions: HashMap::new(),
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

    fn check_local_declared(&self, name: &str) -> Result<(), SemanticError> {
        for scope in self.scopes.iter().rev() {
            if scope.contains(name) {
                return Ok(());
            }
        }
        return Err(SemanticError {
            message: format!("undeclared local variable '{name}'"),
        });
    }

    fn declare_function(&mut self, function: &Function) -> Result<(), SemanticError> {
        if self.functions.contains_key(&function.name) {
            return Err(SemanticError {
                message: format!("duplicate function '{}'", &function.name),
            });
        }
        // For now, limit to 8 args (no stack-passed arguments)
        if function.params.len() > 8 {
            return Err(SemanticError {
                message: format!(
                    "too many parameters in function {}, got {}, max 8",
                    function.name,
                    function.params.len()
                ),
            });
        }
        self.functions.insert(
            function.name.to_string(),
            FunctionInfo {
                param_count: function.params.len(),
            },
        );
        Ok(())
    }

    fn check_function_declared(&self, name: &str) -> Result<(), SemanticError> {
        if !self.functions.contains_key(name) {
            return Err(SemanticError {
                message: format!("undeclared function '{name}'"),
            });
        }
        Ok(())
    }

    fn check_function_param_count(&self, name: &str, expected: usize) -> Result<(), SemanticError> {
        let num_args = self
            .functions
            .get(name)
            .expect(&format!("function {name} does not exist"))
            .param_count;
        if num_args != expected {
            return Err(SemanticError {
                message: format!(
                    "function call of '{name}' has {expected} arguments, declaration has {num_args}"
                ),
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
            Expr::Call { name, args } => {
                // For now, limit to 8 args (no stack-passed arguments)
                if args.len() > 8 {
                    return Err(SemanticError {
                        message: format!(
                            "too many arguments in call to function {}, got {}, max 8",
                            name,
                            args.len()
                        ),
                    });
                }
                self.check_function_declared(name)?;
                self.check_function_param_count(name, args.len())?;
                for arg in args {
                    self.check_expr(arg)?;
                }
                Ok(())
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
            Statement::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.check_expr(cond)?;
                self.check_statement(then_branch)?;
                if let Some(else_statement) = else_branch {
                    self.check_statement(else_statement)?;
                }
            }
            Statement::While { cond, body } => {
                self.check_expr(cond)?;
                self.check_statement(body)?;
            }
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

    fn check_function(&mut self, function: &Function) -> Result<(), SemanticError> {
        // For now, limit to 8 args (no stack-passed arguments)
        if function.params.len() > 8 {
            return Err(SemanticError {
                message: format!(
                    "too many parameters in function {}, got {}, max 8",
                    function.name,
                    function.params.len()
                ),
            });
        }
        self.enter_scope();
        for param in function.params.clone() {
            self.declare_local(&param)?;
        }
        let res = self.check_statements(&function.body);
        self.exit_scope();
        res
    }

    fn check_program(&mut self, program: &Program) -> Result<(), SemanticError> {
        for function in &program.functions {
            self.declare_function(&function)?;
        }
        if !self.functions.contains_key("main") {
            return Err(SemanticError {
                message: "no 'main' function found".to_string(),
            });
        }
        for function in &program.functions {
            self.check_function(function)?;
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
            functions: vec![Function {
                name: "main".to_string(),
                params: vec![],
                body,
            }],
        }
    }

    fn function(name: &str, body: Vec<Statement>) -> Function {
        Function {
            name: name.to_string(),
            params: vec![],
            body,
        }
    }

    fn function_with_params(name: &str, params: &[&str], body: Vec<Statement>) -> Function {
        Function {
            name: name.to_string(),
            params: params.iter().map(|param| param.to_string()).collect(),
            body,
        }
    }

    #[test]
    fn accepts_multiple_functions() {
        let program = Program {
            functions: vec![
                function("helper", vec![Statement::Return(Expr::IntLiteral(1))]),
                function("main", vec![Statement::Return(Expr::IntLiteral(0))]),
            ],
        };

        check(&program).expect("semantic check should succeed");
    }

    #[test]
    fn accepts_same_local_name_in_different_functions() {
        let program = Program {
            functions: vec![
                function(
                    "first",
                    vec![
                        Statement::VarDecl {
                            name: "x".to_string(),
                            init: Expr::IntLiteral(1),
                        },
                        Statement::Return(Expr::Variable("x".to_string())),
                    ],
                ),
                function(
                    "second",
                    vec![
                        Statement::VarDecl {
                            name: "x".to_string(),
                            init: Expr::IntLiteral(2),
                        },
                        Statement::Return(Expr::Variable("x".to_string())),
                    ],
                ),
                function("main", vec![Statement::Return(Expr::IntLiteral(0))]),
            ],
        };

        check(&program).expect("semantic check should succeed");
    }

    #[test]
    fn rejects_duplicate_function_names() {
        let program = Program {
            functions: vec![
                function("main", vec![Statement::Return(Expr::IntLiteral(0))]),
                function("main", vec![Statement::Return(Expr::IntLiteral(1))]),
            ],
        };

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(err.message, "duplicate function 'main'");
    }

    #[test]
    fn rejects_program_without_main_function() {
        let program = Program {
            functions: vec![function(
                "helper",
                vec![Statement::Return(Expr::IntLiteral(1))],
            )],
        };

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(err.message, "no 'main' function found");
    }

    #[test]
    fn accepts_call_to_declared_function() {
        let program = Program {
            functions: vec![
                function("helper", vec![Statement::Return(Expr::IntLiteral(1))]),
                function(
                    "main",
                    vec![Statement::Return(Expr::Call {
                        name: "helper".to_string(),
                        args: vec![],
                    })],
                ),
            ],
        };

        check(&program).expect("semantic check should succeed");
    }

    #[test]
    fn accepts_forward_call_to_later_function() {
        let program = Program {
            functions: vec![
                function(
                    "main",
                    vec![Statement::Return(Expr::Call {
                        name: "helper".to_string(),
                        args: vec![],
                    })],
                ),
                function("helper", vec![Statement::Return(Expr::IntLiteral(1))]),
            ],
        };

        check(&program).expect("semantic check should succeed");
    }

    #[test]
    fn rejects_call_to_undeclared_function() {
        let program = main_program(vec![Statement::Return(Expr::Call {
            name: "helper".to_string(),
            args: vec![],
        })]);

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(err.message, "undeclared function 'helper'");
    }

    #[test]
    fn accepts_parameter_usage_as_local() {
        let program = Program {
            functions: vec![function_with_params(
                "main",
                &["x", "y"],
                vec![Statement::Return(Expr::Binary {
                    op: BinaryOp::Add,
                    left: Box::new(Expr::Variable("x".to_string())),
                    right: Box::new(Expr::Variable("y".to_string())),
                })],
            )],
        };

        check(&program).expect("semantic check should succeed");
    }

    #[test]
    fn rejects_duplicate_parameter_names() {
        let program = Program {
            functions: vec![function_with_params(
                "main",
                &["x", "x"],
                vec![Statement::Return(Expr::Variable("x".to_string()))],
            )],
        };

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(err.message, "duplicate local variable 'x'");
    }

    #[test]
    fn rejects_local_redeclaring_parameter_in_function_scope() {
        let program = Program {
            functions: vec![function_with_params(
                "main",
                &["x"],
                vec![
                    Statement::VarDecl {
                        name: "x".to_string(),
                        init: Expr::IntLiteral(1),
                    },
                    Statement::Return(Expr::Variable("x".to_string())),
                ],
            )],
        };

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(err.message, "duplicate local variable 'x'");
    }

    #[test]
    fn accepts_inner_block_shadowing_parameter() {
        let program = Program {
            functions: vec![function_with_params(
                "main",
                &["x"],
                vec![Statement::Block(vec![
                    Statement::VarDecl {
                        name: "x".to_string(),
                        init: Expr::IntLiteral(1),
                    },
                    Statement::Return(Expr::Variable("x".to_string())),
                ])],
            )],
        };

        check(&program).expect("semantic check should succeed");
    }

    #[test]
    fn accepts_call_with_matching_argument_count() {
        let program = Program {
            functions: vec![
                function_with_params(
                    "add",
                    &["x", "y"],
                    vec![Statement::Return(Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Variable("x".to_string())),
                        right: Box::new(Expr::Variable("y".to_string())),
                    })],
                ),
                function(
                    "main",
                    vec![Statement::Return(Expr::Call {
                        name: "add".to_string(),
                        args: vec![Expr::IntLiteral(1), Expr::IntLiteral(2)],
                    })],
                ),
            ],
        };

        check(&program).expect("semantic check should succeed");
    }

    #[test]
    fn rejects_call_with_too_few_arguments() {
        let program = Program {
            functions: vec![
                function_with_params(
                    "add",
                    &["x", "y"],
                    vec![Statement::Return(Expr::Variable("x".to_string()))],
                ),
                function(
                    "main",
                    vec![Statement::Return(Expr::Call {
                        name: "add".to_string(),
                        args: vec![Expr::IntLiteral(1)],
                    })],
                ),
            ],
        };

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(
            err.message,
            "function call of 'add' has 1 arguments, declaration has 2"
        );
    }

    #[test]
    fn rejects_call_with_too_many_arguments_for_signature() {
        let program = Program {
            functions: vec![
                function_with_params(
                    "id",
                    &["x"],
                    vec![Statement::Return(Expr::Variable("x".to_string()))],
                ),
                function(
                    "main",
                    vec![Statement::Return(Expr::Call {
                        name: "id".to_string(),
                        args: vec![Expr::IntLiteral(1), Expr::IntLiteral(2)],
                    })],
                ),
            ],
        };

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(
            err.message,
            "function call of 'id' has 2 arguments, declaration has 1"
        );
    }

    #[test]
    fn rejects_function_with_more_than_eight_parameters() {
        let program = Program {
            functions: vec![function_with_params(
                "main",
                &["a", "b", "c", "d", "e", "f", "g", "h", "i"],
                vec![Statement::Return(Expr::IntLiteral(0))],
            )],
        };

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(
            err.message,
            "too many parameters in function main, got 9, max 8"
        );
    }

    #[test]
    fn rejects_call_with_more_than_eight_arguments() {
        let program = Program {
            functions: vec![
                function("helper", vec![Statement::Return(Expr::IntLiteral(0))]),
                function(
                    "main",
                    vec![Statement::Return(Expr::Call {
                        name: "helper".to_string(),
                        args: vec![
                            Expr::IntLiteral(1),
                            Expr::IntLiteral(2),
                            Expr::IntLiteral(3),
                            Expr::IntLiteral(4),
                            Expr::IntLiteral(5),
                            Expr::IntLiteral(6),
                            Expr::IntLiteral(7),
                            Expr::IntLiteral(8),
                            Expr::IntLiteral(9),
                        ],
                    })],
                ),
            ],
        };

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(
            err.message,
            "too many arguments in call to function helper, got 9, max 8"
        );
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

    #[test]
    fn accepts_if_else_with_locals_in_branches() {
        let program = main_program(vec![
            Statement::VarDecl {
                name: "x".to_string(),
                init: Expr::IntLiteral(1),
            },
            Statement::If {
                cond: Expr::Binary {
                    op: BinaryOp::Less,
                    left: Box::new(Expr::Variable("x".to_string())),
                    right: Box::new(Expr::IntLiteral(2)),
                },
                then_branch: Box::new(Statement::Block(vec![
                    Statement::VarDecl {
                        name: "y".to_string(),
                        init: Expr::Variable("x".to_string()),
                    },
                    Statement::Return(Expr::Variable("y".to_string())),
                ])),
                else_branch: Some(Box::new(Statement::Block(vec![
                    Statement::VarDecl {
                        name: "z".to_string(),
                        init: Expr::Variable("x".to_string()),
                    },
                    Statement::Return(Expr::Variable("z".to_string())),
                ]))),
            },
        ]);

        check(&program).expect("semantic check should succeed");
    }

    #[test]
    fn accepts_while_with_local_condition_and_body() {
        let program = main_program(vec![
            Statement::VarDecl {
                name: "x".to_string(),
                init: Expr::IntLiteral(3),
            },
            Statement::While {
                cond: Expr::Variable("x".to_string()),
                body: Box::new(Statement::Block(vec![Statement::Assign {
                    name: "x".to_string(),
                    value: Expr::Binary {
                        op: BinaryOp::Subtract,
                        left: Box::new(Expr::Variable("x".to_string())),
                        right: Box::new(Expr::IntLiteral(1)),
                    },
                }])),
            },
            Statement::Return(Expr::Variable("x".to_string())),
        ]);

        check(&program).expect("semantic check should succeed");
    }

    #[test]
    fn rejects_while_condition_using_undeclared_local() {
        let program = main_program(vec![
            Statement::While {
                cond: Expr::Variable("x".to_string()),
                body: Box::new(Statement::Return(Expr::IntLiteral(0))),
            },
            Statement::Return(Expr::IntLiteral(0)),
        ]);

        let err = check(&program).expect_err("semantic check should fail");

        assert_eq!(err.message, "undeclared local variable 'x'");
    }
}
