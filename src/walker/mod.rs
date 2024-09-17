mod ast;
mod interpreter;
mod parser;

use std::{io, io::Write};

use anyhow::Result;
use colored::Colorize;
use itertools::Itertools;
use thiserror::Error;

use crate::{shared::scanner, walker::interpreter::Interpreter};

pub fn exec(source: &str) -> Result<()> {
    interpret(source)?;

    Ok(())
}

pub fn repl() -> Result<()> {
    println!("Gejang TW REPL");

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let prefix = "🦀> ";
    let bad_prefix = "😵> ";
    let mut error = false;

    loop {
        print!("{}", if !error { prefix } else { bad_prefix });
        stdout.flush()?;
        let mut buffer = String::new();
        stdin.read_line(&mut buffer)?;

        match interpret(&buffer) {
            Ok(_) => error = false,
            Err(_) => error = true,
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
            eprintln!("{}", e.to_string().red());
        }
        return Err(InterpreterError::Scanner);
    }

    let expr = parser::parse(tokens.iter());

    let interpreter = Interpreter::new();

    match expr {
        Ok(expr) => {
            println!("{}", expr.to_string().dimmed());
            let _ = interpreter
                .evaluate(&expr)
                .inspect(|v| println!("{:?}", v))
                .inspect_err(|e| eprintln!("{}", e.to_string().red()))
                .map_err(|_| InterpreterError::Evaluation)?;
            Ok(())
        }
        Err(e) => {
            eprintln!("{}", e.to_string().red());
            Err(InterpreterError::Parser)
        }
    }
}
