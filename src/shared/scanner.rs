use std::str::CharIndices;

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

type LineNumber = usize;

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct Token<'s> {
    typ: TokenType<'s>,
    lexeme: &'s str,
    line: LineNumber,
}

#[derive(Error, Clone, PartialEq, PartialOrd, Debug)]
pub enum ScannerError {
    #[error("Unexpected character on line {line}: {char}")]
    UnexpectedCharacter { line: LineNumber, char: char },
    #[error("Unterminated string on line {line}")]
    UnterminatedString { line: LineNumber },
    #[error("Invalid number on line {line}: {number}")]
    InvalidNumber { line: LineNumber, number: String },
}

type ScannerResult<'s> = Result<Token<'s>, ScannerError>;

#[derive(Clone, Debug)]
struct Scanner<'s> {
    source: &'s str,
    cursor: CharIndices<'s>,
    current_offset: usize,
    lexeme_start: usize,
    line: LineNumber,
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
            self.current_offset = *offset;
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

    fn peek(&self) -> Option<char> {
        // Cloning the cursor is cheap - it's just a reference to the original source and an offset.
        self.cursor.clone().next().map(|(_, c)| c)
    }

    fn peek_peek(&self) -> Option<char> {
        self.cursor.clone().nth(1).map(|(_, c)| c)
    }

    fn lexeme(&self) -> &'s str {
        &self.source[self.lexeme_start..=self.current_offset]
    }

    fn make_token(&self, typ: TokenType<'s>) -> Result<Token<'s>, ScannerError> {
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
        while self.peek().is_some_and(|c| c.is_whitespace()) {
            self.advance();
        }

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
                        while let Some(c) = self.peek() {
                            if c != '\n' {
                                self.advance();
                            } else {
                                break;
                            }
                        }
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
                                self.source[self.lexeme_start + 1..self.current_offset].into(),
                            ));
                        }
                    }
                    Err(ScannerError::UnterminatedString { line: self.line })
                }
                '0'..='9' => {
                    while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                        self.advance();
                    }

                    if self.peek().is_some_and(|c| c == '.')
                        && self.peek_peek().is_some_and(|c| c.is_ascii_digit())
                    {
                        self.advance(); // consume the .
                        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                            self.advance();
                        }
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
                'a'..='z' | 'A'..='Z' | '_' => {
                    while self.peek().is_some_and(|c| c.is_alphanumeric() || c == '_') {
                        self.advance();
                    }

                    match self.lexeme() {
                        "and" => self.make_token(TokenType::And),
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
