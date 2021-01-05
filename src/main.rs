use std::{collections::HashMap, fs::File, io::Write, path::PathBuf, todo};
use structopt::StructOpt;

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
#[derive(Debug, StructOpt)]
// StructOpt crate for command line argument parsing (only the path of input file for now).
struct Cli {
    #[structopt(parse(from_os_str))]
    path: std::path::PathBuf,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum CommandKind {
    ACommand,
    CCommand,
    LCommand,
    ICommand, // An invalid command, returned upon encountering a line that is not an instruction. Ignored completely.
}
#[derive(Debug, PartialEq, Eq, Hash)]
struct ParsedLine {
    // The "parts" of an instruction. CommandKind is either A, C, or L(abel).
    // Depending on the CommandKind, Dest, Comp, and Jump are either Some(value) or None
    // Examples:
    // @100
    // command_type: A_Command, symbol: Some("100"), dest: None, Comp: None, Jump: None
    // D = M + 1;JEQ
    // command_type: C_Command, symbol: None, dest: Some("D"), comp: Some("M + 1"), jump: Some("JEQ")
    // (ORANGE)
    // comamand_type: L_Command, symbol: Some("ORANGE"), ... : None
    command_type: CommandKind,
    symbol: Option<String>,
    dest: Option<String>,
    comp: Option<String>,
    jump: Option<String>,
}

fn parse_line(line: String) -> ParsedLine {
    // Assumes the line has been preprocessed already. That means all comments and whitespace have been removed and the
    // line is not a comment. Therefore, everything to parse is a valid Hack Assembly Language command of some form.

    let mut ct = CommandKind::ACommand;
    let mut sym: Option<String> = None;
    let mut des: Option<String> = None;
    let mut temp_comp: String = "".to_string(); // Our "builder" for comp as it may have multiple parts.
    let mut com: Option<String> = None;
    let mut jmp: Option<String> = None;

    let line_chars = line.chars();
    for char in line_chars {
        if char == '@' {
            // A instruction, take everything until EOL into the vector.
            ct = CommandKind::ACommand;
            sym = Some(line.chars().skip(1).collect::<String>());
            break; // we're done with this line.
        } else {
            ct = CommandKind::CCommand;
            if char == '=' {
                // There's an assignment of some for, ether ABC = B or AZY = B OPERATOR C
                let pos_of_eq = line.chars().position(|x| x == '=').unwrap();
                des = Some(line.chars().take(pos_of_eq).collect::<String>()); // Collect up to the = sign.
                &temp_comp.push(line.chars().nth(pos_of_eq + 1).unwrap());
            } else if char == '+' || char == '&' || char == '|' {
                // '-' has to be handled in its own case as it could be a unary operator.
                if line.contains("=") {
                    // The character is an operator and has two arguments, but the first argument was found above.
                    let pos_of_op = line
                        .chars()
                        .position(|x| x == '+' || x == '&' || x == '|')
                        .unwrap();
                    &temp_comp.push_str(
                        line.chars()
                            .skip(pos_of_op)
                            .take(2)
                            .collect::<String>()
                            .as_str(),
                    );
                } else {
                    // There's no destination but we still have some operation going on, eg D+M
                    // In this case, we won't have handled the first part of the comparison yet.
                    let pos_of_op = line
                        .chars()
                        .position(|x| x == '+' || x == '&' || x == '|')
                        .unwrap();
                    &temp_comp.push_str(
                        line.chars()
                            .skip(pos_of_op - 1)
                            .take(3)
                            .collect::<String>()
                            .as_str(),
                    );
                }
            } else if char == '-' || char == '!' {
                // -X, !X, D=-X, D=!X, D=M-X, M-X
                // - Could be a unary or binary operator, so we must check.
                let pos_of_op = line.chars().position(|x| x == '-' || x == '!').unwrap();
                if pos_of_op != 0 && char == '-' {
                    // Could still be unary but we have a destination
                    if line.chars().nth(pos_of_op - 1).unwrap() == '=' {
                        // Unary (char after '=' will be caught by = case)
                        &temp_comp.push(line.chars().nth(pos_of_op + 1).unwrap());
                    } else {
                        // Binary
                        if line.contains("=") {
                            // Don't capture the character after "=" again.
                            &temp_comp.push_str(
                                line.chars()
                                    .skip(pos_of_op)
                                    .take(2)
                                    .collect::<String>()
                                    .as_str(),
                            );
                        } else {
                            &temp_comp.push_str(
                                line.chars()
                                    .skip(pos_of_op - 1)
                                    .take(3)
                                    .collect::<String>()
                                    .as_str(),
                            );
                        }
                    }
                } else if pos_of_op == 0 {
                    // We're at the start and are - or !
                    &temp_comp.push_str(line.chars().take(2).collect::<String>().as_str());
                } else if char == '!' {
                    // We're a ! instruction and not at the beginning, eg D=!M; the '!' will be caught by '=' case.
                    &temp_comp.push(line.chars().nth(pos_of_op + 1).unwrap());
                }
            } else if char == ';' {
                // Character represents a JMP instruction is to follow
                if line.contains("=") {
                    // We'll have parsed the destination and the comp already; just take care of jump.
                    let i = line.chars().position(|x| x == ';').unwrap();
                    jmp = Some(line.chars().skip(i + 1).take(3).collect::<String>());
                } else {
                    // There is no destination, so we may or may not have parsed the comp.
                    if line.contains(|x| x == '+' || x == '-' || x == '&' || x == '|') {
                        // We'll have parsed the operation and operator. Just parse the Jump.
                        let i = line.chars().position(|x| x == ';').unwrap();
                        jmp = Some(line.chars().skip(i + 1).take(3).collect::<String>());
                    } else {
                        // There's no destination, and no operation; comp is just a register or memory.
                        // In this case, we need to store the comparison register as well as process the jump.
                        let i = line.chars().position(|x| x == ';').unwrap();
                        let com = line.chars().nth(i - 1).unwrap();
                        temp_comp.push(com);
                        jmp = Some(line.chars().skip(i + 1).take(3).collect::<String>());
                    }
                }
            }
        }
    }
    if !temp_comp.is_empty() {
        com = Some(temp_comp.to_string());
    }
    ParsedLine {
        command_type: ct,
        symbol: sym,
        dest: des,
        comp: com,
        jump: jmp,
    }
}

/// Return the contents of the supplied file as a String, or panic.
fn get_file_contents(asm_file: &PathBuf) -> String {
    match std::fs::read_to_string(asm_file) {
        Ok(r) => r,
        Err(e) => panic!("Couldn't read from file! Error: {}", e),
    }
}

fn preprocess_line(line: String) -> Option<String> {
    // Strip comments, whitespaces, and spaces between words from each line.
    /* Lines can be comments, empty, an instruction or label, or a combo of an instruction and comment.
    Examples:
    // File: add.asm
    // adds 100 to whatever's at register 300 and stores it at register 100
    @300
    D = M
    @100
    M = D + A

    The above should become:
    @300
    D=M
    @100
    M=D+A
     */
    let line_trimmed = line.trim();
    if line_trimmed.contains("//") {
        // We have a comment somewhere
        if line_trimmed.starts_with("//") {
            None
        } else {
            let comment_start_index = line_trimmed.chars().position(|x| x == '/').unwrap();
            let line_nocomment = line_trimmed
                .chars()
                .take(comment_start_index - 1)
                .collect::<String>();
            Some(line_nocomment.split_whitespace().collect::<String>())
        }
    } else {
        let potential_instruction = line.split_whitespace().collect::<String>();
        if potential_instruction.is_empty() {
            None
        } else {
            Some(potential_instruction)
        }
    }
}

fn translate(instruction: ParsedLine) -> String {
    /* Translate the parsed content into their corrosponding binary instructions.
    Each piece of ParsedLine (except LCommands, which are special) has one and only one binary representation.
    A instructions are just translated into the binary representation of their symbol, with a leading 0 eg:
        @2 --> 0000000000000010
    LCommands have a non-number value as their symbol and require consulting a symbol table we populate at some point.
        @dog --> | dog | 16 | --> 0000000000010000
    C instructions have multiple parts, one per field with three leading 1s:
        D=A+1;JMP ->  111accccccdddjjj where acccccc are determined by the comp, ddd by dest, and jjj by jump.
        ddd and jjj are 1->1 mapping of code -> bits, but
        acccccc can have different comps lead to the same binary value with the only difference being the a-bit
        However, we should still be able to store each individual comp as a key in a hash map between binary values:
        !M -> 1110001
        !A -> 0110001
        which are examples of instructions that differ only in the a-bit.
    */
    // Rust doesn't have a hash literal, so we have to collect them from something else. These don't work with `const`
    // because they use non-const functions, so here they are.
    let dest_map: HashMap<Option<String>, &str> = [
        (None, "000"),
        (Some("M".to_string()), "001"),
        (Some("D".to_string()), "010"),
        (Some("MD".to_string()), "011"),
        (Some("A".to_string()), "100"),
        (Some("AM".to_string()), "101"),
        (Some("AD".to_string()), "110"),
        (Some("AMD".to_string()), "111"),
    ]
    .iter()
    .cloned()
    .collect();

    let jump_map: HashMap<Option<String>, &str> = [
        (None, "000"),
        (Some("JGT".to_string()), "001"),
        (Some("JEQ".to_string()), "010"),
        (Some("JGE".to_string()), "011"),
        (Some("JLT".to_string()), "100"),
        (Some("JNE".to_string()), "101"),
        (Some("JLE".to_string()), "110"),
        (Some("JMP".to_string()), "111"),
    ]
    .iter()
    .cloned()
    .collect();

    let comp_map: HashMap<Option<String>, &str> = [
        (Some("0".to_string()), "0101010"),
        (Some("1".to_string()), "0111111"),
        (Some("-1".to_string()), "0111010"),
        (Some("D".to_string()), "0001100"),
        (Some("A".to_string()), "0110000"),
        (Some("M".to_string()), "1110000"),
        (Some("!D".to_string()), "0001101"),
        (Some("!A".to_string()), "0110001"),
        (Some("!M".to_string()), "1110001"),
        (Some("-D".to_string()), "0001111"),
        (Some("-A".to_string()), "0110011"),
        (Some("-M".to_string()), "1110011"),
        (Some("D+1".to_string()), "0011111"),
        (Some("A+1".to_string()), "0110111"),
        (Some("M+1".to_string()), "1110111"),
        (Some("D-1".to_string()), "0001110"),
        (Some("A-1".to_string()), "0110010"),
        (Some("M-1".to_string()), "1110010"),
        (Some("D+A".to_string()), "0000010"),
        (Some("D+M".to_string()), "1000010"),
        (Some("D-A".to_string()), "0010011"),
        (Some("D-M".to_string()), "1010011"),
        (Some("A-D".to_string()), "0000111"),
        (Some("M-D".to_string()), "1000111"),
        (Some("D&A".to_string()), "0000000"),
        (Some("D&M".to_string()), "1000000"),
        (Some("D|A".to_string()), "0010101"),
        (Some("D|M".to_string()), "1010101"),
    ]
    .iter()
    .cloned()
    .collect();

    if instruction.command_type == CommandKind::ACommand {
        let addr_as_binstr = format!("{:b}", instruction.symbol.unwrap().parse::<u16>().unwrap());
        let mut final_addr_binstr = addr_as_binstr.to_owned();
        // Pad the ouput with enough zeros to "become" a 16-bit word.
        for _ in 0..(16 - addr_as_binstr.len()) {
            final_addr_binstr.insert(0, '0');
        }
        final_addr_binstr
    } else if instruction.command_type == CommandKind::LCommand {
        // Will handle later. Just have to impliment a symbol table.
        todo!()
    } else {
        // We're a C instruction. The word is 111accccccdddjjj:
        let comp_bits = comp_map.get(&instruction.comp).unwrap();
        let dest_bits = dest_map.get(&instruction.dest).unwrap();
        let jump_bits = jump_map.get(&instruction.jump).unwrap();
        "111".to_string() + comp_bits + dest_bits + jump_bits
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
    output_filename.push_str("-assembled.hack");

    let mut translated_contents = String::new();
    for line in contents.lines() {
        // parsed_line is either a valid instruction or the ICommand, which we just do nothing with.
        let parsed_line = match preprocess_line(line.to_string()) {
            Some(instr) => parse_line(instr),
            None => ParsedLine {
                command_type: CommandKind::ICommand,
                dest: None,
                symbol: None,
                comp: None,
                jump: None,
            },
        };
        if parsed_line.command_type != CommandKind::ICommand {
            // We've got a valid instruction, so translate it and send it to the buffer to be written.
            // This "buffer" might be a long String with a bunch of \n in it, or it may actually be a WriteBuf
            // setup_for_writing(translate(parsed_line));
            translated_contents = if translated_contents.is_empty() {
                translated_contents + translate(parsed_line).as_str()
            } else {
                translated_contents + "\n" + translate(parsed_line).as_str()
            }
        }
    }

    match write_binary_to_file(output_filename, translated_contents.to_owned()) {
        Ok(_) => (),
        Err(e) => panic!("Failed to write output to file: {}", e),
    };
}
