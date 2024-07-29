use anyhow::Result;

mod interpret;
mod ops;

use interpret::VirtualMachine;
use ops::{Chunk, OpCode};

pub fn run() -> Result<()> {
    let chunk = Chunk::new(
        vec![
            OpCode::Constant { index: 0 },
            OpCode::Constant { index: 1 },
            OpCode::Negate,
            OpCode::Add,
            OpCode::Return,
        ],
        vec![1.0, 2.0],
        vec![1, 2, 3, 4, 4],
    )?;
    let mut vm = VirtualMachine::new();
    vm.interpret(&chunk, true)?;

    Ok(())
}
