use std::{fmt, fmt::Display};

use itertools::Itertools;

use crate::shared::scanner::Token;

type BoxedExpr<'s> = Box<Expr<'s>>;
type BoxedStmt<'s> = Box<Stmt<'s>>;
type RefToken<'s> = &'s Token<'s>;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Expr<'s> {
    Assign {
        name: RefToken<'s>,
        value: BoxedExpr<'s>,
    },
    Binary {
        left: BoxedExpr<'s>,
        op: RefToken<'s>,
        right: BoxedExpr<'s>,
    },
    Call {
        callee: BoxedExpr<'s>,
        // paren: RefToken<'s>,
        args: Vec<Expr<'s>>,
    },
    Get {
        object: BoxedExpr<'s>,
        name: RefToken<'s>,
    },
    Unary {
        op: RefToken<'s>,
        right: BoxedExpr<'s>,
    },
    Grouping {
        expr: BoxedExpr<'s>,
    },
    Literal {
        value: RefToken<'s>,
    },
    Logical {
        left: BoxedExpr<'s>,
        op: RefToken<'s>,
        right: BoxedExpr<'s>,
    },
    Set {
        object: BoxedExpr<'s>,
        name: RefToken<'s>,
        value: BoxedExpr<'s>,
    },
    This {
        keyword: RefToken<'s>,
    },
    Variable {
        name: RefToken<'s>,
    },
}

impl Display for Expr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Expr::Assign { name, value } => {
                    format!("(assign {} {})", name.lexeme, value)
                }
                Expr::Binary { left, op, right } => {
                    format!("({} {} {})", op.lexeme, left, right)
                }
                Expr::Call {
                    callee,
                    // paren,
                    args,
                } => {
                    format!(
                        "({} {})",
                        callee,
                        args.iter().map(|a| a.to_string()).join(", ")
                    )
                }
                Expr::Get { object, name } => {
                    format!("(get {} {})", object, name.lexeme)
                }
                Expr::Unary { op, right } => {
                    format!("({} {})", op.lexeme, right)
                }
                Expr::Grouping { expr } => {
                    format!("(grouping {})", expr)
                }
                Expr::Literal { value: token } => token.lexeme.into(),
                Expr::Logical { left, op, right } => {
                    format!("({} {} {}", op.lexeme, left, right)
                }
                Expr::Variable { name } => name.lexeme.into(),
                Expr::Set {
                    object,
                    name,
                    value,
                } => {
                    format!("(set {} {} {})", object, name.lexeme, value)
                }
                Expr::This { .. } => {
                    "this".to_string()
                }
            }
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum Stmt<'s> {
    Block {
        stmts: Vec<Stmt<'s>>,
    },
    Break,
    Class {
        name: RefToken<'s>,
        methods: Vec<Stmt<'s>>,
    },
    Expression {
        expr: BoxedExpr<'s>,
    },
    Function {
        name: RefToken<'s>,
        params: Vec<RefToken<'s>>,
        body: Vec<Stmt<'s>>,
    },
    If {
        condition: BoxedExpr<'s>,
        then: BoxedStmt<'s>,
        els: Option<BoxedStmt<'s>>,
    },
    Print {
        expr: BoxedExpr<'s>,
    },
    Return {
        value: Option<BoxedExpr<'s>>,
    },
    Var {
        name: RefToken<'s>,
        initializer: Option<BoxedExpr<'s>>,
    },
    While {
        condition: BoxedExpr<'s>,
        body: BoxedStmt<'s>,
    },
}

impl Display for Stmt<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Stmt::Block { stmts } => {
                    format!("(block {}", stmts.iter().map(|s| s.to_string()).join(" "))
                }
                Stmt::Expression { expr } => {
                    format!("(expression {expr})")
                }
                Stmt::Function { name, params, body } => {
                    format!(
                        "(function {} ({}) ({})",
                        name.lexeme,
                        params.iter().map(|p| p.lexeme).join(" "),
                        body.iter().map(|s| s.to_string()).join(" ")
                    )
                }
                Stmt::Class { name, methods } => {
                    format!(
                        "(class {} {})",
                        name.lexeme,
                        methods
                            .iter()
                            .map(|m| format!(
                                "({})",
                                if let Stmt::Function { name, .. } = m {
                                    name.lexeme
                                } else {
                                    ""
                                }
                            ))
                            .join(" "),
                    )
                }
                Stmt::If {
                    condition,
                    then,
                    els,
                } => {
                    if let Some(e) = els {
                        format!("(if {} then {} else {}", condition, then, e)
                    } else {
                        format!("(if {} then {}", condition, then)
                    }
                }
                Stmt::Print { expr } => {
                    format!("(print {expr})")
                }
                Stmt::Return { value } => {
                    match value {
                        Some(v) => format!("(return {v})"),
                        None => "(return)".to_string(),
                    }
                }
                Stmt::Var { name, initializer } => {
                    if let Some(init) = initializer {
                        format!("(var {} {init})", name.lexeme)
                    } else {
                        format!("(var {})", name.lexeme)
                    }
                }
                Stmt::While { condition, body } => {
                    format!("(while {} {})", condition, body)
                }
                Stmt::Break => "(break)".into(),
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
        Expr::Binary{
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
        Expr::Binary{
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
