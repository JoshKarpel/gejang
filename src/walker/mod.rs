mod ast;
mod interpreter;
mod parser;

use std::{io, io::Write, path::Path};

use anyhow::Result;
use itertools::Itertools;
use thiserror::Error;

use crate::{shared::scanner, walker::interpreter::Interpreter};

pub fn script(path: &Path) -> Result<()> {
    let source = std::fs::read_to_string(path)?;

    interpret(&source)?;

    Ok(())
}

pub fn repl() -> Result<()> {
    println!("Gejang TW REPL");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("🦀> ");
        stdout.flush()?;
        let mut buffer = String::new();
        stdin.read_line(&mut buffer)?;

        match interpret(&buffer) {
            Ok(_) => {}
            Err(e) => eprintln!("{e}"),
        }
    }
}

#[derive(Error, Clone, PartialEq, PartialOrd, Debug)]
pub enum InterpreterError {
    #[error("Scanner error")]
    Scanner,
    #[error("Parser error")]
    Parser,
    #[error("Evaluation error")]
    Evaluation,
}

fn interpret(source: &str) -> Result<(), InterpreterError> {
    let (tokens, errors): (Vec<_>, Vec<_>) = scanner::scan(source).partition_result();

    if !errors.is_empty() {
        for e in errors {
            eprintln!("{:?}", e);
        }
        return Err(InterpreterError::Scanner);
    }

    let expr = parser::parse(tokens.iter());

    let interpreter = Interpreter::new();

    match expr {
        Ok(expr) => {
            println!("{}", expr);
            let result = interpreter.evaluate(&expr).map_err(|e| {
                eprintln!("{:?}", e);
                InterpreterError::Evaluation
            })?;
            println!("{:?}", result);
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(InterpreterError::Parser);
        }
    }

    Ok(())
}
