use std::collections::{HashMap, VecDeque};

use parser::ast::AstNode::{Decl, EndOfInstruction, SExpr, TypeAlias, TypeSignature};
use parser::ast::Expr::*;
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
    fn store_curried(&mut self, var: String, args: Vec<Expr>, e: Expr) {
        self.map.insert(var, (args, e));
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
                    let res = self.eval_expr(e, &mut vec![]);
                    print!("{:?}", res);
                    Some(())
                }
                EndOfInstruction => None,
            };
        }
        None
    }
}

impl Interpreter {
    pub fn new(program: VecDeque<AstNode>) -> Interpreter {
        Interpreter {
            program,
            env: Environment::new(),
        }
    }

    fn eval_expr(&mut self, expr: Expr, stack: &mut Vec<Expr>) -> Result<Value, RunTimeError> {
        match expr {
            Value(v) => Ok(v),
            Symbol(name) => {
                let (mut args, expr) = self.env.get(&name).ok_or(RunTimeError::SymbolNotFound(name))?;
                let mut new_stack = stack.clone();
                new_stack.append(&mut args);
                self.eval_expr(expr, &mut new_stack)
            },
            Var(d) => {
                let pos = stack.len() - d;
                let e = stack[pos].clone();
                let val = self.eval_expr(e, &mut vec![])?;
                let _ = std::mem::replace(&mut stack[pos], Expr::Value(val.clone()));
                Ok(val)
            }
            Tuple(vals) => {
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
                parser::ast::Op::Add => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Int(lv + rv))
                }
                parser::ast::Op::Sub => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Int(lv - rv))
                }
                parser::ast::Op::Mul => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Int(lv * rv))
                }
                parser::ast::Op::Div => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Int(lv / rv))
                }
                parser::ast::Op::Eq => {
                    let lv = self.eval_expr(*l, stack)?;
                    let rv = self.eval_expr(*r, stack)?;
                    Ok(Value::Bool(lv == rv))
                }
                parser::ast::Op::Neq => {
                    let lv = self.eval_expr(*l, stack)?;
                    let rv = self.eval_expr(*r, stack)?;
                    Ok(Value::Bool(lv != rv))
                }
                parser::ast::Op::Lt => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Bool(lv < rv))
                }
                parser::ast::Op::Gt => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Bool(lv > rv))
                }
                parser::ast::Op::Le => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Bool(lv <= rv))
                }
                parser::ast::Op::Ge => {
                    let lv = self.eval_int(l, stack)?;
                    let rv = self.eval_int(r, stack)?;
                    Ok(Value::Bool(lv >= rv))
                }
                parser::ast::Op::And => {
                    let lv = self.eval_bool(l, stack)?;
                    let rv = self.eval_bool(r, stack)?;
                    Ok(Value::Bool(lv && rv))
                }
                parser::ast::Op::Or => {
                    let lv = self.eval_bool(l, stack)?;
                    let rv = self.eval_bool(r, stack)?;
                    Ok(Value::Bool(lv || rv))
                }
            },
            Application(f, e) => {
                stack.push(*e);
                self.eval_expr(*f, stack)
            },
        }
    }

    fn eval_int(&mut self, l: Box<Expr>, stack: &mut Vec<Expr>) -> Result<i64, RunTimeError> {
        return match self.eval_expr(*l, stack)? {
            Value::Int(v) => Ok(v),
            _ => Err(RunTimeError::TypeError(
                Type::TypeName("Int".to_string()),
                Type::TypeName("Later".to_string()),
            )),
        };
    }
    fn eval_bool(&mut self, l: Box<Expr>, stack: &mut Vec<Expr>) -> Result<bool, RunTimeError> {
        return match self.eval_expr(*l, stack)? {
            Value::Bool(v) => Ok(v),
            _ => Err(RunTimeError::TypeError(
                Type::TypeName("Bool".to_string()),
                Type::TypeName("Later".to_string()),
            )),
        };
    }
}
