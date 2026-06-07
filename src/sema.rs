use std::collections::HashMap;

use crate::ast::{BinaryOp, Expr, Function, Param, Program, Statement, Type};
use crate::lexer::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticError {
    pub message: String,
    pub span: Span,
}

pub struct FunctionInfo {
    return_type: Type,
    name: String,
    params: Vec<Param>,
}

struct Checker {
    scopes: Vec<HashMap<String, Type>>,
    functions: HashMap<String, FunctionInfo>,
    loop_depth: usize,
    current_function_return_type: Option<Type>,
}

impl Checker {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            loop_depth: 0,
            current_function_return_type: None,
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
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn current_scope(&self) -> &HashMap<String, Type> {
        self.scopes
            .last()
            .expect("semantic checker should have an active scope")
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<String, Type> {
        self.scopes
            .last_mut()
            .expect("semantic checker should have an active scope")
    }

    fn is_assignable(target: Type, value: Type) -> bool {
        target == value
            || (target == Type::Char && value == Type::Int)
            || (target == Type::Int && value == Type::Char)
    }

    fn is_integer(ty: Type) -> bool {
        matches!(ty, Type::Int | Type::Char)
    }

    fn declare_local(&mut self, ty: Type, name: &str, span: Span) -> Result<(), SemanticError> {
        if self.current_scope().contains_key(name) {
            return Err(SemanticError {
                message: format!("duplicate local variable '{name}'"),
                span,
            });
        }
        self.current_scope_mut().insert(name.to_string(), ty);
        Ok(())
    }

    fn resolve_local(&self, name: &str, span: Span) -> Result<Type, SemanticError> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Ok(*ty);
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
                return_type: function.return_type,
                name: function.name.to_string(),
                params: function.params.clone(),
            },
        );
        Ok(())
    }

    fn check_function_declared(
        &self,
        name: &str,
        span: Span,
    ) -> Result<&FunctionInfo, SemanticError> {
        self.functions.get(name).ok_or_else(|| SemanticError {
            message: format!("undeclared function '{name}'"),
            span,
        })
    }

    fn check_function_params(
        &self,
        function_info: &FunctionInfo,
        args: &[Expr],
        span: Span,
    ) -> Result<(), SemanticError> {
        // Check number of arguments
        if function_info.params.len() != args.len() {
            return Err(SemanticError {
                message: format!(
                    "function call of '{}' has {} arguments, declaration has {}",
                    function_info.name,
                    args.len(),
                    function_info.params.len()
                ),
                span,
            });
        }

        // Check argument types
        for (param, arg) in function_info.params.iter().zip(args) {
            let arg_type = self.check_expr(arg)?;
            if !Self::is_assignable(param.ty, arg_type) {
                return Err(SemanticError {
                    message: format!(
                        "cannot pass value of type '{arg_type:?}' to parameter of type '{:?}'",
                        param.ty
                    ),
                    span: arg.diagnostic_span(),
                });
            }
        }
        Ok(())
    }

    fn check_binary_op_types(
        &self,
        op: &BinaryOp,
        op_span: &Span,
        left_type: Type,
        right_type: Type,
    ) -> Result<Type, SemanticError> {
        if Self::is_integer(left_type) && Self::is_integer(right_type) {
            Ok(Type::Int)
        } else {
            Err(SemanticError {
                message: format!(
                    "invalid operands to binary operator '{:?}'\n\
                     left operand has type '{:?}'\n\
                     right operand has type '{:?}'",
                    op, left_type, right_type
                ),
                span: *op_span,
            })
        }
    }

    fn check_expr(&self, expr: &Expr) -> Result<Type, SemanticError> {
        match expr {
            Expr::IntLiteral { .. } => Ok(Type::Int),
            Expr::Variable { name, span } => self.resolve_local(name, *span),
            Expr::Unary {
                op: _,
                op_span: _,
                expr,
            } => {
                let _expr_type = self.check_expr(expr)?;
                Ok(Type::Int)
            }
            Expr::Binary {
                op,
                op_span,
                left,
                right,
            } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;

                let result_type = self.check_binary_op_types(op, op_span, left_type, right_type)?;

                Ok(result_type)
            }
            Expr::Call {
                name,
                name_span,
                args,
            } => {
                let function_info = self.check_function_declared(name, *name_span)?;
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
                self.check_function_params(function_info, args, *name_span)?;

                Ok(function_info.return_type)
            }
            Expr::Assign {
                name,
                name_span,
                value,
            } => {
                let target_type = self.resolve_local(name, *name_span)?;
                let value_type = self.check_expr(value)?;

                if !Self::is_assignable(target_type, value_type) {
                    return Err(SemanticError {
                        message: format!(
                            "cannot assign value of type '{value_type:?}' to variable of type '{target_type:?}'"
                        ),
                        span: *name_span,
                    });
                }

                Ok(target_type)
            }
            Expr::CompoundAssign {
                name,
                name_span,
                op,
                op_span,
                value,
            } => {
                let target_type = self.resolve_local(name, *name_span)?;
                let value_type = self.check_expr(value)?;

                let result_type =
                    self.check_binary_op_types(op, op_span, target_type, value_type)?;

                if !Self::is_assignable(target_type, result_type) {
                    return Err(SemanticError {
                        message: format!(
                            "cannot assign result of type '{result_type:?}' to variable of type '{target_type:?}'"
                        ),
                        span: *op_span,
                    });
                }

                Ok(target_type)
            }
        }
    }

    fn check_statement(&mut self, statement: &Statement) -> Result<(), SemanticError> {
        match statement {
            Statement::Return(expr) => {
                let expr_type = self.check_expr(expr)?;

                let return_type = self
                    .current_function_return_type
                    .expect("return statement checked outside function");

                if !Self::is_assignable(return_type, expr_type) {
                    return Err(SemanticError {
                        message: format!(
                            "cannot return value of type '{expr_type:?}' from function returning '{return_type:?}'"
                        ),
                        span: expr.diagnostic_span(),
                    });
                }
            }
            Statement::VarDecl {
                ty,
                name,
                name_span,
                init,
            } => {
                self.declare_local(*ty, name, *name_span)?;
                if let Some(init_expr) = init {
                    let init_type = self.check_expr(init_expr)?;
                    if !Self::is_assignable(*ty, init_type) {
                        return Err(SemanticError {
                            message: format!(
                                "cannot assign value of type '{init_type:?}' to variable of type '{ty:?}'"
                            ),
                            span: init_expr.diagnostic_span(),
                        });
                    }
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
            Statement::DoWhile { body, cond } => {
                self.enter_loop();
                let res = self
                    .check_statement(body)
                    .and_then(|_| self.check_expr(cond));
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
            Statement::ExprStatement(expr) => {
                self.check_expr(expr)?;
            }
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
        let old_return_type = self
            .current_function_return_type
            .replace(function.return_type);

        self.enter_scope();

        let res = (|| -> Result<(), SemanticError> {
            for param in &function.params {
                self.declare_local(param.ty, &param.name, param.name_span)?;
            }
            self.check_statements(&function.body)
        })();

        self.exit_scope();
        self.current_function_return_type = old_return_type;
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
