use nand2tetris::assembler::Assembler;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: HackAssembler <inputfile.asm>");
        std::process::exit(1);
    }

    let asm_file = &args[1];
    let assembler = Assembler::new();
    assembler.assemble(asm_file);
}