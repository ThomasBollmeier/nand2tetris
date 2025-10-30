use crate::vmtrans::ast::{ASTNode, Segment, Segment::*};
use crate::vmtrans::parser::parse_vm_code;

pub fn write_code(static_prefix: &str, vm_code: &str) -> Result<Vec<String>, String> {
    let program = parse_vm_code(vm_code)?;
    let code_writer = CodeWriter::new(static_prefix.to_string());
    code_writer.write_program(&program)
}

struct CodeWriter {
    static_prefix: String,
}

impl CodeWriter {
    pub fn new(static_prefix: String) -> Self {
        Self {
            static_prefix,
        }
    }

    pub fn write_program(&self, program: &ASTNode) -> Result<Vec<String>, String> {
        match program {
            ASTNode::Program { commands } => {
                let mut lines = vec![];
                for command in commands {
                    lines.extend(self.write_command(command)?);
                }
                lines.push("// end of program:".to_string());
                let end_label = format!("{}.END", self.static_prefix.to_ascii_uppercase());
                lines.push(format!("({end_label})"));
                lines.push(format!("@{end_label}"));
                lines.push("0;JMP".to_string());
                Ok(lines)
            }
            _ => Err("Expected Program node".to_string()),
        }
    }

    fn write_command(&self, command: &ASTNode) -> Result<Vec<String>, String> {
        let mut lines = vec![format!("// {}:", command.to_command_string())];
        let command_lines = match command {
            ASTNode::Push { segment, index } =>
                self.write_push(segment, *index),
            ASTNode::Pop { segment, index } =>
                self.write_pop(segment, index),
            ASTNode::Add | ASTNode::Sub => self.write_binary_arithmetic(command),
            _ => return Err("Unsupported command".to_string()),
        };
        lines.extend(command_lines);

        Ok(lines)
    }

    fn write_binary_arithmetic(&self, command: &ASTNode) -> Vec<String> {
        let mut lines = vec![];
        // Pop y
        lines.push("@SP".to_string());
        lines.push("M=M-1".to_string());
        lines.push("A=M".to_string());
        lines.push("D=M".to_string()); // D = y
        // Pop x
        lines.push("@SP".to_string());
        lines.push("M=M-1".to_string());
        lines.push("A=M".to_string());
        match command {
            ASTNode::Add => {
                lines.push("M=D+M".to_string()); // x + y
            }
            ASTNode::Sub => {
                lines.push("M=M-D".to_string()); // x - y
            }
            _ => {}
        }
        // Increment SP
        lines.push("@SP".to_string());
        lines.push("M=M+1".to_string());
        lines
    }

    fn write_pop(&self, segment: &Segment, index: &u16) -> Vec<String> {
        let mut lines = vec![];
        lines.push("@SP".to_string());
        lines.push("M=M-1".to_string());
        lines.push("A=M".to_string());
        lines.push("D=M".to_string());
        lines.extend(self.write_d_to_segment(segment, *index));
        lines
    }

    fn write_push(&self, segment: &Segment, index: u16) -> Vec<String> {
        let mut lines = vec![];
        lines.extend(self.write_segment_to_d(segment, index));
        lines.push("@SP".to_string());
        lines.push("A=M".to_string());
        lines.push("M=D".to_string());
        lines.push("@SP".to_string());
        lines.push("M=M+1".to_string());
        lines
    }

    fn write_segment_to_d(&self, segment: &Segment, index: u16) -> Vec<String> {
        let mut lines = vec![];
        match segment {
            Static => {
                lines.push(format!("@{}.{}", self.static_prefix, index));
                lines.push("D=M".to_string());
            }
            Constant => {
                lines.push(format!("@{}", index));
                lines.push("D=A".to_string());
            }
            _ => {
                lines.extend(self.set_address(segment, index));
                lines.push("D=M".to_string());
            }
        }
        lines
    }

    fn write_d_to_segment(&self, segment: &Segment, index: u16) -> Vec<String> {
        let mut lines = vec![];
        match segment {
            Static => {
                lines.push(format!("@{}.{}", self.static_prefix, index));
                lines.push("M=D".to_string());
            }
            Constant => {
                panic!("Constants cannot be the target of a pop operation");
            }
            _ => {
                lines.extend(self.set_address(segment, index));
                lines.push("M=D".to_string());
            }
        }
        lines
    }

    fn set_address(&self, segment: &Segment, index: u16) -> Vec<String> {
        let label = self.segment_to_asm_label(segment);
        if index > 0 {
            vec![
                format!("@{}", index),
                "D=A".to_string(),
                format!("@{}", label),
                "A=M".to_string(),
                "A=A+D".to_string(),
            ]
        } else {
            vec![format!("@{}", label), "A=M".to_string()]
        }
    }

    fn segment_to_asm_label(&self, segment: &Segment) -> &'static str {
        match segment {
            Argument => "ARG",
            Local => "LCL",
            This => "THIS",
            That => "THAT",
            Pointer => "POINTER",
            Temp => "TEMP",
            _ => panic!("Invalid segment for address setting"),
        }
    }
}