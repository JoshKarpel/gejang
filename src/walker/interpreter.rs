use std::{
    borrow::Cow,
    cell::RefCell,
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

#[derive(Debug, Default)]
struct Environment<'s> {
    values: HashMap<Cow<'s, str>, Value<'s>>,
}

impl<'s> Environment<'s> {
    fn define(&mut self, name: Cow<'s, str>, value: Value<'s>) {
        self.values.insert(name, value);
    }

    fn get(&self, name: &Token<'s>) -> Option<Value<'s>> {
        self.values.get(name.lexeme).cloned() // TODO: clone means we can't mutate objects!
    }
}

#[derive(Debug)]
pub struct Interpreter<'s, 'io, I: Read, O: Write, E: Write> {
    environments: RefCell<Vec<Environment<'s>>>,
    streams: &'io RefCell<Streams<I, O, E>>,
}

impl<'s, 'io: 's, I: Read, O: Write, E: Write> Interpreter<'s, 'io, I, O, E> {
    pub fn new(streams: &'io RefCell<Streams<I, O, E>>) -> Self {
        Self {
            environments: RefCell::new(vec![Environment::default()]),
            streams,
        }
    }

    fn push_env(&self) {
        self.environments.borrow_mut().push(Environment::default());
    }

    fn pop_env(&self) {
        self.environments.borrow_mut().pop();
    }

    fn env_define(&self, name: &Token<'s>, value: Value<'s>) {
        self.environments
            .borrow_mut()
            .last_mut()
            .expect("Unexpectedly empty environment stack!")
            .define(name.lexeme.into(), value);
    }

    fn env_assign(&self, name: &Token<'s>, value: Value<'s>) -> EvaluationResult<'s> {
        self.environments
            .borrow_mut()
            .iter_mut()
            .find(|e| e.get(name).is_some())
            .map(|e| {
                e.define(name.lexeme.into(), value.clone());
                value
            })
            .ok_or_else(|| RuntimeError::UndefinedVariable {
                name: name.lexeme.to_string(),
            })
    }

    fn env_get(&self, name: &Token<'s>) -> EvaluationResult<'s> {
        self.environments
            .borrow()
            .iter()
            .rev()
            .find_map(|e| e.get(name))
            .ok_or_else(|| RuntimeError::UndefinedVariable {
                name: name.lexeme.to_string(),
            })
    }

    pub fn interpret(&'s self, statements: &'s [Stmt<'s>]) -> InterpretResult {
        for stmt in statements {
            self.execute(stmt)?;
        }

        Ok(())
    }

    pub fn execute(&'s self, stmt: &'s Stmt<'s>) -> InterpretResult {
        match stmt {
            Stmt::Block { stmts } => {
                self.push_env();
                for stmt in stmts {
                    self.execute(stmt)?
                }
                self.pop_env();
            }
            Stmt::Expression { expr } => {
                self.evaluate(expr)?;
            }
            Stmt::If {
                condition,
                then,
                els,
            } => {
                if self.evaluate(condition)?.is_truthy() {
                    self.execute(then)?
                } else if let Some(e) = els {
                    self.execute(e)?;
                }
            }
            Stmt::Print { expr } => {
                let value = self.evaluate(expr)?;
                writeln!(self.streams.borrow_mut().output, "{}", &value)
                    .map_err(|_| RuntimeError::PrintFailed)?;
            }
            Stmt::Var { name, initializer } => {
                let ival = if let Some(init) = initializer {
                    self.evaluate(init)?
                } else {
                    Value::Nil
                };

                self.env_define(name, ival);
            }
            Stmt::While { condition, body } => {
                while self.evaluate(condition)?.is_truthy() {
                    self.execute(body)?
                }
            }
        };

        Ok(())
    }

    pub fn evaluate(&'s self, expr: &'s Expr<'s>) -> EvaluationResult<'s> {
        Ok(match expr {
            Expr::Literal { value: token } => Value::from(&token.typ),
            Expr::Grouping { expr } => self.evaluate(expr)?,
            Expr::Unary { op, right } => {
                let eval_right = self.evaluate(right)?;

                match op.typ {
                    TokenType::Minus => match eval_right {
                        Value::Number(value) => Value::Number(-value),
                        _ => {
                            return Err(RuntimeError::Unimplemented {
                                msg: "Cannot negate non-number".into(),
                            })
                        }
                    },
                    TokenType::Bang => Value::Boolean(!eval_right.is_truthy()),
                    _ => unreachable!("Unary operator not implemented: {:?}", op),
                }
            }
            Expr::Binary { left, op, right } => {
                let eval_left = self.evaluate(left)?;
                let eval_right = self.evaluate(right)?;

                match (eval_left, op.typ, eval_right) {
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
                        Value::String(Cow::from(format!("{l}{r}")))
                    }
                    (l, TokenType::EqualEqual, r) => Value::Boolean(l == r),
                    (l, TokenType::BangEqual, r) => Value::Boolean(l != r),
                    // TODO: more specific errors!
                    (l, o, r) => {
                        return Err(RuntimeError::Unimplemented {
                            msg: format!(
                                "Binary operation not implemented: {} {o} {}",
                                l.as_ref(),
                                r.as_ref()
                            ),
                        })
                    }
                }
            }
            Expr::Logical { left, op, right } => {
                let l = self.evaluate(left)?;

                return match (l.is_truthy(), op.typ) {
                    (true, TokenType::Or) => Ok(l),
                    (false, TokenType::Or) => self.evaluate(right),
                    (true, TokenType::And) => self.evaluate(right),
                    (false, TokenType::And) => Ok(l),
                    _ => unreachable!("Unexpected logical result/operator"),
                };
            }
            Expr::Variable { name } => self.env_get(name)?,
            Expr::Assign { name, value } => self.env_assign(name, self.evaluate(value)?)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

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
    #[case("\"foo\" + 1;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String + Number".into() }))]
    #[case("1 + \"foo\";", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + String".into() }))]
    #[case("1 + true;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + Boolean".into() }))]
    #[case("1 + false;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + Boolean".into() }))]
    #[case("1 + nil;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + Nil".into() }))]
    #[case("\"foo\" > true;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String > Boolean".into() }))]
    #[case("\"foo\" > \"bar\";", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String > String".into() }))]
    #[case("\"foo\" > 1;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String > Number".into() }))]
    #[case("\"foo\" > nil;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String > Nil".into() }))]
    fn test_evaluate<'s>(#[case] source: &'s str, #[case] expected: EvaluationResult<'s>) {
        let streams = RefCell::new(Streams::test());
        let interpreter = Interpreter::new(&streams);
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
