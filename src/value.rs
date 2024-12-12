use std::fmt::{self};

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
        match self {
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::String(s) => write!(f, "{}", s),
            Value::Function(v) => write!(f, "{:?}", v),
            Value::SystemException(v) => write!(f, "{}", v),
            _ => Err(fmt::Error),
        }
    }
}
