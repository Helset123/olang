use crate::{
    environment::Environment,
    parser::{Block, Expression, Operator, Parser},
    value::{Function, Value},
};
use anyhow::{anyhow, Result};

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    fn eval_binary(
        &mut self,
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
    ) -> Value {
        let left_value = self.eval_expression(*left);
        let left_int = match left_value {
            Value::Int(v) => v,
            _ => {
                return Value::SystemException("Value is not int in binary expression".to_string())
            }
        };

        let right_value = self.eval_expression(*right);
        let right_int = match right_value {
            Value::Int(v) => v,
            _ => {
                return Value::SystemException("Value is not int in binary expression".to_string())
            }
        };

        match operator {
            Operator::Plus => Value::Int(left_int + right_int),
            Operator::Minus => Value::Int(left_int - right_int),
            Operator::Multiply => Value::Int(left_int * right_int),
            Operator::Divide => Value::Int(left_int / right_int),
            Operator::Modulus => Value::Int(left_int % right_int),
        }
    }

    fn eval_block(&mut self, private_environment: bool, block: Block) -> Value {
        if private_environment {
            self.environment.push();
        }

        for expression in block {
            let value = self.eval_expression(expression);
            match value {
                Value::SystemReturn(_) | Value::SystemException(_) => {
                    return value;
                }
                _ => {}
            }
        }

        if private_environment {
            self.environment.pop();
        }

        Value::Null
    }

    fn eval_identifier(&mut self, id: String) -> Value {
        match self.environment.get(&id) {
            Some(v) => v,
            None => Value::SystemException("Use of undeclared identifier".to_string()),
        }
    }

    fn eval_call(&mut self, id: String, arguments: Vec<Expression>) -> Value {
        let function_value = match self.environment.get(&id) {
            Some(v) => v.clone(),
            _ => return Value::SystemException("attempt to call undeclared function".to_string()),
        };

        match function_value {
            Value::Function(function) => {
                let evaluated_arguments = arguments
                    .iter()
                    .map(|e| self.eval_expression(e.clone()))
                    .collect();

                match function {
                    Function::Builtin(function) => function(evaluated_arguments),
                    Function::Defined(defined) => {
                        self.environment.push();

                        if defined.parameters.len() != arguments.len() {
                            return Value::SystemException(
                                "wrong number of arguments in function call".to_string(),
                            );
                        }

                        for (i, parameter) in defined.parameters.iter().enumerate() {
                            self.environment
                                .declare(parameter.clone(), evaluated_arguments[i].clone());
                        }

                        let block_value = self.eval_block(false, defined.body);
                        let result = match block_value {
                            Value::SystemReturn(v) => *v,
                            _ => block_value,
                        };

                        self.environment.pop();

                        result
                    }
                }
            }
            _ => {
                return Value::SystemException(
                    "attempted to call value that is not a function".to_string(),
                )
            }
        }
    }

    fn eval_declare_variable(&mut self, id: String, expression: Box<Expression>) -> Value {
        let value = self.eval_expression(*expression);
        self.environment.declare(id, value);
        Value::Null
    }

    fn eval_return(&mut self, expression: Box<Expression>) -> Value {
        let value = self.eval_expression(*expression);
        match value {
            Value::SystemReturn(_) => {
                Value::SystemException("Attempted to return return".to_string())
            }
            Value::SystemException(_) => value,
            _ => Value::SystemReturn(Box::new(value)),
        }
    }

    fn eval_expression(&mut self, expression: Expression) -> Value {
        match expression {
            Expression::Int(v) => Value::Int(v),
            Expression::String(v) => Value::String(v),
            Expression::Bool(v) => Value::Bool(v),
            Expression::Null => Value::Null,
            Expression::Return(v) => self.eval_return(v),
            Expression::Function(v) => Value::Function(Function::Defined(v)),
            Expression::Block(v) => self.eval_block(true, v),
            Expression::Identifier(id) => self.eval_identifier(id),
            Expression::Call {
                identifier,
                arguments,
            } => self.eval_call(identifier, arguments),
            Expression::VariableDeclaration {
                identifier,
                expression,
            } => self.eval_declare_variable(identifier, expression),
            Expression::Binary {
                left,
                operator,
                right,
            } => self.eval_binary(left, operator, right),
        }
    }

    pub fn new() -> Interpreter {
        Interpreter {
            environment: Environment::default(),
        }
    }

    pub fn eval(&mut self, source: &String) -> Result<Value> {
        let program = Parser::new(source)?.parse()?;
        let mut result = Value::Null;

        for expression in program.ast {
            let value = self.eval_expression(expression);
            match value {
                Value::SystemReturn(v) => {
                    result = *v;
                    break;
                }
                Value::SystemException(v) => return Err(anyhow!("Unhandeled exception: {}", v)),
                _ => {}
            }
        }

        return Ok(result);
    }
}
