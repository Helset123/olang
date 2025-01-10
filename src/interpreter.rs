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
    #[error("Unhandled exception: {0}")]
    UnhandledException(Exception),
    #[error(transparent)]
    Parser(#[from] ParserError),
    #[error(transparent)]
    Lexer(#[from] LexerError),
}

impl Interpreter {
    fn eval_binary(
        &mut self,
        left_expression: &Box<Expression>,
        operator: &Operator,
        right_expression: &Box<Expression>,
    ) -> Result<Value, ControlFlowValue> {
        let left = self.eval_expression(left_expression)?;
        let right = self.eval_expression(right_expression)?;

        // FIXME: utilize the Eq trait instead of this garbage
        Ok(match operator {
            Operator::Plus => Value::Int(left.into_int()? + right.into_int()?),
            Operator::Minus => Value::Int(left.into_int()? - right.into_int()?),
            Operator::Multiply => Value::Int(left.into_int()? * right.into_int()?),
            Operator::Divide => Value::Int(left.into_int()? / right.into_int()?),
            Operator::Modulus => Value::Int(left.into_int()? % right.into_int()?),
            Operator::IsEqual => Value::Bool(left == right),
            // Operator::IsEqual => match left {
            //     Value::Int(left) => Value::Bool(left == *right.into_int()?),
            //     Value::Bool(left) => Value::Bool(left == *right.into_bool()?),
            //     _ => return Err(ControlFlowValue::Exception(Exception::ValueIsWrongType)),
            // },
            Operator::IsNotEqual => match left {
                Value::Int(left) => Value::Bool(left != *right.into_int()?),
                Value::Bool(left) => Value::Bool(left != *right.into_bool()?),
                _ => return Err(ControlFlowValue::Exception(Exception::ValueIsWrongType)),
            },
            Operator::IsLessThan => Value::Bool(left.into_int()? < right.into_int()?),
            Operator::IsLessThanOrEqual => Value::Bool(left.into_int()? <= right.into_int()?),
            Operator::IsGreaterThan => Value::Bool(left.into_int()? > right.into_int()?),
            Operator::IsGreaterThanOrEqual => Value::Bool(left.into_int()? >= right.into_int()?),
            Operator::And => Value::Bool(*left.into_bool()? && *right.into_bool()?),
            Operator::Or => Value::Bool(*left.into_bool()? || *right.into_bool()?),
        })
    }

    fn eval_block(
        &mut self,
        private_environment: bool,
        block: &Block,
    ) -> Result<Value, ControlFlowValue> {
        if private_environment {
            self.environment.push();
        }

        let mut result = Value::Null;
        for expression in block {
            result = self.eval_expression(expression)?;
        }

        if private_environment {
            self.environment.pop();
        }

        Ok(result)
    }

    fn eval_identifier(&mut self, id: &String) -> Result<Value, ControlFlowValue> {
        match self.environment.get(id) {
            Some(v) => Ok(v),
            None => Err(ControlFlowValue::Exception(Exception::UndeclaredIdentifier)),
        }
    }

    fn eval_call(
        &mut self,
        id: &String,
        arguments: &Vec<Expression>,
    ) -> Result<Value, ControlFlowValue> {
        let function_value = match self.environment.get(id) {
            Some(v) => v,
            _ => return Err(ControlFlowValue::Exception(Exception::UndeclaredIdentifier)),
        };

        match function_value {
            Value::Function(function) => {
                let mut evaluated_arguments = vec![];
                for argument in arguments.iter() {
                    evaluated_arguments.push(self.eval_expression(argument)?)
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

                        let result = self.eval_block(false, &defined.body);

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
        id: &String,
        expression: &Box<Expression>,
    ) -> Result<Value, ControlFlowValue> {
        let value = self.eval_expression(expression)?;
        self.environment.declare(id.clone(), value);
        Ok(Value::Null)
    }

    fn eval_if(
        &mut self,
        test: &Box<Expression>,
        body: &Vec<Expression>,
    ) -> Result<Value, ControlFlowValue> {
        if *self.eval_expression(test)?.into_bool()? {
            self.eval_block(true, body)?;
        }
        Ok(Value::Null)
    }

    fn eval_assign(
        &mut self,
        id: &String,
        expression: &Box<Expression>,
    ) -> Result<Value, ControlFlowValue> {
        let value = self.eval_expression(expression)?;
        self.environment.assign(id.as_str(), value)?;

        Ok(Value::Null)
    }

    fn eval_expression(&mut self, expression: &Expression) -> Result<Value, ControlFlowValue> {
        match &expression.value {
            ExpressionValue::Int(v) => Ok(Value::Int(*v)),
            ExpressionValue::String(v) => Ok(Value::String(v.clone())),
            ExpressionValue::Bool(v) => Ok(Value::Bool(*v)),
            ExpressionValue::Null => Ok(Value::Null),
            ExpressionValue::If { test, body } => self.eval_if(test, body),
            ExpressionValue::While { test, body } => self.eval_while(test, body),
            ExpressionValue::Function(v) => Ok(Value::Function(Function::Defined(v.clone()))),
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
            ExpressionValue::Assign {
                identifier,
                expression,
            } => self.eval_assign(identifier, expression),
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
            match self.eval_expression(&expression) {
                Ok(v) => Ok(v),
                Err(err) => match err {
                    ControlFlowValue::Exception(e) => Err(EvalError::UnhandeledException(e)),

                },
            }?;
        }

        return Ok(result);
    }
}
