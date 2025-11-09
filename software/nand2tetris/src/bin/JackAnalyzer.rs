use clap::Parser;
use nand2tetris::jack::{self, Cli};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Cli::parse();
    let jack_files = get_jack_files(&config.source);

    let output_dir = match &config.output_dir {
        Some(dir) => Some(dir.as_str()),
        None => None,
    };

    for file in jack_files {
        jack::analyze_file(&file, output_dir)?;
    }

    Ok(())
}

fn get_jack_files(source: &str) -> Vec<String> {
    let mut files = Vec::new();

    let path = Path::new(source);
    if path.is_file() {
        if is_jack_file(path) {
            files.push(source.to_string());
        }
    } else if path.is_dir() {
        if let Ok(entries) = std::fs::read_dir(path) {
            entries
                .into_iter()
                .filter_map(|entry| entry.ok())
                .for_each(|entry| {
                    let entry_path = entry.path();
                    if is_jack_file(&entry_path) {
                        if let Some(file_str) = entry_path.to_str() {
                            files.push(file_str.to_string());
                        }
                    }
                });
        }
    }

    files
}

fn is_jack_file(path: &Path) -> bool {
    path.extension().map_or(false, |ext| ext == "jack")
}
