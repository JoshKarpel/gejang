use std::{
    borrow::Cow,
    collections::HashMap,
    io::{Read, Write},
};

use crate::{
    shared::{
        errors::{EvaluationResult, InterpretResult, RuntimeError},
        scanner::{Token, TokenType},
        streams::Streams,
        values::Value,
    },
    walker::ast::{Expr, Stmt},
};

#[derive(Debug)]
struct Environment<'s> {
    values: HashMap<Cow<'s, str>, Value<'s>>,
}

impl<'s> Environment<'s> {
    fn define(&mut self, name: Cow<'s, str>, value: Value<'s>) {
        self.values.insert(name, value);
    }

    fn get(&mut self, name: &Token<'s>) -> EvaluationResult {
        self.values
            .get(name.lexeme)
            .map(|v| v.clone()) // TODO: no! We need to be able to mutate objects
            .ok_or_else(|| RuntimeError::UndefinedVariable {
                name: name.lexeme.to_string(),
            })
    }
}

impl<'s> Default for Environment<'s> {
    fn default() -> Self {
        Self {
            values: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Interpreter<'s, 'io, I: Read, O: Write, E: Write> {
    environment: Environment<'s>,
    streams: &'io mut Streams<I, O, E>,
}

impl<'s, 'io, I: Read, O: Write, E: Write> Interpreter<'s, 'io, I, O, E> {
    pub fn new(streams: &'io mut Streams<I, O, E>) -> Self {
        Self {
            environment: Environment::default(),
            streams,
        }
    }

    pub fn interpret(&mut self, statements: &'s [Stmt<'s>]) -> InterpretResult {
        for stmt in statements {
            self.execute(stmt)?;
        }

        Ok(())
    }

    pub fn execute(&mut self, stmt: &'s Stmt<'s>) -> InterpretResult {
        match stmt {
            Stmt::Expression { expr } => self.evaluate(expr)?,
            Stmt::Print { expr } => {
                let value = self.evaluate(expr)?;
                write!(self.streams.output, "{}", &value).map_err(|_| RuntimeError::PrintFailed)?;
                value
            }
            Stmt::Var { name, initializer } => {
                let ival = if let Some(init) = initializer {
                    self.evaluate(init)?
                } else {
                    Value::Nil
                };

                self.environment.define(name.lexeme.into(), ival);
                Value::Nil
            }
        };

        Ok(())
    }

    pub fn evaluate(&mut self, expr: &'s Expr<'s>) -> EvaluationResult<'s> {
        Ok(match expr {
            Expr::Literal { value: token } => Value::from(&token.typ),
            Expr::Grouping { expr } => self.evaluate(expr)?,
            Expr::Unary { op, right } => {
                let eval_right = self.evaluate(right)?;
                let r = eval_right;

                match op.typ {
                    TokenType::Minus => match r {
                        Value::Number(value) => Value::Number(-value),
                        _ => {
                            return Err(RuntimeError::Unimplemented {
                                msg: "Cannot negate non-number".into(),
                            })
                        }
                    },
                    TokenType::Bang => Value::Boolean(!r.is_truthy()),
                    _ => unreachable!("Unary operator not implemented: {:?}", op),
                }
            }
            Expr::Binary { left, op, right } => {
                let eval_left = self.evaluate(left)?;
                let eval_right = self.evaluate(right)?;

                let result = match (eval_left, op.typ, eval_right) {
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
                            msg: format!("Binary operation not implemented: {l} {o} {r}"),
                        })
                    }
                };

                result
            }
            Expr::Variable { name } => self.environment.get(name)?,
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
    fn test_evaluate<'s>(#[case] source: &'s str, #[case] expected: EvaluationResult<'s>) {
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
