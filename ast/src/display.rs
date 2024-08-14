use crate::ast::Type;
use core::fmt::{Display, Formatter, Result};

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
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
            Type::String => write!(f, "String"),
        }
    }
}
