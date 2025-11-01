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
    Number(u16),
    Name(String),
    Label,
    Goto,
    IfGoto,
    Function,
    Call,
    Return,
    Error(String),
}

impl TokenType {
    fn is_valid_name(s: &str) -> bool {
        s.chars().skip(1).all(&Self::is_valid_name_char)
    }
    fn is_valid_name_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == ':'
    }
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
            "label" => TokenType::Label,
            "goto" => TokenType::Goto,
            "if-goto" => TokenType::IfGoto,
            "function" => TokenType::Function,
            "call" => TokenType::Call,
            "return" => TokenType::Return,
            _ => match s.parse::<u16>() {
                Ok(value) => TokenType::Number(value),
                Err(_) => {
                    if Self::is_valid_name(s) {
                        TokenType::Name(s.to_string())
                    } else {
                        TokenType::Error(format!("Invalid token: {}", s))
                    }
                },
            }
        }
    }
}
