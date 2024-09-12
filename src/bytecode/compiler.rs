use std::iter::Peekable;

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

struct Compiler<I>
where
    I: Iterator,
{
    tokens: Peekable<I>,
}

impl<'s, I> From<I> for Compiler<I>
where
    I: Iterator<Item = &'s Token<'s>>,
{
    fn from(tokens: I) -> Self {
        Compiler {
            tokens: tokens.peekable(),
        }
    }
}

impl<'s, I> Compiler<I>
where
    I: Iterator<Item = &'s Token<'s>>,
{
    fn expression(&mut self) -> CompileResult<'s> {
        Ok(Chunk::default())
    }
}

pub fn compile<'s, I>(tokens: I) -> CompileResult<'s>
where
    I: IntoIterator<Item = &'s Token<'s>>,
{
    let mut compiler = Compiler::from(tokens.into_iter());
    compiler.expression()
}
