use std::fmt::Display;

use anyhow::{bail, Result};
use itertools::Itertools;
use strum_macros::{AsRefStr, IntoStaticStr};

use crate::shared::values::Value;

#[derive(Debug, AsRefStr, IntoStaticStr)]
pub enum OpCode {
    Constant { index: u32 }, // the size of this int controls how many constants a block can have
    Add,
    Subtract,
    Multiply,
    Divide,
    Negate,
    Return,
}

#[derive(Debug, Default)]
pub struct Chunk<'s> {
    pub code: Vec<OpCode>,
    pub constants: Vec<Value<'s>>,
    pub lines: Vec<u64>,
}

impl<'s> Chunk<'s> {
    #[allow(dead_code)]
    pub fn new(code: Vec<OpCode>, constants: Vec<Value<'s>>, lines: Vec<u64>) -> Result<Chunk> {
        if code.len() != lines.len() {
            bail!("Chunk code and lines must have same length, but they did not: len(code)={}, len(lines)={}", code.len(), lines.len())
        }

        Ok(Chunk {
            code,
            constants,
            lines,
        })
    }

    pub fn fmt_instruction(&self, offset: usize) -> Option<String> {
        let op = &self.code.get(offset)?;
        let line = self.lines.get(offset)?;

        let o = op.as_ref().to_ascii_uppercase();

        Some(match op {
            OpCode::Return => {
                format!("{offset:04} {line:04} {o}")
            }
            OpCode::Add => {
                format!("{offset:04} {line:04} {o}")
            }
            OpCode::Subtract => {
                format!("{offset:04} {line:04} {o}")
            }
            OpCode::Multiply => {
                format!("{offset:04} {line:04} {o}")
            }
            OpCode::Divide => {
                format!("{offset:04} {line:04} {o}")
            }
            OpCode::Negate => {
                format!("{offset:04} {line:04} {o}")
            }
            OpCode::Constant { index } => {
                format!(
                    "{offset:04} {line:04} {o} {:?}",
                    self.constants[*index as usize]
                )
            }
        })
    }
}

impl<'s> Display for Chunk<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}",
            (0..self.code.len())
                .map(|offset| self.fmt_instruction(offset).unwrap().to_string())
                .join("\n")
        )
    }
}
