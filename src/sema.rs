use std::collections::HashMap;

use crate::ast::{BinaryOp, Expr, Function, Param, Program, Statement, Type};
use crate::lexer::Span;
use crate::typed_ast::{
    TypedExpr, TypedExprKind, TypedFunction, TypedParam, TypedProgram, TypedStatement,
};

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

    fn check_lvalue(&self, expr: &Expr, op_span: Span) -> Result<TypedExpr, SemanticError> {
        match expr {
            Expr::Variable { name, span } => {
                let ty = self.resolve_local(name, *span)?;

                Ok(TypedExpr {
                    kind: TypedExprKind::Variable {
                        name: name.clone(),
                        span: *span,
                    },
                    ty,
                })
            }
            _ => Err(SemanticError {
                message: "cannot assign to non-variable expression".to_string(),
                span: op_span,
            }),
        }
    }

    fn check_inc_dec(
        &self,
        expr: &Expr,
        op_span: Span,
        make_kind: impl FnOnce(Box<TypedExpr>, Span) -> TypedExprKind,
    ) -> Result<TypedExpr, SemanticError> {
        let typed_lvalue = self.check_lvalue(expr, op_span)?;
        let ty = typed_lvalue.ty;
        Ok(TypedExpr {
            kind: make_kind(Box::new(typed_lvalue), op_span),
            ty,
        })
    }

    fn check_expr(&self, expr: &Expr) -> Result<TypedExpr, SemanticError> {
        match expr {
            Expr::IntLiteral { value, span } => Ok(TypedExpr {
                kind: TypedExprKind::IntLiteral {
                    value: *value,
                    span: *span,
                },
                ty: Type::Int,
            }),
            Expr::Variable { name, span } => {
                let ty = self.resolve_local(name, *span)?;

                Ok(TypedExpr {
                    kind: TypedExprKind::Variable {
                        name: name.clone(),
                        span: *span,
                    },
                    ty,
                })
            }
            Expr::Unary { op, op_span, expr } => {
                let typed_expr = self.check_expr(expr)?;
                Ok(TypedExpr {
                    kind: TypedExprKind::Unary {
                        op: *op,
                        op_span: *op_span,
                        expr: Box::new(typed_expr),
                    },
                    ty: Type::Int,
                })
            }
            Expr::Binary {
                op,
                op_span,
                left,
                right,
            } => {
                let typed_left = self.check_expr(left)?;
                let typed_right = self.check_expr(right)?;

                let result_type =
                    self.check_binary_op_types(op, op_span, typed_left.ty, typed_right.ty)?;

                Ok(TypedExpr {
                    kind: TypedExprKind::Binary {
                        op: *op,
                        op_span: *op_span,
                        left: Box::new(typed_left),
                        right: Box::new(typed_right),
                    },
                    ty: result_type,
                })
            }
            Expr::Call {
                name,
                name_span,
                args,
            } => {
                let function_info = self.check_function_declared(name, *name_span)?;

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

                if function_info.params.len() != args.len() {
                    return Err(SemanticError {
                        message: format!(
                            "function call of '{}' has {} arguments, declaration has {}",
                            function_info.name,
                            args.len(),
                            function_info.params.len()
                        ),
                        span: *name_span,
                    });
                }

                let typed_args = args
                    .iter()
                    .map(|arg| self.check_expr(arg))
                    .collect::<Result<Vec<_>, _>>()?;

                for (param, typed_arg) in function_info.params.iter().zip(&typed_args) {
                    if !Self::is_assignable(param.ty, typed_arg.ty) {
                        return Err(SemanticError {
                            message: format!(
                                "cannot pass value of type '{:?}' to parameter of type '{:?}'",
                                typed_arg.ty, param.ty
                            ),
                            span: typed_arg.diagnostic_span(),
                        });
                    }
                }

                Ok(TypedExpr {
                    ty: function_info.return_type,
                    kind: TypedExprKind::Call {
                        name: name.clone(),
                        name_span: *name_span,
                        args: typed_args,
                    },
                })
            }
            Expr::Assign {
                target,
                op_span,
                value,
            } => {
                let typed_target = self.check_lvalue(target, *op_span)?;
                let typed_value = self.check_expr(value)?;

                if !Self::is_assignable(typed_target.ty, typed_value.ty) {
                    return Err(SemanticError {
                        message: format!(
                            "cannot assign value of type '{:?}' to variable of type '{:?}'",
                            typed_value.ty, typed_target.ty
                        ),
                        span: *op_span,
                    });
                }

                let ty = typed_target.ty;
                Ok(TypedExpr {
                    kind: TypedExprKind::Assign {
                        target: Box::new(typed_target),
                        op_span: *op_span,
                        value: Box::new(typed_value),
                    },
                    ty,
                })
            }
            Expr::CompoundAssign {
                target,
                op,
                op_span,
                value,
            } => {
                let typed_target = self.check_lvalue(target, *op_span)?;
                let typed_value = self.check_expr(value)?;

                let result_type =
                    self.check_binary_op_types(op, op_span, typed_target.ty, typed_value.ty)?;

                if !Self::is_assignable(typed_target.ty, result_type) {
                    return Err(SemanticError {
                        message: format!(
                            "cannot assign value of type '{:?}' to variable of type '{:?}'",
                            result_type, typed_target.ty
                        ),
                        span: *op_span,
                    });
                }

                let ty = typed_target.ty;
                Ok(TypedExpr {
                    kind: TypedExprKind::CompoundAssign {
                        target: Box::new(typed_target),
                        op: *op,
                        op_span: *op_span,
                        value: Box::new(typed_value),
                    },
                    ty,
                })
            }
            Expr::PrefixInc { expr, op_span } => {
                self.check_inc_dec(expr, *op_span, |expr, op_span| TypedExprKind::PrefixInc {
                    expr,
                    op_span,
                })
            }

            Expr::PrefixDec { expr, op_span } => {
                self.check_inc_dec(expr, *op_span, |expr, op_span| TypedExprKind::PrefixDec {
                    expr,
                    op_span,
                })
            }

            Expr::PostfixInc { expr, op_span } => {
                self.check_inc_dec(expr, *op_span, |expr, op_span| TypedExprKind::PostfixInc {
                    expr,
                    op_span,
                })
            }

            Expr::PostfixDec { expr, op_span } => {
                self.check_inc_dec(expr, *op_span, |expr, op_span| TypedExprKind::PostfixDec {
                    expr,
                    op_span,
                })
            }
        }
    }

    fn check_statement(&mut self, statement: &Statement) -> Result<TypedStatement, SemanticError> {
        match statement {
            Statement::Return(expr) => {
                let typed_expr = self.check_expr(expr)?;

                let return_type = self
                    .current_function_return_type
                    .expect("return statement checked outside function");

                if !Self::is_assignable(return_type, typed_expr.ty) {
                    return Err(SemanticError {
                        message: format!(
                            "cannot return value of type '{:?}' from function returning '{return_type:?}'",
                            typed_expr.ty
                        ),
                        span: expr.diagnostic_span(),
                    });
                }
                Ok(TypedStatement::Return(typed_expr))
            }
            Statement::VarDecl {
                ty,
                name,
                name_span,
                init,
            } => {
                self.declare_local(*ty, name, *name_span)?;
                let typed_init = if let Some(init_expr) = init {
                    let typed_init = self.check_expr(init_expr)?;

                    if !Self::is_assignable(*ty, typed_init.ty) {
                        return Err(SemanticError {
                            message: format!(
                                "cannot assign value of type '{:?}' to variable of type '{ty:?}'",
                                typed_init.ty,
                            ),
                            span: typed_init.diagnostic_span(),
                        });
                    }

                    Some(typed_init)
                } else {
                    None
                };

                Ok(TypedStatement::VarDecl {
                    ty: *ty,
                    name: name.clone(),
                    name_span: *name_span,
                    init: typed_init,
                })
            }
            Statement::Block(body) => self.check_block(body),
            Statement::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let typed_cond = self.check_expr(cond)?;
                let typed_then = self.check_statement(then_branch)?;
                let typed_else = if let Some(else_statement) = else_branch {
                    Some(self.check_statement(else_statement)?)
                } else {
                    None
                };

                Ok(TypedStatement::If {
                    cond: typed_cond,
                    then_branch: Box::new(typed_then),
                    else_branch: typed_else.map(Box::new),
                })
            }
            Statement::While { cond, body } => {
                self.enter_loop();
                let res = (|| -> Result<TypedStatement, SemanticError> {
                    let typed_cond = self.check_expr(cond)?;
                    let typed_body = self.check_statement(body)?;

                    Ok(TypedStatement::While {
                        cond: typed_cond,
                        body: Box::new(typed_body),
                    })
                })();
                self.exit_loop();
                res
            }
            Statement::DoWhile { body, cond } => {
                self.enter_loop();
                let res = (|| -> Result<TypedStatement, SemanticError> {
                    let typed_body = self.check_statement(body)?;
                    let typed_cond = self.check_expr(cond)?;

                    Ok(TypedStatement::DoWhile {
                        body: Box::new(typed_body),
                        cond: typed_cond,
                    })
                })();
                self.exit_loop();
                res
            }
            Statement::For {
                init,
                cond,
                post,
                body,
            } => {
                self.enter_loop();
                self.enter_scope();

                let res = (|| -> Result<TypedStatement, SemanticError> {
                    let typed_init = if let Some(init_statement) = init {
                        Some(self.check_statement(init_statement)?)
                    } else {
                        None
                    };
                    let typed_cond = if let Some(cond_expr) = cond {
                        Some(self.check_expr(cond_expr)?)
                    } else {
                        None
                    };
                    let typed_post = if let Some(post_expr) = post {
                        Some(self.check_expr(post_expr)?)
                    } else {
                        None
                    };
                    let typed_body = self.check_statement(body)?;

                    Ok(TypedStatement::For {
                        init: typed_init.map(Box::new),
                        cond: typed_cond,
                        post: typed_post,
                        body: Box::new(typed_body),
                    })
                })();
                self.exit_scope();
                self.exit_loop();
                res
            }
            Statement::Empty => Ok(TypedStatement::Empty),
            Statement::ExprStatement(expr) => {
                let typed_expr = self.check_expr(expr)?;
                Ok(TypedStatement::ExprStatement(typed_expr))
            }
            Statement::Break { span } => {
                if !self.in_loop() {
                    return Err(SemanticError {
                        message: format!("cannot use 'break' outside of a loop"),
                        span: *span,
                    });
                }
                Ok(TypedStatement::Break { span: *span })
            }
            Statement::Continue { span } => {
                if !self.in_loop() {
                    return Err(SemanticError {
                        message: format!("cannot use 'continue' outside of a loop"),
                        span: *span,
                    });
                }
                Ok(TypedStatement::Continue { span: *span })
            }
        }
    }

    fn check_block(&mut self, body: &[Statement]) -> Result<TypedStatement, SemanticError> {
        self.enter_scope();
        let res = (|| -> Result<TypedStatement, SemanticError> {
            let mut typed_body = vec![];
            for statement in body {
                typed_body.push(self.check_statement(statement)?);
            }

            Ok(TypedStatement::Block(typed_body))
        })();
        self.exit_scope();
        res
    }

    fn check_function(&mut self, function: &Function) -> Result<TypedFunction, SemanticError> {
        self.enter_scope();
        let old_return_type = self
            .current_function_return_type
            .replace(function.return_type);

        let res = (|| -> Result<TypedFunction, SemanticError> {
            let mut typed_params = vec![];
            for param in &function.params {
                self.declare_local(param.ty, &param.name, param.name_span)?;

                typed_params.push(TypedParam {
                    ty: param.ty,
                    name: param.name.clone(),
                    name_span: param.name_span,
                });
            }
            let mut typed_body = vec![];
            for statement in &function.body {
                typed_body.push(self.check_statement(statement)?);
            }

            Ok(TypedFunction {
                return_type: function.return_type,
                name: function.name.clone(),
                name_span: function.name_span,
                params: typed_params,
                body: typed_body,
            })
        })();

        self.current_function_return_type = old_return_type;
        self.exit_scope();
        res
    }

    fn check_main_function(&mut self, span: Span) -> Result<(), SemanticError> {
        if !self.functions.contains_key("main") {
            return Err(SemanticError {
                message: "no 'main' function found".to_string(),
                span,
            });
        }
        Ok(())
    }
}

pub fn check(program: &Program) -> Result<TypedProgram, SemanticError> {
    let mut checker = Checker::new();

    for function in &program.functions {
        checker.declare_function(function)?;
    }

    checker.check_main_function(program.eof_span)?;

    let mut functions = vec![];
    for function in &program.functions {
        functions.push(checker.check_function(function)?);
    }

    Ok(TypedProgram {
        functions,
        eof_span: program.eof_span,
    })
}
