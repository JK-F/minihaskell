use ast::ast::Decl;
use eval::RTResult;

mod env;
mod error;
mod eval;
mod value;

pub fn eval(program: Vec<Decl>) -> RTResult<()> {
    eval::eval(program)
}
