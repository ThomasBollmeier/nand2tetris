use crate::jack::ast::*;
use crate::jack::parse_tree::{ParseTreeNode, ParseTreeNodeData};
use crate::jack::token_type::TokenTypeCategory;

pub fn convert_parse_tree_to_ast(class_node: &ParseTreeNode) -> Result<Class, String> {
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
    let mut names = vec![];

    for child in &class_var_dec_node.children {
        if let ParseTreeNode::Terminal(token) = child {
            if token.token_type.get_category() == TokenTypeCategory::Identifier {
                names.push(token.lexeme.clone());
            }
        }
    }

    Ok(ClassVarDec{
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
        Some(ParseTreeNode::Terminal(token)) if token.lexeme == "function" => SubroutineCategory::Function,
        Some(ParseTreeNode::Terminal(token)) if token.lexeme == "method" => SubroutineCategory::Method,
        Some(ParseTreeNode::Terminal(token)) if token.lexeme == "constructor" => SubroutineCategory::Constructor,
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

    let mut parameters= vec![];
    let body = vec![];

    for child in children {
        if let ParseTreeNode::NonTerminal(node) = child {
            match node.name.as_str() {
                "parameterList" => { parameters = convert_parameter_list(node)?; }
                "subroutineBody" => { /* TODO: handle body */ }
                _ => {}
            }
        }
    }

    Ok(SubroutineDec{
        category,
        return_type,
        name,
        parameters,
        body,
    })
}

fn convert_parameter_list(param_list_node: &ParseTreeNodeData) -> Result<Vec<(Type, String)>, String> {
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
    use crate::grammarous::StringCharStream;
    use crate::jack::{lexer, parser};
    use super::*;

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

            method void setMarried(boolean isMarried) {
                var int answer;
                let answer = 41 + 1;

            }

            method void sayHello() {
                do Output.printString("Hallo Welt!");
                return;
            }
        }
        "#;

        let mut stream = StringCharStream::new(code);
        let mut lexer = lexer::Lexer::new(&mut stream);
        let mut parser = parser::Parser::new(&mut lexer);

        let parse_tree = parser.parse_class().expect("Failed to parse class");
        let ast = convert_parse_tree_to_ast(&parse_tree).expect("Failed to convert parse tree to AST");

        dbg!(&ast);
        assert_eq!(ast.name, "Person");
        assert!(!ast.class_var_declarations.is_empty());
        assert!(!ast.subroutine_declarations.is_empty());
    }
}
