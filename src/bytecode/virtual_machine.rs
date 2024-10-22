use colored::Colorize;
use itertools::Itertools;
use thiserror::Error;

use crate::bytecode::{
    ops::{Chunk, OpCode},
    values::Value,
};

#[derive(Error, Clone, Debug, PartialEq)]
pub enum RuntimeError {
    #[error("{msg}")]
    Unimplemented { msg: String },
    #[error("Invalid instruction pointer: {ip}")]
    InvalidInstructionPointer { ip: usize },
}

pub type EvaluationResult<'s> = Result<Value<'s>, RuntimeError>;

pub struct VirtualMachine<'s> {
    #[allow(dead_code)]
    stack: Vec<Value<'s>>, // Book uses a fixed-size stack
}

impl<'s> VirtualMachine<'s> {
    pub fn new() -> Self {
        VirtualMachine { stack: Vec::new() }
    }

    #[allow(dead_code)]
    pub fn interpret(&mut self, chunk: &Chunk<'s>, trace: bool) -> EvaluationResult<'s> {
        let mut ip = 0;

        loop {
            if trace {
                let instruction = chunk
                    .fmt_instruction(ip)
                    .ok_or(RuntimeError::InvalidInstructionPointer { ip })?;
                let stack = {
                    let s = self
                        .stack
                        .iter()
                        .enumerate()
                        .rev()
                        .map(|(s, v)| format!("{s} -> {v:?}"))
                        .join("\n│ ");

                    // Fix the prefix for the first line
                    if s.is_empty() {
                        s
                    } else {
                        format!("\n│ {}", s)
                    }
                };
                println!(
                    "{}",
                    format!("┌─ {}{}\n└──────────────────────", instruction, stack).dimmed()
                );
            }

            match chunk.code[ip] {
                OpCode::Return => {
                    return Ok(self.stack.pop().expect("Popped from empty stack"));
                }
                OpCode::Add => {
                    let b = self.stack.pop().expect("Popped from empty stack");
                    let a = self.stack.pop().expect("Popped from empty stack");
                    self.stack.push(match (&a, &b) {
                        (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
                        _ => {
                            return Err(RuntimeError::Unimplemented {
                                msg: format!("Binary operation not implemented: {:?} + {:?}", a, b),
                            })
                        }
                    });
                    ip += 1;
                }
                OpCode::Subtract => {
                    let b = self.stack.pop().expect("Popped from empty stack");
                    let a = self.stack.pop().expect("Popped from empty stack");
                    self.stack.push(match (&a, &b) {
                        (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
                        _ => {
                            return Err(RuntimeError::Unimplemented {
                                msg: format!("Binary operation not implemented: {:?} - {:?}", a, b),
                            })
                        }
                    });
                    ip += 1;
                }
                OpCode::Multiply => {
                    let b = self.stack.pop().expect("Popped from empty stack");
                    let a = self.stack.pop().expect("Popped from empty stack");
                    self.stack.push(match (&a, &b) {
                        (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
                        _ => {
                            return Err(RuntimeError::Unimplemented {
                                msg: format!("Binary operation not implemented: {:?} * {:?}", a, b),
                            })
                        }
                    });
                    ip += 1;
                }
                OpCode::Divide => {
                    let b = self.stack.pop().expect("Popped from empty stack");
                    let a = self.stack.pop().expect("Popped from empty stack");
                    self.stack.push(match (&a, &b) {
                        (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
                        _ => {
                            return Err(RuntimeError::Unimplemented {
                                msg: format!("Binary operation not implemented: {:?} / {:?}", a, b),
                            })
                        }
                    });
                    ip += 1;
                }

                OpCode::Negate => {
                    let value = self.stack.pop().expect("Popped from empty stack");
                    self.stack.push(match value {
                        Value::Number(ref value) => Value::Number(-value),
                        _ => {
                            return Err(RuntimeError::Unimplemented {
                                msg: "Cannot negate non-number".into(),
                            })
                        }
                    });
                    ip += 1;
                }
                OpCode::Constant { index } => {
                    self.stack.push(chunk.constants[index].clone()); // TODO: clone here, can we use COW?
                    ip += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {}
