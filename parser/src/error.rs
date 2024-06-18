use thiserror::Error;

use crate::parse::Rule;

#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("symbol {0} is used but never declared")]
    UnknownSymbol(String),
    #[error("Ran into error while parsing {0}")]
    PestError(#[from] pest::error::Error<Rule>),
    #[error("Critical Error in Language Grammar")]
    GrammarError,
}
