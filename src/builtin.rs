use crate::value::Value;

pub fn print_ln(arguments: Vec<Value>) -> Value {
    println!("{:?}", arguments);
    Value::Null
}

pub fn to_string(arguments: Vec<Value>) -> Value {
    if arguments.len() != 1 {
        // FIXME: this is one of the reasons that systemexception should be an enum, this repeats what is found for defined functions in Interpreter::eval_call
        return Value::SystemException("Wrong number of arguments".to_string());
    }

    Value::String(format!("{}", arguments.first().unwrap()))
}
