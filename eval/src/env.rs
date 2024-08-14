use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

use ast::ast::Expr;


struct Env {
    functions: Rc<RefCell<HashMap<String, Expr>>>,
    vars: Vec<Expr>
}

impl Env {
    pub fn extend(&self, name: String, expr: Expr) -> Env {
        Env {
            functions: Rc::clone(&self.functions),
            vars: {
                let mut v = self.vars.clone();
                v.push(expr);
                v
            }
        }
    }
}
