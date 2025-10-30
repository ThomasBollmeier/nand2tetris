use crate::grammarous::stream::Stream;

pub struct StringCharStream {
    input: Vec<char>,
    position: usize,
}

impl StringCharStream {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }
}

impl Stream<char> for StringCharStream {
    fn advance(&mut self) -> Option<char> {
        if self.position < self.input.len() {
            let ch = self.input[self.position];
            self.position += 1;
            Some(ch)
        } else {
            None
        }
    }
}