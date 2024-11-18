use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use crate::error::RunTimeError;
use crate::value::Value;

type RTResult<T> = Result<T, RunTimeError>;

#[derive(Debug, Clone)]
pub(crate) struct Env {
    functions: Rc<RefCell<HashMap<String, Value>>>,
    env: HashMap<String, Rc<Value>>,
}

impl Env {
    pub fn new() -> Env {
        Env {
            functions: Rc::new(RefCell::new(HashMap::new())),
            env: HashMap::new(),
        }
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

    pub fn contains(&self, name: &String) -> bool {
        self.env.contains_key(name) || (*self.functions).borrow().contains_key(name)
    }

    pub fn get(&self, name: &String) -> RTResult<Value> {
        let var = self.env.get(name);
        match var {
            Some(val) => Ok((**val).clone()),
            None => match (*self.functions).borrow().get(name) {
                Some(val) => Ok(val.clone()),
                None => Err(RunTimeError::VariableNotFound(name.clone())),
            },
        }
    }
    pub fn update_value(&mut self, name: &String, val: Value) {
        if self.env.contains_key(name) {
            self.env.insert(name.to_string(), Rc::new(val));
        }
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}
