use clap::Parser;
use nand2tetris::vmtrans::{Cli, code_writer};

fn main() {
    let config = Cli::parse();
    match code_writer::write_asm_code(&config) {
        Ok(_) => println!("Translation completed successfully."),
        Err(e) => {
            eprintln!("Error during translation: {}", e);
            std::process::exit(1);
        },
    }
 }
