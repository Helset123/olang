use crate::value::{ControlFlowValue, Exception, Value};
use std::{cmp::Reverse, io};

fn expect_num_of_argumets(arguments: &Vec<Value>, num: usize) -> Result<(), ControlFlowValue> {
    if arguments.len() != num {
        Err(ControlFlowValue::Exception(
            Exception::WrongNumberOfArguments,
        ))
    } else {
        Ok(())
    }
}

pub fn print_ln(arguments: Vec<Value>) -> Result<Value, ControlFlowValue> {
    let mut result = String::new();
    for arg in arguments.iter() {
        result.push_str(format!("{}", arg).as_str())
    }

    println!("{}", result);
    Ok(Value::Null)
}

pub fn to_string(arguments: Vec<Value>) -> Result<Value, ControlFlowValue> {
    expect_num_of_argumets(&arguments, 1)?;
    Ok(Value::String(format!("{}", arguments.first().unwrap())))
}

pub fn read_ln(arguments: Vec<Value>) -> Result<Value, ControlFlowValue> {
    expect_num_of_argumets(&arguments, 0)?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|err| ControlFlowValue::Exception(Exception::Custom(err.to_string())))?;

    Ok(Value::String(input.trim().to_string()))
}

pub fn len(arguments: Vec<Value>) -> Result<Value, ControlFlowValue> {
    expect_num_of_argumets(&arguments, 1)?;

    Ok(Value::Int(
        arguments.first().unwrap().into_list()?.len() as i64
    ))
}
