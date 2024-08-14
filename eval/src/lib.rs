use eval::Interpreter;
use ast::ast::Decl;
mod error;
mod env;
mod value;
mod eval;

pub fn interpret(program: Vec<Decl>) {
    let interpreter = Interpreter::new(program);
    for _ in interpreter {
    }
    println!("");
}
