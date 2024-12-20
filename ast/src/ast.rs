pub type Program = Vec<Decl>;
type IntType = i64;

#[derive(Debug, Clone, PartialEq)]
pub enum List<T> {
    Some(Box<T>, Box<List<T>>),
    Empty,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    TypeAlias(String, Type),
    TypeSignature(String, Type),
    FunDecl(String, Vec<String>, Expr),
    SExpr(Expr),
    EndOfInstruction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    TypeVariable(String),
    Function(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    List(Box<Type>),
    Int,
    Bool,
    Char,
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Var(String),
    Application(Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
    Lambda(String, Box<Expr>),
    Case(Box<Expr>, Vec<(Pattern, Expr)>),
    BinOp(Box<Expr>, Op, Box<Expr>),
    Tuple(Vec<Expr>),
    List(List<Expr>),
    Range(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
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
    Var(String),
    Wildcard,
    EmptyList,
    Tuple(Vec<Pattern>),
    FakeTuple(Vec<Pattern>),
    List(Box<Pattern>, Box<Pattern>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    Append,
    Cons,
}
