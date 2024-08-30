use std::iter::Peekable;

use crate::{
    shared::scanner::{Token, TokenType},
    walker::ast::Expr,
};

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

impl<'s, I> Parser<I>
where
    I: Iterator<Item = &'s Token<'s>>,
{
    fn expression(&mut self) -> Expr<'s> {
        self.equality()
    }

    fn equality(&mut self) -> Expr<'s> {
        let mut expr = self.comparison();

        while let Some(operator) = self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::BangEqual | TokenType::EqualEqual))
        {
            let right = self.comparison();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn comparison(&mut self) -> Expr<'s> {
        let mut expr = self.term();

        while let Some(operator) = self.tokens.next_if(|t| {
            matches!(
                t.typ,
                TokenType::Greater
                    | TokenType::GreaterEqual
                    | TokenType::Less
                    | TokenType::LessEqual
            )
        }) {
            let right = self.term();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn term(&mut self) -> Expr<'s> {
        let mut expr = self.factor();

        while let Some(operator) = self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::Minus | TokenType::Plus))
        {
            let right = self.factor();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn factor(&mut self) -> Expr<'s> {
        let mut expr = self.unary();

        while let Some(operator) = self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::Slash | TokenType::Star))
        {
            let right = self.unary();
            expr = Expr::Binary {
                left: Box::new(expr),
                op: operator,
                right: Box::new(right),
            };
        }

        expr
    }

    fn unary(&mut self) -> Expr<'s> {
        if let Some(operator) = self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::Bang | TokenType::Minus))
        {
            let right = self.unary();
            return Expr::Unary {
                op: operator,
                right: Box::new(right),
            };
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr<'s> {
        if let Some(token) = self.tokens.peek() {
            match token.typ {
                TokenType::False => Expr::Literal { token },
                TokenType::True => Expr::Literal { token },
                TokenType::Nil => Expr::Literal { token },
                TokenType::Number(_) => Expr::Literal { token },
                TokenType::String(_) => Expr::Literal { token },
                TokenType::LeftParen => {
                    let expr = self.expression();
                    self.tokens
                        .next_if(|t| matches!(t.typ, TokenType::RightParen))
                        .expect("Expected closing paren"); // TODO: result!
                    Expr::Grouping {
                        expr: Box::new(expr),
                    }
                }
                _ => panic!("Unexpected token: {:?}", token),
            }
        } else {
            panic!("Unexpected end of input");
        }
    }
}
