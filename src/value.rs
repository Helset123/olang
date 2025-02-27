use std::fmt::{self};

use crate::parser::DefinedFunction;
use strum::Display;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Function {
    Defined(DefinedFunction),
    Builtin(fn(Vec<Value>) -> Result<Value, ControlFlowValue>),
}

// FIXME: this implementation is pure bullshit
impl PartialEq for Function {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

impl Eq for Function {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Value {
    Function(Function),
    String(String),
    Int(i64),
    Bool(bool),
    List(Vec<Value>),
    Null,
}

#[derive(Debug, Display, PartialEq)]
pub enum Exception {
    WrongNumberOfArguments,
    NestedReturns,
    UndeclaredIdentifier,
    CalledValueIsNotFunction,
    ValueIsWrongType,
    ExponentiationOverflowed,
    IndexOutOfRange,
    Custom(String),
}

#[derive(Error, Debug, Display)]
pub enum ControlFlowValue {
    Exception(Exception),
    Continue,
    Break,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::String(s) => write!(f, "{}", s),
            Value::Function(v) => write!(f, "{:?}", v),
            Value::Null => write!(f, "null"),
            Value::List(list) => {
                write!(f, "[")?;
                for (i, value) in list.iter().enumerate() {
                    write!(f, "{}", value)?;
                    if i != list.len() - 1 {
                        write!(f, " ")?;
                    }
                }
                write!(f, "]")
            }
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

    pub fn into_str(&self) -> Result<&str, ControlFlowValue> {
        match self {
            Value::String(v) => Ok(v),
            _ => Err(ControlFlowValue::Exception(Exception::ValueIsWrongType)),
        }
    }

    pub fn into_list(&self) -> Result<&Vec<Value>, ControlFlowValue> {
        match self {
            Value::List(v) => Ok(v),
            _ => Err(ControlFlowValue::Exception(Exception::ValueIsWrongType)),
        }
    }
}
