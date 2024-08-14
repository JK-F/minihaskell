use interpreter::Interpreter;
use ast::ast::AstNode;
mod error;
mod env;
mod value;
pub(crate) mod interpreter;

pub fn interpret(program: Vec<AstNode>) {
    let interpreter = Interpreter::new(program);
    for _ in interpreter {
    }
    println!("");
}
