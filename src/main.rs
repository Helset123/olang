use anyhow::Result;

use std::fs;

mod builtin;
mod environment;
mod interpreter;
mod lexer;
mod parser;
mod value;

fn main() -> Result<()> {
    let source = fs::read_to_string("source.olang")?;

    interpreter::Interpreter::new().eval(&source)?;
    Ok(())
}
