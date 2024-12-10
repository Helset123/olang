use crate::lexer::{Lexer, LexerError, Token, TokenValue};
use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulus,
}

pub type Block = Vec<Expression>;

#[derive(Debug, Clone)]
pub struct DefinedFunction {
    pub parameters: Vec<String>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Int(i64),
    String(String),
    Bool(bool),
    Null,
    Return(Box<Expression>),
    Block(Block),
    Identifier(String),
    Binary {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
    },
    VariableDeclaration {
        identifier: String,
        expression: Box<Expression>,
    },
    Function(DefinedFunction),
    Call {
        identifier: String,
        arguments: Vec<Expression>,
    },
}

#[derive(Debug)]
pub struct Program {
    pub ast: Vec<Expression>,
}

pub struct Parser {
    tokens: Vec<Token>,
    t: usize,
}

impl Parser {
    pub fn new(source: &String) -> Result<Parser, LexerError> {
        Ok(Parser {
            tokens: Lexer::new(source).tokenize()?,
            t: 0,
        })
    }

    fn advance(&mut self) {
        self.t += 1
    }

    fn current(&self) -> &TokenValue {
        &self.tokens[self.t].value
    }

    fn parse_block(&mut self) -> Result<Block> {
        match self.current() {
            TokenValue::OpenBrace => Ok(()),
            _ => Err(anyhow!(
                "unexpected token found while parsing block expression"
            )),
        }?;
        self.advance();

        let mut expressions: Vec<Expression> = vec![];
        loop {
            match self.current() {
                TokenValue::CloseBrace => break,
                _ => expressions.push(self.parse_expression()?),
            };
        }
        self.advance(); // skip the closing brace

        Ok(expressions)
    }

    fn parse_return(&mut self) -> Result<Expression> {
        match self.current() {
            TokenValue::KeywordReturn => Ok(()),
            _ => Err(anyhow!(
                "unexpected token found while parsing return expression"
            )),
        }?;
        self.advance();

        Ok(Expression::Return(Box::new(self.parse_expression()?)))
    }

    fn parse_identifier(&mut self) -> Result<Expression> {
        let value = match self.current() {
            TokenValue::Identifier(v) => Ok(v.clone()),
            _ => Err(anyhow!(
                "unexpected token found while parsing int expression"
            )),
        }?;
        self.advance();
        Ok(Expression::Identifier(value))
    }

    fn parse_int(&mut self) -> Result<Expression> {
        let value = match self.current() {
            TokenValue::Int(v) => Ok(*v),
            _ => Err(anyhow!(
                "unexpected token found while parsing int expression"
            )),
        }?;
        self.advance();
        Ok(Expression::Int(value))
    }

    fn parse_string(&mut self) -> Result<Expression> {
        let value = match self.current() {
            TokenValue::String(v) => Ok(v.clone()),
            _ => Err(anyhow!(
                "unexpected token found while parsing string expression"
            )),
        }?;
        self.advance();
        Ok(Expression::String(value))
    }

    fn parse_null(&mut self) -> Result<Expression> {
        match self.current() {
            TokenValue::KeywordNull => Ok(()),
            _ => Err(anyhow!(
                "unexpected token found while parsing null expression"
            )),
        }?;
        self.advance();
        Ok(Expression::Null)
    }

    fn parse_bool(&mut self) -> Result<Expression> {
        let value = match self.current() {
            TokenValue::KeywordTrue => Ok(true),
            TokenValue::KeywordFalse => Ok(false),
            _ => Err(anyhow!(
                "unexpected token found while parsing bool expression"
            )),
        }?;
        self.advance();
        Ok(Expression::Bool(value))
    }

    fn parse_variable_declaration(&mut self) -> Result<Expression> {
        match self.current() {
            TokenValue::KeywordVar => Ok(()),
            _ => Err(anyhow!(
                "unexpected token found while parsing variable declaration expression"
            )),
        }?;
        self.advance();

        let identifier = match self.current() {
            TokenValue::Identifier(v) => Ok(v),
            _ => Err(anyhow!(
                "unexpected token found while parsing variable declaration expression"
            )),
        }?
        .clone();
        self.advance();

        match self.current() {
            TokenValue::EqualSign => Ok(()),
            _ => Err(anyhow!(
                "unexpected token found while parsing variable declaration expression"
            )),
        }?;
        self.advance();

        Ok(Expression::VariableDeclaration {
            identifier,
            expression: Box::new(self.parse_expression()?),
        })
    }

    fn parse_call(&mut self) -> Result<Expression> {
        let identifier = match self.current() {
            TokenValue::Identifier(v) => Ok(v.clone()),
            _ => Err(anyhow!(
                "unexpected token found while parsing call expression"
            )),
        }?;
        self.advance();

        match self.current() {
            TokenValue::OpenParenthesis => Ok(()),
            _ => Err(anyhow!(
                "unexpected token found while parsing call expression"
            )),
        }?;
        self.advance();

        let mut arguments = vec![];
        while *self.current() != TokenValue::CloseParenthesis {
            arguments.push(self.parse_expression()?);
        }
        self.advance(); // skip the clogin parenthesis )

        Ok(Expression::Call {
            identifier,
            arguments,
        })
    }

    fn parse_function(&mut self) -> Result<Expression> {
        match self.current() {
            TokenValue::KeywordFun => Ok(()),
            _ => Err(anyhow!(
                "unexpected token found while parsing variable function expression"
            )),
        }?;
        self.advance();

        match self.current() {
            TokenValue::OpenParenthesis => Ok(()),
            _ => Err(anyhow!(
                "unexpected token found while parsing variable function expression"
            )),
        }?;
        self.advance();

        let mut parameters = vec![];
        loop {
            match self.current() {
                TokenValue::CloseParenthesis => {
                    self.advance();
                    break;
                }
                TokenValue::Identifier(v) => {
                    parameters.push(v.clone());
                }
                _ => return Err(anyhow!("unexpected token found in function parameters")),
            }
            self.advance();
        }

        Ok(Expression::Function(DefinedFunction {
            parameters,
            body: self.parse_block()?,
        }))
    }

    fn parse_primary(&mut self) -> Result<Expression> {
        match self.current() {
            TokenValue::Int(_) => self.parse_int(),
            TokenValue::String(_) => self.parse_string(),
            TokenValue::Identifier(_) => {
                if self.tokens[self.t + 1].value == TokenValue::OpenParenthesis {
                    self.parse_call()
                } else {
                    self.parse_identifier()
                }
            }
            TokenValue::OpenParenthesis => {
                self.advance(); // skip the open parenthesis (

                let expression = self.parse_expression()?;

                match self.current() {
                    TokenValue::CloseParenthesis => Ok(()),
                    _ => Err(anyhow!(
                        "Expected closing parenthesis at end of parenthesis expression"
                    )),
                }?;
                self.advance();

                Ok(expression)
            }
            TokenValue::KeywordNull => self.parse_null(),
            TokenValue::KeywordTrue | TokenValue::KeywordFalse => self.parse_bool(),
            TokenValue::OpenBrace => Ok(Expression::Block(self.parse_block()?)),
            TokenValue::KeywordReturn => self.parse_return(),
            TokenValue::KeywordVar => self.parse_variable_declaration(),
            TokenValue::KeywordFun => self.parse_function(),
            _ => Err(anyhow!(
                "Unexpected token found while parsing primary expression"
            )),
        }
    }

    fn parse_multiplicative(&mut self) -> Result<Expression> {
        let mut left = self.parse_primary()?;

        loop {
            let operator = match self.current() {
                TokenValue::MultiplicationSign => Operator::Multiply,
                TokenValue::DivisionSign => Operator::Divide,
                TokenValue::ModuloSign => Operator::Modulus,
                _ => {
                    break;
                }
            };
            self.advance();

            let right = self.parse_primary()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            }
        }

        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expression> {
        let mut left = self.parse_multiplicative()?;

        loop {
            let operator = match self.current() {
                TokenValue::PlusSign => Operator::Plus,
                TokenValue::MinusSign => Operator::Minus,
                _ => {
                    break;
                }
            };
            self.advance();

            let right = self.parse_multiplicative()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            }
        }

        Ok(left)
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_additive()
    }

    pub fn parse(&mut self) -> Result<Program> {
        self.t = 0;
        let mut program: Program = Program { ast: vec![] };

        while self.tokens[self.t].value != TokenValue::EndOfFile {
            program.ast.push(self.parse_expression()?);
        }

        Ok(program)
    }
}
