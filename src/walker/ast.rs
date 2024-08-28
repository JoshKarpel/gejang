use std::{fmt, fmt::Display};

use crate::shared::scanner::Token;

#[derive(Debug, PartialEq)]
pub enum Expr<'s> {
    Binary {
        left: Box<Expr<'s>>,
        op: &'s Token<'s>,
        right: Box<Expr<'s>>,
    },
    Unary {
        op: &'s Token<'s>,
        right: Box<Expr<'s>>,
    },
    Grouping {
        expr: Box<Expr<'s>>,
    },
    Literal {
        token: &'s Token<'s>,
    },
}

impl Display for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Expr::Binary { left, op, right } => {
                    format!("({} {} {})", op.lexeme, left, right)
                }
                Expr::Unary { op, right } => {
                    format!("({} {})", op.lexeme, right)
                }
                Expr::Grouping { expr } => {
                    format!("(grouping {})", expr)
                }
                Expr::Literal { token } => token.lexeme.into(),
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{
        shared::scanner::{Token, TokenType},
        walker::ast::*,
    };

    #[rstest]
    #[case(
        Expr::Binary {
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
        },
        "(+ 1 2)",
    )]
    #[case(
        Expr::Binary {
            left: Box::new(Expr::Unary {
                op: &Token {
                    typ: TokenType::Minus,
                    lexeme: "-",
                    line: 0,
                },
                right: Box::new(Expr::Literal {
                    token: &Token {
                        typ: TokenType::Number(1.0),
                        lexeme: "1",
                        line: 0,
                    },
                }),
            }),
            op: &Token {
                typ: TokenType::Star,
                lexeme: "*",
                line: 0,
            },
            right: Box::new(Expr::Grouping {
                expr: Box::new(Expr::Literal {
                    token: &Token {
                        typ: TokenType::Number(2.0),
                        lexeme: "2",
                        line: 0,
                    },
                }),
            }),
        },
        "(* (- 1) (grouping 2))",
    )]
    fn test_printer(#[case] input: Expr, #[case] expected: &str) {
        assert_eq!(input.to_string(), expected);
    }
}
