mod ast;
mod interpreter;
mod parser;

use std::{
    cell::RefCell,
    io::{Read, Write},
};

use anyhow::Result;
use colored::Colorize;
use itertools::Itertools;
use thiserror::Error;

use crate::{
    shared::{scanner, streams::Streams},
    walker::interpreter::Interpreter,
};

pub fn exec(source: &str) -> Result<()> {
    interpret(source, &RefCell::new(Streams::new()))?;

    Ok(())
}

pub fn repl() -> Result<()> {
    println!("Gejang TW REPL");

    let prefix = "🦀> ";
    let bad_prefix = "😵> ";
    let mut error = false;

    let streams = RefCell::new(Streams::new());

    loop {
        writeln!(
            streams.borrow_mut().output,
            "{}",
            if !error { prefix } else { bad_prefix }
        )?;
        streams.borrow_mut().output.flush()?;

        let mut buffer = String::new();
        streams.borrow_mut().input.read_line(&mut buffer)?;

        match interpret(&buffer, &streams) {
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
    streams: &RefCell<Streams<I, O, E>>,
) -> Result<(), InterpreterError> {
    let (tokens, errors): (Vec<_>, Vec<_>) = scanner::scan(source).partition_result();

    if !errors.is_empty() {
        for e in errors {
            writeln!(streams.borrow_mut().error, "{}", e.to_string().red())
                .map_err(|_| InterpreterError::Internal)?;
        }
        return Err(InterpreterError::Scanner);
    }

    let (statements, errors): (Vec<_>, Vec<_>) =
        parser::parse(tokens.iter()).into_iter().partition_result();

    if !errors.is_empty() {
        for e in errors {
            writeln!(streams.borrow_mut().error, "{}", e.to_string().red())
                .map_err(|_| InterpreterError::Internal)?;
        }
        return Err(InterpreterError::Parser);
    }

    let interpreter = Interpreter::new(streams);

    interpreter.interpret(&statements).map_err(|e| {
        if writeln!(streams.borrow_mut().error, "{}", e.to_string().red()).is_err() {
            InterpreterError::Internal
        } else {
            InterpreterError::Evaluation
        }
    })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("print 1 + 2;", "3\n")]
    #[case("print 2 * 4 + 3;", "11\n")]
    #[case("print true;", "true\n")]
    #[case("print \"one\";", "one\n")]
    #[case("var foo = \"bar\"; print foo;", "bar\n")]
    #[case("var foo = 1 + 2 * 6; print foo;", "13\n")]
    #[case("var foo; print foo;", "nil\n")]
    #[case("var foo; print foo = 2;", "2\n")]
    #[case(
        r#"
var a = "global a";
var b = "global b";
var c = "global c";
{
  var a = "outer a";
  var b = "outer b";
  {
    var a = "inner a";
    print a;
    print b;
    print c;
  }
  print a;
  print b;
  print c;
}
print a;
print b;
print c;"#,
        "\
inner a
outer b
global c
outer a
outer b
global c
global a
global b
global c
"
    )]
    #[case(
        r#"
var a = 1;
{
  var a = a + 2;
  print a;
}"#,
        "3\n"
    )]
    #[case("if (true) print 1;", "1\n")]
    #[case("if (false) print 1;", "")]
    #[case("if (true) print 1; else print 0;", "1\n")]
    #[case("if (false) print 1; else print 0;", "0\n")]
    #[case("print \"hi\" or 2;", "hi\n")]
    #[case("print nil or \"yes\";", "yes\n")]
    #[case("print false or \"yes\";", "yes\n")]
    #[case("print \"hi\" and 2;", "2\n")]
    #[case("print nil and \"yes\";", "nil\n")]
    #[case("print false and \"yes\";", "false\n")]
    #[case("var i = 0; while (i < 3) {print i; i = i + 1;}", "0\n1\n2\n")]
    fn test_interpreter(#[case] source: &str, #[case] expected: &str) {
        let streams = RefCell::new(Streams::test());
        let r = interpret(source, &streams);
        println!("{}", streams.borrow().get_output().unwrap());
        println!("{}", streams.borrow().get_error().unwrap());
        r.unwrap();
        assert_eq!(streams.borrow().get_output().unwrap(), expected);
    }
}
