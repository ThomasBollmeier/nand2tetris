use std::path::Path;

#[derive(clap::Parser, Debug, Clone)]
#[command(name="JackAnalyzer", version, about="Analyze Jack programs", long_about = None)]
pub struct AnalyzerCli {
    pub source: String,
    #[arg(short, long, help = "Output directory for the analysis results")]
    pub output_dir: Option<String>,
}

#[derive(clap::Parser, Debug, Clone)]
#[command(name="JackCompiler", version, about="Compile Jack programs", long_about = None)]
pub struct CompilerCli {
    pub source: String,
    #[arg(short, long, help = "Output directory for the compiler results")]
    pub output_dir: Option<String>,
}

pub fn get_jack_files(source: &str) -> Vec<String> {
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
