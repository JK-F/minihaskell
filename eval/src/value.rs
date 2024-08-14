use ast::ast::{Expr, List};
use std::fmt::{Formatter, Result, Display};

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Char(char),
    Tuple(Vec<Value>),
    String(String),
    Function0(Box<Expr>),
    Function1(Box<Expr>),
    List(List<Value>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(l), Value::Int(r)) => l == r,
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Char(l), Value::Char(r)) => l == r,
            (Value::String(l), Value::String(r)) => l == r,
            _ => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Value::Int(val) => write!(f, "{}", val),
            Value::Bool(val) => write!(f, "{}", val),
            Value::Char(val) => write!(f, "{}", val),
            Value::Tuple(vs) => vs.into_iter().map(|v| write!(f, "{}", v)).collect(),
            Value::List(ls) => {
                let mut vec = vec![];
                let mut curr = ls;
                while let List::Some(v, vs) = curr {
                    vec.push(v);
                    curr = vs;
                }
                let vec: Vec<String> = vec.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", vec.join(", "))
            }
            Value::String(val) => write!(f, "{}", val),
            Value::Function0(_) => write!(f, "fun[]"),
            Value::Function1(_) => write!(f, "fun[#0]"),
        }
    }
}
