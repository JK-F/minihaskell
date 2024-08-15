use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

use crate::error::RunTimeError;
use crate::value::Value;

type RTResult<T> = Result<T, RunTimeError>;

#[derive(Debug, Clone)]
pub(crate) struct Env {
    functions: Rc<RefCell<HashMap<String, Value>>>,
    vars: Vec<Value>,
}

impl Env {
    pub fn new() -> Env {
        Env {
            functions: Rc::new(RefCell::new(HashMap::new())),
            vars: vec![],
        }
    }
    pub fn extend_function(&self, name: String, val: Value) {
        (*self.functions).borrow_mut().insert(name, val);
    }

    pub fn extended(&self, val: Value) -> Env {
        Env {
            functions: Rc::clone(&self.functions),
            vars: {
                let mut v = self.vars.clone();
                v.push(val);
                v
            },
        }
    }

    pub fn get_function(&self, name: &String) -> RTResult<Value> {
        self.functions
            .borrow()
            .get(name)
            .cloned()
            .ok_or(RunTimeError::SymbolNotFound(name.clone()))
    }

    pub fn get(&self, idx: usize) -> RTResult<Value> {
        let pos = self.vars.len() - 1 - idx;
        self.vars
            .get(pos)
            .cloned()
            .ok_or(RunTimeError::VariableNotFound(idx))
    }

    pub fn replace(&mut self, idx: usize, val: Value) {
        let pos = self.vars.len() - 1 - idx;
        std::mem::replace(&mut self.vars[pos], val);
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}
