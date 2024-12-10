use crate::builtin;
use crate::value::{Function, Value};
use std::collections::HashMap;

pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    // remove a scope to the environment
    pub fn pop(&mut self) {
        self.scopes.pop();
    }
    // add a scope to the environment
    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn get(&self, id: &String) -> Option<Value> {
        for value in self.scopes.iter().rev() {
            match value.get(id) {
                Some(v) => {
                    return Some(v.clone());
                }
                _ => {}
            }
        }

        None
    }

    pub fn declare(&mut self, id: String, value: Value) {
        self.scopes.last_mut().unwrap().insert(id, value);
    }
}

impl Default for Environment {
    fn default() -> Environment {
        let mut environment = Environment::new();
        environment.declare(
            "printLn".to_string(),
            Value::Function(Function::Builtin(builtin::print_ln)),
        );
        environment.declare(
            "toString".to_string(),
            Value::Function(Function::Builtin(builtin::to_string)),
        );
        environment
    }
}
