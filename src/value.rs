use std::fmt::{self};

use crate::parser::DefinedFunction;
use strum::Display;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Function {
    Defined(DefinedFunction),
    Builtin(fn(Vec<Value>) -> Result<Value, ControlFlowValue>),
}

#[derive(Debug, Clone)]
pub enum Value {
    Function(Function),
    String(String),
    Int(i64),
    Bool(bool),
    Null,
}

#[derive(Debug, Display)]
pub enum Exception {
    WrongNumberOfArguments,
    NestedReturns,
    UndeclaredIdentifier,
    CalledValueIsNotFunction,
    ValueIsWrongType,
}

#[derive(Error, Debug, Display)]
pub enum ControlFlowValue {
    Exception(Exception),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::String(s) => write!(f, "{}", s),
            Value::Function(v) => write!(f, "{:?}", v),
            Value::Null => write!(f, "null"),
        }
    }
}

impl Value {
    pub fn into_int(&self) -> Result<&i64, ControlFlowValue> {
        match self {
            Value::Int(v) => Ok(v),
            _ => Err(ControlFlowValue::Exception(Exception::ValueIsWrongType)),
        }
    }

    pub fn into_bool(&self) -> Result<&bool, ControlFlowValue> {
        match self {
            Value::Bool(v) => Ok(v),
            _ => Err(ControlFlowValue::Exception(Exception::ValueIsWrongType)),
        }
    }
}
