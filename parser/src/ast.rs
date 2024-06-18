use core::fmt;

type DebrujinIndex = usize;

#[derive(Debug, Clone)]
pub enum AstNode {
    TypeAlias(String, Type),
    TypeSignature(String, Type),
    Decl(String, Expr),
    SExpr(Expr),
    EndOfInstruction,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Value(Value),
    Var,
}

#[derive(Debug, Clone)]
pub enum Type {
    TypeName(String),
    Function(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::TypeName(name) => write!(f, "{}", name),
            Type::Function(left, right) => write!(f, "{} -> {}", left, right),
            Type::Tuple(vals) => {
                write!(f, "(")?;
                for val in vals{
                    write!(f, "{},", val)?;
                }
                write!(f, ")")?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Symbol(String),
    Var(DebrujinIndex),
    Application(Box<Expr>, Box<Expr>),
    Value(Value),
    Tuple(Vec<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    BinOp(Box<Expr>, Op, Box<Expr>),
}
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Char(char),
    String(String),
    Function(Box<Pattern>, Box<Expr>)
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

#[derive(Debug, Clone)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}
