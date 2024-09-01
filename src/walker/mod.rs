mod ast;
mod parser;

use std::{io, io::Write, path::Path};

use anyhow::Result;
use thiserror::Error;

use crate::shared::scanner;

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
    ScannerError,
    #[error("Parser error")]
    ParserError,
}

fn interpret(source: &str) -> Result<(), InterpreterError> {
    let mut tokens = vec![];
    let mut errors = vec![];
    for r in scanner::scan(source) {
        match r {
            Ok(token) => {
                println!("{:?}", token);
                tokens.push(token);
            }
            Err(e) => {
                errors.push(e);
            }
        }
    }

    if !errors.is_empty() {
        for e in errors {
            eprintln!("{:?}", e);
        }
        return Err(InterpreterError::ScannerError);
    }

    let expr = parser::parse(tokens.iter());

    match expr {
        Ok(expr) => {
            println!("{}", expr);
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(InterpreterError::ParserError);
        }
    }

    Ok(())
}
