use std::collections::HashMap;

use crate::value::Value;
use log::info;
use ast::ast::Decl::{FunDecl, EndOfInstruction, SExpr, TypeAlias, TypeSignature};
use ast::ast::Expr::{Application, BinOp, If, Symbol, Tuple, Var};
use ast::ast::Op::{Add, And, Div, Eq, Ge, Gt, Le, Lt, Mul, Neq, Or, Sub};
use ast::ast::{Decl, Expr, List, Literal, Pattern, Type};

use crate::error::RunTimeError;

pub struct Interpreter {
    program: Vec<Decl>,
    env: Environment,
}

struct Environment {
    map: HashMap<String, Expr>,
}

impl Environment {
    fn new() -> Environment {
        return Environment {
            map: HashMap::new(),
        };
    }
    fn store(&mut self, var: String, e: Expr) {
        self.map.insert(var, e);
    }

    fn get(&mut self, var: &String) -> Option<Expr> {
        self.map.get(var).map(|c| c.clone())
    }
}

impl Iterator for Interpreter {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.program.pop() {
            return match node {
                TypeAlias(_, _) => Some(()),
                TypeSignature(_, _) => Some(()),
                FunDecl(var, e) => Some(self.env.store(var, e)),
                SExpr(e) => {
                    let res = self.eval_expr(e.clone(), &mut vec![]);
                    match res {
                        Ok(val) => println!("> {}", val),
                        Err(err) => println!("Error while running: {}", err),
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
    pub fn new(program: Vec<Decl>) -> Interpreter {
        Interpreter {
            program,
            env: Environment::new(),
        }
    }

    fn eval_expr(&mut self, expr: Expr, stack: &mut Stack) -> Result<Value, RunTimeError> {
        info!("Interpreting with current stack {:?}", stack);
        //        if stack.len() > 10 {
        //            panic!("yeah fuck");
        //        }
        match expr {
            Expr::Value(v) => {
                info!("Interpreting Value {:?}", v);
                Ok(v)
            }
            Symbol(name) => {
                info!("Interpreting function of name {name}");
                let expr = self
                    .env
                    .get(&name)
                    .ok_or(RunTimeError::SymbolNotFound(name))?;
                let mut new_stack = stack.clone();
                self.eval_expr(expr, &mut new_stack)
            }
            Var(d) => {
                info!("Interpreting variable #{d}");
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
                info!(
                    "Interpreting if {:?} then {:?} else {:?} ",
                    test, ethen, eelse
                );
                let tv = self.eval_bool(test, stack)?;
                if tv {
                    return self.eval_expr(*ethen, stack);
                }
                return self.eval_expr(*eelse, stack);
            }
            BinOp(l, op, r) => {
                info!("interpreting {:?} {:?} {:?}", l, op, r);
                return match op {
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
                };
            }
            Application(f, e) => {
                info!("Application of {:?} to {:?}", f, e);
                stack.push((*e, stack.len()));
                let fun = self.eval_expr(*f, stack);
                stack.pop();
                fun
            }
            Expr::Case(e, cases) => {
                let body = self.pattern_match_expr(stack, *e, &cases[..])?;
                self.eval_expr(body.clone(), stack)
            }
            Expr::List(ls) => {
                Ok(Value::List(self.eval_list(ls, stack)?))
            }
        }
    }
    fn eval_list(&mut self, ls: List<Expr>, stack: &mut Stack) -> Result<List<Value>, RunTimeError> {
        match ls {
            List::Some(e, es) => {
                let v = Box::new(self.eval_expr(*e, stack)?);
                let vs = Box::new(self.eval_list(*es, stack)?);
                Ok(List::Some(v, vs))
            },
            List::Empty => Ok(List::Empty)
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

    fn pattern_match_expr<'a>(
        &mut self,
        stack: &mut Stack,
        e: Expr,
        cases: &'a [(Pattern, Expr)],
    ) -> Result<&'a Expr, RunTimeError> {
        info!("Correctly entering pattern matching");
        dbg!(&e);
        if let [(p, body)] = cases {
            if matches!(p, Pattern::Var) {
                stack.push((e, stack.len()));
                return Ok(body);
            }
        }
        if let Expr::List(ls) = &e {
            info!("Correctly entering");
            for (pattern, body) in cases {
                if self.matches_list_expr(stack, ls, pattern)? {
                    return Ok(body)
                }
            }
            info!("Somehow exiting");
        }
        let v = self.eval_expr(e, stack)?;
        for (pattern, body) in cases {
            if matches_value(&v, pattern) {
                return Ok(body);
            }
        }
        Err(RunTimeError::NonExhaustivePattern)
    }

    fn matches_list_expr(&mut self, stack: &mut Stack, ls: &List<Expr>, p: &Pattern) -> Result<bool, RunTimeError> {
        match (ls, p) {
            (_, Pattern::Var) => { 
                stack.push((Expr::List(ls.clone()), stack.len()));
                Ok(true) 
            },
            (List::Empty, Pattern::EmptyList) => Ok(true),
            (List::Some(e, es), Pattern::List(p1, p2)) => {
                let v = self.eval_expr(*e.clone(), stack)?;
                let b = matches_value(&v, p1);
                if b && matches!(*p1.clone(), Pattern::Var) {
                    stack.push((Expr::Value(v), stack.len()));
                }

                Ok(b && self.matches_list_expr(stack, es, p2)?)
            },
            (_, _) => Ok(false)
        }
    }

}

fn matches_value(arg: &Value, p: &Pattern) -> bool {
    match (arg, p) {
        (Value::Int(i1),    Pattern::Literal(Literal::Int(i2))) => i1 == i2,
        (Value::Bool(b1),   Pattern::Literal(Literal::Bool(b2))) => b1 == b2,
        (Value::Char(c1),   Pattern::Literal(Literal::Char(c2))) => c1 == c2,
        (Value::String(s1), Pattern::Literal(Literal::String(s2))) => s1 == s2,
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
