use ast::ast::{Expr, Literal};
use std::fmt::{Debug, Display, Formatter, Result};

use crate::env::Env;

#[derive(Debug, Clone)]
pub enum Value {
    Literal(Literal),
    Tuple(Vec<Value>),
    Closure(Expr, Vec<String>, Env),
    List(Box<Value>, Box<Value>),
    EmptyList,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Literal(l1), Value::Literal(l2)) => l1 == l2,
            _ => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Value::Literal(l) => write!(f, "{}", l),
            Value::Tuple(vs) => write!(
                f,
                "({})",
                vs.iter()
                    .map(|val| format!("{}", val))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::List(head, tail) => {
                let mut vals = vec![head];
                let mut current = tail;
                while let Value::List(elem, next) = current.as_ref() {
                    vals.push(elem);
                    current = next;
                }
                if !matches!(current.as_ref(), Value::EmptyList) {
                    vals.push(current);
                }

                write!(
                    f,
                    "[{}]",
                    vals.iter()
                        .map(|x| format!("{}", x))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Value::EmptyList => write!(f, "[]"),
            Value::Closure(e, args, _) => write!(f, "Closure[{}]{{ {} }}", args.join(", "), e),
        }
    }
}
