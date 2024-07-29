use crate::bytecode::ops::{Chunk, OpCode, Value};
use colored::Colorize;
use itertools::Itertools;

pub enum InterpretResult {
    Ok,
    // CompileError,
    // RuntimeError,
}

pub struct VirtualMachine {
    stack: Vec<Value>, // Book uses a fixed-size stack
}

impl VirtualMachine {
    pub fn new() -> Self {
        VirtualMachine { stack: Vec::new() }
    }

    pub fn interpret(&mut self, chunk: &Chunk, trace: bool) -> InterpretResult {
        let mut ip = 0;

        loop {
            if trace {
                println!(
                    "{}",
                    format!(
                        "┌─ {}{}\n└──────────────────────",
                        chunk.fmt_instruction(ip).unwrap(),
                        {
                            let s = self
                                .stack
                                .iter()
                                .enumerate()
                                .rev()
                                .map(|(s, v)| format!("{s} -> {v}"))
                                .join("\n│ ");

                            if s.is_empty() {
                                s
                            } else {
                                format!("\n│ {}", s)
                            }
                        }
                    )
                    .dimmed()
                );
            }

            match chunk.code[ip] {
                OpCode::Return => {
                    println!("{}", self.stack.pop().unwrap());
                    return InterpretResult::Ok;
                }
                OpCode::Constant { index } => {
                    ip += 1;
                    self.stack.push(chunk.constants[index as usize]);
                }
            }
        }
    }
}
