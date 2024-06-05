use std::collections::VecDeque;

use ast::AstNode;
use error::ParsingError;
pub mod ast;
pub mod error;
pub(crate) mod macros;
pub(crate) mod parse;

pub fn parse(source: &str) -> Result<VecDeque<AstNode>, ParsingError> {
    parse::build_ast(source)
}
