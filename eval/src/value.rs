use ast::ast::{Expr, List, Literal};
use std::fmt::{Formatter, Result, Display};

use crate::env::Env;

#[derive(Debug, Clone)]
pub enum Value {
    Literal(Literal),
    Tuple(Vec<Value>),
    Closure(Expr, Env),
    List(List<Value>),
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
            Value::Tuple(vs) => vs.into_iter().map(|v| write!(f, "{}", v)).collect(),
            Value::List(ls) => write!(f, "{}", ls),
            Value::Closure(e, _) => write!(f, "\\_ -> {}", e),

        }
    }
}
