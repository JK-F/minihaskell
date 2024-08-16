use ast::ast::Decl;
use eval::RTResult;


mod eval;
mod error;
mod env;
mod value;

pub fn eval(program: Vec<Decl>) -> RTResult<()> {
    eval::eval(program)
}

