use crate::jack::ast::Ast;
use crate::jack::lexer::JackToken;

pub struct AstPrinter<'a> {
    indent: usize,
    output: Option<&'a mut dyn Output>,
}

impl <'a> AstPrinter<'a> {
    pub fn new() -> Self {
        AstPrinter {
            indent: 0,
            output: None,
        }
    }

    pub fn set_output(&mut self, output: &'a mut dyn Output) {
        self.output = Some(output);
    }

    pub fn print_ast(&mut self, ast: &Ast) {
        match ast {
            Ast::NonTerminal(data) => {
                self.print_text(&format!("<{}>", data.name));
                self.indent += 1;
                for child in &data.children {
                    self.print_ast(child);
                }
                self.indent -= 1;
                self.print_text(&format!("</{}>", data.name));
            }
            Ast::Terminal(token) => {
                let tag_name = self.get_token_tag_name(token);
                self.print_text(&format!("<{}> {} </{}>",
                    tag_name, Self::escape_xml(&Self::token_value(token)), tag_name));
            }
        }
    }

    fn token_value(token: &JackToken) -> String {
        match token.token_type.get_category() {
            crate::jack::token_type::TokenTypeCategory::StringConstant => {
                token.lexeme.trim_matches('"').to_string()
            }
            _ => token.lexeme.clone(),
        }
    }

    fn escape_xml(text: &str) -> String {
        text.replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
    }

    fn get_token_tag_name(&self, token: &crate::jack::lexer::JackToken) -> &str {
        use crate::jack::token_type::TokenTypeCategory::*;
        match token.token_type.get_category() {
            Class => "keyword",
            Constructor => "keyword",
            Function => "keyword",
            Method => "keyword",
            Field => "keyword",
            Static => "keyword",
            Var => "keyword",
            Int => "keyword",
            Char => "keyword",
            Boolean => "keyword",
            Void => "keyword",
            True => "keyword",
            False => "keyword",
            Null => "keyword",
            This => "keyword",
            Let => "keyword",
            Do => "keyword",
            If => "keyword",
            Else => "keyword",
            While => "keyword",
            Return => "keyword",
            LBrace => "symbol",
            RBrace => "symbol",
            LParen => "symbol",
            RParen => "symbol",
            LBracket => "symbol",
            RBracket => "symbol",
            Dot => "symbol",
            Comma => "symbol",
            Semicolon => "symbol",
            Plus => "symbol",
            Minus => "symbol",
            Asterisk => "symbol",
            Slash => "symbol",
            Ampersand => "symbol",
            Pipe => "symbol",
            LessThan => "symbol",
            GreaterThan => "symbol",
            Equal => "symbol",
            Tilde => "symbol",
            IntegerConstant => "integerConstant",
            StringConstant => "stringConstant",
            Identifier => "identifier",
            Error => "error",
        }
    }

    fn print_text(&mut self, text: &str) {
        match &mut self.output {
            Some(output) => {
                for _ in 0..self.indent {
                    output.print("  ");
                }
                output.println(text);
            },
            None => (),
        }
    }
}

impl <'a> Default for AstPrinter<'a> {
    fn default() -> Self {
        AstPrinter::new()
    }
}

pub trait Output {
    fn print(&mut self, text: &str);
    fn println(&mut self, text: &str) {
        self.print(&format!("{}\n", text));
    }
}

pub struct ConsoleOutput;

impl Output for ConsoleOutput {
    fn print(&mut self, text: &str) {
        print!("{}", text);
    }
}

pub struct StringOutput {
    content: String,
}

impl StringOutput {
    pub fn new() -> Self {
        StringOutput {
            content: String::new(),
        }
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }
}

impl Output for StringOutput {
    fn print(&mut self, text: &str) {
        self.content.push_str(text);
    }
}
