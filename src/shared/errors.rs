use thiserror::Error;

use crate::shared::values::Value;

#[derive(Error, Clone, Debug, PartialEq)]
pub enum RuntimeError {
    #[error("{msg}")]
    Unimplemented { msg: String },
    #[error("Invalid instruction pointer: {ip}")]
    InvalidInstructionPointer { ip: usize },
}

pub type EvaluationResult<'s> = Result<Value<'s>, RuntimeError>;
