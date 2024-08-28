use crate::{
    shared::scanner::{Token, TokenType},
    walker::ast::Expr,
};

struct Parser<'s> {
    tokens: Vec<Token<'s>>,
    current: usize,
}

impl<'s> Parser<'s> {
    fn new(tokens: Vec<Token<'s>>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn advance(&mut self) -> Option<&Token<'s>> {
        self.current += 1;
        self.tokens.get(self.current - 1)
    }

    fn advance_if(&mut self, test: fn(&Token<'s>) -> bool) -> Option<&Token<'s>> {
        self.peek()
            .is_some_and(test)
            .then(|| self.advance().unwrap()) // unwrap is safe here because we know the peek is Some
    }

    fn peek(&self) -> Option<&Token<'s>> {
        self.tokens.get(self.current)
    }

    fn expression(&'s mut self) -> Expr<'s> {
        self.equality()
    }

    fn equality(&'s mut self) -> Expr<'s> {
        let mut expr = self.comparison();

        while let Some(operator) =
            self.advance_if(|t| matches!(t.typ, TokenType::BangEqual | TokenType::EqualEqual))
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

    fn comparison(&'s mut self) -> Expr<'s> {
        let mut expr = self.term();

        while let Some(operator) = self.advance_if(|t| {
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

    fn term(&'s mut self) -> Expr<'s> {
        let mut expr = self.factor();

        while let Some(operator) =
            self.advance_if(|t| matches!(t.typ, TokenType::Minus | TokenType::Plus))
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

    fn factor(&'s mut self) -> Expr<'s> {
        let mut expr = self.unary();

        while let Some(operator) =
            self.advance_if(|t| matches!(t.typ, TokenType::Slash | TokenType::Star))
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

    fn unary(&'s mut self) -> Expr<'s> {
        if let Some(operator) =
            self.advance_if(|t| matches!(t.typ, TokenType::Bang | TokenType::Minus))
        {
            let right = self.unary();
            return Expr::Unary {
                op: operator,
                right: Box::new(right),
            };
        }

        self.primary()
    }

    fn primary(&'s mut self) -> Expr<'s> {
        if let Some(token) = self.advance() {
            match token.typ {
                TokenType::False => Expr::Literal { token },
                TokenType::True => Expr::Literal { token },
                TokenType::Nil => Expr::Literal { token },
                TokenType::Number(_) => Expr::Literal { token },
                TokenType::String(_) => Expr::Literal { token },
                TokenType::LeftParen => {
                    let expr = self.expression();
                    self.advance_if(|t| matches!(t.typ, TokenType::RightParen))
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
