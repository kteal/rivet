use std::collections::HashMap;

use crate::ast::{
    BinaryOp, Expr, ExternalDecl, Function, Initializer, IntLiteralBase, IntLiteralSuffix, Param,
    Program, Statement, Type, UnaryOp,
};
use crate::source::Span;

use crate::typed_ast::{
    LocalId, TypedExpr, TypedExprKind, TypedExternalDecl, TypedFunction, TypedInitializer,
    TypedLocalDecl, TypedParam, TypedProgram, TypedStatement,
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

struct BinaryTypeInfo {
    operand_ty: Type,
    result_ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalSymbol {
    id: LocalId,
    ty: Type,
}

struct Checker {
    scopes: Vec<HashMap<String, LocalSymbol>>,
    functions: HashMap<String, FunctionInfo>,
    loop_depth: usize,
    current_function_return_type: Option<Type>,
    next_local_id: usize,
}

impl Checker {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            loop_depth: 0,
            current_function_return_type: None,
            next_local_id: 0,
        }
    }

    const fn new_local_id(&mut self) -> LocalId {
        let id = LocalId(self.next_local_id);
        self.next_local_id += 1;
        id
    }

    const fn enter_loop(&mut self) {
        self.loop_depth += 1;
    }

    fn exit_loop(&mut self) {
        assert!(self.loop_depth != 0, "cannot have negative loop depth");
        self.loop_depth -= 1;
    }

    const fn in_loop(&self) -> bool {
        self.loop_depth > 0
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn current_scope(&self) -> &HashMap<String, LocalSymbol> {
        self.scopes
            .last()
            .expect("semantic checker should have an active scope")
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<String, LocalSymbol> {
        self.scopes
            .last_mut()
            .expect("semantic checker should have an active scope")
    }

    fn declare_local(
        &mut self,
        ty: &Type,
        name: &str,
        span: Span,
    ) -> Result<LocalSymbol, SemanticError> {
        if self.current_scope().contains_key(name) {
            return Err(SemanticError {
                message: format!("duplicate local variable '{name}'"),
                span,
            });
        }
        let symbol = LocalSymbol {
            id: self.new_local_id(),
            ty: ty.clone(),
        };
        self.current_scope_mut()
            .insert(name.to_string(), symbol.clone());
        Ok(symbol)
    }

    fn resolve_local(&self, name: &str, span: Span) -> Result<LocalSymbol, SemanticError> {
        for scope in self.scopes.iter().rev() {
            if let Some(local) = scope.get(name) {
                return Ok(local.clone());
            }
        }
        Err(SemanticError {
            message: format!("undeclared local variable '{name}'"),
            span,
        })
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
            function.name.clone(),
            FunctionInfo {
                return_type: function.return_type.clone(),
                name: function.name.clone(),
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
        op: BinaryOp,
        op_span: Span,
        left_type: &Type,
        right_type: &Type,
    ) -> Result<BinaryTypeInfo, SemanticError> {
        if let Type::Pointer(inner) = left_type
            && right_type.is_integer()
        {
            if op == BinaryOp::Add || op == BinaryOp::Subtract {
                return Ok(BinaryTypeInfo {
                    operand_ty: Type::Pointer(inner.clone()),
                    result_ty: Type::Pointer(inner.clone()),
                });
            }
            return Err(SemanticError {
                message: format!(
                    "cannot perform binary operation '{op:?}' on types '{left_type:?}' and '{right_type:?}'"
                ),
                span: op_span,
            });
        }

        if let Type::Pointer(inner) = right_type
            && left_type.is_integer()
        {
            if op == BinaryOp::Add {
                return Ok(BinaryTypeInfo {
                    operand_ty: Type::Pointer(inner.clone()),
                    result_ty: Type::Pointer(inner.clone()),
                });
            }
            return Err(SemanticError {
                message: format!(
                    "cannot perform binary operation '{op:?}' on types '{left_type:?}' and '{right_type:?}'"
                ),
                span: op_span,
            });
        }

        if let Type::Pointer(left_inner) = left_type
            && let Type::Pointer(right_inner) = right_type
        {
            if left_inner == right_inner && (op == BinaryOp::Equal || op == BinaryOp::NotEqual) {
                return Ok(BinaryTypeInfo {
                    operand_ty: left_type.clone(),
                    result_ty: Type::Int,
                });
            }
            return Err(SemanticError {
                message: format!(
                    "invalid operands to binary operator '{op:?}'\n\
                     left operand has type '{left_type:?}'\n\
                     right operand has type '{right_type:?}'"
                ),
                span: op_span,
            });
        }

        if !left_type.is_integer() || !right_type.is_integer() {
            return Err(SemanticError {
                message: format!(
                    "invalid operands to binary operator '{op:?}'\n\
                     left operand has type '{left_type:?}'\n\
                     right operand has type '{right_type:?}'"
                ),
                span: op_span,
            });
        }
        let mut operand_ty = Type::usual_arithmetic_type(left_type, right_type);
        let result_ty = if op == BinaryOp::ShiftLeft || op == BinaryOp::ShiftRight {
            left_type
                .promoted()
                .expect("semantic checker should only promote integer types")
        } else {
            match op {
                BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual
                | BinaryOp::LogicalAnd
                | BinaryOp::LogicalOr => Type::Int,

                _ => operand_ty.clone(),
            }
        };
        if op == BinaryOp::ShiftLeft || op == BinaryOp::ShiftRight {
            operand_ty = left_type
                .promoted()
                .expect("semantic checker should only promote integer types");
        }

        Ok(BinaryTypeInfo {
            operand_ty,
            result_ty,
        })
    }

    fn is_assignable_expr(target_ty: &Type, value: &TypedExpr) -> bool {
        target_ty.is_assignable_from(&value.ty)
            || (target_ty.is_pointer() && value.is_null_pointer_constant())
    }

    const fn is_inc_dec_type(ty: &Type) -> bool {
        ty.is_integer() || matches!(ty, Type::Pointer(_))
    }

    fn check_lvalue(&self, expr: &Expr, op_span: Span) -> Result<TypedExpr, SemanticError> {
        match expr {
            Expr::Variable { name, span } => {
                let symbol = self.resolve_local(name, *span)?;

                if matches!(symbol.ty, Type::Array { .. }) {
                    return Err(SemanticError {
                        message: "cannot assign to array expression".to_string(),
                        span: *span,
                    });
                }

                Ok(TypedExpr {
                    kind: TypedExprKind::Variable {
                        id: symbol.id,
                        name: name.clone(),
                        span: *span,
                    },
                    ty: symbol.ty,
                })
            }
            Expr::Unary {
                op: UnaryOp::Dereference,
                op_span,
                expr,
            } => {
                let typed_ptr = self.check_expr(expr)?;

                let Type::Pointer(inner) = &typed_ptr.ty.clone() else {
                    return Err(SemanticError {
                        message: format!(
                            "cannot dereference non-pointer type '{:?}'",
                            typed_ptr.ty
                        ),
                        span: *op_span,
                    });
                };

                Ok(TypedExpr {
                    kind: TypedExprKind::Unary {
                        op: UnaryOp::Dereference,
                        op_span: *op_span,
                        expr: Box::new(typed_ptr),
                    },
                    ty: *inner.clone(),
                })
            }
            Expr::Index { base, index, span } => self.check_index_expr(base, index, *span),
            _ => Err(SemanticError {
                message: "cannot assign to non-lvalue expression".to_string(),
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
        if !Self::is_inc_dec_type(&typed_lvalue.ty) {
            return Err(SemanticError {
                message: format!(
                    "cannot increment or decrement value of type '{:?}'",
                    typed_lvalue.ty
                ),
                span: op_span,
            });
        }
        let ty = typed_lvalue.ty.clone();
        Ok(TypedExpr {
            kind: make_kind(Box::new(typed_lvalue), op_span),
            ty,
        })
    }

    fn check_unary_expr(
        &self,
        op: UnaryOp,
        op_span: Span,
        expr: &Expr,
    ) -> Result<TypedExpr, SemanticError> {
        let typed_expr = self.check_expr(expr)?;
        let ty = match op {
            UnaryOp::LogicalNot => Type::Int,
            UnaryOp::BitwiseNot | UnaryOp::Negate => {
                if !typed_expr.ty.is_integer() {
                    return Err(SemanticError {
                        message: format!(
                            "cannot perform unary operation '{op:?}' on non-integer type '{:?}'",
                            typed_expr.ty
                        ),
                        span: op_span,
                    });
                }
                typed_expr
                    .ty
                    .promoted()
                    .expect("semantic checker should only promote integer types")
            }
            UnaryOp::Dereference => match &typed_expr.ty {
                Type::Pointer(inner) => *inner.clone(),
                _ => {
                    return Err(SemanticError {
                        message: format!(
                            "cannot dereference non-pointer type '{:?}'",
                            typed_expr.ty
                        ),
                        span: op_span,
                    });
                }
            },
        };
        Ok(TypedExpr {
            kind: TypedExprKind::Unary {
                op,
                op_span,
                expr: Box::new(typed_expr),
            },
            ty,
        })
    }

    fn check_call_expr(
        &self,
        name: &str,
        name_span: &Span,
        args: &[Expr],
    ) -> Result<TypedExpr, SemanticError> {
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
            if !Self::is_assignable_expr(&param.ty, typed_arg) {
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
            ty: function_info.return_type.clone(),
            kind: TypedExprKind::Call {
                name: name.to_string(),
                name_span: *name_span,
                args: typed_args,
            },
        })
    }

    fn check_index_expr(
        &self,
        base: &Expr,
        index: &Expr,
        span: Span,
    ) -> Result<TypedExpr, SemanticError> {
        let typed_base = self.check_expr(base)?;
        let typed_index = self.check_expr(index)?;

        if !typed_index.ty.is_integer() {
            return Err(SemanticError {
                message: format!(
                    "array index must be integer type, found '{:?}'",
                    typed_index.ty
                ),
                span,
            });
        }

        let element_ty = match &typed_base.ty {
            Type::Array { element, .. } | Type::Pointer(element) => *element.clone(),
            ty => {
                return Err(SemanticError {
                    message: format!("cannot index expression of type '{ty:?}'"),
                    span,
                });
            }
        };

        Ok(TypedExpr {
            kind: TypedExprKind::Index {
                base: Box::new(typed_base),
                index: Box::new(typed_index),
                span,
            },
            ty: element_ty,
        })
    }

    fn check_cast_expr(
        &self,
        ty: &Type,
        expr: &Expr,
        span: Span,
    ) -> Result<TypedExpr, SemanticError> {
        let typed_expr = self.check_expr(expr)?;

        if !ty.is_integer() || !typed_expr.ty.is_integer() {
            return Err(SemanticError {
                message: format!("cannot cast type '{:?}' to '{ty:?}'", typed_expr.ty),
                span,
            });
        }

        Ok(TypedExpr {
            ty: ty.clone(),
            kind: TypedExprKind::Cast {
                target_ty: ty.clone(),
                expr: Box::new(typed_expr),
                span,
            },
        })
    }

    #[allow(clippy::too_many_lines)]
    fn check_expr(&self, expr: &Expr) -> Result<TypedExpr, SemanticError> {
        match expr {
            Expr::IntLiteral {
                value,
                suffix,
                base,
                span,
            } => {
                let ty = match (suffix, base) {
                    (IntLiteralSuffix::None, IntLiteralBase::Decimal) => {
                        if value <= &(i32::MAX as u64) {
                            Type::Int
                        } else {
                            return Err(SemanticError {
                                message: format!(
                                    "integer literal '{value}' is too large for type '{:?}'",
                                    Type::Int
                                ),
                                span: *span,
                            });
                        }
                    }
                    (IntLiteralSuffix::None, IntLiteralBase::Hex) => {
                        if value <= &(i32::MAX as u64) {
                            Type::Int
                        } else if value <= &u64::from(u32::MAX) {
                            Type::UnsignedInt
                        } else {
                            return Err(SemanticError {
                                message: format!(
                                    "integer literal '{value}' is too large for type '{:?}'",
                                    Type::Int
                                ),
                                span: *span,
                            });
                        }
                    }
                    (IntLiteralSuffix::Unsigned, _) => {
                        if value <= &u64::from(u32::MAX) {
                            Type::UnsignedInt
                        } else {
                            return Err(SemanticError {
                                message: format!(
                                    "integer literal '{value}' is too large for type '{:?}'",
                                    Type::UnsignedInt
                                ),
                                span: *span,
                            });
                        }
                    }
                    (IntLiteralSuffix::Long, _) => {
                        if value <= &(i32::MAX as u64) {
                            Type::Long
                        } else {
                            return Err(SemanticError {
                                message: format!(
                                    "integer literal '{value}' is too large for type '{:?}'",
                                    Type::Long
                                ),
                                span: *span,
                            });
                        }
                    }
                    (IntLiteralSuffix::UnsignedLong, _) => {
                        if value <= &u64::from(u32::MAX) {
                            Type::UnsignedLong
                        } else {
                            return Err(SemanticError {
                                message: format!(
                                    "integer literal '{value}' is too large for type '{:?}'",
                                    Type::UnsignedLong
                                ),
                                span: *span,
                            });
                        }
                    }
                };

                Ok(TypedExpr {
                    kind: TypedExprKind::IntLiteral {
                        value: *value,
                        span: *span,
                    },
                    ty,
                })
            }
            Expr::Variable { name, span } => {
                let symbol = self.resolve_local(name, *span)?;

                if let Type::Array { element, .. } = &symbol.ty {
                    return Ok(TypedExpr {
                        kind: TypedExprKind::Variable {
                            id: symbol.id,
                            name: name.clone(),
                            span: *span,
                        },
                        ty: Type::Pointer(element.clone()),
                    });
                }

                Ok(TypedExpr {
                    kind: TypedExprKind::Variable {
                        id: symbol.id,
                        name: name.clone(),
                        span: *span,
                    },
                    ty: symbol.ty,
                })
            }
            Expr::Unary { op, op_span, expr } => self.check_unary_expr(*op, *op_span, expr),
            Expr::Binary {
                op,
                op_span,
                left,
                right,
            } => {
                let typed_left = self.check_expr(left)?;
                let typed_right = self.check_expr(right)?;

                if matches!(op, BinaryOp::Equal | BinaryOp::NotEqual)
                    && ((typed_left.ty.is_pointer() && typed_right.is_null_pointer_constant())
                        || (typed_right.ty.is_pointer() && typed_left.is_null_pointer_constant()))
                {
                    let operand_ty = if typed_left.ty.is_pointer() {
                        typed_left.ty.clone()
                    } else {
                        typed_right.ty.clone()
                    };

                    return Ok(TypedExpr {
                        kind: TypedExprKind::Binary {
                            op: *op,
                            op_span: *op_span,
                            operand_ty,
                            left: Box::new(typed_left),
                            right: Box::new(typed_right),
                        },
                        ty: Type::Int,
                    });
                }

                let type_info =
                    Self::check_binary_op_types(*op, *op_span, &typed_left.ty, &typed_right.ty)?;

                Ok(TypedExpr {
                    kind: TypedExprKind::Binary {
                        op: *op,
                        op_span: *op_span,
                        operand_ty: type_info.operand_ty,
                        left: Box::new(typed_left),
                        right: Box::new(typed_right),
                    },
                    ty: type_info.result_ty,
                })
            }
            Expr::Call {
                name,
                name_span,
                args,
            } => self.check_call_expr(name, name_span, args),
            Expr::Assign {
                target,
                op_span,
                value,
            } => {
                let typed_target = self.check_lvalue(target, *op_span)?;
                let typed_value = self.check_expr(value)?;

                if !Self::is_assignable_expr(&typed_target.ty, &typed_value) {
                    return Err(SemanticError {
                        message: format!(
                            "cannot assign value of type '{:?}' to variable of type '{:?}'",
                            typed_value.ty, typed_target.ty
                        ),
                        span: *op_span,
                    });
                }

                let ty = typed_target.ty.clone();
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

                let type_info =
                    Self::check_binary_op_types(*op, *op_span, &typed_target.ty, &typed_value.ty)?;

                if !typed_target.ty.is_assignable_from(&type_info.result_ty) {
                    return Err(SemanticError {
                        message: format!(
                            "cannot assign value of type '{:?}' to variable of type '{:?}'",
                            type_info.result_ty, typed_target.ty
                        ),
                        span: *op_span,
                    });
                }

                let ty = typed_target.ty.clone();
                Ok(TypedExpr {
                    kind: TypedExprKind::CompoundAssign {
                        target: Box::new(typed_target),
                        op: *op,
                        op_span: *op_span,
                        operand_ty: type_info.operand_ty,
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
            Expr::Index { base, index, span } => self.check_index_expr(base, index, *span),
            Expr::Cast { ty, expr, span } => self.check_cast_expr(ty, expr, *span),
        }
    }

    fn check_initializer(
        &self,
        target_ty: &Type,
        name_span: Span,
        initializer: &Initializer,
    ) -> Result<TypedInitializer, SemanticError> {
        match (target_ty, initializer) {
            (
                Type::Array {
                    element: element_ty,
                    len: array_len,
                },
                Initializer::List(values),
            ) => {
                // Arrays must be initialized with a List
                let mut typed_values = vec![];

                if values.len() > *array_len {
                    return Err(SemanticError {
                        message: format!(
                            "array initializer list must have <= '{array_len}' elements, has '{}' elements",
                            values.len()
                        ),
                        span: name_span,
                    });
                }

                for value in values {
                    let typed_value = self.check_expr(value)?;
                    if !Self::is_assignable_expr(element_ty, &typed_value) {
                        return Err(SemanticError {
                            message: format!(
                                "cannot assign value of type '{:?}' to array of type '{element_ty:?}'",
                                typed_value.ty,
                            ),
                            span: typed_value.diagnostic_span(),
                        });
                    }
                    typed_values.push(typed_value);
                }

                Ok(TypedInitializer::List(typed_values))
            }
            (
                Type::Char
                | Type::UnsignedChar
                | Type::SignedChar
                | Type::Int
                | Type::UnsignedInt
                | Type::Pointer(_)
                | Type::Long
                | Type::UnsignedLong,
                Initializer::Expr(init_expr),
            ) => {
                // Scalars must be initialized with an Expr
                let typed_init = self.check_expr(init_expr)?;
                if !Self::is_assignable_expr(target_ty, &typed_init) {
                    return Err(SemanticError {
                        message: format!(
                            "cannot assign value of type '{:?}' to variable of type '{target_ty:?}'",
                            typed_init.ty,
                        ),
                        span: typed_init.diagnostic_span(),
                    });
                }

                Ok(TypedInitializer::Expr(typed_init))
            }
            (Type::Array { .. }, Initializer::Expr(_)) => Err(SemanticError {
                message: "array must be initialized with list".to_string(),
                span: name_span,
            }),
            (
                Type::Char
                | Type::UnsignedChar
                | Type::SignedChar
                | Type::Int
                | Type::UnsignedInt
                | Type::Pointer(_)
                | Type::Long
                | Type::UnsignedLong,
                Initializer::List(_),
            ) => Err(SemanticError {
                message: format!("cannot initialize scalar type '{target_ty:?}' with list"),
                span: name_span,
            }),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn check_statement(&mut self, statement: &Statement) -> Result<TypedStatement, SemanticError> {
        match statement {
            Statement::Return(expr) => {
                let typed_expr = self.check_expr(expr)?;

                let return_type = self
                    .current_function_return_type
                    .as_ref()
                    .expect("return statement checked outside function");

                if !Self::is_assignable_expr(return_type, &typed_expr) {
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
            Statement::Decl(declarations) => {
                let mut typed_declarations = vec![];
                for declaration in declarations {
                    let symbol = self.declare_local(
                        &declaration.ty,
                        &declaration.name,
                        declaration.name_span,
                    )?;
                    let typed_init = declaration
                        .init
                        .as_ref()
                        .map(|initializer| {
                            self.check_initializer(
                                &declaration.ty,
                                declaration.name_span,
                                initializer,
                            )
                        })
                        .transpose()?;
                    typed_declarations.push(TypedLocalDecl {
                        id: symbol.id,
                        ty: symbol.ty,
                        name: declaration.name.clone(),
                        name_span: declaration.name_span,
                        init: typed_init,
                    });
                }

                Ok(TypedStatement::Decl(typed_declarations))
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
                        message: "cannot use 'break' outside of a loop".to_string(),
                        span: *span,
                    });
                }
                Ok(TypedStatement::Break { span: *span })
            }
            Statement::Continue { span } => {
                if !self.in_loop() {
                    return Err(SemanticError {
                        message: "cannot use 'continue' outside of a loop".to_string(),
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
            .replace(function.return_type.clone());

        let res = (|| -> Result<TypedFunction, SemanticError> {
            let mut typed_params = vec![];
            for param in &function.params {
                let symbol = self.declare_local(&param.ty, &param.name, param.name_span)?;

                typed_params.push(TypedParam {
                    id: symbol.id,
                    ty: symbol.ty,
                    name: param.name.clone(),
                    name_span: param.name_span,
                });
            }
            let mut typed_body = vec![];
            for statement in &function.body {
                typed_body.push(self.check_statement(statement)?);
            }

            Ok(TypedFunction {
                return_type: function.return_type.clone(),
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

    fn check_main_function(&self, span: Span) -> Result<(), SemanticError> {
        if !self.functions.contains_key("main") {
            return Err(SemanticError {
                message: "no 'main' function found".to_string(),
                span,
            });
        }
        Ok(())
    }
}

/// Performs semantic analysis and returns a typed AST.
///
/// # Errors
///
/// Returns a [`SemanticError`] when the program violates the currently supported
/// semantic rules, such as using undeclared variables, redeclaring locals, or
/// applying operators to unsupported operand types.
pub fn check(program: &Program) -> Result<TypedProgram, SemanticError> {
    let mut checker = Checker::new();

    for decl in &program.declarations {
        if let ExternalDecl::Function(function) = decl {
            checker.declare_function(function)?;
        }
    }

    checker.check_main_function(program.eof_span)?;

    let mut declarations = vec![];
    for decl in &program.declarations {
        if let ExternalDecl::Function(function) = decl {
            declarations.push(TypedExternalDecl::Function(
                checker.check_function(function)?,
            ));
        }
    }

    Ok(TypedProgram {
        declarations,
        eof_span: program.eof_span,
    })
}
