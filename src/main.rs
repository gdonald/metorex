// Metorex CLI
// Command-line interface for the Metorex programming language

use metorex::lexer::Lexer;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: metorex <file.mx>");
        process::exit(1);
    }

    let filename = &args[1];

    // Read the source file
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    };

    // Tokenize
    let lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    // Parse
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(prog) => prog,
        Err(errors) => {
            eprintln!("Parse error(s):");
            for err in errors {
                eprintln!("  {}", err);
            }
            process::exit(1);
        }
    };

    // Execute
    let mut vm = VirtualMachine::new();
    if let Err(err) = vm.execute_program(&program) {
        eprintln!("Runtime error: {}", err);
        process::exit(1);
    }
}
