use clap::Parser;
use nand2tetris::jack::{self, CompilerCli};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = CompilerCli::parse();
    let jack_files = jack::get_jack_files(&config.source);

    let output_dir = match &config.output_dir {
        Some(dir) => Some(dir.as_str()),
        None => None,
    };

    for file in jack_files {
        jack::compile_file(&file, output_dir)?;
    }

    Ok(())
}
