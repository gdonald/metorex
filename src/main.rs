// Metorex CLI
// Command-line interface for the Metorex programming language

use metorex::lexer::Lexer;
use metorex::parser::Parser;
use metorex::repl::Repl;
use metorex::vm::VirtualMachine;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    // No arguments or explicit REPL flag - start REPL mode
    if args.len() == 1 || (args.len() == 2 && (args[1] == "repl" || args[1] == "--repl")) {
        match Repl::new() {
            Ok(mut repl) => {
                if let Err(err) = repl.run() {
                    eprintln!("REPL error: {}", err);
                    process::exit(1);
                }
            }
            Err(err) => {
                eprintln!("Failed to initialize REPL: {}", err);
                process::exit(1);
            }
        }
        return;
    }

    // File execution mode
    let filename = &args[1];

    // Convert filename to absolute path
    let absolute_path = match fs::canonicalize(filename) {
        Ok(path) => path,
        Err(err) => {
            eprintln!("Error resolving file path '{}': {}", filename, err);
            process::exit(1);
        }
    };

    // Read the source file
    let source = match fs::read_to_string(&absolute_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", absolute_path.display(), err);
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

    // Set the current file path and mark it as loaded
    vm.set_current_file(absolute_path.clone());
    vm.mark_file_loaded(absolute_path);

    if let Err(err) = vm.execute_program(&program) {
        eprintln!("Runtime error: {}", err);
        process::exit(1);
    }
}
