use anyhow::{anyhow, bail, Result};

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
}
impl<'s> Iterator for Scanner<'s> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.start = self.current;

        if self.start == self.source.len() {
            return None;
        }

        Some(Err(anyhow!(
            "Unexpected character: {}",
            self.source.chars().nth(self.current).unwrap()
        )))
    }
}

#[derive(Debug)]
pub struct Token {
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
}

pub fn scan(source: &String) -> Result<()> {
    let mut scanner = Scanner::new(source);
    println!("{:?}", scanner);
    let mut line = 0;

    loop {
        if let Some(t) = scanner.next() {
            if let Ok(token) = t {
                if token.line != line {
                    print!("{:4} ", token.line);
                    line = token.line;
                } else {
                    print!("    | ");
                }
                println!("{:?}", token);
            } else {
                bail!("Error while scanning: {:?}", t);
            }
        }
    }
}
