use std::collections::HashMap;
use crate::vmtrans::ast::{ASTNode, Segment, Segment::*};
use crate::vmtrans::parser::parse_vm_code;

pub fn write_code(static_prefix: &str, vm_code: &str) -> Result<Vec<String>, String> {
    let program = parse_vm_code(vm_code)?;
    let mut code_writer = CodeWriter::new(static_prefix.to_string());
    code_writer.write_program(&program)
}

struct CodeWriter {
    static_prefix: String,
    label_counters: HashMap<String, u16>,
}

impl CodeWriter {
    pub fn new(static_prefix: String) -> Self {
        Self {
            static_prefix,
            label_counters: HashMap::new(),
        }
    }

    pub fn write_program(&mut self, program: &ASTNode) -> Result<Vec<String>, String> {
        match program {
            ASTNode::Program { commands } => {
                let mut lines = vec![];
                for command in commands {
                    lines.extend(self.write_command(command)?);
                }
                lines.extend(self.write_end_of_program());
                Ok(lines)
            }
            _ => Err("Expected Program node".to_string()),
        }
    }

    fn write_end_of_program(&self) -> Vec<String> {
        let mut lines = vec![];
        let end_label = format!("{}.END", self.static_prefix.to_ascii_uppercase());
        lines.push("// end of program:".to_string());
        lines.push(format!("({end_label})"));
        lines.push(format!("@{end_label}"));
        lines.push("0;JMP".to_string());
        lines
    }

    fn write_command(&mut self, command: &ASTNode) -> Result<Vec<String>, String> {
        let mut lines = vec![format!("// {}:", command.to_command_string())];
        let command_lines = match command {
            ASTNode::Push { segment, index } =>
                self.write_push(segment, *index),
            ASTNode::Pop { segment, index } =>
                self.write_pop(segment, index),
            ASTNode::Add | ASTNode::Sub | ASTNode::And | ASTNode::Or => self.write_binary(command)?,
            ASTNode::Eq | ASTNode::Lt | ASTNode::Gt => self.write_comparison(command)?,
            ASTNode::Neg | ASTNode::Not => self.write_unary(command)?,
            ASTNode::Label{ name } => self.write_label(name),
            ASTNode::Goto{ label } => self.write_goto(label),
            ASTNode::IfGoto{ label } => self.write_if_goto(label),
            _ => return Err("Unsupported command".to_string()),
        };
        lines.extend(command_lines);

        Ok(lines)
    }

    fn write_if_goto(&self, label: &str) -> Vec<String> {
        vec![
            // Pop value
            "@SP".to_string(),
            "M=M-1".to_string(),
            "A=M".to_string(),
            "D=M".to_string(), // D = value
            format!("@{label}"),
            "D;JNE".to_string() // If D != 0, jump to label
        ]
    }

    fn write_goto(&self, label: &str) -> Vec<String> {
        vec![
            format!("@{}", label),
            "0;JMP".to_string(),
        ]
    }

    fn write_label(&self, name: &str) -> Vec<String> {
        vec![format!("({name})")]
    }


    fn write_unary(&self, unary_op: &ASTNode) -> Result<Vec<String>, String> {
        let mut lines = vec![];
        // Peek x
        lines.push("@SP".to_string());
        lines.push("A=M-1".to_string());
        lines.push("D=M".to_string()); // D = x

        match unary_op {
            ASTNode::Neg => {
                lines.push("M=-D".to_string()); // M = -x
            }
            ASTNode::Not => {
                lines.push("M=!D".to_string()); // M = !x
            }
            _ => {
                return Err("Unsupported unary operation".to_string());
            }
        }

        Ok(lines)
    }

    fn write_comparison(&mut self, comparison: &ASTNode) -> Result<Vec<String>, String> {
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
        lines.push("D=M-D".to_string()); // D = x - y

        let true_label = self.create_unique_label("TRUE");
        let end_label = self.create_unique_label("END");

        lines.push(format!("@{true_label}"));

        match comparison {
            ASTNode::Eq => {
                lines.push("D;JEQ".to_string());
            }
            ASTNode::Lt => {
                lines.push("D;JLT".to_string());
            }
            ASTNode::Gt => {
                lines.push("D;JGT".to_string());
            }
            _ => {
                panic!("Unsupported comparison");
            }
        }

        // Push false (0)
        lines.push("@SP".to_string());
        lines.push("A=M".to_string());
        lines.push("M=0".to_string());

        lines.push(format!("@{end_label}"));
        lines.push("0;JMP".to_string());
        lines.push(format!("({true_label})"));

        // Push true (-1)
        lines.push("@SP".to_string());
        lines.push("A=M".to_string());
        lines.push("M=-1".to_string());

        lines.push(format!("({end_label})"));

        // Increment SP
        lines.push("@SP".to_string());
        lines.push("M=M+1".to_string());

        Ok(lines)
    }

    fn create_unique_label(&mut self, base: &str) -> String {
        let counter = self.label_counters.entry(base.to_string()).or_insert(0);
        let label = format!("{}.{}.{}", self.static_prefix, base, *counter)
            .to_ascii_uppercase();
        *counter += 1;

        label
    }

    fn write_binary(&self, command: &ASTNode) -> Result<Vec<String>, String> {
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
            ASTNode::And => {
                lines.push("M=D&M".to_string()); // x & y
            }
            ASTNode::Or => {
                lines.push("M=D|M".to_string()); // x | y
            }
            _ => {
                return Err("Unsupported binary command".to_string());
            }
        }
        // Increment SP
        lines.push("@SP".to_string());
        lines.push("M=M+1".to_string());
        Ok(lines)
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
                lines.push("@R13".to_string());
                lines.push("M=D".to_string()); // Store D temporarily
                lines.extend(self.set_address(segment, index));
                lines.push("D=A".to_string());
                lines.push("@R14".to_string());
                lines.push("M=D".to_string()); // Store target address
                lines.push("@R13".to_string());
                lines.push("D=M".to_string()); // Retrieve original D
                lines.push("@R14".to_string());
                lines.push("A=M".to_string());
                lines.push("M=D".to_string()); // *target = D
            }
        }
        lines
    }

    fn set_address(&self, segment: &Segment, index: u16) -> Vec<String> {
        match segment {
            Temp => {
                let base_address = 5 + index;
                vec![format!("@{base_address}")]
            }
            Pointer => match index {
                0 => vec!["@THIS".to_string()],
                1 => vec!["@THAT".to_string()],
                _ => panic!("Invalid index for pointer segment"),
            },
            Argument | Local | This | That => {
                let label = self.segment_to_asm_label(segment);
                if index > 0 {
                    vec![
                        format!("@{}", index),
                        "D=A".to_string(),
                        format!("@{}", label),
                        "A=M".to_string(),
                        "A=D+A".to_string(),
                    ]
                } else {
                    vec![format!("@{}", label), "A=M".to_string()]
                }
            }
            _ => panic!("Invalid segment for address setting"),
        }
    }

    fn segment_to_asm_label(&self, segment: &Segment) -> &'static str {
        match segment {
            Argument => "ARG",
            Local => "LCL",
            This => "THIS",
            That => "THAT",
            _ => panic!("Invalid segment for address setting"),
        }
    }

}