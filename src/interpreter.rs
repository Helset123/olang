use crate::{
    environment::Environment,
    lexer::LexerError,
    parser::{Block, Expression, ExpressionValue, Operator, Parser, ParserError},
    value::{ControlFlowValue, Exception, Function, Value},
};
use thiserror::Error;

pub struct Interpreter {
    environment: Environment,
}

#[derive(Error, Debug)]
pub enum EvalError {
    #[error("Unhandeled exception: {0}")]
    UnhandeledException(Exception),
    #[error(transparent)]
    Parser(#[from] ParserError),
    #[error(transparent)]
    Lexer(#[from] LexerError),
}

impl Interpreter {
    fn eval_binary(
        &mut self,
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
    ) -> Result<Value, ControlFlowValue> {
        let left_value = self.eval_expression(*left)?;
        let left_int = match left_value {
            Value::Int(v) => v,
            _ => {
                return Err(ControlFlowValue::Exception(
                    Exception::ValueIsWrongTypeInBinaryOperator,
                ))
            }
        };

        let right_value = self.eval_expression(*right)?;
        let right_int = match right_value {
            Value::Int(v) => v,
            _ => {
                return Err(ControlFlowValue::Exception(
                    Exception::ValueIsWrongTypeInBinaryOperator,
                ))
            }
        };

        Ok(match operator {
            Operator::Plus => Value::Int(left_int + right_int),
            Operator::Minus => Value::Int(left_int - right_int),
            Operator::Multiply => Value::Int(left_int * right_int),
            Operator::Divide => Value::Int(left_int / right_int),
            Operator::Modulus => Value::Int(left_int % right_int),
        })
    }

    fn eval_block(
        &mut self,
        private_environment: bool,
        block: Block,
    ) -> Result<Value, ControlFlowValue> {
        if private_environment {
            self.environment.push();
        }

        for expression in block {
            self.eval_expression(expression)?;
        }

        if private_environment {
            self.environment.pop();
        }

        Ok(Value::Null)
    }

    fn eval_identifier(&mut self, id: String) -> Result<Value, ControlFlowValue> {
        match self.environment.get(&id) {
            Some(v) => Ok(v),
            None => Err(ControlFlowValue::Exception(Exception::UndeclaredIdentifier)),
        }
    }

    fn eval_call(
        &mut self,
        id: String,
        arguments: Vec<Expression>,
    ) -> Result<Value, ControlFlowValue> {
        let function_value = match self.environment.get(&id) {
            Some(v) => v.clone(),
            _ => return Err(ControlFlowValue::Exception(Exception::UndeclaredIdentifier)),
        };

        match function_value {
            Value::Function(function) => {
                let mut evaluated_arguments = vec![];
                for argument in arguments.iter() {
                    evaluated_arguments.push(self.eval_expression(argument.clone())?)
                }

                match function {
                    Function::Builtin(function) => function(evaluated_arguments),
                    Function::Defined(defined) => {
                        self.environment.push();

                        if defined.parameters.len() != arguments.len() {
                            return Err(ControlFlowValue::Exception(
                                Exception::WrongNumberOfArguments,
                            ));
                        }

                        for (i, parameter) in defined.parameters.iter().enumerate() {
                            self.environment
                                .declare(parameter.clone(), evaluated_arguments[i].clone());
                        }

                        let result = match self.eval_block(false, defined.body) {
                            Ok(_) => Ok(Value::Null),
                            Err(err) => match err {
                                ControlFlowValue::Return(v) => Ok(v),
                                ControlFlowValue::Exception(_) => Err(err),
                            },
                        };

                        self.environment.pop();

                        result
                    }
                }
            }
            _ => Err(ControlFlowValue::Exception(
                Exception::CalledValueIsNotFunction,
            )),
        }
    }

    fn eval_declare_variable(
        &mut self,
        id: String,
        expression: Box<Expression>,
    ) -> Result<Value, ControlFlowValue> {
        let value = self.eval_expression(*expression)?;
        self.environment.declare(id, value);
        Ok(Value::Null)
    }

    fn eval_return(&mut self, expression: Box<Expression>) -> ControlFlowValue {
        match self.eval_expression(*expression) {
            Ok(v) => ControlFlowValue::Return(v),
            Err(v) => match v {
                ControlFlowValue::Return(_) => {
                    ControlFlowValue::Exception(Exception::NestedReturns)
                }
                ControlFlowValue::Exception(_) => v,
            },
        }
    }

    fn eval_expression(&mut self, expression: Expression) -> Result<Value, ControlFlowValue> {
        match expression.value {
            ExpressionValue::Int(v) => Ok(Value::Int(v)),
            ExpressionValue::String(v) => Ok(Value::String(v)),
            ExpressionValue::Bool(v) => Ok(Value::Bool(v)),
            ExpressionValue::Null => Ok(Value::Null),
            ExpressionValue::Return(v) => Err(self.eval_return(v)),
            ExpressionValue::Function(v) => Ok(Value::Function(Function::Defined(v))),
            ExpressionValue::Block(v) => self.eval_block(true, v),
            ExpressionValue::Identifier(id) => self.eval_identifier(id),
            ExpressionValue::Call {
                identifier,
                arguments,
            } => self.eval_call(identifier, arguments),
            ExpressionValue::VariableDeclaration {
                identifier,
                expression,
            } => self.eval_declare_variable(identifier, expression),
            ExpressionValue::Binary {
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

    pub fn eval(&mut self, source: &String) -> Result<Value, EvalError> {
        let program = Parser::new(source)?.parse()?;
        let mut result = Value::Null;

        for expression in program.ast {
            match self.eval_expression(expression) {
                Ok(_) => Ok(Value::Null),
                Err(err) => match err {
                    ControlFlowValue::Return(v) => {
                        result = v;
                        break;
                    }
                    ControlFlowValue::Exception(e) => Err(EvalError::UnhandeledException(e)),
                },
            }?;
        }

        return Ok(result);
    }
}
