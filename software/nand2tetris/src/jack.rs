pub mod token_type;
mod lexer;
pub mod parse_tree;
pub mod parse_tree_printer;
mod parser;
mod ast;
pub mod parse_tree_converter;
mod compiler;
mod cli;
mod symbol_table;

use std::path::Path;
pub use lexer::Lexer;
pub use parser::Parser;
pub use cli::{AnalyzerCli, CompilerCli, get_jack_files};

use crate::grammarous::string_char_stream::StringCharStream;
use crate::jack::parse_tree_printer::{ParseTreePrinter, StringOutput};

pub fn analyze_file(file_path: &str, output_dir: Option<&str>) -> Result<(), anyhow::Error> {
    let mut stream = StringCharStream::new_from_file(file_path)?;
    let mut lexer = Lexer::new(&mut stream);
    let mut parser = Parser::new(&mut lexer);

    let ast = parser
        .parse_class()
        .map_err(|e| anyhow::anyhow!("Parsing error in file {}: {}", file_path, e))?;

    let mut output = StringOutput::new();
    let mut ast_printer = ParseTreePrinter::default();
    ast_printer.set_output(&mut output);
    ast_printer.print_ast(&ast);
    let content = output.get_content();

    let (outfile_base_name, output_dir) = derive_base_and_dir(file_path, output_dir)?;
    let output_file_path = format!("{}/{}.xml", &output_dir, &outfile_base_name);
    std::fs::write(output_file_path.clone(), content)?;
    println!("Successfully analyzed file {}. XML written to {}", file_path, output_file_path);

    Ok(())
}

pub fn compile_file(file_path: &str, output_dir: Option<&str>) -> Result<(), anyhow::Error> {
    let mut stream = StringCharStream::new_from_file(file_path)?;
    let mut lexer = Lexer::new(&mut stream);
    let mut parser = Parser::new(&mut lexer);

    let parse_tree = parser
        .parse_class()
        .map_err(|e| anyhow::anyhow!("Parsing error in file {}: {}", file_path, e))?;

    let ast = parse_tree_converter::convert_class(&parse_tree)
        .map_err(|e| anyhow::anyhow!("ParseTree -> AST conversion error in file {}: {}", file_path, e))?;

    let mut compiler = compiler::Compiler::new();
    compiler.compile_class(&ast);
    let vm_code = compiler.get_vm_code();

    let (outfile_base_name, output_dir) = derive_base_and_dir(file_path, output_dir)?;
    let output_file_path = format!("{}/{}.vm", &output_dir, &outfile_base_name);
    std::fs::write(output_file_path.clone(), vm_code)?;
    println!("Successfully compiled file {}. VM code written to {}", file_path, output_file_path);

    Ok(())
}

fn derive_base_and_dir(file_path: &str, output_dir: Option<&str>) -> Result<(String, String), anyhow::Error> {
    let outfile_base_name = Path::new(file_path)
        .file_stem()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid file name for path {}", file_path))?
        .trim_end_matches(".jack");

    let output_dir = match output_dir {
        Some(dir) => dir.to_string(),
        None => {
            let path = Path::new(file_path);
            path.parent()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|| ".".to_string())
        }
    };

    Ok((outfile_base_name.to_string(), output_dir))
}