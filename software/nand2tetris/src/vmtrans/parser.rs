use crate::grammarous::stream::Stream;
use crate::grammarous::string_char_stream::StringCharStream;
use crate::vmtrans::ast::{ASTNode, Segment};
use crate::vmtrans::lexer::Lexer;
use crate::vmtrans::token_type::TokenType::*;

pub fn parse_string(input: &str) -> Result<ASTNode, String> {
    let mut stream = StringCharStream::new(input);
    parse(&mut stream)
}

pub fn parse(stream: &mut dyn Stream<char>) -> Result<ASTNode, String> {
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
        Index(i) => Ok(i),
        _ => Err(format!("Invalid index: {:?}", index_token)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_basic() {
        let input = "push constant 10\npop local 0\nadd\n";
        let ast = parse_string(input).unwrap();
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
}
