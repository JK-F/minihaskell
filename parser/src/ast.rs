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

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Value(Value),
    Var,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    TypeName(String),
    Function(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Int,
    Bool,
    Char,
    String
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::TypeName(name) => write!(f, "{}", name),
            Type::Function(left, right) => write!(f, "{} -> {}", left, right),
            Type::Tuple(vals) => {
                write!(f, "(")?;
                for val in vals {
                    write!(f, "{},", val)?;
                }
                write!(f, ")")?;
                Ok(())
            }
            Type::Int => write!(f, "Int"),
            Type::Bool => write!(f, "Bool"),
            Type::Char => write!(f, "Char"),
            Type::String => write!(f, "String")
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
    Function(Vec<(Pattern, Expr)>),
    ConstantFunction(Box<Expr>)
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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(val) => write!(f, "{}", val),
            Value::Bool(val) => write!(f, "{}", val),
            Value::Char(val) => write!(f, "{}", val),
            Value::String(val) => write!(f, "{}", val),
            Value::ConstantFunction(_) => write!(f, "Constant"),
            Value::Function(_) => write!(f, "Function"),
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
