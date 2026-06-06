use std::collections::{HashMap, HashSet};

use crate::ast::{Expr, Function, Program, Statement};
use crate::lexer::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    pub message: String,
    pub span: Span,
}

pub struct FunctionInfo {
    param_count: usize,
}

struct Checker {
    scopes: Vec<HashSet<String>>,
    functions: HashMap<String, FunctionInfo>,
    loop_depth: usize,
}

impl Checker {
    fn new() -> Self {
        Self {
            scopes: vec![HashSet::new()],
            functions: HashMap::new(),
            loop_depth: 0,
        }
    }

    fn enter_loop(&mut self) {
        self.loop_depth += 1
    }

    fn exit_loop(&mut self) {
        if self.loop_depth == 0 {
            panic!("cannot have negative loop depth")
        }
        self.loop_depth -= 1
    }

    fn in_loop(&self) -> bool {
        self.loop_depth > 0
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

    fn declare_local(&mut self, name: &str, span: Span) -> Result<(), SemanticError> {
        if self.current_scope().contains(name) {
            return Err(SemanticError {
                message: format!("duplicate local variable '{name}'"),
                span,
            });
        }
        self.current_scope_mut().insert(name.to_string());
        Ok(())
    }

    fn check_local_declared(&self, name: &str, span: Span) -> Result<(), SemanticError> {
        for scope in self.scopes.iter().rev() {
            if scope.contains(name) {
                return Ok(());
            }
        }
        return Err(SemanticError {
            message: format!("undeclared local variable '{name}'"),
            span,
        });
    }

    fn declare_function(&mut self, function: &Function) -> Result<(), SemanticError> {
        if self.functions.contains_key(&function.name) {
            return Err(SemanticError {
                message: format!("duplicate function '{}'", &function.name),
                span: function.name_span,
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
                span: function.params[8].name_span,
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

    fn check_function_declared(&self, name: &str, span: Span) -> Result<(), SemanticError> {
        if !self.functions.contains_key(name) {
            return Err(SemanticError {
                message: format!("undeclared function '{name}'"),
                span,
            });
        }
        Ok(())
    }

    fn check_function_param_count(
        &self,
        name: &str,
        actual_arg_count: usize,
        span: Span,
    ) -> Result<(), SemanticError> {
        let num_args = self
            .functions
            .get(name)
            .expect(&format!("function {name} does not exist"))
            .param_count;
        if num_args != actual_arg_count {
            return Err(SemanticError {
                message: format!(
                    "function call of '{name}' has {actual_arg_count} arguments, declaration has {num_args}"
                ),
                span,
            });
        }
        Ok(())
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<(), SemanticError> {
        match expr {
            Expr::IntLiteral(_) => (),
            Expr::Variable { name, span } => self.check_local_declared(name, *span)?,
            Expr::Unary { op: _, expr } => self.check_expr(expr)?,
            Expr::Binary { op: _, left, right } => {
                self.check_expr(left)?;
                self.check_expr(right)?;
            }
            Expr::Call {
                name,
                name_span,
                args,
            } => {
                self.check_function_declared(name, *name_span)?;
                // For now, limit to 8 args (no stack-passed arguments)
                if args.len() > 8 {
                    return Err(SemanticError {
                        message: format!(
                            "too many arguments in call to function {}, got {}, max 8",
                            name,
                            args.len()
                        ),
                        span: *name_span,
                    });
                }
                self.check_function_param_count(name, args.len(), *name_span)?;
                for arg in args {
                    self.check_expr(arg)?;
                }
            }
            Expr::Assign {
                name,
                name_span,
                value,
            } => {
                self.check_local_declared(name, *name_span)?;
                self.check_expr(value)?;
            }
        }
        Ok(())
    }

    fn check_statement(&mut self, statement: &Statement) -> Result<(), SemanticError> {
        match statement {
            Statement::Return(expr) => self.check_expr(expr)?,
            Statement::VarDecl {
                name,
                name_span,
                init,
            } => {
                self.declare_local(name, *name_span)?;
                if let Some(init_expr) = init {
                    self.check_expr(init_expr)?;
                }
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
                self.enter_loop();
                let res = self
                    .check_expr(cond)
                    .and_then(|_| self.check_statement(body));
                self.exit_loop();
                res?;
            }
            Statement::For {
                init,
                cond,
                post,
                body,
            } => {
                self.enter_loop();
                self.enter_scope();
                let res = (|| -> Result<(), SemanticError> {
                    if let Some(init_statement) = init {
                        self.check_statement(init_statement)?;
                    }
                    if let Some(cond_expr) = cond {
                        self.check_expr(cond_expr)?;
                    }
                    if let Some(post_expr) = post {
                        self.check_expr(post_expr)?;
                    }
                    self.check_statement(body)?;

                    Ok(())
                })();
                self.exit_scope();
                self.exit_loop();
                res?
            }
            Statement::Empty => (),
            Statement::ExprStatement(expr) => self.check_expr(expr)?,
            Statement::Break { span } => {
                if !self.in_loop() {
                    return Err(SemanticError {
                        message: format!("cannot use 'break' outside of a loop"),
                        span: *span,
                    });
                }
            }
            Statement::Continue { span } => {
                if !self.in_loop() {
                    return Err(SemanticError {
                        message: format!("cannot use 'continue' outside of a loop"),
                        span: *span,
                    });
                }
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
        self.enter_scope();
        for param in &function.params {
            self.declare_local(&param.name, param.name_span)?;
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
                span: program.eof_span,
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
