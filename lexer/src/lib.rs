pub mod ast;
pub mod macros;
pub mod parse;

pub fn parse(source: &str) -> Result<ast::Program, pest::error::Error<parse::Rule>> {
    return parse::build_ast(source);
}
