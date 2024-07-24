use std::convert::AsRef;
use strum_macros::{AsRefStr, IntoStaticStr};

fn main() {
    let chunk = Chunk {
        code: vec![OpCode::Constant { index: 0 }, OpCode::Return],
        constants: vec![1.0],
        lines: vec![1, 1],
    };
    chunk.disassemble("test chunk");
    interpret(&chunk);
}

#[derive(Debug, AsRefStr, IntoStaticStr)]
enum OpCode {
    Constant { index: u32 }, // the size of this int controls how many constants a block can have
    Return,
}

type Value = f64;

#[derive(Debug)]
struct Chunk {
    code: Vec<OpCode>,
    constants: Vec<Value>,
    lines: Vec<u64>,
}

impl Chunk {
    fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let ops = self.code.iter().zip(self.lines.iter()).enumerate();

        let mut prev_line = 0;

        for (offset, (op, line)) in ops {
            let l = if *line == prev_line {
                "   |".to_string()
            } else {
                format!("{line:04}")
            };
            prev_line = *line;
            let o = op.as_ref().to_ascii_uppercase();

            match op {
                OpCode::Return => {
                    println!("{offset:04} {l:04} {o}");
                }
                OpCode::Constant { index } => {
                    println!(
                        "{offset:04} {l:04} {o} {:?}",
                        self.constants[*index as usize]
                    );
                }
            }
        }
    }
}

enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

fn interpret(chunk: &Chunk) -> InterpretResult {
    let ip = 0;

    loop {
        match chunk.code[ip] {
            OpCode::Return => {
                return InterpretResult::Ok;
            }
            OpCode::Constant { index } => {
                println!("{:?}", chunk.constants[index as usize]);
            }
        }
    }
}
