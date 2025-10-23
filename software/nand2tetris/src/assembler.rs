mod symbol_table;

use std::io::BufRead;
use symbol_table::{Address, SymbolTable};

type BinaryInstruction = String;

pub fn assemble(asm_file: &str) -> Result<(), String> {
    let instruction_lines = read_instruction_lines(asm_file);
    let instructions = parse_instruction_lines(instruction_lines)?;
    let mut symbol_table = create_symbol_table_with_labels(&instructions);
    let binary_instructions = assemble_instructions(&instructions, &mut symbol_table)?;
    for binary_instruction in binary_instructions {
        println!("{}", binary_instruction);
    }
    Ok(())
}

fn assemble_instructions(
    instructions: &Vec<Instruction>,
    symbol_table: &mut SymbolTable,
) -> Result<Vec<BinaryInstruction>, String> {
    let mut binary_instructions = Vec::new();
    let mut next_variable_address = 16 as Address;

    for instruction in instructions {
        if let Some(binary_instruction) =
            assemble_instruction(instruction, symbol_table, &mut next_variable_address)?
        {
            binary_instructions.push(binary_instruction);
        }
    }

    Ok(binary_instructions)
}

fn assemble_instruction(
    instruction: &Instruction,
    symbol_table: &mut SymbolTable,
    next_variable_address: &mut Address,
) -> Result<Option<BinaryInstruction>, String> {
    match instruction {
        Instruction::AInstruction(arg) => match arg {
            AInstructionArg::Value(value) => {
                if *value > 32767 {
                    return Err(format!("A-instruction value {} out of range", value));
                }
                let binary = format!("0{:015b}", value);
                Ok(Some(binary))
            }
            AInstructionArg::Symbol(symbol) => {
                if let Some(address) = symbol_table.lookup(symbol) {
                    let binary = format!("0{:015b}", address);
                    Ok(Some(binary))
                } else {
                    let address = *next_variable_address;
                    symbol_table.add_symbol(symbol, address);
                    *next_variable_address += 1;
                    let binary = format!("0{:015b}", address);
                    Ok(Some(binary))
                }
            }
        }
        Instruction::CInstruction { dest, comp, jump } => {
            // Placeholder for C-instruction assembly logic
            Ok(Some("1110000000000000".to_string()))
        }
        Instruction::Label(_) => Ok(None),
    }
}

fn create_symbol_table_with_labels(instructions: &Vec<Instruction>) -> SymbolTable {
    let mut symbol_table = SymbolTable::new();
    let mut next_address = 0 as Address;

    for instruction in instructions {
        match instruction {
            Instruction::Label(label) => {
                symbol_table.add_label(label, next_address);
            }
            _ => {
                next_address += 1;
            }
        }
    }

    symbol_table
}

fn parse_instruction_lines(instruction_lines: Vec<String>) -> Result<Vec<Instruction>, String> {
    let mut instructions = Vec::new();
    for line in instruction_lines {
        let instruction = parse_instruction_line(&line)
            .map_err(|err| format!("Error parsing line '{}': {}", line, err))?;
        instructions.push(instruction);
    }
    Ok(instructions)
}


fn parse_instruction_line(line: &str) -> Result<Instruction, String> {
    if line.starts_with('@') {
        let arg = &line[1..];
        if let Ok(value) = arg.parse::<u16>() {
            Ok(Instruction::AInstruction(AInstructionArg::Value(value)))
        } else {
            Ok(Instruction::AInstruction(AInstructionArg::Symbol(
                arg.to_string(),
            )))
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

fn read_instruction_lines(file_path: &str) -> Vec<String> {
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
    use super::*;
    use crate::assembler::Instruction::AInstruction;

    #[test]
    fn test_parse_a_instruction_value() {
        let instruction = parse_instruction_line("@123").unwrap();
        assert_eq!(instruction, AInstruction(AInstructionArg::Value(123)));
    }

    #[test]
    fn test_parse_a_instruction_symbol() {
        let instruction = parse_instruction_line("@x").unwrap();
        assert_eq!(
            instruction,
            AInstruction(AInstructionArg::Symbol("x".to_string()))
        );
    }

    #[test]
    fn test_parse_label() {
        let instruction = parse_instruction_line("(LOOP)").unwrap();
        assert_eq!(instruction, Instruction::Label("LOOP".to_string()));
    }

    #[test]
    fn test_parse_c_instruction_dest_comp() {
        let instruction = parse_instruction_line("D=M").unwrap();
        assert_eq!(
            instruction,
            Instruction::CInstruction {
                dest: Some("D".to_string()),
                comp: "M".to_string(),
                jump: None,
            }
        );
    }

    #[test]
    fn test_parse_c_instruction_dest_comp_jump() {
        let instruction = parse_instruction_line("D=M;JGT").unwrap();
        assert_eq!(
            instruction,
            Instruction::CInstruction {
                dest: Some("D".to_string()),
                comp: "M".to_string(),
                jump: Some("JGT".to_string()),
            }
        );
    }
}
