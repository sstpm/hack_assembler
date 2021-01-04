use std::{fs::File, io::Write, path::PathBuf};

/* A minimal assembler for the Hack computer from Nand2Tetris.
    Takes an input file "abc.asm" in the symbolic Hack machine language and writes the corrosponding binary
    to "abc.hack"
    Implementation details and course: 
    https://www.coursera.org/learn/build-a-computer?s
    https://www.nand2tetris.org/project06 
    
   Project to be built in sections:
    1. Basic assembler that can translate files that do not use symbols
        - Parser module that breaks input file into pieces
            - Use structs/enums to collect the different commands that could be created?
                enum CommandKind {
                    A_Command
                    C_Command
                    L_Command // psuedo instruction for labels
                }
                struct ParsedLine {
                    has_more_commands: bool, // maybe not needed, depends on implimentation
                    command_type: CommandKind,
                    symbol: String
                    ...
                }
            - Split each line into commponent pieces, then translate each line into a ParsedLine
        - Code module that translates pieces into "binary instructions"
            - Go through each ParsedLine and translate the various fields into their binary parts
                - Should ParsedLine only have fields that can be translated then?
            - Create the final string of 0s and 1s for each line
            - Write each completed string to the output file
    2. Add the ability to handle symbols like variable names and jump labels
        - SymbolTable module that tracks symbols and labels with their memory addresses.
*/
use structopt::StructOpt;
#[derive(Debug, StructOpt)]
// StructOpt crate for command line argument parsing (only the path of input file for now).
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

/// Return the contents of the supplied file as a String, or panic.
fn get_file_contents(asm_file: &PathBuf) -> String {
    match std::fs::read_to_string(asm_file) {
        Ok(r) => r,
        Err(e) => panic!("Couldn't read from file! Error: {}", e),
    }
}

/// Write the supplied string to the file "filename".
// filename can be any string
// to_write can by any type that implements the lines() function.
fn write_binary_to_file(filename: String, to_write: String) -> std::io::Result<()> {
    let mut output_file = File::create(filename)?;
    for line in to_write.lines() {
        let properly_formatted_line = line.to_owned() + "\n";
        output_file.write(properly_formatted_line.as_bytes())?;
    }
    Ok(())
}

fn main() {
    let args = Cli::from_args();
    let contents = get_file_contents(&args.path);

    let mut output_filename: String = match args.path.file_stem() {
        Some(filename) => String::from(filename.to_str().unwrap()),
        None => panic!("We tried to get the filename from user's input, but one didn't exist!"),
    };
    output_filename.push_str(".hack");

    match write_binary_to_file(output_filename, contents) {
        Ok(_) => (),
        Err(e) => panic!("Failed to write output to file: {}", e),
    };
}
