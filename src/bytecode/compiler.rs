use std::iter::Peekable;

use thiserror::Error;

use crate::{
    bytecode::{
        ops::{Chunk, OpCode},
        values::Value,
    },
    shared::scanner::{Precedence, Token, TokenType},
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

type IntermediateCompileResult<'s> = Result<(), CompilerError<'s>>;
type CompileResult<'s> = Result<Chunk<'s>, CompilerError<'s>>;

struct Compiler<'s, I>
where
    I: Iterator,
{
    tokens: Peekable<I>,
    chunk: Chunk<'s>,
}

impl<'s, I> From<I> for Compiler<'s, I>
where
    I: Iterator<Item = &'s Token<'s>>,
{
    fn from(tokens: I) -> Self {
        Compiler {
            tokens: tokens.peekable(),
            chunk: Chunk::default(),
        }
    }
}

impl<'s, I> Compiler<'s, I>
where
    I: Iterator<Item = &'s Token<'s>>,
{
    fn parse(&mut self, precedence: Precedence) -> IntermediateCompileResult<'s> {
        self.prefix()?;

        while self
            .tokens
            .peek()
            .is_some_and(|token| precedence <= token.typ.precedence())
        {
            self.infix()?
        }

        Ok(())
    }

    fn expression(&mut self) -> IntermediateCompileResult<'s> {
        self.parse(Precedence::Assignment)?;

        Ok(())
    }

    fn prefix(&mut self) -> IntermediateCompileResult<'s> {
        if let Some(token) = self.tokens.next() {
            match token.typ {
                TokenType::LeftParen => {
                    self.expression()?;

                    if let Some(token) = self.tokens.next() {
                        if token.typ != TokenType::RightParen {
                            return Err(CompilerError::UnexpectedToken {
                                expected: TokenType::RightParen,
                                token,
                            });
                        }
                    } else {
                        return Err(CompilerError::UnexpectedEndOfInput);
                    }
                }
                TokenType::Minus => {
                    self.parse(Precedence::Unary)?;
                    self.chunk.write(OpCode::Negate, token.line);
                }
                TokenType::Number(_) => {
                    self.chunk.add_constant(Value::from(&token.typ), token.line);
                }
                _ => {
                    return Err(CompilerError::UnexpectedToken {
                        expected: TokenType::Number(0.0),
                        token,
                    });
                }
            }
        }

        Ok(())
    }

    fn infix(&mut self) -> IntermediateCompileResult<'s> {
        if let Some(token) = self.tokens.next() {
            match token.typ {
                TokenType::Plus | TokenType::Minus | TokenType::Star | TokenType::Slash => {
                    self.parse(token.typ.precedence().next())?;
                    self.chunk.write(
                        match token.typ {
                            TokenType::Plus => OpCode::Add,
                            TokenType::Minus => OpCode::Subtract,
                            TokenType::Star => OpCode::Multiply,
                            TokenType::Slash => OpCode::Divide,
                            _ => unreachable!(""),
                        },
                        token.line,
                    );
                }
                _ => {
                    return Err(CompilerError::UnexpectedToken {
                        expected: TokenType::Number(0.0),
                        token,
                    });
                }
            }
        }

        Ok(())
    }
}

pub fn compile<'s, I>(tokens: I) -> CompileResult<'s>
where
    I: IntoIterator<Item = &'s Token<'s>>,
{
    let mut compiler = Compiler::from(tokens.into_iter());
    compiler.expression()?;
    Ok(compiler.chunk)
}
