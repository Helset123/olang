use core::fmt;
use std::{string::String, vec::Vec};

#[derive(Debug, PartialEq, Clone)]
pub enum TokenValue {
    KeywordReturn,      // return
    KeywordFun,         // fun
    KeywordTrue,        // true
    KeywordFalse,       // false
    KeywordNull,        // null
    KeywordVar,         // var
    EqualSign,          // =
    CloseParenthesis,   // )
    OpenParenthesis,    // (
    OpenBrace,          // {
    CloseBrace,         // }
    PlusSign,           // +
    MinusSign,          // -
    DivisionSign,       // /
    MultiplicationSign, // *
    ModuloSign,         // %
    EndOfFile,          // EOF
    Identifier(String), // print
    String(String),     // "Hello World"
    Int(i64),           // 100
}

#[derive(Debug, Clone)]
pub struct Token {
    pub value: TokenValue,
    region: Region,
}

impl Token {
    pub fn new(region: Region, value: TokenValue) -> Token {
        Token { value, region }
    }
}

#[derive(Debug, Clone)]
pub struct Region {
    start: usize,
    end: usize,
}

#[derive(Debug)]
pub struct LexerError {
    details: String,
    location: usize,
}

impl LexerError {
    fn new(location: usize, details: &str) -> LexerError {
        LexerError {
            details: details.to_string(),
            location,
        }
    }
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error at {}: {}", self.location, self.details)
    }
}

impl std::error::Error for LexerError {}

pub struct Lexer {
    source: Vec<char>,
    c: usize,
}

impl Lexer {
    pub fn new(source: &String) -> Lexer {
        Lexer {
            source: source.chars().collect(),
            c: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut result: Vec<Token> = vec![];
        self.c = 0;

        while self.c < self.source.len() {
            let mut region = Region { start: 0, end: 0 };
            region.start = self.c;
            // match for simple one char poiters
            match match self.source[self.c] {
                '(' => Some(TokenValue::OpenParenthesis),
                ')' => Some(TokenValue::CloseParenthesis),
                '{' => Some(TokenValue::OpenBrace),
                '}' => Some(TokenValue::CloseBrace),
                '=' => Some(TokenValue::EqualSign),
                '+' => Some(TokenValue::PlusSign),
                '-' => Some(TokenValue::MinusSign),
                '*' => Some(TokenValue::MultiplicationSign),
                '/' => Some(TokenValue::DivisionSign),
                '%' => Some(TokenValue::ModuloSign),
                _ => None,
            } {
                Some(v) => {
                    region.end = self.c;
                    result.push(Token::new(region, v));
                    self.c += 1;
                    continue;
                }
                _ => {}
            }

            if self.source[self.c].is_whitespace() {
                self.c += 1;
                continue;
            }

            // string token
            if self.source[self.c] == '"' {
                let mut value = "".to_string();
                self.c += 1;
                while self.c < self.source.len() && self.source[self.c] != '"' {
                    value.push(self.source[self.c]);
                    self.c += 1
                }
                self.c += 1;

                region.end = self.c;
                result.push(Token::new(region, TokenValue::String(value)));
            }
            // int token
            else if self.source[self.c].is_digit(10) || self.source[self.c] == '-' {
                let mut value: i64 = 0;
                let mut negative = false;

                if self.source[self.c] == '-' {
                    negative = true;
                    self.c += 1;
                };

                while self.c < self.source.len() && self.source[self.c].is_digit(10) {
                    value = value * 10
                        + self.source[self.c]
                            .to_digit(10)
                            .ok_or(LexerError::new(self.c, "Expected digit in int token"))?
                            as i64;
                    self.c += 1
                }

                if negative {
                    value *= -1
                }

                region.end = self.c;
                result.push(Token::new(region, TokenValue::Int(value)));
            }
            // identifier or keyword
            else if self.source[self.c].is_alphanumeric() && !self.source[self.c].is_whitespace()
            {
                let mut value = "".to_string();

                while self.c < self.source.len()
                    && self.source[self.c].is_alphanumeric()
                    && !self.source[self.c].is_whitespace()
                {
                    value.push(self.source[self.c]);
                    self.c += 1
                }

                region.end = self.c;
                // check if this matches any keywords
                result.push(Token::new(
                    region,
                    match value.as_str() {
                        "fun" => TokenValue::KeywordFun,
                        "return" => TokenValue::KeywordReturn,
                        "true" => TokenValue::KeywordTrue,
                        "false" => TokenValue::KeywordFalse,
                        "null" => TokenValue::KeywordNull,
                        "var" => TokenValue::KeywordVar,
                        _ => TokenValue::Identifier(value),
                    },
                ))
            } else {
                return Err(LexerError::new(self.c, "Unexpected character"));
            }
        }

        result.push(Token::new(
            Region {
                start: self.source.len() - 1,
                end: self.source.len() - 1,
            },
            TokenValue::EndOfFile,
        ));
        Ok(result)
    }
}
