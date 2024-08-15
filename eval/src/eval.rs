use crate::env::Env;
use crate::value::Value;
use log::info;
use ast::ast::Decl::{FunDecl, EndOfInstruction, SExpr, TypeAlias, TypeSignature};
use ast::ast::Expr::{Application, BinOp, If, Symbol, Tuple, Var};
use ast::ast::Op::{Add, And, Div, Eq, Ge, Gt, Le, Lt, Mul, Neq, Or, Sub};
use ast::ast::{Decl, Expr, List, Literal, Pattern, Type};

use crate::error::RunTimeError;

type RTResult<T> = Result<T, RunTimeError>;

pub fn eval(program: Vec<Decl>) -> RTResult<()> {
    let env = Env::new();
    for decl in program {
        eval_decl(&env, decl)?;
    }
    Ok(())
}

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

fn eval_expr(env: &Env, expr: Expr) -> RTResult<Value> {
    match expr {
        Symbol(name) => {
            info!("Interpreting function of name {name}");
            match env.get_function(&name)? {
                Value::Closure(e, c_env) => eval_expr(&c_env, e),
                val => Ok(val)
            }
        }
        Var(idx) => {
            info!("Interpreting variable #{idx}");
            match env.get(idx)? {
                Value::Closure(e, c_env) => eval_expr(&c_env, e),
                val => Ok(val)
            }
        }
        Expr::Literal(l) => {
            info!("Interpreting Literal {:?}", l);
            Ok(Value::Literal(l))
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
            let (body, new_env) = pattern_match_expr(env, *e, &cases)?;
            eval_expr(&new_env, body)
        }
        Expr::List(ls) => {
            match ls {
                List::Some(head, tail) =>  Ok(
                    Value::List(
                        Box::new(Value::Closure(*head, env.clone())), 
                        Box::new(Value::Closure(Expr::List(*tail), env.clone()))
                    )
                ),
                List::Empty => Ok(Value::EmptyList),
            }
        }
    }
}

fn eval_int(env: &Env, expr: Expr) -> RTResult<i64> {
    match eval_expr(env, expr)? {
        Value::Literal(Literal::Int(v)) => Ok(v),
        _ => Err(RunTimeError::TypeError(
            Type::TypeName("Int".to_string()),
            Type::TypeName("Later".to_string()),
        )),
    }
}
fn eval_bool(env: &Env, expr: Expr) -> RTResult<bool> {
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
) -> RTResult<(Expr, Env)> {
    for (p, body) in cases {
        if let Some(updated_env) = match_pattern(env, p, e.clone())? {
           return Ok((body.clone(), updated_env));
        }
    }
    Err(RunTimeError::NonExhaustivePattern)
}

fn match_pattern(env: &Env, p: &Pattern, e: Expr) -> RTResult<Option<Env>> { 
    match p {
        Pattern::Var => {
            let new_env = env.extended(Value::Closure(e.clone(), env.clone()));
            return Ok(Some(new_env));
        },
        Pattern::EmptyList => {
            let v = eval_expr(env, e.clone())?;
            if let Value::EmptyList = v {
                return Ok(Some(env.clone()));
            }
        },
        Pattern::Literal(l1) => {
            let v = eval_expr(env, e.clone())?;
            if let Value::Literal(l2) = v {
                if *l1 == l2 {
                    return Ok(Some(env.clone()));
                }
            }
        },
        Pattern::List(p, ps) => {
            if let Value::List(head, tail) = eval_expr(env, e)? {
                if let Some(new_env) = matches_value(env, p, &head)? {
                    return matches_value(&new_env, ps, &tail);
                }
            }
        },
    }
    Ok(None)
}

fn matches_value(env: &Env, p: &Pattern, val: &Value) -> RTResult<Option<Env>> {
    Ok(match (val, p) {
        (Value::Literal(l1), Pattern::Literal(l2)) => if l1 == l2 {Some(env.clone())} else {None},
        (v, Pattern::Var) => Some(env.extended(v.clone())),
        (Value::EmptyList, Pattern::EmptyList) => Some(env.clone()),
        (Value::List(_, _), Pattern::List(_, _)) => todo!(),
        (Value::Closure(e, inner_env), p) => {
            let v = eval_expr(inner_env, e.clone())?;
            matches_value(env, p, &v)?
        },
        (_, _) => None,
    })
}
