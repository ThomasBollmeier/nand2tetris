use crate::vmtrans::ast::{ASTNode, Segment, Segment::*};
use crate::vmtrans::parser::parse_vm_code;
use std::collections::HashMap;
use std::path::Path;
use crate::vmtrans::Cli;

pub fn write_asm_code(config: &Cli) -> Result<(), String> {
    let source = &config.source;
    let path = Path::new(source);
    let base_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid source name".to_string())?;

    let mut asm_lines = if !config.no_call_sys_init {
        translate_start_of_program()
    } else {
        vec![]
    };

    asm_lines.extend(if !path.exists() {
        return Err(format!("{source} does not exist"));
    } else if path.is_file() {
        translate_vm_file(source)?
    } else if path.is_dir() {
        translate_vm_directory(source)?
    } else {
        return Err(format!("{source} is not a file or directory"));
    });

    if config.no_call_sys_init {
        asm_lines.extend(translate_end_of_program(base_name));
    }

    let mut asm_file_name = format!("{}.asm", base_name);
    if path.is_dir() {
        asm_file_name = format!(
            "{}/{}.asm",
            path.to_str()
                .ok_or_else(|| "Invalid directory path".to_string())?,
            base_name
        );
    }

    std::fs::write(&asm_file_name, asm_lines.join("\n") + "\n")
        .map_err(|e| format!("Error writing to file {asm_file_name}: {e}"))?;
    println!("Wrote output to {}", asm_file_name);

    Ok(())
}

fn translate_start_of_program() -> Vec<String> {
    let mut lines = vec![];
    // Initialize SP to 256
    lines.push("@256 // <--- Start".to_string());
    lines.push("D=A".to_string());
    lines.push("@SP".to_string());
    lines.push("M=D".to_string());
    // Call Sys.init
    let mut code_writer = CodeWriter::new("Sys".to_string());
    code_writer.write_call("Sys.init", 0);
    lines.extend(code_writer.get_lines());
    lines
}

fn translate_end_of_program(prefix: &str) -> Vec<String> {
    let mut lines = vec![];
    let end_label = format!("{prefix}.end");
    lines.push(format!("({end_label}) // <-- end of program"));
    lines.push(format!("@{end_label}"));
    lines.push("0;JMP".to_string());
    lines
}

fn translate_vm_directory(dir_path: &str) -> Result<Vec<String>, String> {
    let mut all_asm_lines = vec![];
    let entries = std::fs::read_dir(dir_path)
        .map_err(|e| format!("Error reading directory {}: {}", dir_path, e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Error reading directory entry: {e}"))?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "vm" {
                    let file_path = path
                        .to_str()
                        .ok_or_else(|| "Invalid file path".to_string())?;
                    let asm_lines = translate_vm_file(file_path)?;
                    all_asm_lines.extend(asm_lines);
                }
            }
        }
    }

    Ok(all_asm_lines)
}

fn translate_vm_file(file_path: &str) -> Result<Vec<String>, String> {
    let vm_code = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Error reading file {}: {}", file_path, e))?;
    let static_prefix = Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| format!("Invalid file name: {}", file_path))?;

    print!("Translating {file_path}...");
    let program = parse_vm_code(&vm_code)?;
    let mut code_writer = CodeWriter::new(static_prefix.to_string());
    let asm_lines = code_writer.write_program(&program)?;
    println!(" done.");
    Ok(asm_lines)
}

struct CodeWriter {
    static_prefix: String,
    current_function: Option<String>,
    label_counters: HashMap<String, u16>,
    pending_comment: Option<String>,
    lines: Vec<String>,
}

impl CodeWriter {
    pub fn new(static_prefix: String) -> Self {
        Self {
            static_prefix,
            current_function: None,
            label_counters: HashMap::new(),
            pending_comment: None,
            lines: vec![],
        }
    }

    pub fn get_lines(&self) -> Vec<String> {
        self.lines.clone()
    }

    pub fn write_program(&mut self, program: &ASTNode) -> Result<Vec<String>, String> {
        match program {
            ASTNode::Program { commands } => {
                for command in commands {
                    self.write_command(command)?;
                }
                Ok(self.get_lines())
            }
            _ => Err("Expected Program node".to_string()),
        }
    }

    fn write_command(&mut self, command: &ASTNode) -> Result<(), String> {
        self.emit_comment(&command.to_command_string());

        match command {
            ASTNode::Push { segment, index } => self.write_push(segment, *index),
            ASTNode::Pop { segment, index } => self.write_pop(segment, index),
            ASTNode::Add | ASTNode::Sub | ASTNode::And | ASTNode::Or => self.write_binary(command)?,
            ASTNode::Eq | ASTNode::Lt | ASTNode::Gt => self.write_comparison(command)?,
            ASTNode::Neg | ASTNode::Not => self.write_unary(command)?,
            ASTNode::Label { name } => self.write_label(name),
            ASTNode::Goto { label } => self.write_goto(label),
            ASTNode::IfGoto { label } => self.write_if_goto(label),
            ASTNode::Function { name, n_locals } => self.write_function(name, *n_locals),
            ASTNode::Call { name, n_args} => self.write_call(name, *n_args),
            ASTNode::Return => self.write_return(),
            _ => return Err("Unsupported command".to_string()),
        };

        Ok(())
    }

    fn write_return(&mut self)  {
        // FRAME = LCL
        self.emit_code("@LCL");
        self.emit_code("D=M");
        self.emit_code("@R13"); // R13 = FRAME
        self.emit_code("M=D");
        // RET = *(FRAME - 5)
        self.emit_code("@5");
        self.emit_code("A=D-A");
        self.emit_code("D=M");
        self.emit_code("@R14"); // R14 = RET
        self.emit_code("M=D");
        // *ARG = pop()
        self.emit_code("@SP");
        self.emit_code("M=M-1");
        self.emit_code("A=M");
        self.emit_code("D=M");
        self.emit_code("@ARG");
        self.emit_code("A=M");
        self.emit_code("M=D");
        // SP = ARG + 1
        self.emit_code("@ARG");
        self.emit_code("D=M+1");
        self.emit_code("@SP");
        self.emit_code("M=D");
        // THAT = *(FRAME - 1)
        self.emit_code("@R13");
        self.emit_code("AM=M-1");
        self.emit_code("D=M");
        self.emit_code("@THAT");
        self.emit_code("M=D");
        // THIS = *(FRAME - 2)
        self.emit_code("@R13");
        self.emit_code("AM=M-1");
        self.emit_code("D=M");
        self.emit_code("@THIS");
        self.emit_code("M=D");
        // ARG = *(FRAME - 3)
        self.emit_code("@R13");
        self.emit_code("AM=M-1");
        self.emit_code("D=M");
        self.emit_code("@ARG");
        self.emit_code("M=D");
        // LCL = *(FRAME - 4)
        self.emit_code("@R13");
        self.emit_code("AM=M-1");
        self.emit_code("D=M");
        self.emit_code("@LCL");
        self.emit_code("M=D");
        // goto RET
        self.emit_code("@R14");
        self.emit_code("A=M");
        self.emit_code("0;JMP");
    }

    fn write_call(&mut self, callee_name: &str, n_args: u16)  {
        let return_label = self.create_unique_label("ret");
        // Push return address
        self.emit_code(&format!("@{}", return_label));
        self.emit_code("D=A");
        self.push_d_to_stack();
        // Push LCL
        self.emit_code("@LCL");
        self.emit_code("D=M");
        self.push_d_to_stack();
        // Push ARG
        self.emit_code("@ARG");
        self.emit_code("D=M");
        self.push_d_to_stack();
        // Push THIS
        self.emit_code("@THIS");
        self.emit_code("D=M");
        self.push_d_to_stack();
        // Push THAT
        self.emit_code("@THAT");
        self.emit_code("D=M");
        self.push_d_to_stack();
        // Reposition ARG
        self.emit_code("@SP");
        self.emit_code("D=M");
        self.emit_code(&format!("@{}", n_args + 5));
        self.emit_code("D=D-A");
        self.emit_code("@ARG");
        self.emit_code("M=D");
        // Reposition LCL
        self.move_content("SP", "LCL");
        // Transfer control to callee
        self.emit_code(&format!("@{callee_name}"));
        self.emit_code("0;JMP");
        // Set label for return address
        self.emit_code(&format!("({return_label})"));
    }

    fn move_content(&mut self, from: &str, to: &str) {
        self.emit_code(&format!("@{}", from));
        self.emit_code("D=M");
        self.emit_code(&format!("@{}", to));
        self.emit_code("M=D");
    }

    fn push_d_to_stack(&mut self) {
        self.emit_code("@SP");
        self.emit_code("A=M");
        self.emit_code("M=D");
        self.emit_code("@SP");
        self.emit_code("M=M+1");
    }

    fn write_function(&mut self, name: &str, n_locals: u16) {
        self.current_function = Some(name.to_string());
        self.emit_code(&format!("({name})"));
        for _ in 0..n_locals {
            self.emit_code("@SP");
            self.emit_code("A=M");
            self.emit_code("M=0");
            self.emit_code("@SP");
            self.emit_code("M=M+1");
        }
    }

    fn write_if_goto(&mut self, label: &str) {
        let label = self.create_label(label);
        // Pop value
        self.emit_code("@SP");
        self.emit_code("M=M-1");
        self.emit_code("A=M");
        self.emit_code("D=M"); // D = value
        self.emit_code(&format!("@{label}"));
        self.emit_code("D;JNE"); // If D != 0, jump to label
    }

    fn write_goto(&mut self, label: &str) {
        let label = self.create_label(label);
        self.emit_code(&format!("@{label}"));
        self.emit_code("0;JMP");
    }

    fn write_label(&mut self, name: &str) {
        let label = self.create_label(name);
        self.emit_code(&format!("({label})"));
    }

    fn write_unary(&mut self, unary_op: &ASTNode) -> Result<(), String> {
        // Peek x
        self.emit_code("@SP");
        self.emit_code("A=M-1");
        self.emit_code("D=M"); // D = x

        match unary_op {
            ASTNode::Neg => {
                self.emit_code("M=-D"); // M = -x
            }
            ASTNode::Not => {
                self.emit_code("M=!D"); // M = !x
            }
            _ => {
                return Err("Unsupported unary operation".to_string());
            }
        }

        Ok(())
    }

    fn write_comparison(&mut self, comparison: &ASTNode) -> Result<(), String> {
        // Pop y
        self.emit_code("@SP");
        self.emit_code("M=M-1");
        self.emit_code("A=M");
        self.emit_code("D=M"); // D = y
        // Pop x
        self.emit_code("@SP");
        self.emit_code("M=M-1");
        self.emit_code("A=M");
        self.emit_code("D=M-D"); // D = x - y

        let true_label = self.create_unique_label("true");
        let end_label = self.create_unique_label("end");

        self.emit_code(&format!("@{true_label}"));

        match comparison {
            ASTNode::Eq => {
                self.emit_code("D;JEQ");
            }
            ASTNode::Lt => {
                self.emit_code("D;JLT");
            }
            ASTNode::Gt => {
                self.emit_code("D;JGT");
            }
            _ => {
                panic!("Unsupported comparison");
            }
        }

        // Push false (0)
        self.emit_code("@SP");
        self.emit_code("A=M");
        self.emit_code("M=0");

        self.emit_code(&format!("@{end_label}"));
        self.emit_code("0;JMP");
        self.emit_code(&format!("({true_label})"));

        // Push true (-1)
        self.emit_code("@SP");
        self.emit_code("A=M");
        self.emit_code("M=-1");

        self.emit_code(&format!("({end_label})"));

        // Increment SP
        self.emit_code("@SP");
        self.emit_code("M=M+1");

        Ok(())
    }

    fn create_unique_label(&mut self, base: &str) -> String {
        let prefix = if let Some(func_name) = &self.current_function {
            func_name
        } else {
            &self.static_prefix
        };
        let counter = self.label_counters.entry(base.to_string()).or_insert(0);
        let label = format!("{}${}.{}", prefix, base, *counter);
        *counter += 1;

        label
    }

    fn create_label(&mut self, label: &str) -> String {
        let prefix = if let Some(func_name) = &self.current_function {
            func_name
        } else {
            &self.static_prefix
        };
        format!("{prefix}${label}")
    }

    fn write_binary(&mut self, command: &ASTNode) -> Result<(), String> {
        // Pop y
        self.emit_code("@SP");
        self.emit_code("M=M-1");
        self.emit_code("A=M");
        self.emit_code("D=M"); // D = y
        // Pop x
        self.emit_code("@SP");
        self.emit_code("M=M-1");
        self.emit_code("A=M");
        match command {
            ASTNode::Add => {
                self.emit_code("M=D+M"); // x + y
            }
            ASTNode::Sub => {
                self.emit_code("M=M-D"); // x - y
            }
            ASTNode::And => {
                self.emit_code("M=D&M"); // x & y
            }
            ASTNode::Or => {
                self.emit_code("M=D|M"); // x | y
            }
            _ => {
                return Err("Unsupported binary command".to_string());
            }
        }
        // Increment SP
        self.emit_code("@SP");
        self.emit_code("M=M+1");
        Ok(())
    }

    fn write_pop(&mut self, segment: &Segment, index: &u16) {
        self.emit_code("@SP");
        self.emit_code("M=M-1");
        self.emit_code("A=M");
        self.emit_code("D=M");
        self.write_d_to_segment(segment, *index);
    }

    fn write_push(&mut self, segment: &Segment, index: u16) {
        self.write_segment_to_d(segment, index);
        self.emit_code("@SP");
        self.emit_code("A=M");
        self.emit_code("M=D");
        self.emit_code("@SP");
        self.emit_code("M=M+1");
    }

    fn write_segment_to_d(&mut self, segment: &Segment, index: u16) {
        match segment {
            Static => {
                self.emit_code(&format!("@{}.{}", self.static_prefix, index));
                self.emit_code("D=M");
            }
            Constant => {
                self.emit_code(&format!("@{}", index));
                self.emit_code("D=A");
            }
            _ => {
                self.set_address(segment, index);
                self.emit_code("D=M");
            }
        }
    }

    fn write_d_to_segment(&mut self, segment: &Segment, index: u16) {
        match segment {
            Static => {
                self.emit_code(&format!("@{}.{}", self.static_prefix, index));
                self.emit_code("M=D");
            }
            Constant => {
                panic!("Constants cannot be the target of a pop operation");
            }
            _ => {
                self.emit_code("@R13");
                self.emit_code("M=D"); // Store D temporarily
                self.set_address(segment, index);
                self.emit_code("D=A");
                self.emit_code("@R14");
                self.emit_code("M=D"); // Store target address
                self.emit_code("@R13");
                self.emit_code("D=M"); // Retrieve original D
                self.emit_code("@R14");
                self.emit_code("A=M");
                self.emit_code("M=D"); // *target = D
            }
        }
    }

    fn set_address(&mut self, segment: &Segment, index: u16) {
        match segment {
            Temp => {
                let base_address = 5 + index;
                self.emit_code(&format!("@{base_address}"));
            }
            Pointer => match index {
                0 => self.emit_code("@THIS"),
                1 => self.emit_code("@THAT"),
                _ => panic!("Invalid index for pointer segment"),
            },
            Argument | Local | This | That => {
                let label = self.segment_to_asm_label(segment);
                if index > 0 {
                    self.emit_code(&format!("@{}", index));
                    self.emit_code("D=A");
                    self.emit_code(&format!("@{}", label));
                    self.emit_code("A=M");
                    self.emit_code("A=D+A");
                } else {
                    self.emit_code(&format!("@{}", label));
                    self.emit_code("A=M");
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

    fn emit_comment(&mut self, comment: &str) {
        self.pending_comment = Some(comment.to_string());
    }

    fn emit_code(&mut self, code: &str) {
        if let Some(comment) = self.pending_comment.take() {
            self.lines.push(format!("{} // <- {}", code, comment));
            self.pending_comment = None;
        } else {
            self.lines.push(code.to_string());
        }
    }
}
