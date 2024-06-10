use parser::ast::{Expr, Pattern};

use crate::error::RunTimeError;
use crate::error::RunTimeError::*;

pub struct Environment {
    var_stack: Vec<Cases>,
}

impl Environment {
    pub fn store(&mut self, index: usize, args: Vec<Pattern>, expr: Expr) {
    }

    pub fn eval(&mut self, index: usize) -> Result<&Expr, RunTimeError> {
        let var_count = self.var_stack.len();
        let cases = self.var_stack.get(var_count - index)
            .ok_or(OutOfScope)?;




        


        Ok(&Expr::Literal(parser::ast::Literal::Int(0)))
    }
}

pub struct Cases;
