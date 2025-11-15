use crate::grammarous::stream::{BufferedStream, Stream};
use crate::jack::parse_tree::{ParseTreeNode, ParseTreeNodeData};
use crate::jack::lexer::JackToken;
use crate::jack::token_type::TokenTypeCategory::{self, *};

pub struct Parser<'a> {
    stream: BufferedStream<'a, JackToken>,
}

impl<'a> Parser<'a> {
    pub fn new(stream: &'a mut dyn Stream<JackToken>) -> Self {
        Self {
            stream: BufferedStream::new(stream),
        }
    }

    pub fn parse_class(&mut self) -> Result<ParseTreeNode, String> {
        let mut class_data = ParseTreeNodeData::new("class", None);

        let mut token = self.consume(Class)?;
        class_data.add_token(token);

        token = self.consume(Identifier)?;
        class_data.add_token(token);

        token = self.consume(LBrace)?;
        class_data.add_token(token);

        self.parse_class_var_declarations(&mut class_data)?;
        self.parse_subroutine_declarations(&mut class_data)?;

        token = self.consume(RBrace)?;
        class_data.add_token(token);

        Ok(ParseTreeNode::NonTerminal(class_data))
    }

    fn parse_subroutine_declarations(&mut self, class_data: &mut ParseTreeNodeData) -> Result<(), String> {
        loop {
            let next_token = self.peek();
            if next_token.is_none() {
                return Err(
                    "Unexpected end of input while parsing subroutine declarations".to_string(),
                );
            }
            let next_token = next_token.unwrap();
            match next_token.token_type.get_category() {
                Constructor | Function | Method => {
                    let mut subroutine_dec = ParseTreeNodeData::new("subroutineDec", None);

                    let token = self.consume_any_of(&[Constructor, Function, Method])?;
                    subroutine_dec.add_token(token);

                    let token = self.consume_type(true)?;
                    subroutine_dec.add_token(token);

                    let token = self.consume(Identifier)?;
                    subroutine_dec.add_token(token);

                    let token = self.consume(LParen)?;
                    subroutine_dec.add_token(token);

                    self.parse_parameter_list(&mut subroutine_dec)?;

                    let token = self.consume(RParen)?;
                    subroutine_dec.add_token(token);

                    self.parse_subroutine_body(&mut subroutine_dec)?;

                    class_data.add_child(subroutine_dec);
                }
                _ => break,
            }
        }

        Ok(())
    }

    fn parse_subroutine_body(&mut self, subroutine_dec: &mut ParseTreeNodeData) -> Result<(), String> {
        let mut subroutine_body = ParseTreeNodeData::new("subroutineBody", None);

        let token = self.consume(LBrace)?;
        subroutine_body.add_token(token);

        self.parse_var_declarations(&mut subroutine_body)?;

        subroutine_body.add_child(self.parse_statements()?);

        let token = self.consume(RBrace)?;
        subroutine_body.add_token(token);

        subroutine_dec.add_child(subroutine_body);

        Ok(())
    }

    fn parse_statements(&mut self) -> Result<ParseTreeNodeData, String> {
        let mut statements = ParseTreeNodeData::new("statements", None);
        loop {
            let next_token = self.peek();
            if next_token.is_none() {
                return Err("Unexpected end of input while parsing statements".to_string());
            }
            let next_token = next_token.unwrap();
            let statement = match next_token.token_type.get_category() {
                Let => self.parse_let_statement(),
                If => self.parse_if_statement(),
                While => self.parse_while_statement(),
                Do => self.parse_do_statement(),
                Return => self.parse_return_statement(),
                _ => break,
            };
            statements.add_child(statement?);
        }

        Ok(statements)
    }

    fn parse_return_statement(&mut self) -> Result<ParseTreeNodeData, String> {
        let mut return_statement = ParseTreeNodeData::new("returnStatement", None);

        let token = self.consume(Return)?;
        return_statement.add_token(token);

        let next_token = self.peek();
        if let Some(token) = next_token {
            if token.token_type.get_category() != Semicolon {
                let expr = self.parse_expression()?;
                return_statement.add_child(expr);
            }
        } else {
            return Err("Unexpected end of input while parsing return statement".to_string());
        }

        let token = self.consume(Semicolon)?;
        return_statement.add_token(token);

        Ok(return_statement)
    }

    fn parse_do_statement(&mut self) -> Result<ParseTreeNodeData, String> {
        let mut do_statement = ParseTreeNodeData::new("doStatement", None);

        let token = self.consume(Do)?;
        do_statement.add_token(token);

        self.parse_subroutine_call(&mut do_statement)?;

        let token = self.consume(Semicolon)?;
        do_statement.add_token(token);

        Ok(do_statement)
    }

    fn parse_while_statement(&mut self) -> Result<ParseTreeNodeData, String> {
        let mut while_statement = ParseTreeNodeData::new("whileStatement", None);

        let token = self.consume(While)?;
        while_statement.add_token(token);

        let token = self.consume(LParen)?;
        while_statement.add_token(token);

        let expr = self.parse_expression()?;
        while_statement.add_child(expr);

        let token = self.consume(RParen)?;
        while_statement.add_token(token);

        let token = self.consume(LBrace)?;
        while_statement.add_token(token);

        let statements = self.parse_statements()?;
        while_statement.add_child(statements);

        let token = self.consume(RBrace)?;
        while_statement.add_token(token);

        Ok(while_statement)
    }

    fn parse_if_statement(&mut self) -> Result<ParseTreeNodeData, String> {
        let mut if_statement = ParseTreeNodeData::new("ifStatement", None);

        let token = self.consume(If)?;
        if_statement.add_token(token);

        let token = self.consume(LParen)?;
        if_statement.add_token(token);

        let expr = self.parse_expression()?;
        if_statement.add_child(expr);

        let token = self.consume(RParen)?;
        if_statement.add_token(token);

        let token = self.consume(LBrace)?;
        if_statement.add_token(token);

        let statements = self.parse_statements()?;
        if_statement.add_child(statements);

        let token = self.consume(RBrace)?;
        if_statement.add_token(token);

        let next_token = self.peek();
        if let Some(token) = next_token {
            if token.token_type.get_category() == Else {
                let token = self.consume(Else)?;
                if_statement.add_token(token);

                let token = self.consume(LBrace)?;
                if_statement.add_token(token);

                let statements = self.parse_statements()?;
                if_statement.add_child(statements);

                let token = self.consume(RBrace)?;
                if_statement.add_token(token);
            }
        }

        Ok(if_statement)
    }

    fn parse_let_statement(&mut self) -> Result<ParseTreeNodeData, String> {
        let mut let_statement = ParseTreeNodeData::new("letStatement", None);

        let token = self.consume(Let)?;
        let_statement.add_token(token);

        let token = self.consume(Identifier)?;
        let_statement.add_token(token);

        let next_token = self.peek();
        if let Some(token) = next_token {
            if token.token_type.get_category() == LBracket {
                let token = self.consume(LBracket)?;
                let_statement.add_token(token);
                let expr = self.parse_expression()?;
                let_statement.add_child(expr);
                let token = self.consume(RBracket)?;
                let_statement.add_token(token);
            }
        } else {
            return Err("Unexpected end of input while parsing let statement".to_string());
        }

        let token = self.consume(Equal)?;
        let_statement.add_token(token);

        let expr = self.parse_expression()?;
        let_statement.add_child(expr);

        let token = self.consume(Semicolon)?;
        let_statement.add_token(token);

        Ok(let_statement)
    }

    fn parse_expression(&mut self) -> Result<ParseTreeNodeData, String> {
        let mut expression = ParseTreeNodeData::new("expression", None);

        self.parse_term(&mut expression)?;

        loop {
            let next_token = self.peek();
            if let Some(token) = next_token {
                match token.token_type.get_category() {
                    Plus
                    | Minus
                    | Asterisk
                    | Slash
                    | Ampersand
                    | Pipe
                    | LessThan
                    | GreaterThan
                    | Equal => {
                        let token = self.consume_any()?;
                        expression.add_token(token);

                        self.parse_term(&mut expression)?;
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }

        Ok(expression)
    }

    fn parse_term(&mut self, expression: &mut ParseTreeNodeData) -> Result<(), String> {
        let mut term = ParseTreeNodeData::new("term", None);
        let next_token = self.peek();
        if next_token.is_none() {
            return Err("Unexpected end of input while parsing term".to_string());
        }
        let next_token = next_token.unwrap();
        match next_token.token_type.get_category() {
            IntegerConstant | StringConstant | True | False | Null | This => {
                let token = self.consume_any()?;
                term.add_token(token);
            }
            LParen => {
                let token = self.consume(LParen)?;
                term.add_token(token);

                let expr = self.parse_expression()?;
                term.add_child(expr);

                let token = self.consume(RParen)?;
                term.add_token(token);
            }
            Minus | Tilde => {
                let token = self.consume_any_of(&[Minus, Tilde])?;
                term.add_token(token);

                self.parse_term(&mut term)?;
            }
            Identifier => {
                let lookahead = self.peek_nth(1);
                match lookahead {
                    Some(token)
                        if token.token_type.get_category() == LParen
                            || token.token_type.get_category() == Dot =>
                    {
                        self.parse_subroutine_call(&mut term)?;
                    }
                    Some(token) if token.token_type.get_category() == LBracket => {
                        self.parse_array_access(&mut term)?;
                    }
                    _ => {
                        let token = self.consume(Identifier)?;
                        term.add_token(token);
                    }
                }
            }
            _ => return Err("Got unknown token while parsing term".to_string()),
        }

        expression.add_child(term);

        Ok(())
    }

    fn parse_array_access(&mut self, data: &mut ParseTreeNodeData) -> Result<(), String> {
        let token = self.consume(Identifier)?;
        data.add_token(token);

        let token = self.consume(LBracket)?;
        data.add_token(token);

        data.add_child(self.parse_expression()?);

        let token = self.consume(RBracket)?;
        data.add_token(token);

        Ok(())
    }

    fn parse_subroutine_call(&mut self, data: &mut ParseTreeNodeData) -> Result<(), String> {
        let token = self.consume(Identifier)?;
        data.add_token(token);

        let next_token = self.peek();
        if let Some(token) = next_token {
            match token.token_type.get_category() {
                Dot => {
                    let token = self.consume(Dot)?;
                    data.add_token(token);

                    let token = self.consume(Identifier)?;
                    data.add_token(token);
                }
                _ => {}
            }
        } else {
            return Err("Unexpected end of input while parsing subroutine call".to_string());
        }

        let token = self.consume(LParen)?;
        data.add_token(token);

        data.add_child(self.parse_expression_list()?);

        let token = self.consume(RParen)?;
        data.add_token(token);

        Ok(())
    }

    fn parse_expression_list(&mut self) -> Result<ParseTreeNodeData, String> {
        let mut expression_list = ParseTreeNodeData::new("expressionList", None);

        loop {
            let next_token = self.peek();
            if let Some(token) = next_token {
                if token.token_type.get_category() == RParen {
                    break;
                }
            } else {
                return Err("Unexpected end of input while parsing expression list".to_string());
            }

            let expr = self.parse_expression()?;
            expression_list.add_child(expr);

            let next_token = self.peek();
            if let Some(token) = next_token {
                if token.token_type.get_category() == Comma {
                    expression_list.add_token(self.consume(Comma)?);
                } else {
                    break;
                }
            } else {
                return Err("Unexpected end of input while parsing expression list".to_string());
            }
        }

        Ok(expression_list)
    }

    fn parse_var_declarations(&mut self, subroutine_dec: &mut ParseTreeNodeData) -> Result<(), String> {
        loop {
            let next_token = self.peek();
            if next_token.is_none() {
                return Err(
                    "Unexpected end of input while parsing variable declarations".to_string(),
                );
            }
            let next_token = next_token.unwrap();
            match next_token.token_type.get_category() {
                Var => {
                    let mut var_dec = ParseTreeNodeData::new("varDec", None);

                    let token = self.consume(Var)?;
                    var_dec.add_token(token);

                    let token = self.consume_type(false)?;
                    var_dec.add_token(token);

                    self.add_comma_separated_identifiers(&mut var_dec)?;

                    let token = self.consume(Semicolon)?;
                    var_dec.add_token(token);

                    subroutine_dec.add_child(var_dec);
                }
                _ => break,
            }
        }

        Ok(())
    }

    fn parse_parameter_list(&mut self, subroutine_dec: &mut ParseTreeNodeData) -> Result<(), String> {
        let mut param_list = ParseTreeNodeData::new("parameterList", None);

        let next_token = self.peek();
        if let Some(token) = next_token {
            if token.token_type.get_category() == RParen {
                subroutine_dec.add_child(param_list);
                return Ok(());
            }
        } else {
            return Err("Unexpected end of input while parsing parameter list".to_string());
        }

        loop {
            let token = self.consume_type(false)?;
            param_list.add_token(token);

            let token = self.consume(Identifier)?;
            param_list.add_token(token);

            let next_token = self.peek();
            if let Some(token) = next_token {
                if token.token_type.get_category() == Comma {
                    let token = self.consume(Comma)?;
                    param_list.add_token(token);
                } else {
                    break;
                }
            } else {
                return Err("Unexpected end of input in parameter list".to_string());
            }
        }

        subroutine_dec.add_child(param_list);

        Ok(())
    }

    fn parse_class_var_declarations(&mut self, class_data: &mut ParseTreeNodeData) -> Result<(), String> {
        loop {
            let next_token = self.peek();
            if next_token.is_none() {
                return Err(
                    "Unexpected end of input while parsing variable declarations".to_string(),
                );
            }
            let next_token = next_token.unwrap();
            match next_token.token_type.get_category() {
                Static | Field => {
                    let mut class_var_dec = ParseTreeNodeData::new("classVarDec", None);

                    let token = self.consume_any_of(&[Static, Field])?;
                    class_var_dec.add_token(token);

                    let token = self.consume_type(false)?;
                    class_var_dec.add_token(token);

                    self.add_comma_separated_identifiers(&mut class_var_dec)?;

                    let token = self.consume(Semicolon)?;
                    class_var_dec.add_token(token);

                    class_data.add_child(class_var_dec);
                }
                _ => break,
            }
        }

        Ok(())
    }

    fn add_comma_separated_identifiers(&mut self, data: &mut ParseTreeNodeData) -> Result<(), String> {
        let token = self.consume(Identifier)?;
        data.add_token(token);

        loop {
            let next_token = self.peek();
            if let Some(token) = next_token {
                if token.token_type.get_category() == Comma {
                    let token = self.consume(Comma)?;
                    data.add_token(token);

                    let token = self.consume(Identifier)?;
                    data.add_token(token);
                } else {
                    break;
                }
            } else {
                return Err(
                    "Unexpected end of input while parsing comma-separated identifiers".to_string(),
                );
            }
        }
        Ok(())
    }

    fn peek(&mut self) -> Option<JackToken> {
        self.stream.peek()
    }

    fn peek_nth(&mut self, n: usize) -> Option<JackToken> {
        self.stream.peek_nth(n)
    }

    fn consume_type(&mut self, include_void: bool) -> Result<JackToken, String> {
        if include_void {
            return self.consume_any_of(&[Void, Int, Char, Boolean, Identifier]);
        }
        self.consume_any_of(&[Int, Char, Boolean, Identifier])
    }

    fn consume(&mut self, expected: TokenTypeCategory) -> Result<JackToken, String> {
        match self.stream.advance() {
            Some(token) if token.token_type.get_category() == expected => Ok(token),
            Some(token) => Err(format!(
                "[{}, {}]: Expected {:?}, found {:?}",
                token.line, token.column, expected, token.token_type
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

    fn consume_any(&mut self) -> Result<JackToken, String> {
        self.stream
            .advance()
            .ok_or("Unexpected end of input".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammarous::string_char_stream::StringCharStream;
    use crate::jack::lexer::Lexer;
    use crate::jack::parse_tree_printer::{ParseTreePrinter, ConsoleOutput};

    #[test]
    fn test_parse_class() {
        run_code(r#"
        class Person {
            field boolean isMarried, isMale;

            method void setMarried(boolean isMarried) {
                var int answer;
                let answer = 41 + 1;

            }

            method void sayHello() {
                do Output.printString("Hallo Welt!");
                return;
            }
        }
        "#);
    }

    fn run_code(code: &str) {
        let mut char_stream = StringCharStream::new(code);
        let mut lexer = Lexer::new(&mut char_stream);
        let mut parser = Parser::new(&mut lexer);

        let ast = parser.parse_class();
        assert!(
            ast.is_ok(),
            "Failed to parse class: {:?}",
            ast.err().unwrap()
        );

        let mut ast_printer = ParseTreePrinter::default();
        let mut output = ConsoleOutput {};
        ast_printer.set_output(&mut output);

        ast_printer.print_ast(&ast.unwrap());
    }
}
