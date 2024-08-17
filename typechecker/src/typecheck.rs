use std::collections::HashMap;
use std::iter::zip;

use parser::ast::{AstNode, Expr, Op, Pattern, Type, Value};

use crate::error::TypeCheckingError;
use crate::TCResult;

struct TypingContext {
    type_context: HashMap<String, Type>,
    alias_context: HashMap<String, Type>,
}

impl TypingContext {
    fn new() -> TypingContext {
        return TypingContext {
            type_context: HashMap::new(),
            alias_context: HashMap::new(),
        };
    }

    fn add_alias(&mut self, name: String, t: Type) -> TCResult<()> {
        if self.alias_context.get(&name).is_some() {
            return Err(TypeCheckingError::MultipleDefinitions(name));
        }
        self.alias_context.insert(name, t);
        Ok(())
    }

    fn get_alias(&self, name: &String) -> Option<&Type> {
        self.alias_context.get(name)
    }

    fn declare_type(&mut self, symbol: String, t: Type) -> TCResult<()> {
        if self.type_context.get(&symbol).is_some() {
            return Err(TypeCheckingError::MultipleDefinitions(symbol));
        }
        self.type_context.insert(symbol, t);
        Ok(())
    }

    fn lookup_type(&self, symbol: &String) -> Option<&Type> {
        self.type_context.get(symbol)
    }
}

pub(crate) fn type_check(program: &Vec<AstNode>) -> TCResult<()> {
    let mut ctx = TypingContext::new();
    for decl in program {
        type_declare_decl(&mut ctx, decl)?;
    }
    for decl in program {
        type_check_decl(&mut ctx, decl)?;
    }
    Ok(())
}

fn type_check_decl(ctx: &mut TypingContext, line: &AstNode) -> TCResult<()> {
    return match line {
        AstNode::Decl(symbol, e) => match ctx.lookup_type(symbol) {
            Some(t) => type_check_expr(ctx, &mut vec![], t, e),
            None => {
                let t = type_produce_expr(ctx, &mut vec![], e)?;
                ctx.declare_type(symbol.to_string(), t)?;
                Ok(())
            }
        },
        AstNode::SExpr(e) => {
            type_produce_expr(ctx, &mut vec![], e)?;
            Ok(())
        }
        _ => Ok(()),
    };
}

fn type_check_expr(ctx: &TypingContext, arg_stack: &mut Vec<Type>, t: &Type, e: &Expr) -> TCResult<()> {
    if let Type::TypeName(name) = t {
        if let Some(aliased_type) = ctx.get_alias(name) {
            return type_check_expr(ctx, arg_stack, aliased_type, e);
        }
    }
    match e {
        Expr::Symbol(s) => {
            let symbol_type = ctx
                .lookup_type(s)
                .ok_or(TypeCheckingError::NoTypeDeclaration(s.clone()))?;
            types_match(&t, &symbol_type)
        }
        Expr::Var(idx) => {
            let var_type = arg_stack.get(arg_stack.len() - 1 - idx).ok_or(TypeCheckingError::UnboundArgument)?;
            types_match(&t, &var_type)
        }
        Expr::Application(f, e) => {
            let e_type = type_produce_expr(ctx, arg_stack, e)?;
            arg_stack.push(e_type.clone());
            let fun_type = type_produce_expr(ctx, arg_stack, f)?;
            arg_stack.pop();
            return match fun_type {
                Type::Function(arg_type, te) => {
                    types_match(&e_type, &arg_type)?;
                    types_match(&te, t)
                }
                _ => Err(TypeCheckingError::TypeMismatch(
                    fun_type.clone(),
                    Type::Function(Box::new(e_type.clone()), Box::new(t.clone())),
                )),
            };
        }
        Expr::Value(v) => match v {
            Value::Int(_) => types_match(t, &Type::Int),
            Value::Bool(_) => types_match(t, &Type::Bool),
            Value::Char(_) => types_match(t, &Type::Char),
            Value::String(_) => types_match(t, &Type::String),
            Value::ConstantFunction(e) => {
                let e_type = type_produce_expr(ctx, arg_stack, e)?;
                types_match(t, &e_type)
            }
            Value::Function(vec) => {
                let mut type_vec = vec![];
                for (pattern, e) in vec {
                    let pattern_type = match pattern {
                        Pattern::Value(Value::Int(_)) => Some(Type::Int),
                        Pattern::Value(Value::Bool(_)) => Some(Type::Bool),
                        Pattern::Value(Value::Char(_)) => Some(Type::Char),
                        Pattern::Value(Value::String(_)) => Some(Type::String),
                        Pattern::Var => None,
                        _ => {
                            return Err(TypeCheckingError::UnexpectedPattern);
                        }
                    };
                    let e_type = type_produce_expr(ctx, arg_stack, e)?;
                    type_vec.push((pattern_type, e_type));
                }

                if let Type::Function(arg_type, e_type) = t {
                    for (optional, expr_type) in type_vec {
                        if let Some(pattern_type) = optional {
                            types_match(&arg_type, &pattern_type)?;
                        }
                        types_match(&e_type, &expr_type)?;
                    }
                    return Ok(());
                }
                Err(TypeCheckingError::ExpectedFunction)
            }
        },
        Expr::Tuple(es) => {
            let ts = es
                .iter()
                .map(|e| type_produce_expr(ctx, arg_stack, e))
                .collect::<TCResult<Vec<_>>>()?;
            match t {
                Type::Tuple(declared_ts) => {
                    zip(declared_ts, ts)
                        .map(|(a, b)| types_match(&a, &b))
                        .collect::<TCResult<_>>()?;
                    Ok(())
                }
                _ => Err(TypeCheckingError::TypeMismatch(t.clone(), Type::Tuple(ts))),
            }
        }
        Expr::If(test, ife, elsee) => {
            type_check_expr(ctx, arg_stack, &Type::TypeName("Bool".to_string()), test)?;
            type_check_expr(ctx, arg_stack, t, ife)?;
            type_check_expr(ctx, arg_stack, t, elsee)?;
            Ok(())
        }
        Expr::BinOp(l, op, r) => {
            match op {
                Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Lt | Op::Gt | Op::Le | Op::Ge => {
                    type_check_expr(ctx, arg_stack, &Type::Int, l)?;
                    type_check_expr(ctx, arg_stack, &Type::Int, r)?;
                }
                Op::Eq | Op::Neq => {
                    let t = type_produce_expr(ctx, arg_stack, l)?;
                    type_check_expr(ctx, arg_stack, &t, r)?;
                }
                Op::And | Op::Or => {
                    type_check_expr(ctx, arg_stack, &Type::Bool, l)?;
                    type_check_expr(ctx, arg_stack, &Type::Bool, r)?;
                }
            }
            types_match(t, &Type::Bool)
        }
    }
}

fn types_match(type_a: &Type, type_b: &Type) -> TCResult<()> {
    if type_a.eq(type_b) {
        return Ok(());
    }
    Err(TypeCheckingError::TypeMismatch(
        type_a.clone(),
        type_b.clone(),
    ))
}

fn type_produce_expr(ctx: &TypingContext, arg_stack: &mut Vec<Type>, e: &Expr) -> TCResult<Type> {
    match e {
        Expr::Symbol(symbol) => ctx
            .lookup_type(symbol)
            .ok_or(TypeCheckingError::NoTypeDeclaration(symbol.to_string()))
            .cloned(),
        Expr::Var(idx) => arg_stack
            .get(arg_stack.len() - 1 - idx)
            .ok_or(TypeCheckingError::UnboundArgument)
            .cloned(),
        Expr::Application(f, e) => {
            let e_type = type_produce_expr(ctx, arg_stack, e)?;
            arg_stack.push(e_type.clone());
            let f_type = type_produce_expr(ctx, arg_stack, f)?;
            arg_stack.pop();
            match f_type {
                Type::Function(arg_type, res_type) => {
                    types_match(&e_type, &arg_type)?;
                    return Ok(*res_type);
                }
                _ => Err(TypeCheckingError::ExpectedFunction),
            }
        }
        Expr::Value(Value::Int(_)) => Ok(Type::Int),
        Expr::Value(Value::Bool(_)) => Ok(Type::Bool),
        Expr::Value(Value::Char(_)) => Ok(Type::Char),
        Expr::Value(Value::String(_)) => Ok(Type::String),
        Expr::Value(Value::ConstantFunction(e)) => type_produce_expr(ctx, arg_stack, e),
        Expr::Value(Value::Function(vec)) => {
            let mut arg_type = None;
            let mut e_type = None;
            for (pattern, e) in vec {
                let pattern_type = match pattern {
                    Pattern::Value(Value::Int(_)) => Some(Type::Int),
                    Pattern::Value(Value::Bool(_)) => Some(Type::Bool),
                    Pattern::Value(Value::Char(_)) => Some(Type::Char),
                    Pattern::Value(Value::String(_)) => Some(Type::String),
                    Pattern::Var => None,
                    _ => {
                        return Err(TypeCheckingError::UnexpectedPattern);
                    }
                };
                let expr_type = type_produce_expr(ctx, arg_stack, e)?;
                if arg_type.is_none()  {
                    if let Some(ref t) = pattern_type {
                        arg_type = Some(t);
                    }
                }
                if e_type.is_none() {
                    e_type = Some(&expr_type);
                }
                if let Some(ref a) = arg_type {
                    if let Some(ref b) = pattern_type {
                        types_match(&a, &b)?;
                    }
                }
                if let Some(ref a) = e_type {
                    types_match(&a, &expr_type)?;
                }
            }
            let arg_type = Box::new(arg_type.ok_or(TypeCheckingError::ArgumentTypeUnknown)?.clone());
            let e_type = Box::new(e_type.unwrap().clone());
            Ok(Type::Function(arg_type, e_type))
        },
        Expr::Tuple(es) => {
            let mut ts = vec![];
            for e in es {
                ts.push(type_produce_expr(ctx, arg_stack, e)?.clone());
            }
            Ok(Type::Tuple(ts))
        },
        Expr::If(_, ife, elsee) => {
            let if_type = type_produce_expr(ctx, arg_stack, ife)?;
            let else_type = type_produce_expr(ctx, arg_stack, elsee)?;
            types_match(&if_type, &else_type)?;
            Ok(if_type)
        }
        Expr::BinOp(l, op, r) => {
            let l_type = type_produce_expr(ctx, arg_stack, l)?;
            let r_type = type_produce_expr(ctx, arg_stack, r)?;
            types_match(&l_type, &r_type)?;
            match op {
                Op::Add |
                Op::Sub |
                Op::Mul |
                Op::Div => Ok(Type::Int),
                Op::Eq |
                Op::Neq |
                Op::Lt |
                Op::Gt |
                Op::Le |
                Op::Ge |
                Op::And |
                Op::Or => Ok(Type::Bool),
            }
        },
    }
}

fn type_declare_decl(ctx: &mut TypingContext, decl: &AstNode) -> TCResult<()> {
    return match decl {
        AstNode::TypeAlias(name, t) => ctx.add_alias(name.clone(), t.clone()),
        AstNode::TypeSignature(symbol, t) => {
            let t = remove_aliases(ctx, t);
            ctx.declare_type(symbol.clone(), t)
        },
        _ => Ok(()),
    };
}

fn remove_aliases(ctx: &mut TypingContext, t: &Type) -> Type {
    match t {
        Type::TypeName(name) => ctx.get_alias(&name).cloned().unwrap_or(Type::TypeName(name.clone())),
        Type::Function(ft, et) => Type::Function(Box::new(remove_aliases(ctx, ft)), Box::new(remove_aliases(ctx, et))),
        Type::Tuple(ts) => {
            let collect = ts.iter().map(|a_type| remove_aliases(ctx, a_type).clone()).collect::<Vec<_>>();
            Type::Tuple(collect)
        },
        _ => t.clone(),
    }
}
