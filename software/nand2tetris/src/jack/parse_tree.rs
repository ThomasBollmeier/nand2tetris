use crate::jack::lexer::JackToken;

pub struct ParseTreeNodeData {
    pub name: String,
    pub value: Option<String>,
    pub children: Vec<ParseTreeNode>,
}

impl ParseTreeNodeData {
    pub fn new(name: &str, value: Option<String>) -> Self {
        ParseTreeNodeData {
            name: name.to_string(),
            value,
            children: Vec::new(),
        }
    }

    pub fn add_token(&mut self, token: JackToken) {
        self.children.push(ParseTreeNode::Terminal(token));
    }

    pub fn add_child(&mut self, child: ParseTreeNodeData) {
        self.children.push(ParseTreeNode::NonTerminal(child));
    }
}

pub enum ParseTreeNode {
    NonTerminal(ParseTreeNodeData),
    Terminal(JackToken),
}


