use std::{
    io,
    io::{Read, Stderr, Stdin, Stdout, Write},
};

#[derive(Debug)]
pub struct Streams<I: Read, O: Write, E: Write> {
    pub input: I,
    pub output: O,
    pub error: E,
}

impl Streams<Stdin, Stdout, Stderr> {
    pub fn new() -> Self {
        Streams {
            input: io::stdin(),
            output: io::stdout(),
            error: io::stderr(),
        }
    }
}

#[cfg(test)]
impl Streams<&[u8], Vec<u8>, Vec<u8>> {
    pub fn test() -> Self {
        Streams {
            input: &[],
            output: Vec::new(),
            error: Vec::new(),
        }
    }
}