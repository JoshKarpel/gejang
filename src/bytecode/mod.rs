use std::{io, io::Write};

use anyhow::Result;
use colored::Colorize;
use itertools::Itertools;

use crate::{
    bytecode::{ops::OpCode, virtual_machine::VirtualMachine},
    shared::scanner,
    walker::InterpreterError,
};

mod compiler;
mod ops;
mod virtual_machine;

pub fn exec(source: &str) -> Result<()> {
    interpret(source)?;

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

fn interpret(source: &str) -> Result<(), InterpreterError> {
    let (tokens, errors): (Vec<_>, Vec<_>) = scanner::scan(source).partition_result();

    if !errors.is_empty() {
        for e in errors {
            eprintln!("{}", e.to_string().red());
        }
        return Err(InterpreterError::Scanner);
    }

    let mut chunk = compiler::compile(tokens.iter()).map_err(|e| {
        eprintln!("{}", e.to_string().red());
        InterpreterError::Evaluation
    })?;

    chunk.write(OpCode::Return, 0); // TODO: Remove this

    println!("{}", chunk.to_string().dimmed());

    let mut vm = VirtualMachine::new();

    vm.interpret(&chunk, true)
        .map_err(|e| {
            eprintln!("{}", e.to_string().red());
            InterpreterError::Evaluation
        })
        .inspect(|x| println!("{:?}", x))?;

    Ok(())
}
