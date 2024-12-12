use crate::value::{ControlFlowValue, Exception, Value};

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
    println!("{:?}", arguments);
    Ok(Value::Null)
}

pub fn to_string(arguments: Vec<Value>) -> Result<Value, ControlFlowValue> {
    expect_num_of_argumets(&arguments, 1)?;
    Ok(Value::String(format!("{}", arguments.first().unwrap())))
}
