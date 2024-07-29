mod interpret;
mod ops;

use interpret::VirtualMachine;
use ops::{Chunk, OpCode};

pub fn run() {
    let chunk = Chunk {
        code: vec![
            OpCode::Constant { index: 0 },
            OpCode::Constant { index: 1 },
            OpCode::Return,
        ],
        constants: vec![1.0, 2.0],
        lines: vec![1, 2, 3],
    };
    // chunk.disassemble("test chunk");
    let mut vm = VirtualMachine::new();
    vm.interpret(&chunk, true);
}
