use crate::shared::tokens::TokenType;
use anyhow::{anyhow, Result};
use std::str::CharIndices;

#[derive(Debug)]
struct Scanner<'s> {
    source: &'s str,
    cursor: CharIndices<'s>,
    current_offset: usize,
    lexeme_start: usize,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    typ: TokenType,
    lexeme: String, // TODO: it should be possible to make this a &str pointing back to the original source
}

impl<'s> From<&'s str> for Scanner<'s> {
    fn from(source: &'s str) -> Self {
        Self {
            source,
            cursor: source.char_indices(),
            current_offset: 0,
            lexeme_start: 0,
        }
    }
}

impl<'s> Scanner<'s> {
    fn advance(&mut self) -> Option<(usize, char)> {
        if let Some((offset, c)) = self.cursor.next() {
            self.current_offset = offset;
            Some((offset, c))
        } else {
            None
        }
    }

    fn peek(&self) -> Option<char> {
        self.cursor.clone().next().map(|(_, c)| c)
    }

    fn peek_peek(&self) -> Option<char> {
        self.cursor.clone().nth(1).map(|(_, c)| c)
    }

    fn match_char(&mut self, expected: char) -> bool {
        if let Some(c) = self.peek() {
            if c == expected {
                self.advance();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn lexeme(&self) -> &str {
        &self.source[self.lexeme_start..=self.current_offset]
    }

    fn make_token(&self, typ: TokenType) -> Option<Result<Token>> {
        Some(Ok(Token {
            typ,
            lexeme: self.lexeme().into(),
        }))
    }
}

impl<'s> Iterator for Scanner<'s> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((lexeme_start, c)) = self.advance() {
            self.lexeme_start = lexeme_start;
            match c {
                '(' => self.make_token(TokenType::LeftParen),
                ')' => self.make_token(TokenType::RightParen),
                '{' => self.make_token(TokenType::LeftBrace),
                '}' => self.make_token(TokenType::RightBrace),
                ',' => self.make_token(TokenType::Comma),
                '.' => self.make_token(TokenType::Dot),
                '-' => self.make_token(TokenType::Minus),
                '+' => self.make_token(TokenType::Plus),
                ';' => self.make_token(TokenType::Semicolon),
                '*' => self.make_token(TokenType::Star),
                '!' => {
                    if self.match_char('=') {
                        self.make_token(TokenType::BangEqual)
                    } else {
                        self.make_token(TokenType::Bang)
                    }
                }
                '=' => {
                    if self.match_char('=') {
                        self.make_token(TokenType::EqualEqual)
                    } else {
                        self.make_token(TokenType::Equal)
                    }
                }
                '<' => {
                    if self.match_char('=') {
                        self.make_token(TokenType::LessEqual)
                    } else {
                        self.make_token(TokenType::Less)
                    }
                }
                '>' => {
                    if self.match_char('=') {
                        self.make_token(TokenType::GreaterEqual)
                    } else {
                        self.make_token(TokenType::Greater)
                    }
                }
                '/' => {
                    if self.match_char('/') {
                        while let Some(c) = self.peek() {
                            if c != '\n' {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        self.make_token(TokenType::Comment(self.lexeme().into()))
                    } else {
                        self.make_token(TokenType::Slash)
                    }
                }
                ' ' | '\r' | '\t' | '\n' => self.make_token(TokenType::Whitespace),
                '"' => {
                    while let Some((_, c)) = self.advance() {
                        if c == '"' {
                            // TODO: strip the quotes off
                            return self.make_token(TokenType::String(self.lexeme().into()));
                        }
                    }
                    Some(Err(anyhow!("Unterminated string")))
                }
                '0'..='9' => {
                    while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        self.advance();
                    }

                    if self.peek() == Some('.')
                        && self
                            .peek_peek()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or(false)
                    {
                        self.advance(); // consume the .
                        while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                            self.advance();
                        }
                    }

                    if let Ok(number) = self.lexeme().parse() {
                        self.make_token(TokenType::Number(number))
                    } else {
                        Some(Err(anyhow!("Invalid number: {:?}", self.lexeme())))
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    while self
                        .peek()
                        .map(|c| c.is_alphanumeric() || c == '_')
                        .unwrap_or(false)
                    {
                        self.advance();
                    }

                    match self.lexeme() {
                        "and" => self.make_token(TokenType::And),
                        "class" => self.make_token(TokenType::Class),
                        "else" => self.make_token(TokenType::Else),
                        "false" => self.make_token(TokenType::False),
                        "for" => self.make_token(TokenType::For),
                        "fun" => self.make_token(TokenType::Fun),
                        "if" => self.make_token(TokenType::If),
                        "nil" => self.make_token(TokenType::Nil),
                        "or" => self.make_token(TokenType::Or),
                        "print" => self.make_token(TokenType::Print),
                        "return" => self.make_token(TokenType::Return),
                        "super" => self.make_token(TokenType::Super),
                        "this" => self.make_token(TokenType::This),
                        "true" => self.make_token(TokenType::True),
                        "var" => self.make_token(TokenType::Var),
                        "while" => self.make_token(TokenType::While),
                        _ => self.make_token(TokenType::Identifier(self.lexeme().into())),
                    }
                }
                _ => Some(Err(anyhow!("Unexpected character: {c:?}"))),
            }
        } else {
            None
        }
    }
}

pub fn scan(source: &str) -> impl Iterator<Item = Result<Token>> + '_ {
    Scanner::from(source)
}
