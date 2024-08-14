type DebrujinIndex = usize;

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

#[derive(Debug, Clone, PartialEq)]
pub enum List<T> {
    Some(Box<T>, Box<List<T>>),
    Empty,
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

type IntType = i64;

#[derive(Debug, Clone)]
pub enum Decl {
    TypeAlias(String, Type),
    TypeSignature(String, Type),
    FunDecl(String, Expr),
    SExpr(Expr),
    EndOfInstruction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    TypeName(String),
    Function(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Int,
    Bool,
    Char,
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Symbol(String),
    Var(DebrujinIndex),
    Application(Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Case(Box<Expr>, Vec<(Pattern, Expr)>),
    BinOp(Box<Expr>, Op, Box<Expr>),
    Tuple(Vec<Expr>),
    List(List<Expr>),
    Literal(Literal),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(IntType),
    Bool(bool),
    String(String),
    Char(char),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Literal),
    Var,
    EmptyList,
    List(Box<Pattern>, Box<Pattern>),
}

#[derive(Debug, Clone, PartialEq)]
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
