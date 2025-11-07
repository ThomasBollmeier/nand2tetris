use crate::grammarous::stream::{BufferedStream, Stream};
use crate::jack::ast::{Ast, AstData};
use crate::jack::lexer::JackToken;
use crate::jack::token_type::{TokenTypeCategory::{self, *}};

pub struct Parser<'a> {
    stream: BufferedStream<'a, JackToken>,
}


impl<'a> Parser<'a> {
    pub fn new(stream: &'a mut dyn Stream<JackToken>) -> Self {
        Self {
            stream: BufferedStream::new(stream),
        }
    }

    pub fn parse_class(&mut self) -> Result<Ast, String> {
        let mut class_data = AstData::new("class", None);

        let mut token = self.consume(Class)?;
        class_data.add_token(token);

        token = self.consume(Identifier)?;
        class_data.add_token(token);

        token = self.consume(LBrace)?;
        class_data.add_token(token);

        self.parse_var_declarations(&mut class_data)?;

        token = self.consume(RBrace)?;
        class_data.add_token(token);

        Ok(Ast::NonTerminal(class_data))
    }

    fn parse_var_declarations(&mut self, class_data: &mut AstData) -> Result<(), String> {
        loop {
            let next_token  = self.peek();
            if next_token.is_none() {
                return Err("Unexpected end of input while parsing variable declarations".to_string());
            }
            let next_token = next_token.unwrap();
            match next_token.token_type.get_category() {
                Static | Field => {
                    let mut class_var_dec = AstData::new("classVarDec", None);

                    let token = self.consume_any_of(&[Static, Field])?;
                    class_var_dec.add_token(token);

                    let token = self.consume_any_of(&[Int, Char, Boolean, Identifier])?;
                    class_var_dec.add_token(token);

                    let token = self.consume(Identifier)?;
                    class_var_dec.add_token(token);

                    loop {
                        let next_token = self.peek();
                        if let Some(token) = next_token {
                            if token.token_type.get_category() == Comma {
                                let token = self.consume(Comma)?;
                                class_var_dec.add_token(token);

                                let token = self.consume(Identifier)?;
                                class_var_dec.add_token(token);
                            } else {
                                break;
                            }
                        } else {
                            return Err("Unexpected end of input in variable declaration".to_string());
                        }
                    }

                    let token = self.consume(Semicolon)?;
                    class_var_dec.add_token(token);

                    class_data.add_child(class_var_dec);
                },
                _ => break,
            }

        }

        Ok(())
    }

    fn peek(&mut self) -> Option<JackToken> {
        self.stream.peek()
    }

    fn peek_n(&mut self, n: usize) -> Vec<JackToken> {
        self.stream.peek_n(n)
    }

    fn consume(&mut self, expected: TokenTypeCategory) -> Result<JackToken, String> {
        match self.stream.advance() {
            Some(token) if token.token_type.get_category() == expected => Ok(token),
            Some(token) => Err(format!(
                "Expected {:?}, found {:?}",
                expected, token.token_type
            )),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn consume_any_of(&mut self, expected: &[TokenTypeCategory]) -> Result<JackToken, String> {
        match self.stream.advance() {
            Some(token) if expected.contains(&token.token_type.get_category()) => Ok(token),
            Some(token) => Err(format!(
                "Expected one of {:?}, found {:?}",
                expected, token.token_type
            )),
            None => Err("Unexpected end of input".to_string()),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammarous::string_char_stream::StringCharStream;
    use crate::jack::lexer::Lexer;

    #[test]
    fn test_parse_class() {
        let input = r#"
        class Person {
            field boolean isMarried, isMale;
        }
        "#;
        let mut char_stream = StringCharStream::new(input);
        let mut lexer = Lexer::new(&mut char_stream);
        let mut parser = Parser::new(&mut lexer);

        let ast = parser.parse_class();
        assert!(ast.is_ok());
    }
}
