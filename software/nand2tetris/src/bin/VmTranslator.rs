use nand2tetris::vmtrans::code_writer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: VmTranslator <Program.vm>");
        std::process::exit(1);
    }

    let vm_file = &args[1];
    match std::fs::read_to_string(vm_file) {
        Ok(vm_code) => {
            let static_prefix = vm_file.replace(".vm", "");
            match code_writer::write_code(&static_prefix, &vm_code) {
                Ok(assembly_code) => {
                    let asm_file = vm_file.replace(".vm", ".asm");
                    std::fs::write(&asm_file, assembly_code.join("\n") + "\n").expect("Unable to write file");
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