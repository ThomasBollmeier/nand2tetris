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