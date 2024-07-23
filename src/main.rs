use std::convert::AsRef;
use strum_macros::{AsRefStr, IntoStaticStr};

fn main() {
    let chunk = Chunk {
        code: vec![OpCode::Return],
    };
    chunk.disassemble("test chunk");
}

#[derive(Debug, AsRefStr, IntoStaticStr)]
enum OpCode {
    Return,
}

struct Chunk {
    code: Vec<OpCode>,
}

impl Chunk {
    fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let ops = self.code.iter().enumerate();

        for (offset, op) in ops {
            match op {
                OpCode::Return => {
                    println!("{:04} {}", offset, op.as_ref());
                }
            }
        }
    }
}
