use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use log::info;

use crate::error::RunTimeError;
use crate::value::Value;

type RTResult<T> = Result<T, RunTimeError>;

#[derive(Debug, Clone)]
pub(crate) struct Env {
    functions: Rc<RefCell<HashMap<String, Value>>>,
    env: HashMap<String, Value>
}

impl Env {
    pub fn new() -> Env {
        Env {
            functions: Rc::new(RefCell::new(HashMap::new())),
            env: HashMap::new(),
        }
    }

    pub fn debug(&self) {
        for (n, v) in (*self.functions).borrow().iter() {
            info!("{}: {}", n, v);
        };
        for (n, v) in self.env.iter() {
            info!("{}: {}", n, v);
        };
    }

    pub fn add_function(&self, name: String, v: Value) {
        (*self.functions).borrow_mut().insert(name, v);
    }

    pub fn extended(&self, name: String, val: Value) -> Env {
        let mut new_env = self.env.clone();
        new_env.insert(name, val);
        Env { functions: Rc::clone(&self.functions), env: new_env } 
    }

    pub fn get(&self, name: &String) -> RTResult<Value> {
        let var = self.env .get(name).cloned();
        let fun = (*self.functions).borrow().get(name).cloned();
        match var.or(fun) {
            Some(val) => Ok(val),
            None => Err(RunTimeError::VariableNotFound(name.clone())),
        }

    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}
