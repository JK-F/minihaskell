use std::collections::HashMap;

use parser::ast::Expr;


struct Env {
    vars: HashMap<String, Expr>
}

impl Env {
    pub fn extend(&mut self, name: String, expr: Expr) {
        self.vars.insert(name, expr);
    }
}
