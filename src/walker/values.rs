use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::{Debug, Display},
};

use strum_macros::{AsRefStr, IntoStaticStr};

use crate::{shared::scanner::TokenType, walker::ast::Stmt};

#[derive(Debug, Clone, PartialEq, AsRefStr, IntoStaticStr)]
pub enum Value<'s> {
    // Is it really worth bringing those strings all the way from the source to here?
    #[allow(dead_code)]
    Object(HashMap<Cow<'s, str>, Value<'s>>),
    Number(f64),
    String(Cow<'s, str>),
    Boolean(bool),
    Nil,
    NativeFunction {
        name: &'static str,
        arity: usize,
        f: fn(&[Value<'s>]) -> Value<'s>,
    },
    Function {
        name: &'s str,
        params: Vec<&'s str>,
        body: &'s Vec<Stmt<'s>>,
    },
}

impl<'s> From<&TokenType<'s>> for Value<'s> {
    fn from(token: &TokenType<'s>) -> Self {
        match token {
            TokenType::Number(value) => Value::Number(*value),
            TokenType::String(value) => Value::String(Cow::from(*value)),
            TokenType::True => Value::Boolean(true),
            TokenType::False => Value::Boolean(false),
            TokenType::Nil => Value::Nil,
            _ => unreachable!("Cannot get a literal value from token {:?}", token),
        }
    }
}

impl Value<'_> {
    pub fn is_truthy(&self) -> bool {
        match self {
            // TODO: implement Python-like truthiness
            Value::Boolean(value) => *value,
            Value::Nil => false,
            _ => true,
        }
    }
}

impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Value::Object(_) => "<object>".to_string(), // TODO: implement better object display
                Value::Number(value) => value.to_string(),
                Value::String(value) => value.to_string(),
                Value::Boolean(value) => value.to_string(),
                Value::Nil => "nil".to_string(),
                Value::NativeFunction { name, f: _, arity } =>
                    format!("<native fn {name}/{arity}>"),
                Value::Function {
                    name,
                    params,
                    body: _,
                } => format!("<fn {}/{}>", name, params.len()),
            }
        )
    }
}
