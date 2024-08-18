use crate::walker::ast::Expr;

pub fn format_ast(expression: &Expr) -> String {
    match expression {
        Expr::Binary { left, op, right } => {
            format!("({} {} {})", op.lexeme, format_ast(left), format_ast(right))
        }
        Expr::Grouping { expr } => {
            format!("(grouping {})", format_ast(expr))
        }
        Expr::Literal { token } => token.lexeme.into(),
        Expr::Unary { op, right } => {
            format!("({} {})", op.lexeme, format_ast(right))
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use rstest::rstest;

    use crate::{
        shared::scanner::{Token, TokenType},
        walker::{ast::*, ast_formatter::format_ast},
    };

    #[rstest]
    #[case(
        Expr::Binary {
            left: Box::new(Expr::Literal {
                token: Token {
                    typ: TokenType::Number(1.0),
                    lexeme: "1",
                    line: 0,
                },
            }),
            op: Token {
                typ: TokenType::Plus,
                lexeme: "+",
                line: 0,
            },
            right: Box::new(Expr::Literal {
                token: Token {
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
                op: Token {
                    typ: TokenType::Minus,
                    lexeme: "-",
                    line: 0,
                },
                right: Box::new(Expr::Literal {
                    token: Token {
                        typ: TokenType::Number(1.0),
                        lexeme: "1",
                        line: 0,
                    },
                }),
            }),
            op: Token {
                typ: TokenType::Star,
                lexeme: "*",
                line: 0,
            },
            right: Box::new(Expr::Grouping {
                expr: Box::new(Expr::Literal {
                    token: Token {
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
        assert_eq!(format_ast(&input), expected);
    }
}
