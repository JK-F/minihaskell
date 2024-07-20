use ast::AstNode;
use error::ParsingError;
pub mod ast;
pub mod error;
pub(crate) mod macros;
pub(crate) mod parse;

pub fn parse(source: &str) -> Result<Vec<AstNode>, ParsingError> {
    let mut res =parse::build_ast(source.to_string())?;
    res.reverse();
    Ok(res)
}
