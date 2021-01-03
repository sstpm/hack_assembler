/* A minimal assembler for the Hack computer from Nand2Tetris.
    Takes an input file "abc.asm" in the symbolic Hack machine language and writes the corrosponding binary
    to "abc.hack"
    Implementation details and course: 
    https://www.coursera.org/learn/build-a-computer?s
    https://www.nand2tetris.org/project06 
    
   Project to be built in sections:
    1. Basic assembler that can translate files that do not use symbols
        - Parser module that breaks input file into pieces
        - Code module that translates pieces into "binary instructoins"
    2. Add the ability to handle symbols like variable names and jump labels
        - SymbolTable module that tracks symbols and labels with their memory addresses.
*/
fn main() {
    println!("Hello, world!");
}
