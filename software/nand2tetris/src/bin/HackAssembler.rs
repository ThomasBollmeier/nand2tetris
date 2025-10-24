use nand2tetris::assembler;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: HackAssembler <inputfile.asm>");
        std::process::exit(1);
    }

    let asm_file = &args[1];
    match assembler::assemble(asm_file) {
        Ok(binary_code) => {
            let hack_file = asm_file.replace(".asm", ".hack");
            std::fs::write(&hack_file, binary_code.join("\n") + "\n").expect("Unable to write file");
            println!("Assembled {} to {}", asm_file, hack_file);
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}