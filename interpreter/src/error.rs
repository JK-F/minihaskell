use parser::ast::Type;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RunTimeError {
    #[error("Symbol {0} was not defined")]
    SymbolNotFound(String),
    #[error("Expected Argument for function but found none")]
    MissingArgument,
    #[error("Expected type {0}, but found {1}")]
    TypeError(Type, Type),
    #[error("Reached end of program")]
    EndOfProgram,
}
