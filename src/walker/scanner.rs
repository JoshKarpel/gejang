use crate::shared::tokens::TokenType;
use anyhow::{anyhow, Result};
use std::str::{CharIndices, Chars};

#[derive(Debug)]
struct Scanner<'s> {
    source: &'s str,
    cursor: CharIndices<'s>,
    current_offset: usize,
    lexeme_start: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'l> {
    typ: TokenType,
    lexeme: &'l str,
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

    fn is_at_end(&self) -> bool {
        self.cursor.as_str().is_empty()
    }

    fn make_token(&self, typ: TokenType) -> Option<Result<Token>> {
        Some(Ok(Token {
            typ,
            lexeme: &self.source[self.lexeme_start..self.current_offset],
        }))
    }
}

impl<'s> Iterator for Scanner<'s> {
    type Item = Result<Token<'s>>;

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
                        self.make_token(TokenType::Comment)
                    } else {
                        self.make_token(TokenType::Slash)
                    }
                }
                ' ' | '\r' | '\t' | '\n' => self.make_token(TokenType::Whitespace),
                '"' => {
                    while let Some((_, c)) = self.advance() {
                        if c == '"' {
                            return self.make_token(TokenType::String);
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

                    self.make_token(TokenType::Number)
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    while self
                        .peek()
                        .map(|c| c.is_alphanumeric() || c == '_')
                        .unwrap_or(false)
                    {
                        self.advance();
                    }

                    // TODO: Add keywords, need to know the lexeme!

                    self.make_token(TokenType::Identifier)
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