use std::collections::HashMap;

use thiserror::Error;

use crate::{shared::scanner::TokenType, walker::ast::Expr};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    #[allow(dead_code)]
    Object(HashMap<String, Value>),
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            // TODO: implement Python-like truthiness
            Value::Boolean(value) => *value,
            Value::Nil => false,
            _ => true,
        }
    }
}

#[derive(Error, Clone, Debug)]
pub enum RuntimeError<'s> {
    #[error("Unimplemented expression: {expr}")]
    Unimplemented { expr: &'s Expr<'s> },
}

type EvaluationResult<'s> = Result<Value, RuntimeError<'s>>;

#[derive(Debug)]
pub struct Interpreter {}

impl Interpreter {
    pub fn new() -> Self {
        Self {}
    }

    pub fn evaluate<'s>(&self, expr: &'s Expr<'s>) -> EvaluationResult<'s> {
        Ok(match expr {
            Expr::Literal { token } => match token.typ {
                TokenType::Number(value) => Value::Number(value),
                TokenType::String(value) => Value::String(value.into()),
                TokenType::True => Value::Boolean(true),
                TokenType::False => Value::Boolean(false),
                TokenType::Nil => Value::Nil,
                _ => unreachable!("Literal token type not implemented: {:?}", token),
            },
            Expr::Grouping { expr } => self.evaluate(expr)?,
            Expr::Unary { op, right } => {
                let right = self.evaluate(right)?;

                match op.typ {
                    TokenType::Minus => match right {
                        Value::Number(value) => Value::Number(-value),
                        _ => return Err(RuntimeError::Unimplemented { expr }),
                    },
                    TokenType::Bang => Value::Boolean(!right.is_truthy()),
                    _ => return Err(RuntimeError::Unimplemented { expr }),
                }
            }
            Expr::Binary { left, op, right } => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match (left, op.typ, right) {
                    (Value::Number(l), TokenType::Plus, Value::Number(r)) => Value::Number(l + r),
                    (Value::Number(l), TokenType::Minus, Value::Number(r)) => Value::Number(l - r),
                    (Value::Number(l), TokenType::Star, Value::Number(r)) => Value::Number(l * r),
                    (Value::Number(l), TokenType::Slash, Value::Number(r)) => Value::Number(l / r),
                    (Value::Number(l), TokenType::Greater, Value::Number(r)) => {
                        Value::Boolean(l > r)
                    }
                    (Value::Number(l), TokenType::GreaterEqual, Value::Number(r)) => {
                        Value::Boolean(l >= r)
                    }
                    (Value::Number(l), TokenType::Less, Value::Number(r)) => Value::Boolean(l < r),
                    (Value::Number(l), TokenType::LessEqual, Value::Number(r)) => {
                        Value::Boolean(l <= r)
                    }
                    (Value::String(l), TokenType::Plus, Value::String(r)) => Value::String(l + &r),
                    (l, TokenType::EqualEqual, r) => Value::Boolean(l == r),
                    (l, TokenType::BangEqual, r) => Value::Boolean(l != r),
                    // TODO: more specific errors!
                    _ => return Err(RuntimeError::Unimplemented { expr }),
                }
            }
        })
    }
}
