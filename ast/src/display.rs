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
            Type::TypeName(name) => write!(f, "{}", name),
            Type::Function(left, right) => write!(f, "{} -> {}", left, right),
            Type::Tuple(vals) => {
                write!(f, "(")?; for val in vals {
                    write!(f, "{},", val)?;
                }
                write!(f, ")")?;
                Ok(())
            }
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
            Literal::Bool(val) => write!(f, "{}", val),
            Literal::String(val) => write!(f, "{}", val),
            Literal::Char(val) => write!(f, "{}", val),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Expr::Symbol(name) => write!(f, "{}", name),
            Expr::Var(idx) => write!(f, "#{}", idx ),
            Expr::Application(fun, arg) => write!(f, "({} {})", fun, arg),
            Expr::If(a, b, c) =>  write!(f, "if {}, then {}, else {}", a, b, c),
            Expr::Tuple(es) => {
                write!(f, "(")?;
                fmt_vec(f, es)?;
                write!(f, ")")
            },
            Expr::List(ls) => write!(f, "{}", ls),
            Expr::BinOp(l, op, r) => write!(f, "({} {} {})", l, op, r),
            Expr::Literal(l) => write!(f, "{}", l),
            Expr::Case(e, cases) => {
                write!(f, "match {} with {{", e)?;
                for (p, body) in cases {
                    write!(f, "{} -> {};", p, body)?;

                }
                write!(f, "}}")
            },
        }
    }
}

impl Display for Pattern { 
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Pattern::Literal(l) => write!(f, "{}", l),
            Pattern::Var => write!(f, "var"),
            Pattern::EmptyList => write!(f, "[]"),
            Pattern::Wildcard => write!(f, "_"),
            Pattern::Tuple(ps) => {
                write!(f, "(")?;
                fmt_vec(f, ps)?;
                write!(f, ")")
            }
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
            Op::Eq  => write!(f, "=="),
            Op::Neq => write!(f, "!="),
            Op::Lt  => write!(f, "<"),
            Op::Gt  => write!(f, ">"),
            Op::Le  => write!(f, "<="),
            Op::Ge  => write!(f, ">="),
            Op::And => write!(f, "&&"),
            Op::Or  => write!(f, "||"),
        }
    }
}

impl <T> Display for List<T>
    where T: Display {
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

fn fmt_vec<T>(f: &mut Formatter<'_>, v: &Vec<T>) -> Result 
    where T: Display {
    match &v[..] {
        [] => Ok(()),
        [xs @ .., last] => {
            for x in xs {
                write!(f, "{},", x)?;
            }
            write!(f, "{}", last)
        }
    }
}
