use ast::ast::Decl;
use error::ParsingError;
mod error;
pub(crate) mod macros;
pub(crate) mod parse;
mod util;

pub fn parse(source: &str) -> Result<Vec<Decl>, ParsingError> {
    let res = parse::build_ast(source.to_string())?;
    Ok(res)
}
