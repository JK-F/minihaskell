use std::collections::{HashMap, VecDeque};


use log::info;
use parser::ast::AstNode::{Decl, EndOfInstruction, SExpr, TypeAlias, TypeSignature};
use parser::ast::Expr::{Symbol, Var, Application, Tuple, If, BinOp};
use parser::ast::Op::{Add, Sub, Mul, Div, Eq, Neq, Lt, Gt, Le, Ge, And, Or};
use parser::ast::{AstNode, Expr, Type, Value};

use crate::error::RunTimeError;

pub struct Interpreter {
    program: VecDeque<AstNode>,
    env: Environment,
}

struct Environment {
    map: HashMap<String, (Vec<Expr>, Expr)>
}

impl Environment {
    fn new() -> Environment {
        return Environment { map: HashMap::new() };
    }
    fn store(&mut self, var: String, e: Expr) {
        self.map.insert(var, (vec![], e));
    }

    fn get(&mut self, var: &String) -> Option<(Vec<Expr>, Expr)> {
        self.map.get(var).map(|c| c.clone())
    }
}

impl Iterator for Interpreter {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.program.pop_front() {
            return match node {
                TypeAlias(_, _) => Some(()),
                TypeSignature(_, _) => Some(()),
                Decl(var, e) => Some(self.env.store(var, e)),
                SExpr(e) => {
                    let res = self.eval_expr(e.clone(), &mut vec![]);
                    match res {
                        Ok(val) => println!("> {}", val),
                        Err(err) => println!("Error at while running: {}", err),
                    }
                    Some(())
                }
                EndOfInstruction => None,
            };
        }
        None
    }
}

type Stack = Vec<(Expr, usize)>;

impl Interpreter {
    pub fn new(program: VecDeque<AstNode>) -> Interpreter {
        Interpreter {
            program,
            env: Environment::new(),
        }
    }

    fn eval_expr(&mut self, expr: Expr, stack: &mut Stack) -> Result<Value, RunTimeError> {
        info!("Interpreting expression: {:?}, stack:{:?}", expr, stack);
        match expr {
            Expr::Value(Value::Function(_args, expr)) => {
                self.eval_expr(*expr, stack)
            },
            Expr::Value(v) => Ok(v),
            Symbol(name) => {
                let (mut args, expr) = self.env.get(&name).ok_or(RunTimeError::SymbolNotFound(name))?;
                let mut new_stack = stack.clone();
                let appended_at = new_stack.len();
                for arg in args {
                    new_stack.push((arg, appended_at));
                }
                self.eval_expr(expr, &mut new_stack)
            },
            Var(d) => {
                let pos = stack.len() - 1 - d;
                let (e, eval_size) = stack[pos].clone();
                let val = self.eval_expr(e, &mut stack[0..eval_size].to_vec())?;
                let _ = std::mem::replace(&mut stack[pos], (Expr::Value(val.clone()), pos));
                Ok(val)
            }
            Tuple(_vals) => {
                todo!()
            }
            If(test, ethen, eelse) => {
                let tv = self.eval_bool(test, stack)?;
                if tv {
                    return self.eval_expr(*ethen, stack);
                }
                return self.eval_expr(*eelse, stack);
            }
            BinOp(l, op, r) => match op {
                Add => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Int(lv + rv))
                }
                Sub => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Int(lv - rv))
                }
                Mul => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Int(lv * rv))
                }
                Div => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Int(lv / rv))
                }
                Eq => {
                    let lv = self.eval_expr(*l, stack)?;
                    let rv = self.eval_expr(*r, stack)?;
                    Ok(Value::Bool(lv == rv))
                }
                Neq => {
                    let lv = self.eval_expr(*l, stack)?;
                    let rv = self.eval_expr(*r, stack)?;
                    Ok(Value::Bool(lv != rv))
                }
                Lt => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Bool(lv < rv))
                }
                Gt => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Bool(lv > rv))
                }
                Le => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Bool(lv <= rv))
                }
                Ge => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Bool(lv >= rv))
                }
                And => {
                    let lv = self.eval_bool(l, stack)?;
                    let rv = self.eval_bool(r, stack)?;
                    Ok(Value::Bool(lv && rv))
                }
                Or => {
                    let lv = self.eval_bool(l, stack)?;
                    let rv = self.eval_bool(r, stack)?;
                    Ok(Value::Bool(lv || rv))
                }
            },
            Application(f, e) => {
                stack.push((*e, stack.len()));
                self.eval_expr(*f, stack)
            },
        }
    }

    fn eval_int(&mut self, expr: Box<Expr>, stack: &mut Stack) -> Result<i64, RunTimeError> {
        return match self.eval_expr(*expr, stack)? {
            Value::Int(v) => Ok(v),
            _ => Err(RunTimeError::TypeError(
                Type::TypeName("Int".to_string()),
                Type::TypeName("Later".to_string()),
            )),
        };
    }
    fn eval_bool(&mut self, expr: Box<Expr>, stack: &mut Stack) -> Result<bool, RunTimeError> {
        return match self.eval_expr(*expr, stack)? {
            Value::Bool(v) => Ok(v),
            _ => Err(RunTimeError::TypeError(
                Type::TypeName("Bool".to_string()),
                Type::TypeName("Later".to_string()),
            )),
        };
    }
}
