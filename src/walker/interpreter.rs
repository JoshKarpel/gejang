use std::{borrow::Cow, collections::HashMap};

use strum_macros::{AsRefStr, IntoStaticStr};
use thiserror::Error;

use crate::{shared::scanner::TokenType, walker::ast::Expr};

#[derive(Debug, Clone, PartialEq, AsRefStr, IntoStaticStr)]
pub enum Value<'s> {
    // Is it really worth bringing those strings all the way from the source to here?
    #[allow(dead_code)]
    Object(HashMap<Cow<'s, str>, Value<'s>>),
    Number(f64),
    String(Cow<'s, str>),
    Boolean(bool),
    Nil,
}

impl<'s> Value<'s> {
    pub fn is_truthy(&self) -> bool {
        match self {
            // TODO: implement Python-like truthiness
            Value::Boolean(value) => *value,
            Value::Nil => false,
            _ => true,
        }
    }
}

#[derive(Error, Clone, Debug, PartialEq)]
pub enum RuntimeError {
    #[error("{msg}")]
    Unimplemented { msg: String },
}

type EvaluationResult<'s> = Result<Value<'s>, RuntimeError>;

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
                TokenType::String(value) => Value::String(Cow::from(value)),
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
                        _ => {
                            return Err(RuntimeError::Unimplemented {
                                msg: "Cannot negate non-number".into(),
                            })
                        }
                    },
                    TokenType::Bang => Value::Boolean(!right.is_truthy()),
                    _ => unreachable!("Unary operator not implemented: {:?}", op),
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
                    (Value::String(ref l), TokenType::Plus, Value::String(ref r)) => {
                        Value::String(Cow::from(format!("{}{}", l, r)))
                    }
                    (l, TokenType::EqualEqual, r) => Value::Boolean(l == r),
                    (l, TokenType::BangEqual, r) => Value::Boolean(l != r),
                    // TODO: more specific errors!
                    (l, o, r) => {
                        return Err(RuntimeError::Unimplemented {
                            msg: format!(
                                "Binary operation not implemented: {} {} {}",
                                l.as_ref(),
                                o,
                                r.as_ref()
                            ),
                        })
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{
        shared::scanner::{scan, Token},
        walker::{
            interpreter::{EvaluationResult, Interpreter, RuntimeError, Value},
            parser::parse,
        },
    };

    #[rstest]
    #[case("1", Ok(Value::Number(1.0)))]
    #[case("\"foo\"", Ok(Value::String("foo".into())))]
    #[case("true", Ok(Value::Boolean(true)))]
    #[case("false", Ok(Value::Boolean(false)))]
    #[case("nil", Ok(Value::Nil))]
    #[case("1 + 2", Ok(Value::Number(3.0)))]
    #[case("\"foo\" + \"bar\"", Ok(Value::String("foobar".into())))]
    #[case("\"foo\" + 1", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String + Number".into() }))]
    #[case("1 + \"foo\"", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + String".into() }))]
    #[case("1 + true", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + Boolean".into() }))]
    #[case("1 + false", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + Boolean".into() }))]
    #[case("1 + nil", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + Nil".into() }))]
    fn test_interpreter(#[case] source: &str, #[case] expected: EvaluationResult) {
        let interpreter = Interpreter::new();
        let tokens: Vec<Token> = scan(source).try_collect().unwrap();
        let expr = parse(tokens.iter()).unwrap();
        let result = interpreter.evaluate(&expr);
        assert_eq!(result, expected);
    }
}
