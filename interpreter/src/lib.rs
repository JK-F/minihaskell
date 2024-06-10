use std::collections::VecDeque;

use interpreter::Interpreter;
use parser::ast::AstNode;
pub mod error;
pub mod env;
pub(crate) mod interpreter;

pub fn interpret(program: VecDeque<AstNode>) {
    let interpreter = Interpreter::new(program);
}
