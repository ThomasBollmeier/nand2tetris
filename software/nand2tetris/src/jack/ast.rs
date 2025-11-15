// Abstract Syntax Tree (AST) definitions for the Jack programming language.

#[derive(Debug, Clone)]
pub struct Class {
    pub name: String,
    pub class_var_declarations: Vec<ClassVarDec>,
    pub subroutine_declarations: Vec<SubroutineDec>,
}

#[derive(Debug, Clone)]
pub struct ClassVarDec {
    pub category: ClassVarCategory,
    pub var_type: Type,
    pub names: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Char,
    Boolean,
    Class(String),
}


#[derive(Debug, Clone)]
pub enum ClassVarCategory {
    Static,
    Field,
}

#[derive(Debug, Clone)]
pub struct SubroutineDec {
    pub category: SubroutineCategory,
    pub return_type: Option<Type>,
    pub name: String,
    pub parameters: Vec<(Type, String)>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum SubroutineCategory {
    Constructor,
    Function,
    Method,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        var_name: String,
        index_expression: Option<Expression>,
        value_expression: Expression,
    },
    If {
        condition: Expression,
        if_statements: Vec<Statement>,
        else_statements: Option<Vec<Statement>>,
    },
    While {
        condition: Expression,
        body_statements: Vec<Statement>,
    },
    Do {
        subroutine_call: SubroutineCall,
    },
    Return {
        value: Option<Expression>,
    },
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub term: Box<Term>,
    pub rest: Vec<(Operator, Box<Term>)>,
}

#[derive(Debug, Clone)]
pub enum Term {
    IntegerConstant(u16),
    StringConstant(String),
    KeywordConstant(KeywordConstant),
    VarName(String),
    VarNameWithIndex {
        var_name: String,
        index_expression: Box<Expression>,
    },
    ExpressionInParens(Box<Expression>),
    UnaryOp {
        operator: UnaryOperator,
        term: Box<Term>,
    },
    SubroutineCall(SubroutineCall),
}

#[derive(Debug, Clone)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    And,
    Or,
    LessThan,
    GreaterThan,
    Equal,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Negate,
    Not,
}

#[derive(Debug,  Clone)]
pub enum KeywordConstant {
    True,
    False,
    Null,
    This,
}

#[derive(Debug, Clone)]
pub struct SubroutineCall {
    pub class_or_instance_name: Option<String>,
    pub subroutine_name: String,
    pub arguments: Vec<Expression>,
}