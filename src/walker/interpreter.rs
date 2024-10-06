use std::{
    borrow::Cow,
    io::{Read, Write},
};

use crate::{
    shared::{
        errors::{EvaluationResult, RuntimeError},
        scanner::TokenType,
        streams::Streams,
        values::Value,
    },
    walker::ast::{Expr, Stmt},
};

#[derive(Debug)]
pub struct Interpreter<'io, I: Read, O: Write, E: Write> {
    streams: &'io mut Streams<I, O, E>,
}

impl<'io, I: Read, O: Write, E: Write> Interpreter<'io, I, O, E> {
    pub fn new(streams: &'io mut Streams<I, O, E>) -> Self {
        Self { streams }
    }

    pub fn interpret<'s>(&mut self, statements: &'s [Stmt<'s>]) -> EvaluationResult<'s> {
        for stmt in statements {
            self.execute(stmt)?;
        }

        // TODO: Not really supposed to return anything but maybe it doesn't matter?
        // I guess that's what an expression oriented language is for!
        Ok(Value::Nil)
    }

    pub fn execute<'s>(&mut self, stmt: &'s Stmt<'s>) -> EvaluationResult<'s> {
        match stmt {
            Stmt::Expression { expr } => self.evaluate(expr)?,
            Stmt::Print { expr } => {
                let value = self.evaluate(expr)?;
                write!(self.streams.output, "{}", &value).map_err(|_| RuntimeError::PrintFailed)?;
                value
            }
        };
        Ok(Value::Nil)
    }

    pub fn evaluate<'s>(&mut self, expr: &'s Expr<'s>) -> EvaluationResult<'s> {
        Ok(match expr {
            Expr::Literal { token } => Value::from(&token.typ),
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
        shared::{
            errors::{EvaluationResult, RuntimeError},
            scanner::{scan, Token},
            streams::Streams,
            values::Value,
        },
        walker::{ast::Stmt, interpreter::Interpreter, parser::parse},
    };

    #[rstest]
    #[case("1;", Ok(Value::Number(1.0)))]
    #[case("\"foo\";", Ok(Value::String("foo".into())))]
    #[case("true;", Ok(Value::Boolean(true)))]
    #[case("false;", Ok(Value::Boolean(false)))]
    #[case("nil;", Ok(Value::Nil))]
    #[case("!true;", Ok(Value::Boolean(false)))]
    #[case("!false;", Ok(Value::Boolean(true)))]
    #[case("!1;", Ok(Value::Boolean(false)))]
    #[case("!\"foo\";", Ok(Value::Boolean(false)))]
    #[case("!nil;", Ok(Value::Boolean(true)))]
    #[case("1 + 2;", Ok(Value::Number(3.0)))]
    #[case("1 - 2;", Ok(Value::Number(-1.0)))]
    #[case("1 / 2;", Ok(Value::Number(0.5)))]
    #[case("2 * 2;", Ok(Value::Number(4.0)))]
    #[case("1 / 0;", Ok(Value::Number(f64::INFINITY)))]
    #[case("2 == 2;", Ok(Value::Boolean(true)))]
    #[case("2 != 2;", Ok(Value::Boolean(false)))]
    #[case("1 == 2;", Ok(Value::Boolean(false)))]
    #[case("1 != 2;", Ok(Value::Boolean(true)))]
    #[case("1 <= 2;", Ok(Value::Boolean(true)))]
    #[case("2 <= 2;", Ok(Value::Boolean(true)))]
    #[case("3 <= 2;", Ok(Value::Boolean(false)))]
    #[case("1 < 2;", Ok(Value::Boolean(true)))]
    #[case("2 < 2;", Ok(Value::Boolean(false)))]
    #[case("3 < 2;", Ok(Value::Boolean(false)))]
    #[case("1 >= 2;", Ok(Value::Boolean(false)))]
    #[case("2 >= 2;", Ok(Value::Boolean(true)))]
    #[case("3 >= 2;", Ok(Value::Boolean(true)))]
    #[case("1 > 2;", Ok(Value::Boolean(false)))]
    #[case("2 > 2;", Ok(Value::Boolean(false)))]
    #[case("3 > 2;", Ok(Value::Boolean(true)))]
    #[case("\"foo\" + \"bar\";", Ok(Value::String("foobar".into())))]
    #[case("\"foo\" + 1;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String + Number".into() }
    ))]
    #[case("1 + \"foo\";", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + String".into() }
    ))]
    #[case("1 + true;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + Boolean".into() }
    ))]
    #[case("1 + false;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + Boolean".into() }
    ))]
    #[case("1 + nil;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + Nil".into() }
    ))]
    #[case("\"foo\" > true;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String > Boolean".into() }
    ))]
    #[case("\"foo\" > \"bar\";", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String > String".into() }
    ))]
    #[case("\"foo\" > 1;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String > Number".into() }
    ))]
    #[case("\"foo\" > nil;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String > Nil".into() }
    ))]
    fn test_evaluate(#[case] source: &str, #[case] expected: EvaluationResult) {
        let mut streams = Streams::test();
        let mut interpreter = Interpreter::new(&mut streams);
        let tokens: Vec<Token> = scan(source).try_collect().expect("Failed to scan tokens");
        let stmt = parse(tokens.iter())
            .pop()
            .expect("Expected one statement")
            .expect("Failed to parse statement");
        if let Stmt::Expression { expr } = stmt {
            let result = interpreter.evaluate(&expr);
            assert_eq!(result, expected);
        } else {
            panic!("Expected expression statement");
        }
    }
}
