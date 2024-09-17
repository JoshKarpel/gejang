use std::{borrow::Cow, collections::HashMap};

use strum_macros::{AsRefStr, IntoStaticStr};

use crate::shared::scanner::TokenType;

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
