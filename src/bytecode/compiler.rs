use thiserror::Error;

use crate::{
    bytecode::ops::Chunk,
    shared::scanner::{Token, TokenType},
};

#[derive(Error, Clone, PartialEq, PartialOrd, Debug)]
pub enum CompilerError<'s> {
    #[error("Expected {expected} on line {}, but got {}", .token.line, .token.typ)]
    UnexpectedToken {
        expected: TokenType<'s>,
        token: &'s Token<'s>,
    },
    #[error("Unexpected end of input")]
    UnexpectedEndOfInput,
}

type CompileResult<'s> = Result<Chunk, CompilerError<'s>>;

pub fn compile<'s, I>(tokens: I) -> CompileResult<'s>
where
    I: IntoIterator<Item = &'s Token<'s>>,
{
    Ok(Chunk::default())
}
