use crate::bytecode::scanner::TokenType::EndOfFile;
use anyhow::{bail, Result};

#[derive(Debug)]
pub struct Scanner<'s> {
    source: &'s String,
    start: usize,   // The start of the current lexeme
    current: usize, // The current character under consideration
    line: usize,
}

impl<'s> Scanner<'s> {
    fn new(source: &'s String) -> Scanner {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn token(&mut self) -> Result<Token> {
        self.start = self.current;

        if self.start == self.source.len() {
            return Ok(Token {
                typ: EndOfFile,
                start: self.start,
                length: 0,
                line: self.line,
            });
        }

        bail!("Unexpected character")
    }
}

#[derive(Debug)]
struct Token {
    typ: TokenType,
    start: usize,
    length: usize,
    line: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Error,
    EndOfFile,
}

pub fn scan(source: &String) -> Result<()> {
    let mut scanner = Scanner::new(source);
    println!("{:?}", scanner);
    let mut line = 0;

    loop {
        let token = scanner.token()?;
        if token.line != line {
            print!("{:4} ", token.line);
            line = token.line;
        } else {
            print!("    | ");
        }
        println!("{:?}", token);

        if token.typ == TokenType::EndOfFile {
            return Ok(());
        }
    }
}
