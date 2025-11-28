use crate::jack::lexer::JackToken;
use crate::jack::token_type::TokenTypeCategory;

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

impl ParseTreeNode {
    pub fn apply_action(&self, action: &ParseTreeAction) -> Option<&ParseTreeNode> {
        match action {
            ParseTreeAction::CheckName{name} => {
                match self {
                    ParseTreeNode::NonTerminal(data) => {
                        if &data.name == name {
                            Some(self)
                        } else {
                            None
                        }
                    },
                    ParseTreeNode::Terminal(token) => None,
                }
            },
            ParseTreeAction::CheckTokenTypeCategory{token_type_category} => {
                match self {
                    ParseTreeNode::Terminal(token) => {
                        if &token.token_type.get_category() == token_type_category {
                            Some(self)
                        } else {
                            None
                        }
                    },
                    _ => None,
                }
            },
            ParseTreeAction::NavigateToChildByName{name} => {
                match self {
                    ParseTreeNode::NonTerminal(data) => {
                        for child in &data.children {
                            match child {
                                ParseTreeNode::NonTerminal(child_data) => {
                                    if &child_data.name == name {
                                        return Some(child);
                                    }
                                },
                                ParseTreeNode::Terminal(token) => { },
                            }
                        }
                        None
                    },
                    _ => None,
                }
            },
            ParseTreeAction::NavigateToChildByIndex{index} => {
                match self {
                    ParseTreeNode::NonTerminal(data) => {
                        data.children.get(*index)
                    },
                    _ => None,
                }
            },
        }
    }

    pub fn apply_actions(&self, actions: &[ParseTreeAction]) -> Option<&ParseTreeNode> {
        let mut current_node = self;
        for action in actions {
            match current_node.apply_action(action) {
                Some(node) => current_node = node,
                None => return None,
            }
        }
        Some(current_node)
    }
}

pub enum ParseTreeAction {
    CheckName{name: String},
    CheckTokenTypeCategory{token_type_category: TokenTypeCategory},
    NavigateToChildByName{name: String},
    NavigateToChildByIndex{index: usize},
}


