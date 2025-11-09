#[derive(clap::Parser, Debug, Clone)]
#[command(name="JackAnalyzer", version, about="Analyze Jack programs", long_about = None)]
pub struct Cli {
    pub source: String,
    #[arg(short, long, help="Output directory for the analysis results")]
    pub output_dir: Option<String>,
}

