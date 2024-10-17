use ast::ast::Type;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypingError {
    #[error("Cannot unify types {0} and {1} in a list")]
    HetrogenousList(Type, Type),
    #[error("Cannot find type of {0}")]
    UnknownIdentifier(String),
    #[error("Cannot unify types {0} and {1}")]
    CannotUnify(Type, Type),
    #[error("Duplicate Type Variable")]
    DuplicateTypeVariable(String),
}
