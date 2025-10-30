use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token<T: Clone> {
    pub token_type: T,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl <T: Clone> Token<T> {
    pub fn new(token_type: T, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            column,
        }
    }
}