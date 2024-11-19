use crate::ast::Decl;
use crate::ast::Expr;
use crate::ast::List;
use crate::ast::Literal;
use crate::ast::Op;
use crate::ast::Pattern;
use crate::ast::Type;
use core::fmt::{Display, Formatter, Result};

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Type::TypeVariable(name) => write!(f, "{}", name),
            Type::Function(left, right) => write!(f, "({} -> {})", left, right),
            Type::List(t) => write!(f, "[{}]", t),
            Type::Tuple(vals) => write!(
                f,
                "({})",
                vals.into_iter()
                    .map(|val| format!("{}", val))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Type::Int => write!(f, "Int"),
            Type::Bool => write!(f, "Bool"),
            Type::Char => write!(f, "Char"),
            Type::String => write!(f, "String"),
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Literal::Int(val) => write!(f, "{}", val),
            Literal::Bool(val) => write!(f, "{}", if *val {"True"} else {"False"}),
            Literal::String(val) => write!(f, "\"{}\"", val),
            Literal::Char(val) => write!(f, "'{}'", val),
        }
    }
}

impl Display for Decl {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Decl::TypeAlias(type_var, type_expr) => write!(f, "type {} = {}", type_var, type_expr),
            Decl::TypeSignature(var_name, type_expr) => write!(f, "{} :: {}", var_name, type_expr),
            Decl::FunDecl(var_name, args, expr) => {
                write!(f, "{} {}= {}", var_name, args.join(" ") + " ", expr)
            }
            Decl::SExpr(expr) => write!(f, "{}", expr),
            Decl::EndOfInstruction => write!(f, ""),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Expr::Var(name) => write!(f, "{}", name),
            Expr::Application(fun, arg) => write!(f, "({} {})", fun, arg),
            Expr::If(a, b, c) => write!(f, "if {}, then {}, else {}", a, b, c),
            Expr::Tuple(es) => fmt_vec(f, es, "(", ")", ", "),
            Expr::List(ls) => write!(f, "{}", ls),
            Expr::BinOp(l, op, r) => write!(f, "({} {} {})", l, op, r),
            Expr::Literal(l) => write!(f, "{}", l),
            Expr::Case(e, cases) => {
                write!(f, "case {} of {{", e)?;
                for (p, body) in cases {
                    write!(f, "{} -> {}; ", p, body)?;
                }
                write!(f, "}}")
            }
            Expr::Range(start, step, last) => match last {
                Some(last) => write!(f, "[{}, ..{}.., {}] ", start, step, last),
                None => write!(f, "[{}, ..{}..] ", start, step),
            },
            Expr::Lambda(arg, expr) => write!(f, "(\\{} -> {})", arg, expr),
            Expr::Let(x, expr1, expr2) => write!(f, "let {} = {} in {}", x, expr1, expr2),
        }
    }
}

impl Display for Pattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Pattern::Literal(l) => write!(f, "{}", l),
            Pattern::Var(name) => write!(f, "{}", name),
            Pattern::EmptyList => write!(f, "[]"),
            Pattern::Wildcard => write!(f, "_"),
            Pattern::Tuple(ps) => fmt_vec(f, ps, "(", ")", ", "),
            Pattern::FakeTuple(ps) => fmt_vec(f, ps, "", "", " "),
            Pattern::List(p1, p2) => write!(f, "({}:{})", p1, p2),
        }
    }
}

impl Display for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Op::Add => write!(f, "+"),
            Op::Sub => write!(f, "-"),
            Op::Mul => write!(f, "*"),
            Op::Div => write!(f, "/"),
            Op::Mod => write!(f, "`mod`"),
            Op::Eq => write!(f, "=="),
            Op::Neq => write!(f, "/="),
            Op::Lt => write!(f, "<"),
            Op::Gt => write!(f, ">"),
            Op::Le => write!(f, "<="),
            Op::Ge => write!(f, ">="),
            Op::And => write!(f, "&&"),
            Op::Or => write!(f, "||"),
            Op::Append => write!(f, "++"),
            Op::Cons => write!(f, ":"),
        }
    }
}

impl<T> Display for List<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut vec = vec![];
        let mut curr = self;
        while let List::Some(v, vs) = curr {
            vec.push(v);
            curr = vs;
        }
        let vec: Vec<String> = vec.iter().map(|v| format!("{}", v)).collect();
        write!(f, "[{}]", vec.join(", "))
    }
}

fn fmt_vec<T>(f: &mut Formatter<'_>, v: &Vec<T>, open: &str, close: &str, join: &str) -> Result
where
    T: Display,
{
    match &v[..] {
        [] => {
            write!(f, "{}{}", open, close)
        }
        [xs @ .., last] => {
            write!(f, "{}", open)?;
            for x in xs {
                write!(f, "{}{}", x, join)?;
            }
            write!(f, "{}{}", last, close)
        }
    }
}
