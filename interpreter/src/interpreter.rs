use std::collections::VecDeque;

use parser::ast::{AstNode, Expr, Pattern};
use parser::ast::AstNode::*;

use crate::error::RunTimeError;

pub struct Interpreter<'a> {
    program: VecDeque<AstNode<'a>>,
    env: Vec<(Vec<Pattern<'a>>, Expr<'a>)>,
}

impl Interpreter<'_> {
    pub fn new(program: VecDeque<AstNode>) -> Interpreter {
        Interpreter { program, env: vec![] }
    }

    pub fn execute(&mut self) -> Result<(), RunTimeError> {
        if let Some(node) = self.program.pop_front() {
            return match node {
                TypeAlias(_, _) => Ok(()),
                TypeSignature(_, _) => Ok(()),
                Decl(f, x, e) => self.decl(f, x, e),
                EndOfInstruction => Ok(())
            }
        }
        Err(RunTimeError::EndOfProgram)
    }

    pub fn decl(
        &mut self,
        f: &str,
        x: Vec<Pattern>,
        e: Expr,
    ) -> Result<(), RunTimeError> {
        
        Ok(())
    }
}
