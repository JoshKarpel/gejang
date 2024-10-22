use std::{fmt::Display, str::CharIndices};

use thiserror::Error;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum TokenType<'s> {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier(&'s str),
    String(&'s str),
    Number(f64),
    And,
    Break,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    Comment(&'s str),
    While,
}

impl Display for TokenType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TokenType::LeftParen => "(",
                TokenType::RightParen => ")",
                TokenType::LeftBrace => "{",
                TokenType::RightBrace => "}",
                TokenType::Comma => ",",
                TokenType::Dot => ".",
                TokenType::Minus => "-",
                TokenType::Plus => "+",
                TokenType::Semicolon => ";",
                TokenType::Slash => "/",
                TokenType::Star => "*",
                TokenType::Bang => "!",
                TokenType::BangEqual => "!=",
                TokenType::Equal => "=",
                TokenType::EqualEqual => "==",
                TokenType::Greater => ">",
                TokenType::GreaterEqual => ">=",
                TokenType::Less => "<",
                TokenType::LessEqual => "<=",
                TokenType::Identifier(_) => "an identifier",
                TokenType::String(_) => "a string",
                TokenType::Number(_) => "a number",
                TokenType::And => "and",
                TokenType::Break => "break",
                TokenType::Class => "class",
                TokenType::Else => "else",
                TokenType::False => "false",
                TokenType::For => "for",
                TokenType::Fun => "fun",
                TokenType::If => "if",
                TokenType::Nil => "nil",
                TokenType::Or => "or",
                TokenType::Print => "print",
                TokenType::Return => "return",
                TokenType::Super => "super",
                TokenType::This => "this",
                TokenType::True => "true",
                TokenType::Var => "var",
                TokenType::Comment(_) => "a comment",
                TokenType::While => "while",
            }
        )
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary,
}

impl Precedence {
    pub fn next(&self) -> Self {
        match self {
            Precedence::None => Precedence::Assignment,
            Precedence::Assignment => Precedence::Or,
            Precedence::Or => Precedence::And,
            Precedence::And => Precedence::Equality,
            Precedence::Equality => Precedence::Comparison,
            Precedence::Comparison => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Unary,
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => unreachable!("No precedence higher than Primary"),
        }
    }
}

impl TokenType<'_> {
    pub fn precedence(&self) -> Precedence {
        match self {
            TokenType::Plus | TokenType::Minus => Precedence::Term,
            TokenType::Star | TokenType::Slash => Precedence::Factor,
            _ => Precedence::None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Token<'s> {
    pub typ: TokenType<'s>,
    pub lexeme: &'s str,
    pub line: usize,
}

#[derive(Error, Clone, PartialEq, PartialOrd, Debug)]
pub enum ScannerError {
    #[error("Unexpected character on line {line}: {char}")]
    UnexpectedCharacter { line: usize, char: char },
    #[error("Unterminated string on line {line}")]
    UnterminatedString { line: usize },
    #[error("Invalid number on line {line}: {number}")]
    InvalidNumber { line: usize, number: String },
}

type ScannerResult<'s> = Result<Token<'s>, ScannerError>;

#[derive(Clone, Debug)]
struct Scanner<'s> {
    source: &'s str,
    cursor: CharIndices<'s>,
    current_offset: usize,
    lexeme_start: usize,
    line: usize,
}

impl<'s> From<&'s str> for Scanner<'s> {
    fn from(source: &'s str) -> Self {
        Self {
            source,
            cursor: source.char_indices(),
            current_offset: 0,
            lexeme_start: 0,
            line: 0,
        }
    }
}

impl<'s> Scanner<'s> {
    fn advance(&mut self) -> Option<(usize, char)> {
        self.cursor.next().inspect(|(offset, c)| {
            self.current_offset = *offset + c.len_utf8();
            if *c == '\n' {
                self.line += 1;
            }
        })
    }

    fn advance_if_match(&mut self, expected: char) -> bool {
        // Peekable has a similar interface, but it wouldn't go through our custom advance(),
        // so we wouldn't get to update the line number and offset.
        self.peek()
            .is_some_and(|c| c == expected)
            .then(|| self.advance())
            .is_some()
    }

    fn advance_while(&mut self, predicate: fn(char) -> bool) {
        while self.peek().is_some_and(predicate) {
            self.advance();
        }
    }

    fn peek(&self) -> Option<char> {
        // Cloning the cursor is cheap - it's just a reference to the original source and an offset.
        self.cursor.clone().next().map(|(_, c)| c)
    }

    fn peek_peek(&self) -> Option<char> {
        self.cursor.clone().nth(1).map(|(_, c)| c)
    }

    fn lexeme(&self) -> &'s str {
        &self.source[self.lexeme_start..self.current_offset]
    }

    fn make_token(&self, typ: TokenType<'s>) -> ScannerResult<'s> {
        Ok(Token {
            typ,
            lexeme: self.lexeme(),
            line: self.line,
        })
    }
}

impl<'s> Iterator for Scanner<'s> {
    type Item = ScannerResult<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance_while(|c| c.is_whitespace());

        self.advance().map(|(lexeme_start, c)| {
            self.lexeme_start = lexeme_start;
            match c {
                '(' => self.make_token(TokenType::LeftParen),
                ')' => self.make_token(TokenType::RightParen),
                '{' => self.make_token(TokenType::LeftBrace),
                '}' => self.make_token(TokenType::RightBrace),
                ',' => self.make_token(TokenType::Comma),
                '.' => self.make_token(TokenType::Dot),
                '-' => self.make_token(TokenType::Minus),
                '+' => self.make_token(TokenType::Plus),
                ';' => self.make_token(TokenType::Semicolon),
                '*' => self.make_token(TokenType::Star),
                '!' => {
                    if self.advance_if_match('=') {
                        self.make_token(TokenType::BangEqual)
                    } else {
                        self.make_token(TokenType::Bang)
                    }
                }
                '=' => {
                    if self.advance_if_match('=') {
                        self.make_token(TokenType::EqualEqual)
                    } else {
                        self.make_token(TokenType::Equal)
                    }
                }
                '<' => {
                    if self.advance_if_match('=') {
                        self.make_token(TokenType::LessEqual)
                    } else {
                        self.make_token(TokenType::Less)
                    }
                }
                '>' => {
                    if self.advance_if_match('=') {
                        self.make_token(TokenType::GreaterEqual)
                    } else {
                        self.make_token(TokenType::Greater)
                    }
                }
                '/' => {
                    if self.advance_if_match('/') {
                        self.advance_while(|c| c != '\n');
                        self.make_token(TokenType::Comment(self.lexeme()))
                    } else {
                        self.make_token(TokenType::Slash)
                    }
                }
                '"' => {
                    while let Some((_, c)) = self.advance() {
                        if c == '"' {
                            return self.make_token(TokenType::String(
                                // Adjusting the bounds manually here to strip the quotes off is safe,
                                // because we know that the lexeme is bounded by ASCII quote characters.
                                self.source[self.lexeme_start + 1..self.current_offset - 1].into(),
                            ));
                        }
                    }
                    Err(ScannerError::UnterminatedString { line: self.line })
                }
                '0'..='9' => {
                    self.advance_while(|c| c.is_ascii_digit());

                    if self.peek().is_some_and(|c| c == '.')
                        && self.peek_peek().is_some_and(|c| c.is_ascii_digit())
                    {
                        self.advance(); // consume the .
                        self.advance_while(|c| c.is_ascii_digit());
                    }

                    if let Ok(number) = self.lexeme().parse() {
                        self.make_token(TokenType::Number(number))
                    } else {
                        Err(ScannerError::InvalidNumber {
                            line: self.line,
                            number: self.lexeme().into(),
                        })
                    }
                }
                c if c.is_alphabetic() => {
                    self.advance_while(|c| c.is_alphanumeric() || c == '_');

                    match self.lexeme() {
                        "and" => self.make_token(TokenType::And),
                        "break" => self.make_token(TokenType::Break),
                        "class" => self.make_token(TokenType::Class),
                        "else" => self.make_token(TokenType::Else),
                        "false" => self.make_token(TokenType::False),
                        "for" => self.make_token(TokenType::For),
                        "fun" => self.make_token(TokenType::Fun),
                        "if" => self.make_token(TokenType::If),
                        "nil" => self.make_token(TokenType::Nil),
                        "or" => self.make_token(TokenType::Or),
                        "print" => self.make_token(TokenType::Print),
                        "return" => self.make_token(TokenType::Return),
                        "super" => self.make_token(TokenType::Super),
                        "this" => self.make_token(TokenType::This),
                        "true" => self.make_token(TokenType::True),
                        "var" => self.make_token(TokenType::Var),
                        "while" => self.make_token(TokenType::While),
                        lexeme => self.make_token(TokenType::Identifier(lexeme)),
                    }
                }
                _ => Err(ScannerError::UnexpectedCharacter {
                    char: c,
                    line: self.line,
                }),
            }
        })
    }
}

pub fn scan(source: &str) -> impl Iterator<Item = ScannerResult> + '_ {
    Scanner::from(source)
}

#[cfg(test)]
mod tests {
    extern crate test;

    use itertools::Itertools;
    use rstest::rstest;
    use test::Bencher;

    use super::*;

    #[rstest]
    #[case("1 + 2", vec![
        Ok(Token {
            typ: TokenType::Number(1.0),
            lexeme: "1",
            line: 0,
        }),
        Ok(Token {
            typ: TokenType::Plus,
            lexeme: "+",
            line: 0,
        }),
        Ok(Token {
            typ: TokenType::Number(2.0),
            lexeme: "2",
            line: 0,
        }),
    ])]
    #[case("\"foo", vec![
        Err(ScannerError::UnterminatedString { line: 0 }),
    ])]
    #[case("\"foo\"", vec![
        Ok(Token {
            typ: TokenType::String("foo"),
            lexeme: "\"foo\"",
            line: 0,
        }),
    ])]
    #[case("\"foo\"\n\"bar\"", vec![
        Ok(Token {
            typ: TokenType::String("foo"),
            lexeme: "\"foo\"",
            line: 0,
        }),
        Ok(Token {
            typ: TokenType::String("bar"),
            lexeme: "\"bar\"",
            line: 1,
        }),
    ])]
    #[case("123", vec![
        Ok(Token {
            typ: TokenType::Number(123.0),
            lexeme: "123",
            line: 0,
        }),
    ])]
    #[case("123.123", vec![
        Ok(Token {
            typ: TokenType::Number(123.123),
            lexeme: "123.123",
            line: 0,
        }),
    ])]
    #[case("123.", vec![
        Ok(Token {
            typ: TokenType::Number(123.0),
            lexeme: "123",
            line: 0,
        }),
        Ok(Token {
            typ: TokenType::Dot,
            lexeme: ".",
            line: 0,
        }),
    ])]
    #[case("123.foo", vec![
        Ok(Token {
            typ: TokenType::Number(123.0),
            lexeme: "123",
            line: 0,
        }),
        Ok(Token {
            typ: TokenType::Dot,
            lexeme: ".",
            line: 0,
        }),
        Ok(Token {
            typ: TokenType::Identifier("foo"),
            lexeme: "foo",
            line: 0,
        }),
    ])]
    #[case("printfoo", vec![
        Ok(Token {
            typ: TokenType::Identifier("printfoo"),
            lexeme: "printfoo",
            line: 0,
        }),
    ])]
    #[case("print foo", vec![
        Ok(Token {
            typ: TokenType::Print,
            lexeme: "print",
            line: 0,
        }),
        Ok(Token {
            typ: TokenType::Identifier("foo"),
            lexeme: "foo",
            line: 0,
        }),
    ])]
    #[case("안녕하세요", vec![
        Ok(Token {
            typ: TokenType::Identifier("안녕하세요"),
            lexeme: "안녕하세요",
            line: 0,
        }),
    ])]
    #[case("λ", vec![
        Ok(Token {
            typ: TokenType::Identifier("λ"),
            lexeme: "λ",
            line: 0,
        }),
    ])]
    #[case("λ + bar", vec![
        Ok(Token {
            typ: TokenType::Identifier("λ"),
            lexeme: "λ",
            line: 0,
        }),
        Ok(Token {
            typ: TokenType::Plus,
            lexeme: "+",
            line: 0,
        }),
        Ok(Token {
            typ: TokenType::Identifier("bar"),
            lexeme: "bar",
            line: 0,
        }),
    ])]
    #[case("λ.bar", vec![
        Ok(Token {
            typ: TokenType::Identifier("λ"),
            lexeme: "λ",
            line: 0,
        }),
        Ok(Token {
            typ: TokenType::Dot,
            lexeme: ".",
            line: 0,
        }),
        Ok(Token {
            typ: TokenType::Identifier("bar"),
            lexeme: "bar",
            line: 0,
        }),
    ])]
    #[case("\"λ\"", vec![
        Ok(Token {
            typ: TokenType::String("λ"),
            lexeme: "\"λ\"",
            line: 0,
        }),
    ])]
    // TODO: support emoji identifiers
    // #[case("🦀", vec![
    //     Ok(Token {
    //         typ: TokenType::Identifier("🦀"),
    //         lexeme: "🦀",
    //         line: 0,
    //     }),
    // ])]
    // #[case("🦀 + bar", vec![
    //     Ok(Token {
    //         typ: TokenType::Identifier("🦀"),
    //         lexeme: "🦀",
    //         line: 0,
    //     }),
    //     Ok(Token {
    //         typ: TokenType::Plus,
    //         lexeme: "+",
    //         line: 0,
    //     }),
    //     Ok(Token {
    //         typ: TokenType::Identifier("bar"),
    //         lexeme: "bar",
    //         line: 0,
    //     }),
    // ])]
    // #[case("🦀.bar", vec![
    //     Ok(Token {
    //         typ: TokenType::Identifier("🦀"),
    //         lexeme: "🦀",
    //         line: 0,
    //     }),
    //     Ok(Token {
    //         typ: TokenType::Dot,
    //         lexeme: ".",
    //         line: 0,
    //     }),
    //     Ok(Token {
    //         typ: TokenType::Identifier("bar"),
    //         lexeme: "bar",
    //         line: 0,
    //     }),
    // ])]
    // #[case("🦀.λ", vec![
    //     Ok(Token {
    //         typ: TokenType::Identifier("🦀"),
    //         lexeme: "🦀",
    //         line: 0,
    //     }),
    //     Ok(Token {
    //         typ: TokenType::Dot,
    //         lexeme: ".",
    //         line: 0,
    //     }),
    //     Ok(Token {
    //         typ: TokenType::Identifier("λ"),
    //         lexeme: "λ",
    //         line: 0,
    //     }),
    // ])]
    fn test_scanner(#[case] source: &str, #[case] expected: Vec<Result<Token, ScannerError>>) {
        assert_eq!(scan(source).collect_vec(), expected);
    }

    #[test]
    fn scan_hello_world() {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/hello_world.🦀"
        ));

        let tokens: Vec<_> = scan(source).collect();

        assert!(tokens.iter().all(|t| t.is_ok()));
        assert_eq!(tokens.len(), 437);
    }

    #[bench]
    fn bench_scan_hello_world(b: &mut Bencher) {
        let source = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/examples/hello_world.🦀"
        ))
        .repeat(100);

        b.iter(|| {
            scan(&source).for_each(drop);
        });
    }
}
