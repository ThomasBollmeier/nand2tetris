pub mod token_type;
mod lexer;
pub mod ast;
pub mod ast_printer;
mod parser;
mod cli;

use std::path::Path;
pub use lexer::Lexer;
pub use parser::Parser;
pub use cli::Cli;

use crate::grammarous::string_char_stream::StringCharStream;
use crate::jack::ast_printer::{AstPrinter, StringOutput};

pub fn analyze_file(file_path: &str, output_dir: Option<&str>) -> Result<(), anyhow::Error> {
    let mut stream = StringCharStream::new_from_file(file_path)?;
    let mut lexer = lexer::Lexer::new(&mut stream);
    let mut parser = parser::Parser::new(&mut lexer);

    let ast = parser
        .parse_class()
        .map_err(|e| anyhow::anyhow!("Parsing error in file {}: {}", file_path, e))?;

    let mut output = StringOutput::new();
    let mut ast_printer = AstPrinter::default();
    ast_printer.set_output(&mut output);
    ast_printer.print_ast(&ast);
    let content = output.get_content();

    let outfile_base_name = std::path::Path::new(file_path)
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

    let output_file_path = format!("{}/{}.xml", &output_dir, &outfile_base_name);

    std::fs::write(output_file_path.clone(), content)?;
    println!("Successfully analyzed file {}. XML written to {}", file_path, output_file_path);

    Ok(())
}