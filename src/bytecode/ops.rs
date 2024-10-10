use std::fmt::Display;

use anyhow::{bail, Result};
use itertools::Itertools;
use strum_macros::{AsRefStr, IntoStaticStr};

use crate::shared::values::Value;

#[derive(Debug, AsRefStr, IntoStaticStr)]
pub enum OpCode {
    Constant { index: usize },
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
    pub lines: Vec<usize>,
}

impl<'s> Chunk<'s> {
    #[allow(dead_code)]
    pub fn new(
        code: Vec<OpCode>,
        constants: Vec<Value<'s>>,
        lines: Vec<usize>,
    ) -> Result<Chunk<'s>> {
        if code.len() != lines.len() {
            bail!("Chunk code and lines must have same length, but they did not: len(code)={}, len(lines)={}", code.len(), lines.len())
        }

        Ok(Chunk {
            code,
            constants,
            lines,
        })
    }

    pub fn add_constant(&mut self, value: Value<'s>, line: usize) {
        self.constants.push(value);
        self.code.push(OpCode::Constant {
            index: self.constants.len() - 1,
        });
        self.lines.push(line)
    }

    pub fn write(&mut self, op: OpCode, line: usize) {
        self.code.push(op);
        self.lines.push(line);
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
                format!("{offset:04} {line:04} {o} {:?}", self.constants[*index])
            }
        })
    }
}

impl Display for Chunk<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            (0..self.code.len())
                .map(|offset| self.fmt_instruction(offset).unwrap().to_string())
                .join("\n")
        )
    }
}
