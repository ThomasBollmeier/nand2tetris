use nand2tetris::grammarous::StringCharStream;
use nand2tetris::jack::Lexer;
use nand2tetris::jack::Parser;

#[test]
fn test_parser_works() {
    parse_jack_file("tests/input/Square.jack");
    parse_jack_file("tests/input/SquareGame.jack");
    parse_jack_file("tests/input/Main.jack");
}

fn parse_jack_file(file_path: &str) {
    let mut stream =
        StringCharStream::new_from_file(file_path).expect("Failed to create char stream");
    let mut lexer = Lexer::new(&mut stream);
    let mut parser = Parser::new(&mut lexer);

    let ast = parser.parse_class();

    assert!(ast.is_ok(), "{}", ast.err().unwrap());
}
