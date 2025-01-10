use crate::builtin::*;
use crate::value::{ControlFlowValue, Exception, Function, Value};
use std::collections::HashMap;

pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            scopes: vec![HashMap::new()],
        }
    }

    // remove a scope to the environment
    pub fn pop(&mut self) -> &mut Self {
        self.scopes.pop();
        self
    }
    // add a scope to the environment
    pub fn push(&mut self) -> &mut Self {
        self.scopes.push(HashMap::new());
        self
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

    pub fn assign(&mut self, id: &str, value: Value) -> Result<(), ControlFlowValue> {
        let mut success = false;

        for scope in self.scopes.iter_mut().rev() {
            if scope.get(id).is_some() {
                scope.insert(id.to_string(), value);
                success = true;
                break;
            }
        }

        if success {
            Ok(())
        } else {
            Err(ControlFlowValue::Exception(Exception::UndeclaredIdentifier))
        }
    }

    pub fn declare(&mut self, id: String, value: Value) -> &mut Self {
        self.scopes.last_mut().unwrap().insert(id, value);
        self
    }

    fn declare_builtin(
        &mut self,
        id: String,
        function: fn(Vec<Value>) -> Result<Value, ControlFlowValue>,
    ) -> &mut Self {
        self.declare(id, Value::Function(Function::Builtin(function)));
        self
    }
}

impl Default for Environment {
    fn default() -> Self {
        let mut env = Environment::new();
        env.declare_builtin("printLn".to_string(), print_ln)
            .declare_builtin("toString".to_string(), to_string);
        env
    }
}
