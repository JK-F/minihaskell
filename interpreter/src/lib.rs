use interpreter::Interpreter;
use parser::ast::AstNode;
mod error;
mod env;
pub(crate) mod interpreter;

pub fn interpret(program: Vec<AstNode>) {
    let interpreter = Interpreter::new(program);
    for _ in interpreter {
    }
    println!("");
}
