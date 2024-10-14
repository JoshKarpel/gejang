use std::{fmt, fmt::Display};

use itertools::Itertools;

use crate::shared::scanner::Token;

type E<'s> = Box<Expr<'s>>;
type T<'s> = &'s Token<'s>;

#[derive(Debug, PartialEq)]
pub enum Expr<'s> {
    Binary {
        left: E<'s>,
        op: T<'s>,
        right: E<'s>,
    },
    Unary {
        op: T<'s>,
        right: E<'s>,
    },
    Grouping {
        expr: E<'s>,
    },
    Literal {
        value: T<'s>,
    },
    Variable {
        name: T<'s>,
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
                Expr::Literal { value: token } => token.lexeme.into(),
                Expr::Variable { name } => name.lexeme.into(),
            }
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum Stmt<'s> {
    Block {
        stmts: Vec<Stmt<'s>>,
    },
    Expression {
        expr: E<'s>,
    },
    Print {
        expr: E<'s>,
    },
    Var {
        name: T<'s>,
        initializer: Option<E<'s>>,
    },
}

impl Display for Stmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Stmt::Block { stmts } => {
                    format!("(block {}", stmts.iter().map(|s| s.to_string()).join(", "))
                }
                Stmt::Expression { expr } => {
                    format!("(expression {expr})")
                }
                Stmt::Print { expr } => {
                    format!("(print {expr})")
                }
                Stmt::Var { name, initializer } => {
                    if let Some(init) = initializer {
                        format!("(var {} {init})", name.lexeme)
                    } else {
                        format!("(var {})", name.lexeme)
                    }
                }
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
                    value: &Token {
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
                    value: &Token {
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
