use std::{io, io::Write, path::Path};

use anyhow::Result;

use crate::bytecode::interpret::VirtualMachine;

mod compiler;
mod interpret;
mod ops;

pub fn script(path: &Path) -> Result<()> {
    let source = std::fs::read_to_string(path)?;

    interpret(&source)?;

    Ok(())
}

pub fn repl() -> Result<()> {
    println!("Gejang VM REPL");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("🦀> ");
        stdout.flush().unwrap();
        let mut buffer = String::new();
        stdin.read_line(&mut buffer)?;

        match interpret(&buffer) {
            Ok(_) => {}
            Err(e) => eprintln!("{e}"),
        }
    }
}

fn interpret(source: &str) -> Result<()> {
    compiler::compile(source)?;

    let _ = VirtualMachine::new();

    Ok(())
}
