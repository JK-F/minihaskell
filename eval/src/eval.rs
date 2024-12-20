use std::iter::zip;

use crate::env::Env;
use crate::value::Value;
use ast::ast::Decl::*;
use ast::ast::Op::*;
use ast::ast::{Decl, Expr, List, Literal, Pattern, Type};
use log::info;

use crate::error::RunTimeError;

pub type RTResult<T> = Result<T, RunTimeError>;

pub fn eval(program: Vec<Decl>) -> RTResult<()> {
    let mut env = Env::new();
    for decl in program {
        eval_decl(&mut env, decl)?;
    }
    Ok(())
}

fn eval_decl(env: &mut Env, decl: Decl) -> RTResult<()> {
    match decl {
        TypeAlias(_, _) => Ok(()),
        TypeSignature(_, _) => Ok(()),
        FunDecl(name, args, e) => {
            let closure = Value::Closure(e, args, env.clone());
            info!("Storing {}: {} to env", name, closure);
            env.add_function(name, closure);
            Ok(())
        }
        SExpr(e) => {
            let v = eval_expr(env, e)?;
            let v = force_eval(v)?;
            println!("> {}", v);
            Ok(())
        }
        EndOfInstruction => Ok(()),
    }
}

fn force_eval(v: Value) -> RTResult<Value> {
    match v {
        Value::Tuple(vs) => Ok(Value::Tuple(
            vs.into_iter()
                .map(|v| force_eval(v))
                .collect::<RTResult<Vec<_>>>()?,
        )),
        Value::Closure(e, vars, mut env) => {
            if vars.iter().all(|var| env.contains(var)) {
                force_eval(eval_expr(&mut env, e)?)
            } else {
                Ok(Value::Closure(e, vars, env))
            }
        }
        Value::List(x, xs) => Ok(Value::List(
            Box::new(force_eval(*x)?),
            Box::new(force_eval(*xs)?),
        )),
        x => Ok(x),
    }
}

fn eval_expr(env: &mut Env, expr: Expr) -> RTResult<Value> {
    info!("Interpreting Expression {} with env:", expr);
    info!("{env:?}");
    match expr {
        Expr::Var(name) => {
            let v = env.get(&name)?;
            let v = handle_closure(v)?;
            env.update_value(&name, v.clone());
            Ok(v)
        }
        Expr::Literal(l) => Ok(Value::Literal(l)),
        Expr::Tuple(es) => Ok(Value::Tuple(
            es.into_iter()
                .map(|e| Value::Closure(e, vec![], env.clone()))
                .collect(),
        )),
        Expr::If(test, ethen, eelse) => {
            let tv = eval_bool(env, *test)?;
            if tv {
                return eval_expr(env, *ethen);
            }
            return eval_expr(env, *eelse);
        }
        Expr::Lambda(var_name, expr) => Ok(Value::Closure(*expr, vec![var_name], env.clone())),
        Expr::Let(var_name, expr1, expr2) => {
            let var = Value::Closure(*expr1, vec![], env.clone());
            eval_expr(&mut env.extended(var_name, var), *expr2)
        }
        Expr::BinOp(l, op, r) => {
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
                Mod => {
                    let lv = eval_int(env, *l)?;
                    let rv = eval_int(env, *r)?;
                    Ok(Value::Literal(Literal::Int(lv % rv)))
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
                            Value::Closure(expr, _, mut env) => curr = eval_expr(&mut env, expr)?,
                            tail => curr = tail,
                        }
                    }
                    let big_list = elements
                        .into_iter()
                        .rev()
                        .fold(Box::new(rv), |acc, x| Box::new(Value::List(x, acc)));
                    Ok(*big_list)
                }
                Cons => Ok(Value::List(
                    Box::new(Value::Closure(*l, vec![], env.clone())),
                    Box::new(Value::Closure(*r, vec![], env.clone())),
                )),
            };
        }
        Expr::Application(f, e) => {
            let e_closure = Value::Closure(*e, vec![], env.clone());
            match eval_expr(env, *f)? {
                // Typechecker should ensure f is a function with arity > 0
                Value::Closure(e, v, c_env) => {
                    let mut v = v;
                    let name = v.pop().unwrap();
                    info!("Pushing {}: {} to env", name, e);
                    let mut new_env = c_env.extended(name, e_closure);
                    match &v[..] {
                        [] => eval_expr(&mut new_env, e),
                        _ => Ok(Value::Closure(e, v, new_env)),
                    }
                }
                _ => unreachable!(),
            }
        }
        Expr::Case(e, cases) => {
            let (body, mut new_env) = pattern_match_expr(env, *e, &cases)?;
            eval_expr(&mut new_env, body)
        }
        Expr::List(ls) => match ls {
            List::Some(head, tail) => Ok(Value::List(
                Box::new(Value::Closure(*head, vec![], env.clone())),
                Box::new(Value::Closure(Expr::List(*tail), vec![], env.clone())),
            )),
            List::Empty => Ok(Value::EmptyList),
        },
        Expr::Range(start, step, stop) => {
            let start = eval_int(env, *start)?;
            let step = eval_int(env, *step)?;
            let stop = stop.map(|stop| eval_int(env, *stop)).transpose()?;
            if let Some(stop) = stop {
                if start > stop {
                    return Ok(Value::EmptyList);
                }
            }
            return Ok(Value::List(
                Box::new(Value::Literal(Literal::Int(start))),
                Box::new(Value::Closure(
                    Expr::Range(
                        Box::new(Expr::Literal(Literal::Int(start + step))),
                        Box::new(Expr::Literal(Literal::Int(step))),
                        stop.map(|x| Box::new(Expr::Literal(Literal::Int(x)))),
                    ),
                    vec![],
                    env.clone(),
                )),
            ));
        }
    }
}

fn handle_closure(v: Value) -> RTResult<Value> {
    match v {
        Value::Closure(e, args, mut c_env) => {
            if args.is_empty() {
                eval_expr(&mut c_env, e)
            } else {
                Ok(Value::Closure(e, args, c_env))
            }
        }
        val => Ok(val),
    }
}

fn eval_int(env: &mut Env, expr: Expr) -> RTResult<i64> {
    info!("Evaluating {} to int", &expr);
    match eval_expr(env, expr)? {
        Value::Literal(Literal::Int(v)) => Ok(v),
        _ => Err(RunTimeError::TypeError(
            Type::TypeVariable("Int".to_string()),
            Type::TypeVariable("Later".to_string()),
        )),
    }
}
fn eval_bool(env: &mut Env, expr: Expr) -> RTResult<bool> {
    let v = handle_closure(eval_expr(env, expr)?)?;
    match v {
        Value::Literal(Literal::Bool(v)) => Ok(v),
        _ => Err(RunTimeError::TypeError(
            Type::TypeVariable("Bool".to_string()),
            Type::TypeVariable("Later".to_string()),
        )),
    }
}

fn pattern_match_expr<'a>(
    env: &mut Env,
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

fn match_pattern(env: &mut Env, p: &Pattern, e: Expr) -> RTResult<Option<Env>> {
    info!("Matching pattern: {}", p);
    match p {
        Pattern::Var(name) => {
            let new_env =
                env.extended(name.clone(), Value::Closure(e.clone(), vec![], env.clone()));
            return Ok(Some(new_env));
        }
        Pattern::Wildcard => {
            return Ok(Some(env.clone()));
        }
        Pattern::EmptyList => {
            let v = eval_expr(env, e.clone())?;
            if let Value::EmptyList = v {
                return Ok(Some(env.clone()));
            }
        }
        Pattern::Literal(l1) => {
            let v = eval_expr(env, e.clone())?;
            if let Value::Literal(l2) = v {
                if *l1 == l2 {
                    return Ok(Some(env.clone()));
                }
            }
        }
        Pattern::List(_, _) => {
            let v = eval_expr(env, e)?;
            return matches_value(env, p, &v);
        }
        Pattern::FakeTuple(ps) => {
            return match_pattern(env, &Pattern::Tuple(ps.to_vec()), e);
        }
        Pattern::Tuple(ps) => {
            let vs = eval_expr(env, e)?;
            return matches_value(env, &Pattern::Tuple(ps.clone()), &vs);
        }
    }
    Ok(None)
}

fn matches_value(env: &mut Env, p: &Pattern, v: &Value) -> RTResult<Option<Env>> {
    info!("Matching val {} on pattern {}", v, p);
    Ok(match (p, v) {
        (Pattern::Wildcard, _) => Some(env.clone()),
        (Pattern::Literal(l2), Value::Literal(l1)) => l1.eq(l2).then_some(env.clone()),
        (Pattern::Var(name), v) => Some(env.extended(name.clone(), v.clone())),
        (Pattern::EmptyList, Value::EmptyList) => Some(env.clone()),
        (Pattern::List(p1, p2), Value::List(v1, v2)) => match matches_value(env, p1, v1)? {
            Some(mut new_env) => matches_value(&mut new_env, p2, v2)?,
            None => None,
        },
        (p, Value::Closure(e, v, inner_env)) => {
            if v.len() != 0 {
                return Ok(None);
            }
            let v = eval_expr(&mut inner_env.clone(), e.clone())?;
            matches_value(env, p, &v)?
        }
        (Pattern::Tuple(ps), Value::Tuple(vs)) => {
            let mut curr = env.clone();
            for (p, v) in zip(ps, vs) {
                match matches_value(&mut curr, p, v)? {
                    Some(new_env) => curr = new_env,
                    None => {
                        return Ok(None);
                    }
                }
            }
            Some(curr)
        }
        (Pattern::FakeTuple(ps), _) => matches_value(env, &Pattern::Tuple(ps.to_vec()), v)?,
        (_, _) => None,
    })
}
