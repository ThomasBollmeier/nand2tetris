use nand2tetris::assembler;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: HackAssembler <inputfile.asm>");
        std::process::exit(1);
    }

    let asm_file = &args[1];
    let _ = assembler::assemble(asm_file);
}