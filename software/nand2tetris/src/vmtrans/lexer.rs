use crate::grammarous::stream::{BufferedStream, Stream};
use crate::grammarous::token::Token;
use crate::vmtrans::token_type::TokenType;

pub type VmToken = Token<TokenType>;

pub struct Lexer<'a> {
    stream: BufferedStream<'a, char>,
    line: usize,
    column: usize,
}

impl <'a> Lexer<'a> {
    pub fn new(stream: &'a mut dyn Stream<char>) -> Self {
        Self {
            stream: BufferedStream::new(stream),
            line: 1,
            column: 1,
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.stream.advance();
        if let Some(c) = ch {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.stream.peek() {
            if ch.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        // Skip the '//' characters
        self.next_char();
        self.next_char();
        while let Some(ch) = self.next_char() {
            if ch == '\n' {
                break;
            }
        }
    }

    fn handle_chars(&mut self, predicate_fn: &dyn Fn(char) -> bool) -> VmToken {
        let line = self.line;
        let column = self.column;
        let mut lexeme = String::new();

        let first_char = self.next_char().unwrap();
        lexeme.push(first_char);

        while let Some(ch) = self.stream.peek() {
            if predicate_fn(ch) {
                lexeme.push(ch);
                self.next_char();
            } else {
                break;
            }
        }

        VmToken::new(TokenType::from(lexeme.as_str()), lexeme, line, column)
    }

    fn handle_unexpected_char(&mut self, ch: char) -> VmToken {
        let error_token = VmToken::new(
            TokenType::Error(format!("Unexpected character '{}' at line {}", ch, self.line)),
            "".to_string(),
            self.line,
            self.column,
        );
        self.next_char();
        error_token
    }
}

impl Stream<VmToken> for Lexer<'_> {
    fn advance(&mut self) -> Option<VmToken> {
        loop {
            self.skip_whitespace();
            let next_chars = self.stream.peek_n(2);
            if next_chars.is_empty() {
                return None;
            }
            match next_chars[0] {
                '/' => {
                    if next_chars.len() > 1 && next_chars[1] == '/' {
                        self.skip_line_comment();
                        continue;
                    } else {
                        return Some(self.handle_unexpected_char('/'));
                    }
                }
                ch => {
                    return if ch.is_ascii_alphabetic() {
                        Some(self.handle_chars(&|c: char| !c.is_whitespace()))
                    } else if ch.is_digit(10) {
                        Some(self.handle_chars(&|c: char| c.is_digit(10)))
                    } else {
                        Some(self.handle_unexpected_char(ch))
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammarous::string_char_stream::StringCharStream;

    #[test]
    fn test_lexer_basic() {
        let input = "push constant 10\n\npop local 0\n// This is a comment\nadd\n";
        let mut char_stream = StringCharStream::new(input);
        let mut lexer = Lexer::new(&mut char_stream);

        let expected_tokens = vec![
            VmToken::new(TokenType::Push, "push".to_string(), 1, 1),
            VmToken::new(TokenType::Constant, "constant".to_string(), 1, 6),
            VmToken::new(TokenType::Index(10), "10".to_string(), 1, 15),
            VmToken::new(TokenType::Pop, "pop".to_string(), 3, 1),
            VmToken::new(TokenType::Local, "local".to_string(), 3, 5),
            VmToken::new(TokenType::Index(0), "0".to_string(), 3, 11),
            VmToken::new(TokenType::Add, "add".to_string(), 5, 1),
        ];

        for expected in expected_tokens {
            let token = lexer.advance().unwrap();
            assert_eq!(token, expected);
        }

        assert!(lexer.advance().is_none());
    }
}
