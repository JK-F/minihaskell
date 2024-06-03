pub type Program<'a> = Vec<AstNode<'a>>;

#[derive(Debug)]
pub enum AstNode<'a> {
    TypeAlias(&'a str, Type<'a>),
    TypeSignature(&'a str, Type<'a>),
    Decl(&'a str, Vec<Pattern<'a>>, Expr<'a>),
}

#[derive(Debug)]
pub enum Pattern<'a> {
    Literal(Literal<'a>),
    Var(&'a str),
}

#[derive(Debug)]
pub enum Type<'a> {
    TypeName(&'a str),
    Function(Box<Type<'a>>, Box<Type<'a>>),
    Tuple(Vec<Type<'a>>),
}

#[derive(Debug)]
pub enum Expr<'a> {
    Var(&'a str),
    Application(&'a str, Vec<Expr<'a>>),
    Literal(Literal<'a>),
    Tuple(Vec<Expr<'a>>),
    If(Box<Expr<'a>>, Box<Expr<'a>>, Box<Expr<'a>>),
    BinOp(Box<Expr<'a>>, Op, Box<Expr<'a>>),
}

#[derive(Debug)]
pub enum Literal<'a> {
    Int(i64),
    Bool(bool),
    Char(char),
    String(&'a str),
}

#[derive(Debug)]
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
