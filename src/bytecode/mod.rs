mod interpret;
mod ops;

use interpret::interpret;
use ops::{Chunk, OpCode};

pub fn run() {
    let chunk = Chunk {
        code: vec![OpCode::Constant { index: 0 }, OpCode::Return],
        constants: vec![1.0],
        lines: vec![1, 1],
    };
    chunk.disassemble("test chunk");
    interpret(&chunk);
}
