use crate::shared::tokens::TokenType;
use anyhow::{anyhow, Result};
use std::str::Chars;

#[derive(Debug)]
struct Scanner<'s> {
    source: Chars<'s>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    typ: TokenType,
}

impl<'s> From<&'s str> for Scanner<'s> {
    fn from(source: &'s str) -> Self {
        Scanner {
            source: source.chars(),
        }
    }
}

impl<'s> Scanner<'s> {
    fn advance(&mut self) -> Option<char> {
        self.source.next()
    }

    fn peek(&self) -> Option<char> {
        self.source.clone().next()
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
        self.source.as_str().is_empty()
    }

    fn make_token(&self, typ: TokenType) -> Option<Result<Token>> {
        Some(Ok(Token { typ }))
    }
}

impl<'s> Iterator for Scanner<'s> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(c) = self.advance() {
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
                    while let Some(c) = self.advance() {
                        if c == '"' {
                            return self.make_token(TokenType::String);
                        }
                    }
                    Some(Err(anyhow!("Unterminated string")))
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
