mod ast;
mod printer;

use std::{io, io::Write, path::Path};

use anyhow::Result;

use crate::shared::scanner;

pub fn script(path: &Path) -> anyhow::Result<()> {
    let source = std::fs::read_to_string(path)?;

    interpret(&source)?;

    Ok(())
}

pub fn repl() -> anyhow::Result<()> {
    println!("Gejang TW REPL");

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
    for token in scanner::scan(source) {
        println!("{:?}", token);
    }

    Ok(())
}
