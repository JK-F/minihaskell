use parser::ast::Type;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TypeCheckingError {
    #[error("Found multiple definitions of '{0}' while typechecking")]
    MultipleDefinitions(String),
    #[error("Found no declarataion for the type of '{0}' but it is used by other functions")]
    NoTypeDeclaration(String),
    #[error("Couldn't match expected type '{0}' with actual type '{0}'")]
    TypeMismatch(Type, Type),
    #[error("Cannot parse this pattern pattern")]
    UnexpectedPattern,
    #[error("Expected Function Type for Function Value")]
    ExpectedFunction,
    #[error("Expected additional argument")]
    UnboundArgument,
    #[error("Could not Infer type of this argument")]
    ArgumentTypeUnknown,
}
