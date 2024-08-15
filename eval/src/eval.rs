
use crate::env::Env;
use crate::value::Value;
use log::{info, warn};
use ast::ast::Decl::{FunDecl, EndOfInstruction, SExpr, TypeAlias, TypeSignature};
use ast::ast::Expr::{Application, BinOp, If, Symbol, Tuple, Var};
use ast::ast::Op::{Add, And, Div, Eq, Ge, Gt, Le, Lt, Mul, Neq, Or, Sub};
use ast::ast::{Decl, Expr, List, Literal, Pattern, Type};

use crate::error::RunTimeError;

type RTResult<T> = Result<T, RunTimeError>;

fn eval_decl(env: &Env, decl: Decl) -> RTResult<()> {
    match decl {
        TypeAlias(_, _) => Ok(()),
        TypeSignature(_, _) => Ok(()),
        FunDecl(name, e) => {
            let closure = Value::Closure(e, env.clone());
            env.extend_function(name, closure);
            Ok(())
        }
        SExpr(e) => {
            let v = eval_expr(env, e)?;
            println!("> {}", v);
            Ok(())
        }
        EndOfInstruction => Ok(()),
    }
}

fn eval_expr(env: &Env, expr: Expr) -> Result<Value, RunTimeError> {
    //        if stack.len() > 10 {
    //            panic!("yeah fuck");
    //        }
    match expr {
        Expr::Literal(l) => {
            info!("Interpreting Literal {:?}", l);
            Ok(Value::Literal(l))
        }
        Symbol(name) => {
            info!("Interpreting function of name {name}");
            let val = env.get_function(&name)?;
            Ok(val)
        }
        Var(idx) => {
            info!("Interpreting variable #{idx}");
            env.get(idx)
        }
        Tuple(_vals) => {
            todo!()
        }
        If(test, ethen, eelse) => {
            info!(
                "Interpreting if {:?} then {:?} else {:?} ",
                test, ethen, eelse
            );
            let tv = eval_bool(env, *test)?;
            if tv {
                return eval_expr(env, *ethen);
            }
            return eval_expr(env, *eelse);
        }
        BinOp(l, op, r) => {
            info!("interpreting {:?} {:?} {:?}", l, op, r);
            return match op {
                Add => {
                    let lv = eval_int(env, *l)?;
                    let rv = eval_int(env, *r)?;
                    Ok(Value::Literal(Literal::Int(lv + rv)))
                }
                Sub => {
                    let lv = eval_int(env, *l)?;
                    let rv = eval_int(env, *r)?;
                    Ok(Value::Literal(Literal::Int(lv - rv)))
                }
                Mul => {
                    let lv = eval_int(env, *l)?;
                    let rv = eval_int(env, *r)?;
                    Ok(Value::Literal(Literal::Int(lv * rv)))
                }
                Div => {
                    let lv = eval_int(env, *l)?;
                    let rv = eval_int(env, *r)?;
                    Ok(Value::Literal(Literal::Int(lv / rv)))
                }
                Eq => {
                    let lv = eval_expr(env, *l)?;
                    let rv = eval_expr(env, *r)?;
                    Ok(Value::Literal(Literal::Bool(lv == rv)))
                }
                Neq => {
                    let lv = eval_expr(env, *l)?;
                    let rv = eval_expr(env, *r)?;
                    Ok(Value::Literal(Literal::Bool(lv != rv)))
                }
                Lt => {
                    let lv = eval_int(env, *l)?;
                    let rv = eval_int(env, *r)?;
                    Ok(Value::Literal(Literal::Bool(lv < rv)))
                }
                Gt => {
                    let lv = eval_int(env, *l)?;
                    let rv = eval_int(env, *r)?;
                    Ok(Value::Literal(Literal::Bool(lv > rv)))
                }
                Le => {
                    let lv = eval_int(env, *l)?;
                    let rv = eval_int(env, *r)?;
                    Ok(Value::Literal(Literal::Bool(lv <= rv)))
                }
                Ge => {
                    let lv = eval_int(env, *l)?;
                    let rv = eval_int(env, *r)?;
                    Ok(Value::Literal(Literal::Bool(lv >= rv)))
                }
                And => {
                    let lv = eval_bool(env, *l)?;
                    let rv = eval_bool(env, *r)?;
                    Ok(Value::Literal(Literal::Bool(lv && rv)))
                }
                Or => {
                    let lv = eval_bool(env, *l)?;
                    let rv = eval_bool(env, *r)?;
                    Ok(Value::Literal(Literal::Bool(lv || rv)))
                }
            };
        }
        Application(f, e) => {
            info!("Application of {:?} to {:?}", f, e);
            let new_env = env.extended(Value::Closure(*e, env.clone()));
            eval_expr(&new_env, *f)
        }
        Expr::Case(e, cases) => {
            let body = pattern_match_expr(env, *e, &cases)?;
            eval_expr(env, body.clone())
        }
        Expr::List(ls) => {
            Ok(Value::List(eval_list(env, ls)?))
        }
    }
}
fn eval_list(env: &Env, ls: List<Expr> ) -> Result<List<Value>, RunTimeError> {
    match ls {
        List::Some(e, es) => {
            let v = Box::new(eval_expr(env, *e)?);
            let vs = Box::new(eval_list(env, *es)?);
            Ok(List::Some(v, vs))
        },
        List::Empty => Ok(List::Empty)
    }
}

fn eval_int(env: &Env, expr: Expr) -> Result<i64, RunTimeError> {
    match eval_expr(env, expr)? {
        Value::Literal(Literal::Int(v)) => Ok(v),
        _ => Err(RunTimeError::TypeError(
            Type::TypeName("Int".to_string()),
            Type::TypeName("Later".to_string()),
        )),
    }
}
fn eval_bool(env: &Env, expr: Expr) -> Result<bool, RunTimeError> {
    match eval_expr(env, expr)? {
        Value::Literal(Literal::Bool(v)) => Ok(v),
        _ => Err(RunTimeError::TypeError(
            Type::TypeName("Bool".to_string()),
            Type::TypeName("Later".to_string()),
        )),
    }
}

fn pattern_match_expr<'a>(
    env: &Env,
    e: Expr,
    cases: &[(Pattern, Expr)],
) -> Result<(Expr, Env), RunTimeError> {
    for (pattern, body) in cases {
        match pattern {
            Pattern::Var => {
                let new_env = env.extended(Value::Closure(e, env.clone()));
                return Ok((body.clone(), new_env));
            },
            Pattern::EmptyList => {
                let v = eval_expr(env, e.clone())?;
                if let Value::List(List::Empty) = v {
                    return Ok((body.clone(), env.clone()))
                }
            },
            Pattern::Literal(l1) => {
                let v = eval_expr(env, e.clone())?;
                if let Value::Literal(l2) = v {
                    if *l1 == l2 {
                        return Ok((body.clone(), env.clone()))
                    }
                }
            },
            Pattern::List(p1, ps) => {

            },
        }
    }
    Err(RunTimeError::NonExhaustivePattern)
}

fn matches_value(arg: &Value, p: &Pattern) -> bool {
    match (arg, p) {
        (Value::Literal(l1), Pattern::Literal(l2)) => l1 == l2,
        (Value::List(ls), p) => matches_list(ls, p),
        (_, Pattern::Var) => true,
        _ => false,
    }
}

fn matches_list(ls: &List<Value>, p: &Pattern) -> bool {
    match (ls, p) {
        (List::Some(v, vs), Pattern::List(p, tail_pattern)) => matches_value(v, p) && matches_list(vs, tail_pattern),
        (List::Empty, Pattern::EmptyList) => true,
        (_, _) => false,
    }
}
