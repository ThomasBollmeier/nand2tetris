pub trait Stream<T:Clone> {
    fn advance(&mut self) -> Option<T>;
}

pub struct BufferedStream<'a, T: Clone> {
    buffer: Vec<T>,
    stream: &'a mut dyn Stream<T>,
}

impl<'a, T:Clone> BufferedStream<'a, T> {
    pub fn new(stream: &'a mut dyn Stream<T>) -> Self {
        Self {
            buffer: Vec::new(),
            stream,
        }
    }
    
    pub fn peek(&mut self) -> Option<T> {
        self.peek_nth(0)
    }
    
    pub fn peek_n(&mut self, n: usize) -> Vec<T> {
        self.fill_buffer(n);
        self.buffer.iter().take(n).cloned().collect()
    }
    
    pub fn peek_nth(&mut self, n: usize) -> Option<T> {
        self.fill_buffer(n + 1);
        self.buffer.get(n).cloned()
    }
    
    pub fn advance(&mut self) -> Option<T> {
        if !self.buffer.is_empty() {
            return self.buffer.remove(0).into();
        }
        self.stream.advance()
    }

    fn fill_buffer(&mut self, n: usize) {
        while self.buffer.len() < n {
            if let Some(item) = self.stream.advance() {
                self.buffer.push(item);
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct TestStream {
        data: Vec<char>,
        position: usize,
    }
    impl TestStream {
        fn new(data: &str) -> Self {
            Self {
                data: data.chars().collect(),
                position: 0,
            }
        }
    }
    impl Stream<char> for TestStream {
        fn advance(&mut self) -> Option<char> {
            if self.position < self.data.len() {
                let ch = self.data[self.position];
                self.position += 1;
                Some(ch)
            } else {
                None
            }
        }
    }

    #[test]
    fn test_buffered_stream() {
        let mut test_stream = TestStream::new("abcde");
        let mut buffered_stream = BufferedStream::new(&mut test_stream);

        assert_eq!(buffered_stream.peek(), Some('a'));
        assert_eq!(buffered_stream.peek_nth(2), Some('c'));
        assert_eq!(buffered_stream.peek_n(3), vec!['a', 'b', 'c']);

        assert_eq!(buffered_stream.advance(), Some('a'));
        assert_eq!(buffered_stream.advance(), Some('b'));
        assert_eq!(buffered_stream.peek(), Some('c'));

        assert_eq!(buffered_stream.advance(), Some('c'));
        assert_eq!(buffered_stream.advance(), Some('d'));
        assert_eq!(buffered_stream.advance(), Some('e'));
        assert_eq!(buffered_stream.advance(), None);
    }
}