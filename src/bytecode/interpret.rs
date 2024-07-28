use crate::bytecode::ops::{Chunk, OpCode};

pub enum InterpretResult {
    Ok,
    // CompileError,
    // RuntimeError,
}

pub fn interpret(chunk: &Chunk) -> InterpretResult {
    let mut ip = 0;

    loop {
        match chunk.code[ip] {
            OpCode::Return => {
                return InterpretResult::Ok;
            }
            OpCode::Constant { index } => {
                println!("{:?}", chunk.constants[index as usize]);
                ip += 1;
            }
        }
    }
}
