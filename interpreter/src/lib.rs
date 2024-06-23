use std::collections::VecDeque;

use interpreter::Interpreter;
use parser::ast::AstNode;
pub mod error;
pub(crate) mod interpreter;

pub fn interpret(program: VecDeque<AstNode>) {
    let interpreter = Interpreter::new(program);
    for _ in interpreter {
    }
    println!("");
}
