mod ast;
mod interpreter;
mod parser;

use std::io::{Read, Write};

use anyhow::Result;
use colored::Colorize;
use itertools::Itertools;
use thiserror::Error;

use crate::{
    shared::{scanner, streams::Streams},
    walker::interpreter::Interpreter,
};

pub fn exec(source: &str) -> Result<()> {
    interpret(source, &mut Streams::new())?;

    Ok(())
}

pub fn repl() -> Result<()> {
    println!("Gejang TW REPL");

    let prefix = "🦀> ";
    let bad_prefix = "😵> ";
    let mut error = false;

    let mut streams = Streams::new();

    loop {
        print!("{}", if !error { prefix } else { bad_prefix });
        streams.output.flush()?;
        let mut buffer = String::new();
        streams.input.read_line(&mut buffer)?;

        match interpret(&buffer, &mut streams) {
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
    #[error("Internal error")]
    Internal,
}

fn interpret<I: Read, O: Write, E: Write>(
    source: &str,
    streams: &mut Streams<I, O, E>,
) -> Result<(), InterpreterError> {
    let (tokens, errors): (Vec<_>, Vec<_>) = scanner::scan(source).partition_result();

    if !errors.is_empty() {
        for e in errors {
            write!(streams.error, "{}", e.to_string().red())
                .map_err(|_| InterpreterError::Internal)?;
        }
        return Err(InterpreterError::Scanner);
    }

    let (statements, errors): (Vec<_>, Vec<_>) =
        parser::parse(tokens.iter()).into_iter().partition_result();

    if !errors.is_empty() {
        for e in errors {
            write!(streams.error, "{}", e.to_string().red())
                .map_err(|_| InterpreterError::Internal)?;
        }
        return Err(InterpreterError::Parser);
    }

    let mut interpreter = Interpreter::new(streams);

    interpreter.interpret(&statements).map_err(|e| {
        if write!(streams.error, "{}", e.to_string().red()).is_err() {
            InterpreterError::Internal
        } else {
            InterpreterError::Evaluation
        }
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("print 1 + 2;", "3")]
    #[case("print 2 * 4 + 3;", "11")]
    #[case("print true;", "true")]
    #[case("print \"one\";", "one")]
    fn test_interpreter(#[case] source: &str, #[case] expected: &str) {
        let mut streams = Streams::test();
        interpret(source, &mut streams).unwrap();
        assert_eq!(String::from_utf8(streams.output).unwrap(), expected);
    }
}
