use std::{
    borrow::Cow,
    cell::RefCell,
    collections::HashMap,
    io::{Read, Write},
    ops::{Deref, DerefMut},
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use thiserror::Error;

use crate::{
    shared::{scanner::TokenType, streams::Streams},
    walker::{
        ast::{Expr, Stmt},
        resolver::Locals,
        values::Value,
    },
};

pub type LoxPointer<'s> = Rc<RefCell<Value<'s>>>;

impl<'s> From<Value<'s>> for LoxPointer<'s> {
    fn from(value: Value<'s>) -> Self {
        Rc::new(RefCell::new(value))
    }
}

#[derive(Error, Clone, Debug, PartialEq)]
pub enum RuntimeError<'s> {
    #[error("{msg}")]
    Unimplemented { msg: String },
    #[error("Print failed")]
    PrintFailed,
    #[error("Undefined variable {name}")]
    UndefinedVariable { name: String },
    #[error("Value of type {typ} is not callable")]
    NotCallable { typ: String },
    #[error("Wrong number of arguments: expected {arity}, got {got}")]
    WrongNumberOfArgs { arity: usize, got: usize },
    #[error("Only instances have attributes")]
    OnlyInstancesHaveAttributes,
    #[error("Returning")]
    Return { value: LoxPointer<'s> },
    #[error("Breaking loop")]
    Break,
}

pub type InterpretResult<'s> = Result<(), RuntimeError<'s>>;
pub type EvaluationResult<'s> = Result<LoxPointer<'s>, RuntimeError<'s>>;

#[derive(Debug, Clone, Default, PartialEq)]
struct Environment<'s> {
    values: HashMap<Cow<'s, str>, LoxPointer<'s>>,
}

impl<'s> Environment<'s> {
    fn global() -> Self {
        let mut e = Self::default();

        e.define(
            Cow::from("clock"),
            Value::NativeFunction {
                name: "clock",
                arity: 0,
                f: |_| {
                    let now = SystemTime::now();
                    Value::Number(
                        now.duration_since(UNIX_EPOCH)
                            .expect("Are you living in the past?")
                            .as_secs_f64(),
                    )
                    .into()
                },
            }
            .into(),
        );

        e.define(
            Cow::from("tsp2cup"),
            Value::NativeFunction {
                name: "tsp2cup",
                arity: 1,
                f: |args| {
                    let tsp = match args.first().expect("Missing argument").borrow().deref() {
                        Value::Number(v) => *v,
                        _ => unreachable!(),
                    };

                    Value::Number(tsp / 48.0).into()
                },
            }
            .into(),
        );

        e
    }

    fn define(&mut self, name: Cow<'s, str>, value: LoxPointer<'s>) {
        self.values.insert(name, value);
    }

    fn get(&self, name: &Cow<'s, str>) -> Option<&LoxPointer<'s>> {
        self.values.get(name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnvironmentStack<'s>(Vec<Rc<RefCell<Environment<'s>>>>);

impl<'s> EnvironmentStack<'s> {
    fn global() -> Self {
        EnvironmentStack(vec![Rc::new(RefCell::new(Environment::global()))])
    }

    fn push(&mut self) {
        self.0.push(Rc::new(RefCell::new(Environment::default())))
    }

    fn pop(&mut self) {
        self.0.pop();
    }

    fn define(&self, name: Cow<'s, str>, value: LoxPointer<'s>) {
        self.0
            .last()
            .expect("Empty environment stack")
            .borrow_mut()
            .define(name, value);
    }

    fn assign(
        &self,
        name: &Cow<'s, str>,
        value: LoxPointer<'s>,
        depth: Option<&usize>,
    ) -> EvaluationResult<'s> {
        if let Some(&d) = depth {
            self.0
                .get(d + 1)
                .expect("Environment lookup resolved to missing depth during assignment")
                .borrow_mut()
                .define(name.clone(), value.clone())
        } else {
            self.0
                .first()
                .expect("Environment stack was unexpectedly empty")
                .borrow_mut()
                .define(name.clone(), value.clone())
        }
        Ok(value)
    }

    fn get(&self, name: &Cow<'s, str>, depth: Option<&usize>) -> EvaluationResult<'s> {
        let v: Option<LoxPointer<'s>> = if let Some(&d) = depth {
            println!("Looking up at depth {} on {}", d, name);
            self.0
                .get(d + 1)
                .expect("Environment lookup resolved to missing depth during lookup")
        } else {
            self.0
                .first()
                .expect("Environment stack was unexpectedly empty")
        }
        .borrow()
        .get(name)
        .cloned();
        v.ok_or_else(|| RuntimeError::UndefinedVariable {
            name: name.to_string(),
        })
    }
}

#[derive(Debug)]
pub struct Interpreter<'s, 'io, I: Read, O: Write, E: Write> {
    environments: RefCell<EnvironmentStack<'s>>,
    streams: &'io RefCell<Streams<I, O, E>>,
    locals: Locals<'s>,
}

impl<'s, 'io: 's, I: Read, O: Write, E: Write> Interpreter<'s, 'io, I, O, E> {
    pub fn new(streams: &'io RefCell<Streams<I, O, E>>, locals: Locals<'s>) -> Self {
        Self {
            environments: EnvironmentStack::global().into(),
            streams,
            locals,
        }
    }

    pub fn interpret(&'s self, statements: &'s [Stmt<'s>]) -> InterpretResult<'s> {
        for stmt in statements {
            self.execute(stmt)?;
        }

        Ok(())
    }

    pub fn execute(&'s self, stmt: &'s Stmt<'s>) -> InterpretResult<'s> {
        match stmt {
            Stmt::Block { stmts } => {
                self.environments.borrow_mut().push();
                for stmt in stmts {
                    self.execute(stmt)?
                }
                self.environments.borrow_mut().pop();
            }
            Stmt::Expression { expr } => {
                self.evaluate(expr)?;
            }
            Stmt::Function { name, params, body } => self.environments.borrow().define(
                Cow::from(name.lexeme),
                Value::Function {
                    name: name.lexeme,
                    params: params.iter().map(|p| p.lexeme).collect(),
                    body,
                    closure: self.environments.borrow().clone(),
                }
                .into(),
            ),
            Stmt::Class { name, methods } => {
                self.environments
                    .borrow()
                    .define(Cow::from(name.lexeme), Value::Nil.into());
                let methods = methods
                    .iter()
                    .map(|m| {
                        if let Stmt::Function { name, params, body } = m {
                            (
                                Cow::from(name.lexeme),
                                Value::Function {
                                    name: name.lexeme,
                                    params: params.iter().map(|p| p.lexeme).collect(),
                                    body,
                                    closure: self.environments.borrow().clone(),
                                }
                                .into(),
                            )
                        } else {
                            unreachable!("Class method not a function")
                        }
                    })
                    .collect();
                let cls = Value::Class {
                    name: name.lexeme,
                    methods,
                };
                self.environments
                    .borrow()
                    .define(Cow::from(name.lexeme), cls.into());
            }
            Stmt::If {
                condition,
                then,
                els,
            } => {
                if self.evaluate(condition)?.borrow().is_truthy() {
                    self.execute(then)?
                } else if let Some(e) = els {
                    self.execute(e)?;
                }
            }
            Stmt::Print { expr } => {
                let value = self.evaluate(expr)?;
                writeln!(self.streams.borrow_mut().output, "{}", &value.borrow())
                    .map_err(|_| RuntimeError::PrintFailed)?;
            }
            Stmt::Var { name, initializer } => {
                let ival = if let Some(init) = initializer {
                    self.evaluate(init)?
                } else {
                    Value::Nil.into()
                };

                self.environments.borrow().define(name.lexeme.into(), ival);
            }
            Stmt::While { condition, body } => {
                while self.evaluate(condition)?.borrow().is_truthy() {
                    let r = self.execute(body);
                    if let Err(RuntimeError::Break) = r {
                        break;
                    } else if let e @ Err(_) = r {
                        return e;
                    }
                }
            }
            Stmt::Return { value } => {
                let v = if let Some(e) = value {
                    self.evaluate(e)?
                } else {
                    Value::Nil.into()
                };

                return Err(RuntimeError::Return { value: v });
            }
            Stmt::Break => return Err(RuntimeError::Break),
        };

        Ok(())
    }

    pub fn evaluate(&'s self, expr: &'s Expr<'s>) -> EvaluationResult<'s> {
        Ok(match expr {
            Expr::Literal { value: token } => Value::from(&token.typ).into(),
            Expr::Grouping { expr } => self.evaluate(expr)?,
            Expr::Unary { op, right } => {
                let eval_right = self.evaluate(right)?;

                match op.typ {
                    TokenType::Minus => match eval_right.borrow().deref() {
                        Value::Number(value) => Value::Number(-value).into(),
                        _ => {
                            return Err(RuntimeError::Unimplemented {
                                msg: "Cannot negate non-number".into(),
                            })
                        }
                    },
                    TokenType::Bang => {
                        Value::Boolean(!eval_right.borrow().deref().is_truthy()).into()
                    }
                    _ => unreachable!("Unary operator not implemented: {:?}", op),
                }
            }
            Expr::Binary { left, op, right } => {
                let eval_left = self.evaluate(left)?;
                let eval_right = self.evaluate(right)?;

                let x = match (
                    eval_left.borrow().deref(),
                    op.typ,
                    eval_right.borrow().deref(),
                ) {
                    (Value::Number(l), TokenType::Plus, Value::Number(r)) => {
                        Value::Number(l + r).into()
                    }
                    (Value::Number(l), TokenType::Minus, Value::Number(r)) => {
                        Value::Number(l - r).into()
                    }
                    (Value::Number(l), TokenType::Star, Value::Number(r)) => {
                        Value::Number(l * r).into()
                    }
                    (Value::Number(l), TokenType::Slash, Value::Number(r)) => {
                        Value::Number(l / r).into()
                    }
                    (Value::Number(l), TokenType::Greater, Value::Number(r)) => {
                        Value::Boolean(l > r).into()
                    }
                    (Value::Number(l), TokenType::GreaterEqual, Value::Number(r)) => {
                        Value::Boolean(l >= r).into()
                    }
                    (Value::Number(l), TokenType::Less, Value::Number(r)) => {
                        Value::Boolean(l < r).into()
                    }
                    (Value::Number(l), TokenType::LessEqual, Value::Number(r)) => {
                        Value::Boolean(l <= r).into()
                    }
                    (Value::String(ref l), TokenType::Plus, Value::String(ref r)) => {
                        Value::String(Cow::from(format!("{l}{r}"))).into()
                    }
                    (l, TokenType::EqualEqual, r) => Value::Boolean(l == r).into(),
                    (l, TokenType::BangEqual, r) => Value::Boolean(l != r).into(),
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
                };
                x
            }
            Expr::Logical { left, op, right } => {
                let l = self.evaluate(left)?;

                return match (l.clone().borrow().is_truthy(), op.typ) {
                    (true, TokenType::Or) => Ok(l),
                    (false, TokenType::Or) => self.evaluate(right),
                    (true, TokenType::And) => self.evaluate(right),
                    (false, TokenType::And) => Ok(l),
                    _ => unreachable!("Unexpected logical result/operator"),
                };
            }
            Expr::Variable { name } => self
                .environments
                .borrow()
                .get(&Cow::from(name.lexeme), self.locals.get(expr))?,
            Expr::Assign { name, value } => self.environments.borrow().assign(
                &Cow::from(name.lexeme),
                self.evaluate(value)?,
                self.locals.get(expr),
            )?,
            Expr::Set {
                object,
                name,
                value,
            } => {
                let o = self.evaluate(object)?;

                let x = if let Value::Instance { ref mut fields, .. } = o.borrow_mut().deref_mut() {
                    let v = self.evaluate(value)?;
                    fields.insert(Cow::from(name.lexeme), v.clone());
                    v
                } else {
                    return Err(RuntimeError::OnlyInstancesHaveAttributes);
                };
                x
            }
            Expr::Call { callee, args } => {
                let c = self.evaluate(callee)?;

                let a = args
                    .iter()
                    .map(|arg| self.evaluate(arg))
                    .collect::<Result<Vec<LoxPointer>, RuntimeError>>()?;

                let num_args = a.len();

                let r = match c.borrow().deref().clone() {
                    // TODO: clone here is weird
                    Value::NativeFunction { name: _, f, arity } => {
                        if num_args != arity {
                            return Err(RuntimeError::WrongNumberOfArgs {
                                arity,
                                got: num_args,
                            });
                        }

                        Ok(f(&a))
                    }
                    Value::Function {
                        name: _,
                        params,
                        body,
                        closure,
                    } => {
                        let num_params = params.len();
                        if num_args != num_params {
                            return Err(RuntimeError::WrongNumberOfArgs {
                                arity: num_params,
                                got: num_args,
                            });
                        };

                        let old_env = self.environments.replace(closure);

                        self.environments.borrow_mut().push();

                        a.iter().zip(params.iter()).for_each(|(arg, &param)| {
                            self.environments
                                .borrow()
                                .define(Cow::from(param), arg.clone()) // TODO another clone
                        });

                        let rv = self.interpret(body);

                        self.environments.borrow_mut().pop(); // must pop the env whether we succeeded or failed, to handle returns

                        self.environments.replace(old_env);

                        rv.map(|_| Value::Nil.into())
                    }
                    ref class @ Value::Class { ref methods, .. } => {
                        let instance: LoxPointer = Value::Instance {
                            class: Box::new(class.clone().into()), // TODO: this seems wrong, should be able to use original Rc
                            fields: methods.clone(),
                        }
                        .into();

                        if let Some(init) = methods.get("init") {
                            if let Value::Function {
                                name: _,
                                params,
                                body,
                                closure,
                            } = init.borrow().deref()
                            {
                                let num_params = params.len();
                                if num_args != num_params {
                                    return Err(RuntimeError::WrongNumberOfArgs {
                                        arity: num_params,
                                        got: num_args,
                                    });
                                };

                                let old_env = self.environments.replace(closure.clone());

                                self.environments.borrow_mut().push();

                                self.environments
                                    .borrow()
                                    .define(Cow::from("this"), instance.clone());

                                self.environments.borrow_mut().push();

                                a.iter().zip(params.iter()).for_each(|(arg, &param)| {
                                    self.environments
                                        .borrow()
                                        .define(Cow::from(param), arg.clone()) // TODO another clone
                                });

                                self.interpret(body)?;

                                self.environments.borrow_mut().pop();
                                self.environments.borrow_mut().pop();

                                self.environments.replace(old_env);
                            }
                        }

                        Ok(instance)
                    }
                    _ => Err(RuntimeError::NotCallable {
                        typ: c.borrow().to_string(),
                    }),
                };

                match r {
                    Ok(v) => v,
                    Err(RuntimeError::Return { value }) => value,
                    Err(e) => return Err(e),
                }
            }
            Expr::Get { object, name } => {
                let o = self.evaluate(object)?;
                let x = if let Value::Instance {
                    fields: attributes, ..
                } = o.borrow().deref()
                {
                    attributes
                        .get(&Cow::from(name.lexeme))
                        .cloned()
                        .ok_or_else(|| RuntimeError::UndefinedVariable {
                            name: name.lexeme.to_string(),
                        })?
                } else {
                    return Err(RuntimeError::OnlyInstancesHaveAttributes);
                };
                if let Value::Function {
                    name,
                    params,
                    body,
                    closure,
                } = x.borrow().deref()
                {
                    let mut closure_with_this = closure.clone();
                    closure_with_this.push();
                    if let Some(e) = closure_with_this.0.last_mut() {
                        e.borrow_mut().define(Cow::from("this"), o.clone());
                    }
                    return Ok(Value::Function {
                        name,
                        params: params.clone(),
                        body,
                        closure: closure_with_this,
                    }
                    .into());
                }
                x
            }
            Expr::This { keyword } => self
                .environments
                .borrow()
                .get(&Cow::from(keyword.lexeme), self.locals.get(expr))?,
        })
    }
}

#[cfg(test)]
mod tests {

    // TODO: FIX!
    // #[rstest]
    // #[case("1;", Ok(Value::Number(1.0)))]
    // #[case("\"foo\";", Ok(Value::String("foo".into())))]
    // #[case("true;", Ok(Value::Boolean(true)))]
    // #[case("false;", Ok(Value::Boolean(false)))]
    // #[case("nil;", Ok(Value::Nil))]
    // #[case("!true;", Ok(Value::Boolean(false)))]
    // #[case("!false;", Ok(Value::Boolean(true)))]
    // #[case("!1;", Ok(Value::Boolean(false)))]
    // #[case("!\"foo\";", Ok(Value::Boolean(false)))]
    // #[case("!nil;", Ok(Value::Boolean(true)))]
    // #[case("1 + 2;", Ok(Value::Number(3.0)))]
    // #[case("1 - 2;", Ok(Value::Number(-1.0)))]
    // #[case("1 / 2;", Ok(Value::Number(0.5)))]
    // #[case("2 * 2;", Ok(Value::Number(4.0)))]
    // #[case("1 / 0;", Ok(Value::Number(f64::INFINITY)))]
    // #[case("2 == 2;", Ok(Value::Boolean(true)))]
    // #[case("2 != 2;", Ok(Value::Boolean(false)))]
    // #[case("1 == 2;", Ok(Value::Boolean(false)))]
    // #[case("1 != 2;", Ok(Value::Boolean(true)))]
    // #[case("1 <= 2;", Ok(Value::Boolean(true)))]
    // #[case("2 <= 2;", Ok(Value::Boolean(true)))]
    // #[case("3 <= 2;", Ok(Value::Boolean(false)))]
    // #[case("1 < 2;", Ok(Value::Boolean(true)))]
    // #[case("2 < 2;", Ok(Value::Boolean(false)))]
    // #[case("3 < 2;", Ok(Value::Boolean(false)))]
    // #[case("1 >= 2;", Ok(Value::Boolean(false)))]
    // #[case("2 >= 2;", Ok(Value::Boolean(true)))]
    // #[case("3 >= 2;", Ok(Value::Boolean(true)))]
    // #[case("1 > 2;", Ok(Value::Boolean(false)))]
    // #[case("2 > 2;", Ok(Value::Boolean(false)))]
    // #[case("3 > 2;", Ok(Value::Boolean(true)))]
    // #[case("\"foo\" + \"bar\";", Ok(Value::String("foobar".into())))]
    // #[case("\"foo\" + 1;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String + Number".into() }
    // ))]
    // #[case("1 + \"foo\";", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + String".into() }
    // ))]
    // #[case("1 + true;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + Boolean".into() }
    // ))]
    // #[case("1 + false;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + Boolean".into() }
    // ))]
    // #[case("1 + nil;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: Number + Nil".into() }
    // ))]
    // #[case("\"foo\" > true;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String > Boolean".into() }
    // ))]
    // #[case("\"foo\" > \"bar\";", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String > String".into() }
    // ))]
    // #[case("\"foo\" > 1;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String > Number".into() }
    // ))]
    // #[case("\"foo\" > nil;", Err(RuntimeError::Unimplemented { msg: "Binary operation not implemented: String > Nil".into() }
    // ))]
    // fn test_evaluate<'s>(#[case] source: &'s str, #[case] expected: EvaluationResult<'s>) {
    //     let streams = RefCell::new(Streams::test());
    //     let interpreter = Interpreter::new(&streams);
    //     let tokens: Vec<Token> = scan(source).try_collect().expect("Failed to scan tokens");
    //     let stmt = parse(tokens.iter())
    //         .pop()
    //         .expect("Expected one statement")
    //         .expect("Failed to parse statement");
    //     if let Stmt::Expression { ref expr } = stmt {
    //         let result = interpreter.evaluate(expr);
    //         assert_eq!(result, expected);
    //     } else {
    //         panic!("Expected expression statement");
    //     }
    // }
}
