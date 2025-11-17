use crate::jack::ast::*;
use crate::jack::lexer::JackToken;
use crate::jack::parse_tree::{ParseTreeNode, ParseTreeNodeData};
use crate::jack::token_type::TokenTypeCategory;

pub fn convert_class(class_node: &ParseTreeNode) -> Result<Class, String> {
    let class_node = match class_node {
        ParseTreeNode::NonTerminal(node) if node.name == "class" => node,
        _ => return Err("Expected class parse tree node".to_string()),
    };
    let name;
    let mut class_var_declarations = vec![];
    let mut subroutine_declarations = vec![];

    if let Some(ParseTreeNode::Terminal(token)) = class_node.children.get(1) {
        name = token.lexeme.clone();
    } else {
        return Err("Invalid className node structure".to_string());
    }

    for child in &class_node.children {
        match child {
            ParseTreeNode::NonTerminal(node) if node.name == "classVarDec" => {
                let class_var_dec = convert_class_var_declaration(node)?;
                class_var_declarations.push(class_var_dec);
            }
            ParseTreeNode::NonTerminal(node) if node.name == "subroutineDec" => {
                let subroutine_dec = convert_subroutine_declaration(node)?;
                subroutine_declarations.push(subroutine_dec);
            }
            _ => {}
        }
    }

    Ok(Class {
        name,
        class_var_declarations,
        subroutine_declarations,
    })
}

fn convert_class_var_declaration(
    class_var_dec_node: &ParseTreeNodeData,
) -> Result<ClassVarDec, String> {
    let children = &class_var_dec_node.children;

    let catg_node = children.get(0).ok_or("Missing category node")?;
    let category = match catg_node {
        ParseTreeNode::Terminal(token) if token.lexeme == "static" => ClassVarCategory::Static,
        ParseTreeNode::Terminal(token) if token.lexeme == "field" => ClassVarCategory::Field,
        _ => return Err("Invalid category node".to_string()),
    };

    let type_node = children.get(1).ok_or("Missing type node")?;
    let var_type = convert_type_node(type_node)?;
    let names = get_identifiers(&children[2..]);

    Ok(ClassVarDec {
        category,
        var_type,
        names,
    })
}

fn convert_type_node(type_node: &ParseTreeNode) -> Result<Type, String> {
    match type_node {
        ParseTreeNode::Terminal(token) => match token.lexeme.as_str() {
            "int" => Ok(Type::Int),
            "char" => Ok(Type::Char),
            "boolean" => Ok(Type::Boolean),
            class_name => Ok(Type::Class(class_name.to_string())),
        },
        _ => Err("Invalid type node".to_string()),
    }
}

fn convert_subroutine_declaration(
    subroutine_dec_node: &ParseTreeNodeData,
) -> Result<SubroutineDec, String> {
    let children = &subroutine_dec_node.children;

    let category = match children.get(0) {
        Some(ParseTreeNode::Terminal(token)) if token.lexeme == "function" => {
            SubroutineCategory::Function
        }
        Some(ParseTreeNode::Terminal(token)) if token.lexeme == "method" => {
            SubroutineCategory::Method
        }
        Some(ParseTreeNode::Terminal(token)) if token.lexeme == "constructor" => {
            SubroutineCategory::Constructor
        }
        _ => return Err("Invalid subroutine category node".to_string()),
    };

    let return_type = match children.get(1) {
        Some(ParseTreeNode::Terminal(token)) if token.lexeme == "void" => None,
        Some(type_node) => Some(convert_type_node(type_node)?),
        _ => return Err("Invalid return type node".to_string()),
    };

    let name = if let Some(ParseTreeNode::Terminal(token)) = children.get(2) {
        token.lexeme.clone()
    } else {
        return Err("Invalid subroutine name node".to_string());
    };

    let mut parameters = vec![];
    let mut body = None;

    for child in children {
        if let ParseTreeNode::NonTerminal(node) = child {
            match node.name.as_str() {
                "parameterList" => {
                    parameters = convert_parameter_list(node)?;
                }
                "subroutineBody" => {
                    body = Some(convert_subroutine_body(node)?);
                }
                _ => {}
            }
        }
    }

    let body = body.ok_or("Missing subroutine body")?;

    Ok(SubroutineDec {
        category,
        return_type,
        name,
        parameters,
        body,
    })
}

fn convert_subroutine_body(subroutine_body_node: &ParseTreeNodeData) -> Result<Body, String> {
    let children = &subroutine_body_node.children;
    let mut var_declarations = vec![];
    let mut statements = vec![];

    for child in children {
        if let ParseTreeNode::NonTerminal(node) = child {
            match node.name.as_str() {
                "varDec" => {
                    var_declarations.push(convert_var_declaration(node)?);
                }
                "statements" => {
                    statements = convert_statements(node)?;
                }
                _ => {}
            }
        }
    }

    Ok(Body {
        var_declarations,
        statements,
    })
}

fn convert_statements(statements_node: &ParseTreeNodeData) -> Result<Vec<Statement>, String> {
    let children = &statements_node.children;
    let mut statements = vec![];

    for child in children {
        if let ParseTreeNode::NonTerminal(node) = child {
            match node.name.as_str() {
                "letStatement" => {
                    statements.push(convert_let_statement(node)?);
                }
                "ifStatement" => {
                    statements.push(convert_if_statement(node)?);
                }
                "whileStatement" => {
                    // Implement conversion for whileStatement
                }
                "doStatement" => {
                    // Implement conversion for doStatement
                }
                "returnStatement" => {
                    statements.push(convert_return_statement(node)?);
                }
                _ => {}
            }
        }
    }

    Ok(statements)
}

fn convert_return_statement(return_stmt_node: &ParseTreeNodeData) -> Result<Statement, String> {
    let children = &return_stmt_node.children;

    let return_expression = if children.len() > 2 {
        let expr_node = get_non_terminal_at(children, 1, "expression")?;
        Some(convert_expression(expr_node)?)
    } else {
        None
    };

    Ok(Statement::Return {
        value: return_expression,
    })
}

fn convert_if_statement(if_stmt_node: &ParseTreeNodeData) -> Result<Statement, String> {
    let children = &if_stmt_node.children;

    let condition_node = get_non_terminal_at(children, 2, "expression")?;
    let condition = convert_expression(condition_node)?;

    let statements_node = get_non_terminal_at(children, 5, "statements")?;
    let if_statements = convert_statements(statements_node)?;
    let mut else_statements = None;

    if children.len() > 6 {
        let else_statements_node = get_non_terminal_at(children, 9, "statements")?;
        else_statements = Some(convert_statements(else_statements_node)?);
    }

    Ok(Statement::If {
        condition,
        if_statements,
        else_statements,
    })
}

fn get_non_terminal_at<'a>(
    nodes: &'a [ParseTreeNode],
    idx: usize,
    name: &str,
) -> Result<&'a ParseTreeNodeData, String> {
    let node = nodes
        .get(idx)
        .ok_or("Child index out of bounds".to_string())?;
    if let ParseTreeNode::NonTerminal(n) = node {
        if n.name == name {
            return Ok(n);
        }
    }
    Err("No non-terminal found".to_string())
}

fn get_terminal_at(
    nodes: &[ParseTreeNode],
    idx: usize,
    token_type_category: TokenTypeCategory,
) -> Result<&JackToken, String> {
    let node = nodes
        .get(idx)
        .ok_or("Child index out of bounds".to_string())?;
    if let ParseTreeNode::Terminal(token) = node {
        if token.token_type.get_category() == token_type_category {
            return Ok(token);
        }
    }
    Err("No terminal found".to_string())
}

fn convert_let_statement(let_stmt_node: &ParseTreeNodeData) -> Result<Statement, String> {
    let children = &let_stmt_node.children;

    let var_name = if let Some(ParseTreeNode::Terminal(token)) = children.get(1) {
        token.lexeme.clone()
    } else {
        return Err("Invalid variable name node in let statement".to_string());
    };

    let lbracket_node = children
        .get(2)
        .ok_or("Missing node after variable name in let statement")?;

    let index_expression = match lbracket_node {
        ParseTreeNode::Terminal(token)
            if token.token_type.get_category() == TokenTypeCategory::LBracket =>
        {
            let expr_node = children.get(3).ok_or("Missing expression")?;
            let expr = match expr_node {
                ParseTreeNode::NonTerminal(node) if node.name == "expression" => {
                    convert_expression(node)?
                }
                _ => return Err("Invalid index expression node in let statement".to_string()),
            };
            Some(expr)
        }
        _ => None,
    };

    let value_expression = if let Some(expr_node) = children.get(children.len() - 2) {
        match expr_node {
            ParseTreeNode::NonTerminal(node) if node.name == "expression" => {
                convert_expression(node)?
            }
            _ => return Err("Invalid value expression node in let statement".to_string()),
        }
    } else {
        return Err("Invalid value expression node in let statement".to_string());
    };

    Ok(Statement::Let {
        var_name,
        index_expression,
        value_expression,
    })
}

fn convert_expression(expression_node: &ParseTreeNodeData) -> Result<Expression, String> {
    let children = &expression_node.children;
    let term_node = get_non_terminal_at(children, 0, "term")?;
    let term = convert_term(term_node)?;
    let mut rest = vec![];

    if children.len() > 1 {
        let n = children.len() / 2;
        for i in 0..n {
            let op_node = children
                .get(2 * i + 1)
                .ok_or("Missing operator node in expression")?;
            let operator = match op_node {
                ParseTreeNode::Terminal(token) => match token.lexeme.as_str() {
                    "+" => Operator::Plus,
                    "-" => Operator::Minus,
                    "*" => Operator::Multiply,
                    "/" => Operator::Divide,
                    "&" => Operator::And,
                    "|" => Operator::Or,
                    "<" => Operator::LessThan,
                    ">" => Operator::GreaterThan,
                    "=" => Operator::Equal,
                    _ => return Err("Invalid operator in expression".to_string()),
                },
                _ => return Err("Invalid operator node in expression".to_string()),
            };

            let next_term_node = get_non_terminal_at(children, 2 * i + 2, "term")?;
            let next_term = convert_term(next_term_node)?;

            rest.push((operator, Box::new(next_term)));
        }
    }

    Ok(Expression {
        term: Box::new(term),
        rest,
    })
}

fn convert_term(term_node: &ParseTreeNodeData) -> Result<Term, String> {
    let children = &term_node.children;
    let first_child = children.get(0).ok_or("Empty term node")?;

    match first_child {
        ParseTreeNode::Terminal(token) => match token.token_type.get_category() {
            TokenTypeCategory::IntegerConstant => {
                let value = token
                    .lexeme
                    .parse::<u16>()
                    .map_err(|_| "Invalid integer constant".to_string())?;
                Ok(Term::IntegerConstant(value))
            }
            TokenTypeCategory::StringConstant => {
                let value = token.lexeme.trim_matches('"').to_string();
                Ok(Term::StringConstant(value))
            }
            TokenTypeCategory::True => Ok(Term::KeywordConstant(KeywordConstant::True)),
            TokenTypeCategory::False => Ok(Term::KeywordConstant(KeywordConstant::False)),
            TokenTypeCategory::Null => Ok(Term::KeywordConstant(KeywordConstant::Null)),
            TokenTypeCategory::This => Ok(Term::KeywordConstant(KeywordConstant::This)),
            TokenTypeCategory::Identifier => convert_expression_w_identifier(children),
            TokenTypeCategory::LParen => convert_grouped_expression(children),
            TokenTypeCategory::Minus | TokenTypeCategory::Tilde => convert_unary_operation(term_node),
            _ => Err("Invalid term node".to_string()),
        },
        _ => Err("Invalid term node".to_string()),
    }
}

fn convert_unary_operation(term_node: &ParseTreeNodeData) -> Result<Term, String> {
    let children = &term_node.children;

    let operator_node = children.get(0).ok_or("Missing operator in unary operation")?;
    let operator = match operator_node {
        ParseTreeNode::Terminal(token) => match token.lexeme.as_str() {
            "-" => UnaryOperator::Negate,
            "~" => UnaryOperator::Not,
            _ => return Err("Invalid unary operator".to_string()),
        },
        _ => return Err("Invalid operator node in unary operation".to_string()),
    };

    let term_node = get_non_terminal_at(children, 1, "term")?;
    let term = convert_term(term_node)?;

    Ok(Term::UnaryOp {
        operator,
        term: Box::new(term),
    })
}

fn convert_grouped_expression(nodes: &Vec<ParseTreeNode>) -> Result<Term, String> {
    if nodes.len() != 3 {
        return Err("Invalid grouped expression structure".to_string());
    }

    let expr_node = match &nodes[1] {
        ParseTreeNode::NonTerminal(node) if node.name == "expression" => node,
        _ => return Err("Invalid expression node in grouped expression".to_string()),
    };

    let expression = convert_expression(expr_node)?;
    Ok(Term::ExpressionInParens(Box::new(expression)))
}

fn convert_expression_w_identifier(nodes: &Vec<ParseTreeNode>) -> Result<Term, String> {
    let first_ident = if let ParseTreeNode::Terminal(token) = &nodes[0] {
        Term::VarName(token.lexeme.clone())
    } else {
        return Err("Invalid identifier term".to_string());
    };

    if nodes.len() == 1 {
        return Ok(first_ident);
    }

    let second_node = &nodes[1];
    match second_node {
        ParseTreeNode::Terminal(token) => match token.token_type.get_category() {
           TokenTypeCategory::Dot | TokenTypeCategory::LParen => {
                let subroutine_call = convert_subroutine_call(nodes)?;
                Ok(Term::SubroutineCall(subroutine_call))
            }
            _ => Err("Invalid term structure after identifier".to_string()),
        },
        _ => Err("Invalid term structure after identifier".to_string()),
    }
}

fn convert_subroutine_call(
    nodes: &Vec<ParseTreeNode>,
) -> Result<SubroutineCall, String> {
    let class_or_instance_name = None;
    let subroutine_name;
    let mut arguments = vec![];

    let first = get_terminal_at(nodes, 0, TokenTypeCategory::Identifier)?;

    let second = match &nodes[1] {
        ParseTreeNode::Terminal(token) => token,
        _ => return Err("Invalid subroutine call structure".to_string()),
    };
    match second.token_type.get_category() {
        TokenTypeCategory::Dot => {
            return Err("Not implemented yet".to_string());
        }
        TokenTypeCategory::LParen => {
            subroutine_name = first.lexeme.clone();
        }
        _ => return Err("Invalid subroutine call structure".to_string()),
    }

    Ok(SubroutineCall {
        class_or_instance_name,
        subroutine_name,
        arguments,
    })
}

fn convert_var_declaration(var_dec_node: &ParseTreeNodeData) -> Result<VarDec, String> {
    let children = &var_dec_node.children;

    let type_node = children.get(1).ok_or("Missing type node")?;
    let var_type = convert_type_node(type_node)?;
    let names = get_identifiers(&children[2..]);

    Ok(VarDec { var_type, names })
}

fn get_identifiers(nodes: &[ParseTreeNode]) -> Vec<String> {
    let mut identifiers = vec![];
    for node in nodes {
        if let ParseTreeNode::Terminal(token) = node {
            if token.token_type.get_category() == TokenTypeCategory::Identifier {
                identifiers.push(token.lexeme.clone());
            }
        }
    }

    identifiers
}

fn convert_parameter_list(
    param_list_node: &ParseTreeNodeData,
) -> Result<Vec<(Type, String)>, String> {
    let mut parameters = vec![];
    let children = &param_list_node.children;
    let mut i = 0;
    while i < children.len() {
        if let Some(type_node) = children.get(i) {
            let param_type = convert_type_node(type_node)?;
            if let Some(ParseTreeNode::Terminal(token)) = children.get(i + 1) {
                let param_name = token.lexeme.clone();
                parameters.push((param_type, param_name));
                i += 3; // Move to the next type
            } else {
                return Err("Invalid parameter name node".to_string());
            }
        } else {
            break;
        }
    }

    Ok(parameters)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammarous::StringCharStream;
    use crate::jack::parse_tree_printer;
    use crate::jack::{lexer, parser};

    #[test]
    fn test_convert_parse_tree_to_ast() {
        let code = r#"
        class Person {
            field boolean isMarried, isMale;

            constructor Person new(boolean isMarried_, boolean isMale_) {
                let isMarried = isMarried_;
                let isMale = isMale_;
                return this;
            }

            method void setMarried(boolean isMarried_) {
                var boolean changed;
                if (isMarried = isMarried_) {
                    return;
                } else {
                    let changed = true;
                }

                let isMarried = isMarried_;
                return;
            }

            method void sayHello() {
                var String greeting;
                let greeting = "Hallo Welt";
                do Output.printString(greeting);
                return;
            }
        }
        "#;

        let ast = run_code_to_ast_conversion(code).expect("Failed to convert code to AST");

        dbg!(&ast);
        assert_eq!(ast.name, "Person");
        assert!(!ast.class_var_declarations.is_empty());
        assert!(!ast.subroutine_declarations.is_empty());
    }

    #[test]
    fn test_if_statement_conversion() {
        let code = r#"
        class Test {

            static Environment env;

            method void testIf(boolean condition) {
                if (condition) {
                    return;
                } else {
                    let condition = false;
                }
                return;
            }
        }
        "#;

        let ast = run_code_to_ast_conversion(code).expect("Failed to convert code to AST");

        dbg!(&ast);

        let subroutine = &ast.subroutine_declarations[0];
        let body = &subroutine.body;
        let statements = &body.statements;

        assert_eq!(statements.len(), 2);
        if let Statement::If {
            condition: _,
            if_statements: _,
            else_statements: Some(else_stmts),
        } = &statements[0]
        {
            assert_eq!(else_stmts.len(), 2);
        } else {
            panic!("Expected an if statement with else branch");
        }
    }

    #[test]
    fn test_expression_conversion() {
        let code = r#"
        class Calculator {
            method int calcSomething(int a, int b) {
                var int answer;
                let answer = add(41, 1);
                return -a + b * (a - b);
            }
            method int add(int x, int y) {
                return x + y;
            }
        }
        "#;

        let ast = run_code_to_ast_conversion(code).expect("Failed to convert code to AST");

        dbg!(&ast);

        let subroutine = &ast.subroutine_declarations[0];
        let body = &subroutine.body;
        let statements = &body.statements;

        assert_eq!(statements.len(), 2);
    }

    fn run_code_to_ast_conversion(code: &str) -> Result<Class, String> {
        let mut stream = StringCharStream::new(code);
        let mut lexer = lexer::Lexer::new(&mut stream);
        let mut parser = parser::Parser::new(&mut lexer);

        let parse_tree = parser
            .parse_class()
            .map_err(|e| format!("Parsing error: {}", e))?;
        parse_tree_printer::print_parse_tree(&parse_tree);
        convert_class(&parse_tree)
    }
}
