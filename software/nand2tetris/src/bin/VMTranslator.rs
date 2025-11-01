use clap::Parser;
use nand2tetris::vmtrans::{Cli, code_writer};

fn main() {
    let config = Cli::parse();
    let vm_file = &config.infile;

    match std::fs::read_to_string(vm_file) {
        Ok(vm_code) => {
            let static_prefix = vm_file.replace(".vm", "");
            match code_writer::write_code(&static_prefix, &vm_code) {
                Ok(assembly_code) => {
                    let asm_file = match &config.outfile {
                        Some(outfile) => outfile.clone(),
                        None => vm_file.replace(".vm", ".asm"),
                    };
                    std::fs::write(&asm_file, assembly_code.join("\n") + "\n")
                        .expect("Unable to write file");
                    println!("Translated {} to {}", vm_file, asm_file);
                }
                Err(e) => {
                    eprintln!("Error during translation: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading file {}: {}", vm_file, e);
            std::process::exit(1);
        }
    }
}
