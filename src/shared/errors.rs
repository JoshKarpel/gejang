use thiserror::Error;

use crate::shared::values::Value;

#[derive(Error, Clone, Debug, PartialEq)]
pub enum RuntimeError {
    #[error("{msg}")]
    Unimplemented { msg: String },
    #[error("Invalid instruction pointer: {ip}")]
    InvalidInstructionPointer { ip: usize },
    #[error("Print failed")]
    PrintFailed,
    #[error("Undefined variable {name}")]
    UndefinedVariable { name: String },
}

pub type InterpretResult = Result<(), RuntimeError>;
pub type EvaluationResult<'s> = Result<Value<'s>, RuntimeError>;
