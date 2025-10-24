mod symbol_table;

use std::io::BufRead;
use symbol_table::{Address, SymbolTable};

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

type BinaryInstruction = String;

pub fn assemble(asm_file: &str) -> Result<Vec<BinaryInstruction>, String> {
    let instruction_lines = read_instruction_lines(asm_file);
    let instructions = parse_instruction_lines(instruction_lines)?;
    let mut symbol_table = create_symbol_table_with_label_entries(&instructions);

    assemble_instructions(&instructions, &mut symbol_table)
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
        Instruction::AInstruction(arg) => {
            assemble_a_instruction(arg, symbol_table, next_variable_address).map(Some)
        }
        Instruction::CInstruction { dest, comp, jump } => {
            assemble_c_instruction(dest, comp, jump).map(Some)
        }
        Instruction::Label(_) => Ok(None),
    }
}

fn assemble_c_instruction(
    dest: &Option<String>,
    comp: &String,
    jump: &Option<String>,
) -> Result<BinaryInstruction, String> {
    let dest_bits = create_dest_bits(dest)?;
    let comp_bits = create_comp_bits(comp)?;
    let jump_bits = create_jump_bits(jump)?;
    Ok(format!("111{comp_bits}{dest_bits}{jump_bits}"))
}

fn create_comp_bits(comp: &String) -> Result<String, String> {
    match comp.as_str() {
        "0" => Ok("0101010".to_string()),
        "1" => Ok("0111111".to_string()),
        "-1" => Ok("0111010".to_string()),
        "D" => Ok("0001100".to_string()),
        "A" => Ok("0110000".to_string()),
        "M" => Ok("1110000".to_string()),
        "!D" => Ok("0001101".to_string()),
        "!A" => Ok("0110001".to_string()),
        "!M" => Ok("1110001".to_string()),
        "-D" => Ok("0001111".to_string()),
        "-A" => Ok("0110011".to_string()),
        "-M" => Ok("1110011".to_string()),
        "D+1" => Ok("0011111".to_string()),
        "A+1" => Ok("0110111".to_string()),
        "M+1" => Ok("1110111".to_string()),
        "D-1" => Ok("0001110".to_string()),
        "A-1" => Ok("0110010".to_string()),
        "M-1" => Ok("1110010".to_string()),
        "D+A" => Ok("0000010".to_string()),
        "D+M" => Ok("1000010".to_string()),
        "D-A" => Ok("0010011".to_string()),
        "D-M" => Ok("1010011".to_string()),
        "A-D" => Ok("0000111".to_string()),
        "M-D" => Ok("1000111".to_string()),
        "D&A" => Ok("0000000".to_string()),
        "D&M" => Ok("1000000".to_string()),
        "D|A" => Ok("0010101".to_string()),
        "D|M" => Ok("1010101".to_string()),
        _ => Err(format!("{comp} is not a valid comp mnemonic")),
    }
}

fn create_dest_bits(dest: &Option<String>) -> Result<String, String> {
    if dest.is_none() {
        return Ok("000".to_string());
    }

    let dest = dest.as_ref().unwrap();
    let mut bits = 0;
    if dest.contains('M') {
        bits += 1;
    }
    if dest.contains('D') {
        bits += 2;
    }
    if dest.contains('A') {
        bits += 4;
    }
    Ok(format!("{:03b}", bits))
}

fn create_jump_bits(jump: &Option<String>) -> Result<String, String> {
    match jump {
        None => Ok("000".to_string()),
        Some(j) => match j.as_str() {
            "JGT" => Ok("001".to_string()),
            "JEQ" => Ok("010".to_string()),
            "JGE" => Ok("011".to_string()),
            "JLT" => Ok("100".to_string()),
            "JNE" => Ok("101".to_string()),
            "JLE" => Ok("110".to_string()),
            "JMP" => Ok("111".to_string()),
            _ => Err(format!("Invalid jump mnemonic: {}", j)),
        }
    }
}

fn assemble_a_instruction(
    arg: &AInstructionArg,
    symbol_table: &mut SymbolTable,
    next_variable_address: &mut Address,
) -> Result<BinaryInstruction, String> {
    match arg {
        AInstructionArg::Value(value) => {
            if *value > 32767 {
                return Err(format!("A-instruction value {} out of range", value));
            }
            let binary = format!("0{:015b}", value);
            Ok(binary)
        }
        AInstructionArg::Symbol(symbol) => {
            if let Some(address) = symbol_table.lookup(symbol) {
                let binary = format!("0{:015b}", address);
                Ok(binary)
            } else {
                let address = *next_variable_address;
                symbol_table.add_symbol(symbol, address);
                *next_variable_address += 1;
                let binary = format!("0{:015b}", address);
                Ok(binary)
            }
        }
    }}

fn create_symbol_table_with_label_entries(instructions: &Vec<Instruction>) -> SymbolTable {
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
