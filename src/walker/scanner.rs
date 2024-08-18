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

    fn is_at_end(&self) -> bool {
        self.peek().is_none()
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
