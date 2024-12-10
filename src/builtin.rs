use crate::value::Value;

pub fn olang_print_ln(arguments: Vec<Value>) -> Value {
    println!("{:?}", arguments.join(" "));
    Value::Null
}
