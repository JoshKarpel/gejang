use std::iter::Peekable;

use thiserror::Error;

use crate::{
    shared::scanner::{Token, TokenType},
    walker::ast::{Expr, Stmt},
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
    #[error("Invalid assignment target")] // better debug info
    InvalidAssignmentTarget,
}

type ParserExprResult<'s> = Result<Expr<'s>, ParserError<'s>>;
type ParserStmtResult<'s> = Result<Stmt<'s>, ParserError<'s>>;

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

const TRUE: Token = Token {
    typ: TokenType::True,
    lexeme: "true",
    line: 0,
};

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

    fn parse(&mut self) -> Vec<ParserStmtResult<'s>> {
        let mut statements = Vec::new();
        while self.tokens.peek().is_some() {
            statements.push(self.declaration());
        }
        statements
    }

    fn declaration(&mut self) -> ParserStmtResult<'s> {
        (if self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::Var))
            .is_some()
        {
            self.variable_declaration()
        } else {
            self.statement()
        })
        .inspect_err(|_| self.synchronize())
    }

    fn variable_declaration(&mut self) -> ParserStmtResult<'s> {
        if let Some(name) = self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::Identifier(_)))
        {
            let initializer = if self
                .tokens
                .next_if(|t| matches!(t.typ, TokenType::Equal))
                .is_some()
            {
                Some(Box::new(self.expression()?))
            } else {
                None
            };

            self.require_token(TokenType::Semicolon)?;

            Ok(Stmt::Var { name, initializer })
        } else {
            Err(self
                .tokens
                .peek()
                .map_or(ParserError::UnexpectedEndOfInput, |token| {
                    ParserError::UnexpectedToken {
                        expected: TokenType::Semicolon,
                        token,
                    }
                }))
        }
    }

    fn statement(&mut self) -> ParserStmtResult<'s> {
        // TODO: can we avoid matching twice?
        if let Some(token) = self.tokens.next_if(|t| {
            matches!(
                t.typ,
                TokenType::For
                    | TokenType::Print
                    | TokenType::LeftBrace
                    | TokenType::If
                    | TokenType::While
                    | TokenType::Fun
                    | TokenType::Return
                    | TokenType::Break
            )
        }) {
            match token.typ {
                TokenType::For => self.for_statement(),
                TokenType::Print => self.print_statement(),
                TokenType::LeftBrace => self.block(),
                TokenType::If => self.if_statement(),
                TokenType::While => self.while_statement(),
                TokenType::Fun => self.function(),
                TokenType::Return => self.return_statement(),
                TokenType::Break => {
                    self.require_token(TokenType::Semicolon)?;
                    Ok(Stmt::Break)
                }
                _ => unreachable!("Unimplemented statement type"),
            }
        } else {
            self.expression_statement()
        }
    }

    fn for_statement(&mut self) -> ParserStmtResult<'s> {
        self.require_token(TokenType::LeftParen)?;

        let initializer = if self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::Semicolon))
            .is_some()
        {
            None
        } else if self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::Var))
            .is_some()
        {
            Some(self.variable_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self
            .tokens
            .peek()
            .is_some_and(|t| !matches!(t.typ, TokenType::Semicolon))
        {
            self.expression()?
        } else {
            Expr::Literal { value: &TRUE }
        };

        self.require_token(TokenType::Semicolon)?;

        let increment = if self
            .tokens
            .peek()
            .is_some_and(|t| !matches!(t.typ, TokenType::RightParen))
        {
            Some(self.expression()?)
        } else {
            None
        };

        self.require_token(TokenType::RightParen)?;

        let mut body = self.statement()?;

        if let Some(i) = increment {
            body = Stmt::Block {
                stmts: vec![body, Stmt::Expression { expr: Box::new(i) }],
            }
        }

        body = Stmt::While {
            condition: Box::new(condition),
            body: Box::new(body),
        };

        if let Some(i) = initializer {
            body = Stmt::Block {
                stmts: vec![i, body],
            }
        }

        Ok(body)
    }

    fn print_statement(&mut self) -> ParserStmtResult<'s> {
        let expr = self.expression()?;
        self.require_token(TokenType::Semicolon)?;
        Ok(Stmt::Print {
            expr: Box::new(expr),
        })
    }

    fn block(&mut self) -> ParserStmtResult<'s> {
        let mut stmts = Vec::new();

        while self
            .tokens
            .peek()
            .is_some_and(|t| !matches!(t.typ, TokenType::RightBrace))
        {
            stmts.push(self.declaration()?);
        }

        self.require_token(TokenType::RightBrace)?;

        Ok(Stmt::Block { stmts })
    }

    fn if_statement(&mut self) -> ParserStmtResult<'s> {
        self.require_token(TokenType::LeftParen)?;
        let condition = Box::new(self.expression()?);
        self.require_token(TokenType::RightParen)?;
        let then = Box::new(self.statement()?);
        let els = if self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::Else))
            .is_some()
        {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then,
            els,
        })
    }

    fn while_statement(&mut self) -> ParserStmtResult<'s> {
        self.require_token(TokenType::LeftParen)?;
        let condition = Box::new(self.expression()?);
        self.require_token(TokenType::RightParen)?;
        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    fn function(&mut self) -> ParserStmtResult<'s> {
        let name = self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::Identifier(_)))
            .ok_or_else(|| ParserError::UnexpectedToken {
                expected: TokenType::Identifier(""),
                token: self
                    .tokens
                    .peek()
                    .ok_or(ParserError::UnexpectedEndOfInput)
                    .unwrap(), // TODO what?
            })?;

        self.require_token(TokenType::LeftParen)?;

        let mut params = vec![];

        while self
            .tokens
            .peek()
            .is_some_and(|t| !matches!(t.typ, TokenType::RightParen))
        {
            params.push(
                self.tokens
                    .next_if(|t| matches!(t.typ, TokenType::Identifier(_)))
                    .ok_or_else(|| ParserError::UnexpectedToken {
                        expected: TokenType::Identifier(""),
                        token: self
                            .tokens
                            .peek()
                            .ok_or(ParserError::UnexpectedEndOfInput)
                            .unwrap(), // TODO what?
                    })?,
            );
            if !self
                .tokens
                .peek()
                .is_some_and(|t| matches!(t.typ, TokenType::Comma))
            {
                break;
            }
        }

        self.require_token(TokenType::RightParen)?;

        self.require_token(TokenType::LeftBrace)?;

        let mut body = vec![];
        while self
            .tokens
            .peek()
            .is_some_and(|t| !matches!(t.typ, TokenType::RightBrace))
        {
            body.push(self.declaration()?);
        }

        self.require_token(TokenType::RightBrace)?;

        Ok(Stmt::Function { name, params, body })
    }

    fn return_statement(&mut self) -> ParserStmtResult<'s> {
        let value = if self
            .tokens
            .peek()
            .is_some_and(|t| !matches!(t.typ, TokenType::Semicolon))
        {
            Some(Box::new(self.expression()?))
        } else {
            None
        };

        self.require_token(TokenType::Semicolon)?;

        Ok(Stmt::Return { value })
    }

    fn expression_statement(&mut self) -> ParserStmtResult<'s> {
        let expr = self.expression()?;
        self.require_token(TokenType::Semicolon)?;
        Ok(Stmt::Expression {
            expr: Box::new(expr),
        })
    }

    fn require_token(&mut self, typ: TokenType<'s>) -> Result<&Token<'s>, ParserError<'s>> {
        self.tokens.next_if(|t| t.typ == typ).ok_or_else(|| {
            self.tokens
                .peek()
                .map_or(ParserError::UnexpectedEndOfInput, |token| {
                    ParserError::UnexpectedToken {
                        expected: typ,
                        token,
                    }
                })
        })
    }

    fn expression(&mut self) -> ParserExprResult<'s> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParserExprResult<'s> {
        let expr = self.or()?;

        if self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::Equal))
            .is_some()
        {
            let value = Box::new(self.assignment()?);

            if let Expr::Variable { name } = expr {
                Ok(Expr::Assign { name, value })
            } else {
                Err(ParserError::InvalidAssignmentTarget)
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> ParserExprResult<'s> {
        let mut expr = self.and()?;

        while let Some(op) = self.tokens.next_if(|t| matches!(t.typ, TokenType::Or)) {
            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn and(&mut self) -> ParserExprResult<'s> {
        let mut expr = self.equality()?;

        while let Some(op) = self.tokens.next_if(|t| matches!(t.typ, TokenType::And)) {
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                op,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ParserExprResult<'s> {
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

    fn comparison(&mut self) -> ParserExprResult<'s> {
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

    fn term(&mut self) -> ParserExprResult<'s> {
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

    fn factor(&mut self) -> ParserExprResult<'s> {
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

    fn unary(&mut self) -> ParserExprResult<'s> {
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

        self.call()
    }

    fn call(&mut self) -> ParserExprResult<'s> {
        let mut expr = self.primary()?;

        while self
            .tokens
            .next_if(|t| matches!(t.typ, TokenType::LeftParen))
            .is_some()
        {
            let mut args = vec![];

            while self
                .tokens
                .peek()
                .is_some_and(|t| !matches!(t.typ, TokenType::RightParen))
            {
                args.push(self.expression()?);
                if !self
                    .tokens
                    .peek()
                    .is_some_and(|t| matches!(t.typ, TokenType::Comma))
                {
                    break;
                }
            }

            self.require_token(TokenType::RightParen)?;

            expr = Expr::Call {
                callee: Box::new(expr),
                // paren,  // trouble with multiple mutable borrows here
                args,
            };
        }

        Ok(expr)
    }

    fn primary(&mut self) -> ParserExprResult<'s> {
        if let Some(token) = self.tokens.next() {
            Ok(match token.typ {
                TokenType::False => Expr::Literal { value: token },
                TokenType::True => Expr::Literal { value: token },
                TokenType::Nil => Expr::Literal { value: token },
                TokenType::Number(_) => Expr::Literal { value: token },
                TokenType::String(_) => Expr::Literal { value: token },
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
                TokenType::Identifier(_) => Expr::Variable { name: token },
                _ => {
                    return Err(ParserError::UnexpectedToken {
                        expected: TokenType::Identifier(""), // TODO: What should this be?
                        token,
                    });
                }
            })
        } else {
            Err(ParserError::UnexpectedEndOfInput)
        }
    }
}

pub fn parse<'s, I>(tokens: I) -> Vec<ParserStmtResult<'s>>
// TODO: return an iterator instead of a Vec?
where
    I: IntoIterator<Item = &'s Token<'s>>, // TODO: Iterator or IntoIterator?
{
    let mut parser = Parser::from(tokens.into_iter());
    parser.parse()
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use rstest::rstest;

    use super::*;
    use crate::shared::scanner::scan;

    #[rstest]
    #[case("1 + 2", Ok(Expr::Binary{
        left: Box::new(Expr::Literal {
            value: &Token {
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
            value: &Token {
                typ: TokenType::Number(2.0),
                lexeme: "2",
                line: 0,

            },
        }),
        }))]
    #[case("(1 + 2)", Ok(Expr::Grouping{
        expr: Box::new(Expr::Binary {
            left: Box::new(Expr::Literal {
                value: &Token {
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
                value: &Token {
                    typ: TokenType::Number(2.0),
                    lexeme: "2",
                    line: 0,
                },
            }),
        }),
        }))]
    #[case("clock()", Ok(Expr::Call{
        callee: Box::new(Expr::Variable {
            name: &Token {
                typ: TokenType::Identifier("clock"),
                lexeme: "clock",
                line: 0,
            },
        }),
        args: vec![],
        }))]
    #[case("tsp2cup(15)", Ok(Expr::Call{
        callee: Box::new(Expr::Variable {
            name: &Token {
                typ: TokenType::Identifier("tsp2cup"),
                lexeme: "tsp2cup",
                line: 0,
            },
        }),
        args: vec![Expr::Literal {
            value: &Token {
                typ: TokenType::Number(15.0),
                lexeme: "15",
                line: 0,
            }}
        ],
        }))]
    #[case("(1 + 2", Err(ParserError::UnexpectedEndOfInput))]
    #[case("(1 + 2 foo", Err(ParserError::UnexpectedToken{
        expected: TokenType::RightParen,
        token: &Token {
            typ: TokenType::Identifier("foo"),
            lexeme: "foo",
            line: 0,
        },
        }))]

    fn test_parse(#[case] source: &str, #[case] expected: ParserExprResult) {
        let tokens: Vec<Token> = scan(source).try_collect().unwrap();
        let mut parser = Parser::from(tokens.iter());
        assert_eq!(parser.expression(), expected);
    }

    #[rstest]
    #[case(ParserError::UnexpectedEndOfInput, "Unexpected end of input")]
    #[case(ParserError::UnexpectedToken{
        expected: TokenType::RightParen,
        token: &Token {
            typ: TokenType::Identifier("foo"),
            lexeme: "foo",
            line: 0,
        },
        }, "Expected ) on line 0, but got identifier(foo)")]
    fn test_parse_error_display(#[case] err: ParserError, #[case] expected: &str) {
        assert_eq!(err.to_string(), expected);
    }
}
