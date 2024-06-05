use std::collections::VecDeque;

use ast::AstNode;
use parse::Rule;
pub mod ast;
pub(crate) mod macros;
pub(crate) mod parse;

pub fn parse(source: &str) -> Result<VecDeque<AstNode>, pest::error::Error<Rule>> {
    parse::build_ast(source)
}
