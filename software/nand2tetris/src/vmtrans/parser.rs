use crate::grammarous::stream::Stream;
use crate::grammarous::string_char_stream::StringCharStream;
use crate::vmtrans::ast::{ASTNode, Segment};
use crate::vmtrans::lexer::Lexer;
use crate::vmtrans::token_type::TokenType::*;

pub fn parse_vm_code(code: &str) -> Result<ASTNode, String> {
    let mut stream = StringCharStream::new(code);
    program(&mut stream)
}

fn program(stream: &mut dyn Stream<char>) -> Result<ASTNode, String> {
    let mut lexer = Lexer::new(stream);
    let mut commands = Vec::new();

    loop {
        if let Some(token) = lexer.advance() {
            let command = match token.token_type {
                Push => push(&mut lexer)?,
                Pop => pop(&mut lexer)?,
                Add => ASTNode::Add,
                Sub => ASTNode::Sub,
                Neg => ASTNode::Neg,
                Eq => ASTNode::Eq,
                Gt => ASTNode::Gt,
                Lt => ASTNode::Lt,
                And => ASTNode::And,
                Or => ASTNode::Or,
                Not => ASTNode::Not,
                Label => label(&mut lexer)?,
                Goto => goto_command(&mut lexer)?,
                IfGoto => if_goto(&mut lexer)?,
                Function => function(&mut lexer)?,
                Call => call(&mut lexer)?,
                Return => ASTNode::Return,
                _ => {
                    return Err(format!("Unexpected token: {:?}", token));
                }
            };
            commands.push(command);
        } else {
            break;
        }
    }

    Ok(ASTNode::Program { commands })
}

fn call(lexer: &mut Lexer) -> Result<ASTNode, String> {
    let name_token = lexer.advance().ok_or("Expected function name after 'call'")?;
    let n_args_token = lexer.advance().ok_or("Expected number of arguments after function name")?;

    let name = match name_token.token_type {
        Name(name) => name,
        _ => return Err(format!("Invalid function name: {:?}", name_token)),
    };

    let n_args = match n_args_token.token_type {
        Number(n) => n,
        _ => return Err(format!("Invalid number of arguments: {:?}", n_args_token)),
    };

    Ok(ASTNode::Call { name, n_args })
}

fn function(lexer: &mut Lexer) -> Result<ASTNode, String> {
    let name_token = lexer.advance().ok_or("Expected function name after 'function'")?;
    let n_vars_token = lexer.advance().ok_or("Expected number of local variables after function name")?;

    let name = match name_token.token_type {
        Name(name) => name,
        _ => return Err(format!("Invalid function name: {:?}", name_token)),
    };

    let n_vars = match n_vars_token.token_type {
        Number(n) => n,
        _ => return Err(format!("Invalid number of local variables: {:?}", n_vars_token)),
    };

    Ok(ASTNode::Function { name, n_locals: n_vars })
}

fn if_goto(lexer: &mut Lexer) -> Result<ASTNode, String> {
    let label_token = lexer.advance().ok_or("Expected label name after 'if-goto'")?;
    match label_token.token_type {
        Name(label) => Ok(ASTNode::IfGoto { label }),
        _ => Err(format!("Invalid label name: {:?}", label_token)),
    }
}

fn goto_command(lexer: &mut Lexer) -> Result<ASTNode, String> {
    let label_token = lexer.advance().ok_or("Expected label name after 'goto'")?;
    match label_token.token_type {
        Name(label) => Ok(ASTNode::Goto { label }),
        _ => Err(format!("Invalid label name: {:?}", label_token)),
    }
}

fn label(lexer: &mut Lexer) -> Result<ASTNode, String> {
    let name_token = lexer.advance().ok_or("Expected label name after 'label'")?;
    match name_token.token_type {
        Name(name) => Ok(ASTNode::Label { name }),
        _ => Err(format!("Invalid label name: {:?}", name_token)),
    }
}

fn push(lexer: &mut Lexer) -> Result<ASTNode, String> {
    let segment = segment(lexer)?;
    let index = index(lexer)?;

    Ok(ASTNode::Push { segment, index })
}

fn pop(lexer: &mut Lexer) -> Result<ASTNode, String> {
    let segment = segment(lexer)?;
    let index = index(lexer)?;

    Ok(ASTNode::Pop { segment, index })
}

fn segment(lexer: &mut Lexer) -> Result<Segment, String> {
    let segment_token = lexer.advance().ok_or("Expected segment after 'push'")?;
    Ok(match segment_token.token_type {
        Argument => Segment::Argument,
        Local => Segment::Local,
        Static => Segment::Static,
        Constant => Segment::Constant,
        This => Segment::This,
        That => Segment::That,
        Pointer => Segment::Pointer,
        Temp => Segment::Temp,
        _ => return Err(format!("Invalid segment: {:?}", segment_token)),
    })
}

fn index(lexer: &mut Lexer) -> Result<u16, String> {
    let index_token = lexer.advance().ok_or("Expected index")?;
    match index_token.token_type {
        Number(i) => Ok(i),
        _ => Err(format!("Invalid index: {:?}", index_token)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_basic() {
        let input = "push constant 10\npop local 0\nadd\n";
        let ast = parse_vm_code(input).unwrap();
        dbg!(&ast);
        match ast {
            ASTNode::Program { commands } => {
                assert_eq!(commands.len(), 3);
                match &commands[0] {
                    ASTNode::Push { segment, index } => {
                        assert_eq!(*segment, Segment::Constant);
                        assert_eq!(*index, 10);
                    }
                    _ => panic!("Expected Push command"),
                }
                match &commands[1] {
                    ASTNode::Pop { segment, index } => {
                        assert_eq!(*segment, Segment::Local);
                        assert_eq!(*index, 0);
                    }
                    _ => panic!("Expected Pop command"),
                }
                match &commands[2] {
                    ASTNode::Add => {}
                    _ => panic!("Expected Add command"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_parse_branching() {
        let input = "label LOOP_START\ngoto LOOP_START\nif-goto END\n";
        let ast = parse_vm_code(input).unwrap();
        dbg!(&ast);
        match ast {
            ASTNode::Program { commands } => {
                assert_eq!(commands.len(), 3);
                match &commands[0] {
                    ASTNode::Label { name } => {
                        assert_eq!(name, "LOOP_START");
                    }
                    _ => panic!("Expected Label command"),
                }
                match &commands[1] {
                    ASTNode::Goto { label } => {
                        assert_eq!(label, "LOOP_START");
                    }
                    _ => panic!("Expected Goto command"),
                }
                match &commands[2] {
                    ASTNode::IfGoto { label } => {
                        assert_eq!(label, "END");
                    }
                    _ => panic!("Expected IfGoto command"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }

    #[test]
    fn test_parse_function_calls() {
        let input = "function SimpleFunction 3\ncall SimpleFunction 2\nreturn\n";
        let ast = parse_vm_code(input).unwrap();
        dbg!(&ast);
        match ast {
            ASTNode::Program { commands } => {
                assert_eq!(commands.len(), 3);
                match &commands[0] {
                    ASTNode::Function { name, n_locals: n_vars } => {
                        assert_eq!(name, "SimpleFunction");
                        assert_eq!(*n_vars, 3);
                    }
                    _ => panic!("Expected Function command"),
                }
                match &commands[1] {
                    ASTNode::Call { name, n_args } => {
                        assert_eq!(name, "SimpleFunction");
                        assert_eq!(*n_args, 2);
                    }
                    _ => panic!("Expected Call command"),
                }
                match &commands[2] {
                    ASTNode::Return => {}
                    _ => panic!("Expected Return command"),
                }
            }
            _ => panic!("Expected Program node"),
        }
    }
}
