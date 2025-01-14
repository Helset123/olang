use crate::{
    environment::Environment,
    lexer::LexerError,
    parser::{Block, Expression, ExpressionValue, IfClause, Operator, Parser, ParserError},
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
    #[error("\"continue\" keyword used outside of loop")]
    ContinueOutsideLoop,
    #[error("\"break\" keyword used outside of loop")]
    BreakOutsideLoop,
    #[error(transparent)]
    Parser(#[from] ParserError),
    #[error(transparent)]
    Lexer(#[from] LexerError),
}

impl EvalError {
    pub fn unwrap_exception(&self) -> &Exception {
        match self {
            Self::UnhandledException(v) => v,
            _ => {
                panic!("called `EvalError::unwrap_exception()` on something else than a `UnhandledException` error")
            }
        }
    }
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
            Operator::Exponentiation => {
                let base = *left.into_int()?;
                let exponent = *right.into_int()?;
                match (base as u64).checked_pow(exponent as u32) {
                    Some(v) => Value::Int(v as i64),
                    None => {
                        return Err(ControlFlowValue::Exception(
                            Exception::ExponentiationOverflowed,
                        ))
                    }
                }
            }
            Operator::IsEqual => Value::Bool(left == right),
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
        clauses: &Vec<IfClause>,
        else_block: &Option<Block>,
    ) -> Result<Value, ControlFlowValue> {
        let mut run_else_block = true;
        let mut result = Value::Null;

        for clause in clauses {
            let test_value = self.eval_expression(clause.test.as_ref())?;
            if *test_value.into_bool()? {
                result = self.eval_block(true, &clause.body)?;
                run_else_block = false;
                break;
            }
        }

        if run_else_block {
            if let Some(block) = else_block {
                result = self.eval_block(true, &block)?;
            }
        }

        Ok(result)
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

    fn eval_loop(
        &mut self,
        init: &Option<Box<Expression>>,
        test: &Option<Box<Expression>>,
        update: &Option<Box<Expression>>,
        body: &Block,
    ) -> Result<Value, ControlFlowValue> {
        let mut result = Value::Null;

        self.environment.push();

        if let Some(init) = init {
            self.eval_expression(init)?;
        }

        loop {
            if let Some(test) = test {
                if !*self.eval_expression(test)?.into_bool()? {
                    break;
                }
            }
            match self.eval_block(false, body) {
                Ok(v) => {
                    result = v;
                }
                Err(ControlFlowValue::Continue) => {}
                Err(ControlFlowValue::Break) => break,
                Err(e) => {
                    return Err(e);
                }
            }
            if let Some(update) = update {
                self.eval_expression(update)?;
            }
        }

        self.environment.pop();

        Ok(result)
    }

    fn eval_expression(&mut self, expression: &Expression) -> Result<Value, ControlFlowValue> {
        match &expression.value {
            ExpressionValue::Int(v) => Ok(Value::Int(*v)),
            ExpressionValue::String(v) => Ok(Value::String(v.clone())),
            ExpressionValue::Bool(v) => Ok(Value::Bool(*v)),
            ExpressionValue::Null => Ok(Value::Null),
            ExpressionValue::If {
                clauses,
                else_block,
            } => self.eval_if(clauses, else_block),
            ExpressionValue::Loop {
                init,
                test,
                update,
                body,
            } => self.eval_loop(init, test, update, body),
            ExpressionValue::Continue => Err(ControlFlowValue::Continue),
            ExpressionValue::Break => Err(ControlFlowValue::Break),
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

    pub fn eval(&mut self, source: &str) -> Result<Value, EvalError> {
        let program = Parser::new(source)?.parse()?;
        let mut result = Value::Null;

        for expression in program.ast {
            match self.eval_expression(&expression) {
                Ok(v) => Ok(result = v),
                Err(err) => match err {
                    ControlFlowValue::Exception(e) => Err(EvalError::UnhandledException(e)),
                    ControlFlowValue::Continue => Err(EvalError::ContinueOutsideLoop),
                    ControlFlowValue::Break => Err(EvalError::BreakOutsideLoop),
                },
            }?;
        }

        return Ok(result);
    }
}
