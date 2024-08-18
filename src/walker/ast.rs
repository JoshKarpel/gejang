use crate::shared::scanner::Token;

pub enum Expr<'s> {
    Binary {
        left: Box<Expr<'s>>,
        op: Token<'s>,
        right: Box<Expr<'s>>,
    },
    Grouping {
        expr: Box<Expr<'s>>,
    },
    Literal {
        token: Token<'s>,
    },
    Unary {
        op: Token<'s>,
        right: Box<Expr<'s>>,
    },
}
