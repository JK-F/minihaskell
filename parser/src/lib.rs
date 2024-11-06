use ast::ast::Decl;
use error::ParsingError;
use log::info;
mod error;
pub(crate) mod macros;
pub(crate) mod parse;
mod util;

pub fn parse(source: &str) -> Result<Vec<Decl>, ParsingError> {
    let program = parse::build_ast(source.to_string())?;
    info!("Parsed program: ");
    for decl in &program {
        info!("{}", decl);
    }
    Ok(program)
}
