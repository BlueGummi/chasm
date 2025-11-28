use chasm::tokens::TokenKind;
use chasm::parser::Parser;
use logos::Logos;

fn main() {
    // Test input demonstrating many features
    let input = r#"
@define SIZE 32
var x = 10
const y = 20

include "testfile.asm"

macro_rules! add2(reg1, reg2) {
    %tmp:
    nand %tmp, %tmp
}

for!(var i = 0; i < 4; i++)

{
    R1 = R1 + i
}

label:
.local_label:
::global_label:

"Hello\nWorld"
0xFF 0b1011 0o77 1234 'A' '\n'

    "#;

    println!("raw string");
    println!("{input}");
    println!("=== LEXING + PARSING ===");

    let mut parser = Parser::new(input);
    let ast = parser.parse();

    println!("AST:");
    for stmt in ast {
        println!("{:?}", stmt);
    }
}
