use std::fmt::{self, Write};
use std::rc::Rc;

use crate::parser::DefinedFunction;

#[derive(Debug, Clone)]
pub enum Function {
    Defined(DefinedFunction),
    Builtin(fn(Vec<Value>) -> Value),
}

#[derive(Debug, Clone)]
pub enum Value {
    SystemException(String),
    SystemReturn(Box<Value>),
    Function(Function),
    String(String),
    Int(i64),
    Bool(bool),
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match Value {
            Value::Bool(v) => f.write_str(format!("{}", v).as_str()),
        }
        Ok(())
    }
}
