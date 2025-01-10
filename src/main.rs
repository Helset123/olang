use anyhow::Result;
use interpreter::EvalError;
use value::Value;

use std::fs;

mod builtin;
mod environment;
mod interpreter;
mod lexer;
mod parser;
mod value;

#[cfg(test)]
mod tests;

pub fn eval(source: &str) -> Result<Value, EvalError> {
    interpreter::Interpreter::new().eval(source)
}

fn main() -> Result<()> {
    let source = fs::read_to_string("source.olang")?;

    // println!("AST: {:#?}", Parser::new(&source)?.parse()?);
    eval(source.as_str())?;
    Ok(())
}
