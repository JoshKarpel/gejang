use std::iter::Peekable;

use thiserror::Error;

use crate::{
    shared::scanner::{Token, TokenType},
    walker::ast::Expr,
};

#[derive(Error, Clone, PartialEq, PartialOrd, Debug)]
pub enum ParserError<'s> {
    #[error("Expected {expected} on line {}, but got {}", .token.line, .token.typ)]
    UnexpectedToken {
        expected: TokenType<'s>,
        token: &'s Token<'s>,
    },
    #[error("Unexpected end of input")]
    UnexpectedEndOfInput,
    #[error("Expected an expression, but got {token:?}")]
    ExpectedExpression { token: &'s Token<'s> },
}

type ParserResult<'s> = Result<Expr<'s>, ParserError<'s>>;

/*
Trying to write this the way the book wants doesn't work
because of Rust's borrowing rules.
The problem is that the Parser struct borrows itself mutably,
which means that it can't borrow itself immutably at the same time.
That's a problem because the data structures returned by the parser
are references to the tokens in the parser (immutable borrows).
But as we descend recursively, we need a *mutable* borrow on Parser.current,
but because we have to borrow the Parser struct itself mutably,
we're screwed.

How to work around this?

Using a single iterator seems to be the way to go.
We still need a mutable reference to the Parser struct
(though now that's roughly equivalent to just
having a mutable reference to the iterator itself).
But the things we're returning are immutable borrows
to the iterator's underlying data, not the iterator itself,
so we can have multiple of those at the same time.

Note that the explicit lifetime on the return values
(`Expr<'s>`) is necessary because otherwise Rust will
infer the lifetime to be the same as the
Parser's `&mut self` lifetime instead of
the data inside the iterator.
*/

struct Parser<I>
where
    I: Iterator,
{
    tokens: Peekable<I>,
}

impl<'s, I> From<I> for Parser<I>
where
    I: Iterator<Item = &'s Token<'s>>,
{
    fn from(tokens: I) -> Self {
        Parser {
            tokens: tokens.peekable(),
        }
    }
}

impl<'s, I> Parser<I>
where
    I: Iterator<Item = &'s Token<'s>>,
{
    #[allow(dead_code)]
    fn synchronize(&mut self) {
        while let Some(token) = self.tokens.next() {
            // If we are at the end of the current statement...
            if matches!(token.typ, TokenType::Semicolon) {
                return;
            }

            // ... or at the beginning of a new statement
            if let Some(next) = self.tokens.peek() {
                if matches!(
                    next.typ,
                    TokenType::Class
                        | TokenType::Fun
                        | TokenType::Var
                        | TokenType::For
                        | TokenType::If
                        | TokenType::While
                        | TokenType::Print
                        | TokenType::Return
                ) {
                    return;
                }
            }
        }
    }

    fn expression(&mut self) -> ParserResult<'s> {
        self.equality()
    }

    fn equality(&mut self) -> ParserResult<'s> {
        let mut expr = self.comparison()?;

        while let Some(operator) = self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::BangEqual | TokenType::EqualEqual))
        {
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParserResult<'s> {
        let mut expr = self.term()?;

        while let Some(operator) = self.tokens.next_if(|t| {
            matches!(
                t.typ,
                TokenType::Greater
                    | TokenType::GreaterEqual
                    | TokenType::Less
                    | TokenType::LessEqual
            )
        }) {
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParserResult<'s> {
        let mut expr = self.factor()?;

        while let Some(operator) = self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::Minus | TokenType::Plus))
        {
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParserResult<'s> {
        let mut expr = self.unary()?;

        while let Some(operator) = self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::Slash | TokenType::Star))
        {
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParserResult<'s> {
        if let Some(operator) = self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::Bang | TokenType::Minus))
        {
            let right = self.unary()?;
            return Ok(Expr::Unary {
                op: operator,
                right: Box::new(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> ParserResult<'s> {
        if let Some(token) = self.tokens.next() {
            Ok(match token.typ {
                TokenType::False => Expr::Literal { token },
                TokenType::True => Expr::Literal { token },
                TokenType::Nil => Expr::Literal { token },
                TokenType::Number(_) => Expr::Literal { token },
                TokenType::String(_) => Expr::Literal { token },
                TokenType::LeftParen => {
                    let expr = self.expression()?;
                    self.tokens
                        .next_if(|t| matches!(t.typ, TokenType::RightParen))
                        .ok_or_else(|| {
                            self.tokens
                                .peek()
                                .map_or(ParserError::UnexpectedEndOfInput, |token| {
                                    ParserError::UnexpectedToken {
                                        expected: TokenType::RightParen,
                                        token,
                                    }
                                })
                        })?;
                    Expr::Grouping {
                        expr: Box::new(expr),
                    }
                }
                _ => return Err(ParserError::ExpectedExpression { token }),
            })
        } else {
            Err(ParserError::UnexpectedEndOfInput)
        }
    }
}

pub fn parse<'s, I>(tokens: I) -> ParserResult<'s>
where
    I: IntoIterator<Item = &'s Token<'s>>, // TODO: Iterator or IntoIterator?
{
    let mut parser = Parser::from(tokens.into_iter());
    parser.expression()
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use rstest::rstest;

    use super::*;
    use crate::shared::scanner::scan;

    #[rstest]
    #[case("1 + 2", Ok(Expr::Binary {
        left: Box::new(Expr::Literal {
            token: &Token {
                typ: TokenType::Number(1.0),
                lexeme: "1",
                line: 0,

            },
        }),
        op: &Token {
            typ: TokenType::Plus,
            lexeme: "+",
            line: 0,

        },
        right: Box::new(Expr::Literal {
            token: &Token {
                typ: TokenType::Number(2.0),
                lexeme: "2",
                line: 0,

            },
        }),
    }))]
    #[case("(1 + 2)", Ok(Expr::Grouping {
        expr: Box::new(Expr::Binary {
            left: Box::new(Expr::Literal {
                token: &Token {
                    typ: TokenType::Number(1.0),
                    lexeme: "1",
                    line: 0,
                },
            }),
            op: &Token {
                typ: TokenType::Plus,
                lexeme: "+",
                line: 0,
            },
            right: Box::new(Expr::Literal {
                token: &Token {
                    typ: TokenType::Number(2.0),
                    lexeme: "2",
                    line: 0,
                },
            }),
        }),
    }))]
    #[case("(1 + 2", Err(ParserError::UnexpectedEndOfInput))]
    #[case("(1 + 2 foo", Err(ParserError::UnexpectedToken {
        expected: TokenType::RightParen,
        token: &Token {
            typ: TokenType::Identifier("foo"),
            lexeme: "foo",
            line: 0,
        },
    }))]
    fn test_parse(#[case] source: &str, #[case] expected: ParserResult) {
        let tokens: Vec<Token> = scan(source).try_collect().unwrap();
        assert_eq!(parse(tokens.iter()), expected);
    }

    #[rstest]
    #[case(ParserError::UnexpectedEndOfInput, "Unexpected end of input")]
    #[case(ParserError::UnexpectedToken {
        expected: TokenType::RightParen,
        token: &Token {
            typ: TokenType::Identifier("foo"),
            lexeme: "foo",
            line: 0,
        },
    }, "Expected ) on line 0, but got an identifier")]
    fn test_parse_error_display(#[case] err: ParserError, #[case] expected: &str) {
        assert_eq!(err.to_string(), expected);
    }
}
