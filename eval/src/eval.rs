use std::iter::zip;

use crate::env::Env;
use crate::value::Value;
use log::info;
use ast::ast::Decl::{FunDecl, EndOfInstruction, SExpr, TypeAlias, TypeSignature};
use ast::ast::Expr::{Application, BinOp, If, Symbol, Tuple, Var};
use ast::ast::Op::{Add, And, Div, Eq, Ge, Gt, Le, Lt, Mul, Neq, Or, Sub, Append, Cons};
use ast::ast::{Decl, Expr, List, Literal, Pattern, Type};

use crate::error::RunTimeError;

pub type RTResult<T> = Result<T, RunTimeError>;

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
        FunDecl(name, args, e) => {
            let closure = Value::Closure(e, args, env.clone());
            info!("Storing {}: {} to env", name, closure);
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
            let v = env.get_function(&name)?;
            handle_closure(v)
        }
        Var(idx) => {
            info!("Interpreting variable #{idx}");
            let v = env.get(idx)?;
            handle_closure(v)
        }
        Expr::Literal(l) => {
            info!("Interpreting Literal {:?}", l);
            Ok(Value::Literal(l))
        }
        Tuple(es) => {
            info!("Interpreting Tuple {:?}", es);
            Ok(Value::Tuple(es.into_iter().map(|e| Value::Closure(e, 0, env.clone())).collect()))
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
                Append => {
                    let lv = eval_expr(env, *l)?;
                    let rv = eval_expr(env, *r)?;
                    let mut curr = lv;
                    let mut elements = vec![];
                    while let Value::List(x, xs) = curr {
                        elements.push(x);
                        match *xs {
                            Value::Closure(expr, 0, env) => curr = eval_expr(&env, expr)?,
                            tail => curr = tail,
                        }
                    };
                    let big_list = elements.into_iter().rev().fold(Box::new(rv), |acc, x| Box::new(Value::List(x, acc)));
                    Ok(*big_list)
                },
                Cons => {
                    let lv = eval_expr(env, *l)?;
                    let rv = eval_expr(env, *r)?;
                    Ok(Value::List(Box::new(lv), Box::new(rv)))
                },
            };
        }
        Application(f, e) => {
            info!("Application of {:?} to {:?}", f, e);
            info!("Pushing {} to env", e);
            let new_env = env.extended(Value::Closure(*e, 0, env.clone()));
            let return_value = eval_expr(&new_env, *f)?;
            extend_closure(return_value, new_env.get(0)?)
        }
        Expr::Case(e, cases) => {
            info!("Interpreting case on {} => {:?}", e, cases);
            let (body, new_env) = pattern_match_expr(env, *e, &cases)?;
            eval_expr(&new_env, body)
        }
        Expr::List(ls) => {
            info!("Interpreting list {:?}", ls);
            match ls {
                List::Some(head, tail) =>  Ok(
                    Value::List(
                        Box::new(Value::Closure(*head, 0, env.clone())), 
                        Box::new(Value::Closure(Expr::List(*tail), 0, env.clone()))
                    )
                ),
                List::Empty => Ok(Value::EmptyList),
            }
        }
    }
}

fn handle_closure(v: Value) -> RTResult<Value> {
    match v {
        Value::Closure(e, 0, c_env) => eval_expr(&c_env, e),
        val => Ok(val)

    }
}

fn extend_closure(v: Value, arg: Value) -> RTResult<Value> {
    info!("Pushing {} to closure {}", arg, v);
    match v {
        Value::Closure(e, 0, c_env) => eval_expr(&c_env, e),
        Value::Closure(e, 1, c_env) => eval_expr(&c_env.extended(arg), e) ,
        Value::Closure(e, n, c_env) => Ok(Value::Closure(e, n-1, c_env.extended(arg))),
        val => Ok(val)
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
            info!("Matching var pattern");
            let new_env = env.extended(Value::Closure(e.clone(), 0, env.clone()));
            return Ok(Some(new_env));
        },
        Pattern::Wildcard => {
            info!("Matching wildcard pattern");
            return Ok(Some(env.clone()));

        },
        Pattern::EmptyList => {
            info!("Matching [] pattern");
            let v = eval_expr(env, e.clone())?;
            if let Value::EmptyList = v {
                return Ok(Some(env.clone()));
            }
        },
        Pattern::Literal(l1) => {
            info!("Matching literal pattern");
            let v = eval_expr(env, e.clone())?;
            if let Value::Literal(l2) = v {
                if *l1 == l2 {
                    return Ok(Some(env.clone()));
                }
            }
        },
        Pattern::List(p, ps) => {
            info!("Recognized list pattern");
            if let Value::List(head, tail) = eval_expr(env, e)? {
                info!("Evaluated List with head: {}", &head);
                if let Some(new_env) = matches_value(env, p, &head)? {
                    return matches_value(&new_env, ps, &tail);
                }
            }
        },
        Pattern::Tuple(ps) => {
            info!("Matching tuple pattern");
            let vs = eval_expr(env, e)?;
            return matches_value(env, &Pattern::Tuple(ps.clone()), &vs);
        },
    }
    Ok(None)
}

fn matches_value(env: &Env, p: &Pattern, v: &Value) -> RTResult<Option<Env>> {
    info!("Matching val {} on pattern {}", v, p);
    Ok(match (p, v) {
        (Pattern::Wildcard, _) => Some(env.clone()),
        (Pattern::Literal(l2), Value::Literal(l1)) => if l1 == l2 {Some(env.clone())} else {None},
        (Pattern::Var, v) => Some(env.extended(v.clone())),
        (Pattern::EmptyList, Value::EmptyList) => Some(env.clone()),
        (Pattern::List(_, _), Value::List(_, _)) => todo!(),
        (p, Value::Closure(e, 0, inner_env)) => {
            let v = eval_expr(inner_env, e.clone())?;
            matches_value(env, p, &v)?
        },
        (Pattern::Tuple(ps), Value::Tuple(vs)) => {
            info!("Got here");
            let mut curr = env.clone();
            for (p, v) in zip(ps, vs) {
                match matches_value(env, p, v)? {
                    Some(new_env) => {curr = new_env},
                    None => { return Ok(None); },
                }
            }
            Some(curr)
        }
        (_, _) => None,
    })



}
