use anyhow::{anyhow, bail, Result};

#[derive(Debug)]
pub struct Scanner {
    source: Vec<char>, // TODO: this is not ideal, we should be pointing back to the original source string to save memory
    start: usize,      // The start of the current lexeme
    current: usize,    // The current character under consideration
    line: usize,
}

impl Scanner {
    fn new(source: &String) -> Scanner {
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

    fn is_at_end(&mut self) -> bool {
        self.start == self.source.len()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        self.source[self.current - 1]
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }
}

impl Iterator for Scanner {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.start = self.current;

        if self.is_at_end() {
            return None;
        }

        let c = self.advance();

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
            '!' => {
                return Some(Ok(if self.match_char('=') {
                    self.make_token(TokenType::BangEqual)
                } else {
                    self.make_token(TokenType::Bang)
                }));
            }
            '=' => {
                return Some(Ok(if self.match_char('=') {
                    self.make_token(TokenType::EqualEqual)
                } else {
                    self.make_token(TokenType::Equal)
                }));
            }
            '<' => {
                return Some(Ok(if self.match_char('=') {
                    self.make_token(TokenType::LessEqual)
                } else {
                    self.make_token(TokenType::Less)
                }));
            }
            '>' => {
                return Some(Ok(if self.match_char('=') {
                    self.make_token(TokenType::GreaterEqual)
                } else {
                    self.make_token(TokenType::Greater)
                }));
            }
            _ => {}
        }

        Some(Err(anyhow!("Unexpected character: {c}")))
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
    Equal,
    EqualEqual,
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
                    print!("   | ");
                }
                println!("{:?}", token);
            } else {
                bail!("Error while scanning: {:?}", t);
            }
        }
    }
}
