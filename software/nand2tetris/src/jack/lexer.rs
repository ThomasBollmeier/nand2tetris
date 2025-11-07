use crate::grammarous::stream::{BufferedStream, Stream};
use crate::grammarous::token::Token;
use crate::jack::token_type::TokenType;
use std::collections::HashMap;

pub type JackToken = Token<TokenType>;

const MAX_JACK_INT_VALUE: u16 = 32767;

pub struct Lexer<'a> {
    stream: BufferedStream<'a, char>,
    line: usize,
    column: usize,
    symbols: HashMap<char, TokenType>,
    keywords: HashMap<String, TokenType>,
}

impl<'a> Lexer<'a> {
    pub fn new(stream: &'a mut dyn Stream<char>) -> Self {
        Self {
            stream: BufferedStream::new(stream),
            line: 1,
            column: 1,
            symbols: Self::init_symbols(),
            keywords: Self::init_keywords(),
        }
    }

    fn init_symbols() -> HashMap<char, TokenType> {
        let mut symbols = HashMap::new();
        symbols.insert('{', TokenType::LBrace);
        symbols.insert('}', TokenType::RBrace);
        symbols.insert('(', TokenType::LParen);
        symbols.insert(')', TokenType::RParen);
        symbols.insert('[', TokenType::LBracket);
        symbols.insert(']', TokenType::RBracket);
        symbols.insert('.', TokenType::Dot);
        symbols.insert(',', TokenType::Comma);
        symbols.insert(';', TokenType::Semicolon);
        symbols.insert('+', TokenType::Plus);
        symbols.insert('-', TokenType::Minus);
        symbols.insert('*', TokenType::Asterisk);
        symbols.insert('/', TokenType::Slash);
        symbols.insert('&', TokenType::Ampersand);
        symbols.insert('|', TokenType::Pipe);
        symbols.insert('<', TokenType::LessThan);
        symbols.insert('>', TokenType::GreaterThan);
        symbols.insert('=', TokenType::Equal);
        symbols.insert('~', TokenType::Tilde);
        symbols
    }

    fn init_keywords() -> HashMap<String, TokenType> {
        let mut keywords = HashMap::new();
        keywords.insert("class".to_string(), TokenType::Class);
        keywords.insert("constructor".to_string(), TokenType::Constructor);
        keywords.insert("function".to_string(), TokenType::Function);
        keywords.insert("method".to_string(), TokenType::Method);
        keywords.insert("field".to_string(), TokenType::Field);
        keywords.insert("static".to_string(), TokenType::Static);
        keywords.insert("var".to_string(), TokenType::Var);
        keywords.insert("int".to_string(), TokenType::Int);
        keywords.insert("char".to_string(), TokenType::Char);
        keywords.insert("boolean".to_string(), TokenType::Boolean);
        keywords.insert("void".to_string(), TokenType::Void);
        keywords.insert("true".to_string(), TokenType::True);
        keywords.insert("false".to_string(), TokenType::False);
        keywords.insert("null".to_string(), TokenType::Null);
        keywords.insert("this".to_string(), TokenType::This);
        keywords.insert("let".to_string(), TokenType::Let);
        keywords.insert("do".to_string(), TokenType::Do);
        keywords.insert("if".to_string(), TokenType::If);
        keywords.insert("else".to_string(), TokenType::Else);
        keywords.insert("while".to_string(), TokenType::While);
        keywords.insert("return".to_string(), TokenType::Return);
        keywords
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

    fn skip_block_comment(&mut self) {
        // Skip the '/*' characters
        self.next_char();
        self.next_char();
        loop {
            let next_chars = self.stream.peek_n(2).iter().collect::<String>();
            if next_chars.is_empty() {
                break; // End of stream reached without closing comment
            }
            if next_chars == "*/" {
                // Skip the '*/' characters
                self.next_char();
                self.next_char();
                break;
            } else {
                self.next_char(); // Advance one character
            }
        }
    }

    fn integer_constant(&mut self, first_digit: char, line: usize, column: usize) -> JackToken {
        let mut lexeme = String::new();
        lexeme.push(first_digit);

        while let Some(ch) = self.stream.peek() {
            if ch.is_ascii_digit() {
                lexeme.push(ch);
                self.next_char();
            } else {
                break;
            }
        }

        match lexeme.parse::<u16>() {
            Ok(value) => {
                if value <= MAX_JACK_INT_VALUE {
                    JackToken::new(TokenType::IntegerConstant(value), lexeme, line, column)
                } else {
                    let message = format!(
                        "Integer constant out of range: '{}' at line {}, column {}",
                        lexeme, line, column
                    );
                    JackToken::new(TokenType::Error { message }, lexeme, line, column)
                }
            },
            Err(_) => {
                let message = format!(
                    "Invalid integer constant: '{}' at line {}, column {}",
                    lexeme, line, column
                );
                JackToken::new(TokenType::Error { message }, lexeme, line, column)
            }
        }
    }

    fn identifier_or_keyword(&mut self, first_char: char, line: usize, column: usize) -> JackToken {
        let mut lexeme = String::new();
        lexeme.push(first_char);

        while let Some(ch) = self.stream.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                lexeme.push(ch);
                self.next_char();
            } else {
                break;
            }
        }

        if let Some(token_type) = self.keywords.get(&lexeme) {
            JackToken::new(token_type.clone(), lexeme, line, column)
        } else {
            JackToken::new(TokenType::Identifier(lexeme.clone()), lexeme, line, column)
        }
    }

    fn string_constant(&mut self, line: usize, column: usize) -> JackToken {
        let mut lexeme = String::new();
        lexeme.push('"');

        while let Some(ch) = self.next_char() {
            lexeme.push(ch);
            if ch == '"' {
                // Closing quote found
                return JackToken::new(TokenType::StringConstant(lexeme.clone()), lexeme, line, column);
            }
        }

        // If we reach here, the string was not closed
        let message = format!("Unterminated string constant at line {}, column {}", line, column);
        JackToken::new(TokenType::Error { message }, lexeme, line, column)
    }

    fn symbol_token(&mut self, ch: char, line: usize, column: usize) -> JackToken {
        if let Some(token_type) = self.symbols.get(&ch) {
            JackToken::new(token_type.clone(), ch.to_string(), line, column)
        } else {
            Self::error_token_unexpected_char(ch, line, column)
        }
    }

    fn error_token_unexpected_char(ch: char, line: usize, column: usize) -> JackToken {
        let message = format!("Unexpected character: '{ch}' at line {line}, column {column}");
        JackToken::new(TokenType::Error { message }, ch.to_string(), line, column)
    }
}

impl Stream<JackToken> for Lexer<'_> {
    fn advance(&mut self) -> Option<JackToken> {
        loop {
            self.skip_whitespace();

            let next_chars = self.stream.peek_n(2).iter().collect::<String>();

            if next_chars.is_empty() {
                return None;
            }

            if next_chars == "//" {
                self.skip_line_comment();
                continue;
            }

            if next_chars == "/*" {
                self.skip_block_comment();
                continue;
            }

            let line = self.line;
            let column = self.column;

            match self.next_char() {
                Some(ch) if ch.is_ascii_digit() => {
                    return Some(self.integer_constant(ch, line, column));
                }
                Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {
                    return Some(self.identifier_or_keyword(ch, line, column));
                }
                Some('"') => {
                    return Some(self.string_constant(line, column));
                }
                Some(ch) => {
                    return Some(Self::symbol_token(self, ch, line, column));
                }
                None => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammarous::string_char_stream::StringCharStream;

    #[test]
    fn test_lexical_analyzer_simple() {
        let input = r#"
class Main {

    function void main() {
        var int answer;
        let answer = 42; // The answer to everything
        Output.println("Hello, World!");
    }

}"#;
        let mut char_stream = StringCharStream::new(input);
        let mut lexer = Lexer::new(&mut char_stream);

        let mut tokens = Vec::new();
        while let Some(token) = lexer.advance() {
            tokens.push(token);
        }

        let expected_tokens = vec![
            Token::new(TokenType::Class, "class".to_string(), 2, 1),
            Token::new(TokenType::Identifier("Main".to_string()), "Main".to_string(), 2, 7),
            Token::new(TokenType::LBrace, "{".to_string(), 2, 12),
            Token::new(TokenType::Function, "function".to_string(), 4, 5),
            Token::new(TokenType::Void, "void".to_string(), 4, 14),
            Token::new(TokenType::Identifier("main".to_string()), "main".to_string(), 4, 19),
            Token::new(TokenType::LParen, "(".to_string(), 4, 23),
            Token::new(TokenType::RParen, ")".to_string(), 4, 24),
            Token::new(TokenType::LBrace, "{".to_string(), 4, 26),
            Token::new(TokenType::Var, "var".to_string(), 5, 9),
            Token::new(TokenType::Int, "int".to_string(), 5, 13),
            Token::new(TokenType::Identifier("answer".to_string()), "answer".to_string(), 5, 17),
            Token::new(TokenType::Semicolon, ";".to_string(), 5, 23),
            Token::new(TokenType::Let, "let".to_string(), 6, 9),
            Token::new(TokenType::Identifier("answer".to_string()), "answer".to_string(), 6, 13),
            Token::new(TokenType::Equal, "=".to_string(), 6, 20),
            Token::new(TokenType::IntegerConstant(42), "42".to_string(), 6, 22),
            Token::new(TokenType::Semicolon, ";".to_string(), 6, 24),
            Token::new(TokenType::Identifier("Output".to_string()), "Output".to_string(), 7, 9),
            Token::new(TokenType::Dot, ".".to_string(), 7, 15),
            Token::new(TokenType::Identifier("println".to_string()), "println".to_string(), 7, 16),
            Token::new(TokenType::LParen, "(".to_string(), 7, 23),
            Token::new(TokenType::StringConstant("\"Hello, World!\"".to_string()), "\"Hello, World!\"".to_string(), 7, 24),
            Token::new(TokenType::RParen, ")".to_string(), 7, 39),
            Token::new(TokenType::Semicolon, ";".to_string(), 7, 40),
            Token::new(TokenType::RBrace, "}".to_string(), 8, 5),
            Token::new(TokenType::RBrace, "}".to_string(), 10, 1),
        ];

        assert_eq!(tokens.len(), expected_tokens.len());

        for (token, expected) in tokens.iter().zip(expected_tokens.iter()) {
            assert_eq!(token, expected);
        }

    }

}