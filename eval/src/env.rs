use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use log::info;

use crate::error::RunTimeError;
use crate::value::Value;

type RTResult<T> = Result<T, RunTimeError>;

#[derive(Debug, Clone)]
pub(crate) struct Env {
    functions: Rc<RefCell<HashMap<String, Value>>>,
    env: HashMap<String, Rc<Value>>
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

    pub fn add_function(&mut self, name: String, v: Value) {
        (*self.functions).borrow_mut().insert(name, v);
    }

    pub fn extended(&self, name: String, val: Value) -> Env {
        let functions = self.functions.clone();
        let mut env = self.env.clone();
        env.insert(name, Rc::new(val));
        Env { functions, env }
    }

    pub fn get(&self, name: &String) -> RTResult<Value> {
        let var = self.env.get(name);
        match var {
            Some(val) => Ok((**val).clone()),
            None => {
                match (*self.functions).borrow().get(name) {
                    Some(val) => Ok(val.clone()),
                    None => Err(RunTimeError::VariableNotFound(name.clone())),
                }
            }
        }

    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}
