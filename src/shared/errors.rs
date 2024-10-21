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
    #[error("Value of type {typ} is not callable")]
    NotCallable { typ: String },
    #[error("Wrong number of arguments: expected {arity}, got {got}")]
    WrongNumberOfArgs { arity: u8, got: usize },
}

pub type InterpretResult = Result<(), RuntimeError>;
pub type EvaluationResult<'s> = Result<Value<'s>, RuntimeError>;
