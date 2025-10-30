#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    Push,
    Pop,
    Argument,
    Local,
    Static,
    Constant,
    This,
    That,
    Pointer,
    Temp,
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
    Index(u16),
    Error(String),
}

impl From<&str> for TokenType {
    fn from(s: &str) -> Self {
        match s {
            "push" => TokenType::Push,
            "pop" => TokenType::Pop,
            "argument" => TokenType::Argument,
            "local" => TokenType::Local,
            "static" => TokenType::Static,
            "constant" => TokenType::Constant,
            "this" => TokenType::This,
            "that" => TokenType::That,
            "pointer" => TokenType::Pointer,
            "temp" => TokenType::Temp,
            "add" => TokenType::Add,
            "sub" => TokenType::Sub,
            "neg" => TokenType::Neg,
            "eq" => TokenType::Eq,
            "gt" => TokenType::Gt,
            "lt" => TokenType::Lt,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "not" => TokenType::Not,
            _ => match s.parse::<u16>() {
                Ok(index) => TokenType::Index(index),
                Err(_) => TokenType::Error(format!("Unknown token: '{s}'")),
            }
        }
    }
}
