use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum ASTNode {
    Program{commands: Vec<ASTNode>},
    Push{segment: Segment, index: u16},
    Pop{segment: Segment, index: u16},
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
    Label{name: String},
    Goto{label: String},
    IfGoto{label: String},
    Function{name: String, n_locals: u16},
    Call{name: String, n_args: u16},
    Return,
}

impl ASTNode {
    pub fn to_command_string(&self) -> String {
        match self {
            ASTNode::Push { segment, index } => format!("push {} {}", segment, index),
            ASTNode::Pop { segment, index } => format!("pop {} {}", segment, index),
            ASTNode::Add => "add".to_string(),
            ASTNode::Sub => "sub".to_string(),
            ASTNode::Neg => "neg".to_string(),
            ASTNode::Eq => "eq".to_string(),
            ASTNode::Gt => "gt".to_string(),
            ASTNode::Lt => "lt".to_string(),
            ASTNode::And => "and".to_string(),
            ASTNode::Or => "or".to_string(),
            ASTNode::Not => "not".to_string(),
            ASTNode::Program { .. } => "program".to_string(),
            ASTNode::Label { name } => format!("label {}", name),
            ASTNode::Goto { label } => format!("goto {}", label),
            ASTNode::IfGoto { label } => format!("if-goto {}", label),
            ASTNode::Function { name, n_locals} => format!("function {} {}", name, n_locals),
            ASTNode::Call { name, n_args } => format!("call {} {}", name, n_args),
            ASTNode::Return => "return".to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Segment {
    Argument,
    Local,
    Static,
    Constant,
    This,
    That,
    Pointer,
    Temp,
}

impl Segment {
    pub fn to_string(&self) -> &str {
        match self {
            Segment::Argument => "argument",
            Segment::Local => "local",
            Segment::Static => "static",
            Segment::Constant => "constant",
            Segment::This => "this",
            Segment::That => "that",
            Segment::Pointer => "pointer",
            Segment::Temp => "temp",
        }
    }
}

impl Display for Segment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}