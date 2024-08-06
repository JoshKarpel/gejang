use anyhow::{anyhow, bail, Result};

#[derive(Debug)]
pub struct Scanner<'s> {
    source: Vec<char>,
    start: usize,   // The start of the current lexeme
    current: usize, // The current character under consideration
    line: usize,
}

impl<'s> Scanner<'s> {
    fn new(source: &'s String) -> Scanner {
        Scanner {
            source: source.chars().collect(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    fn make_token(&self, typ: TokenType) -> Token {
        Token {
            typ,
            start: self.start,
            length: self.current - self.start,
            line: self.line,
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

        let c = self.source.get(self.current).unwrap();
        self.current += 1;

        match c {
            '(' => {
                return Some(Ok(self.make_token(TokenType::LeftParen)));
            }
            ')' => {
                return Some(Ok(self.make_token(TokenType::RightParen)));
            }
            '{' => {
                return Some(Ok(self.make_token(TokenType::LeftBrace)));
            }
            '}' => {
                return Some(Ok(self.make_token(TokenType::RightBrace)));
            }
            ';' => {
                return Some(Ok(self.make_token(TokenType::Semicolon)));
            }
            ',' => {
                return Some(Ok(self.make_token(TokenType::Comma)));
            }
            '.' => {
                return Some(Ok(self.make_token(TokenType::Dot)));
            }
            '-' => {
                return Some(Ok(self.make_token(TokenType::Minus)));
            }
            '+' => {
                return Some(Ok(self.make_token(TokenType::Plus)));
            }
            '/' => {
                return Some(Ok(self.make_token(TokenType::Slash)));
            }
            '*' => {
                return Some(Ok(self.make_token(TokenType::Star)));
            }
            _ => {}
        }

        Some(Err(anyhow!(
            "Unexpected character: {}",
            self.source.get(self.current).unwrap()
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
