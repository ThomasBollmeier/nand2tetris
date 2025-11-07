use crate::jack::lexer::JackToken;

pub struct AstData {
    pub name: String,
    pub value: Option<String>,
    pub children: Vec<Ast>,
}

impl AstData {
    pub fn new(name: &str, value: Option<String>) -> Self {
        AstData {
            name: name.to_string(),
            value,
            children: Vec::new(),
        }
    }

    pub fn add_token(&mut self, token: JackToken) {
        self.children.push(Ast::Terminal(token));
    }

    pub fn add_child(&mut self, child: AstData) {
        self.children.push(Ast::NonTerminal(child));
    }
}

pub enum Ast {
    NonTerminal(AstData),
    Terminal(JackToken),
}


