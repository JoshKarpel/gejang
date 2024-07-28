use strum_macros::{AsRefStr, IntoStaticStr};

#[derive(Debug, AsRefStr, IntoStaticStr)]
pub enum OpCode {
    Constant { index: u32 }, // the size of this int controls how many constants a block can have
    Return,
}

pub type Value = f64;

#[derive(Debug)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: Vec<Value>,
    pub lines: Vec<u64>,
}

impl Chunk {
    pub fn disassemble(&self, name: &str) {
        println!("== {} ==", name);

        let ops = self.code.iter().zip(self.lines.iter()).enumerate();

        for (offset, (op, line)) in ops {
            let o = op.as_ref().to_ascii_uppercase();

            match op {
                OpCode::Return => {
                    println!("{offset:04} {line:04} {o}");
                }
                OpCode::Constant { index } => {
                    println!(
                        "{offset:04} {line:04} {o} {:?}",
                        self.constants[*index as usize]
                    );
                }
            }
        }
    }
}
