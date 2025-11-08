pub mod token_type;
mod lexer;
pub mod ast;
pub mod ast_printer;
mod parser;
mod cli;

pub use lexer::Lexer;
pub use parser::Parser;
pub use cli::Cli;

use crate::grammarous::string_char_stream::StringCharStream;
use crate::jack::ast_printer::{AstPrinter, StringOutput};

pub fn analyze_file(file_path: &str) -> Result<(), anyhow::Error> {
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
    let output_file_path = format!("{}.my.xml", file_path.trim_end_matches(".jack"));
    std::fs::write(output_file_path.clone(), content)?;

    println!("Successfully analyzed file {}. XML written to {}", file_path, output_file_path);

    Ok(())
}