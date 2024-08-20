use anyhow::{anyhow, Result};
use colored::Colorize;
use itertools::Itertools;

use crate::bytecode::ops::{Chunk, OpCode, Value};

pub struct VirtualMachine {
    stack: Vec<Value>, // Book uses a fixed-size stack
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine { stack: Vec::new() }
    }

    pub fn interpret(&mut self, chunk: &Chunk, trace: bool) -> Result<()> {
        let mut ip = 0;

        loop {
            if trace {
                let instruction = chunk
                    .fmt_instruction(ip)
                    .ok_or(anyhow!("Invalid instruction pointer: {ip}"))?;
                let stack = {
                    let s = self
                        .stack
                        .iter()
                        .enumerate()
                        .rev()
                        .map(|(s, v)| format!("{s} -> {v}"))
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
                    println!("{}", self.stack.pop().unwrap());
                    return Ok(());
                }
                OpCode::Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a + b);
                    ip += 1;
                }
                OpCode::Subtract => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a - b);
                    ip += 1;
                }
                OpCode::Multiply => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a * b);
                    ip += 1;
                }
                OpCode::Divide => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(a / b);
                    ip += 1;
                }

                OpCode::Negate => {
                    let value = self.stack.pop().unwrap();
                    self.stack.push(-value);
                    ip += 1;
                }
                OpCode::Constant { index } => {
                    self.stack.push(chunk.constants[index]);
                    ip += 1;
                }
            }
        }
    }
}
