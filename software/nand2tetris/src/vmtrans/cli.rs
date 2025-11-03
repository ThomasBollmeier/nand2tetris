
#[derive(clap::Parser, Debug, Clone)]
#[command(name="VMTranslator", version, about="Translates VM code to Hack assembly", long_about = None)]
pub struct Cli {
    pub source: String,
    #[arg(short='s', long="no-call-sys-init", help="Suppresses the automatic call to Sys.init")]
    pub no_call_sys_init: bool,
}