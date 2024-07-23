use std::convert::AsRef;
use strum_macros::{AsRefStr, IntoStaticStr};

fn main() {
    let chunk = Chunk {
        code: vec![OpCode::Constant { index: 0 }, OpCode::Return],
        constants: vec![1.0],
        lines: vec![1, 2],
    };
    chunk.disassemble("test chunk");
}

#[derive(Debug, AsRefStr, IntoStaticStr)]
enum OpCode {
    Constant { index: u8 },
    Return,
}

type Value = f64;
struct Chunk {
    code: Vec<OpCode>,
    constants: Vec<Value>,
    lines: Vec<u64>,
}

impl Chunk {
    fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let ops = self.code.iter().zip(self.lines.iter()).enumerate();

        for (offset, (op, line)) in ops {
            let l = line;
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
