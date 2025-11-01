
#[derive(clap::Parser, Debug, Clone)]
#[command(name="VMTranslator", version, about="Translates VM code to Hack assembly", long_about = None)]
pub struct Cli {
    pub infile: String,
    #[arg(short, long, value_name = "FILE")]
    pub outfile: Option<String>,
}