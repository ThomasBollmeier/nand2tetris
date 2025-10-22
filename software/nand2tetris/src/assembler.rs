use std::io::BufRead;

pub struct Assembler {
    // Fields for the Assembler struct
}

impl Assembler {
    pub fn new() -> Self {
        Assembler {}
    }

    pub fn assemble(&self, asm_file: &str) -> Result<(), String> {
        let instruction_lines = self.read_instruction_lines(asm_file);
        let mut instructions = Vec::new();
        for line in instruction_lines {
            let instruction = self.parse_instruction_line(&line)
                .map_err(|err| format!("Error parsing line '{}': {}", line, err))?;
            println!("{:#?}", instruction);
            instructions.push(instruction);
        }
        Ok(())
    }

    fn read_instruction_lines(&self, file_path: &str) -> Vec<String> {
        let mut lines = Vec::new();

        let file = match std::fs::File::open(file_path) {
            Ok(file) => file,
            Err(_) => return lines,
        };
        let reader = std::io::BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                let line = line.trim().to_string();
                if line.is_empty() || line.starts_with("//") {
                    continue;
                }
                lines.push(line);
            }
        }

        lines
    }

    fn parse_instruction_line(&self, line: &str) -> Result<Instruction, String> {
        if line.starts_with('@') {
            let arg = &line[1..];
            if let Ok(value) = arg.parse::<u16>() {
                Ok(Instruction::AInstruction(AInstructionArg::Value(value)))
            } else {
                Ok(Instruction::AInstruction(AInstructionArg::Symbol(arg.to_string())))
            }
        } else if line.starts_with('(') && line.ends_with(')') {
            let label = &line[1..line.len() - 1];
            Ok(Instruction::Label(label.to_string()))
        } else {
            let mut dest = None;
            let mut comp_jump = line;

            if let Some(eq_index) = line.find('=') {
                dest = Some(line[..eq_index].to_string());
                comp_jump = &line[eq_index + 1..];
            }

            let mut comp = comp_jump;
            let mut jump = None;

            if let Some(semi_index) = comp_jump.find(';') {
                comp = &comp_jump[..semi_index];
                jump = Some(comp_jump[semi_index + 1..].to_string());
            }

            Ok(Instruction::CInstruction {
                dest,
                comp: comp.to_string(),
                jump,
            })
        }
    }
}

#[derive(Debug, PartialEq)]
enum Instruction {
    AInstruction(AInstructionArg),
    CInstruction {
        dest: Option<String>,
        comp: String,
        jump: Option<String>,
    },
    Label(String),
}

#[derive(Debug, PartialEq)]
enum AInstructionArg {
    Symbol(String),
    Value(u16),
}

#[cfg(test)]
mod tests {
    use crate::assembler::Instruction::AInstruction;
    use super::*;

    #[test]
    fn test_parse_a_instruction_value() {
        let assembler = Assembler::new();
        let instruction = assembler.parse_instruction_line("@123").unwrap();
        assert_eq!(instruction, AInstruction(AInstructionArg::Value(123)));
    }

    #[test]
    fn test_parse_a_instruction_symbol() {
        let assembler = Assembler::new();
        let instruction = assembler.parse_instruction_line("@x").unwrap();
        assert_eq!(instruction, AInstruction(AInstructionArg::Symbol("x".to_string())));
    }

    #[test]
    fn test_parse_label() {
        let assembler = Assembler::new();
        let instruction = assembler.parse_instruction_line("(LOOP)").unwrap();
        assert_eq!(instruction, Instruction::Label("LOOP".to_string()));
    }

    #[test]
    fn test_parse_c_instruction_dest_comp() {
        let assembler = Assembler::new();
        let instruction = assembler.parse_instruction_line("D=M").unwrap();
        assert_eq!(instruction, Instruction::CInstruction {
            dest: Some("D".to_string()),
            comp: "M".to_string(),
            jump: None,
        });
    }

    #[test]
    fn test_parse_c_instruction_dest_comp_jump() {
        let assembler = Assembler::new();
        let instruction = assembler.parse_instruction_line("D=M;JGT").unwrap();
        assert_eq!(instruction, Instruction::CInstruction {
            dest: Some("D".to_string()),
            comp: "M".to_string(),
            jump: Some("JGT".to_string()),
        });
    }
}