use thiserror::Error;

use crate::{
    bytecode::ops::{Chunk, OpCode::Constant},
    shared::scanner::{scan, LineNumber, ScannerError, Token, TokenType},
};

#[derive(Error, Debug, PartialEq)]
pub enum ParserError {
    #[error("Unexpected token from line {line}")]
    UnknownToken { line: LineNumber },
}

#[derive(Error, Debug, PartialEq)]
pub enum CompilerError {
    #[error(transparent)]
    Compiler(#[from] ParserError),
    #[error(transparent)]
    Scanner(#[from] ScannerError),
}

pub fn compile(source: &str) -> Result<Chunk, CompilerError> {
    scan(source).try_fold(Chunk::default(), |mut chunk: Chunk, token| {
        let t = token?;
        expression(&t, &mut chunk)?;
        println!("{:?}", t);
        Ok(chunk)
    })
}

fn expression(token: &Token<'_>, chunk: &mut Chunk) -> Result<(), ParserError> {
    chunk.lines.push(token.line);

    match token.typ {
        TokenType::Number(n) => {
            chunk.code.push(Constant {
                index: chunk.constants.len(),
            });
            chunk.constants.push(n);
        }
        _ => return Err(ParserError::UnknownToken { line: token.line }),
    }

    Ok(())
}
