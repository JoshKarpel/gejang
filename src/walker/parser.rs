use crate::shared::scanner::Token;

struct Parser<'s> {
    tokens: Vec<Token<'s>>,
    current: usize,
}

impl<'s> Parser<'s> {
    fn new(tokens: Vec<Token<'s>>) -> Self {
        Parser { tokens, current: 0 }
    }
}
