// Metorex CLI
// Command-line interface for the Metorex programming language

use clap::Parser as ClapParser;
use metorex::lexer::Lexer;
use metorex::parser::Parser;
use metorex::repl::Repl;
use metorex::vm::VirtualMachine;
use std::fs;
use std::process;

#[derive(ClapParser)]
#[command(name = "metorex", version, about = "The Metorex programming language")]
struct Cli {
    /// Source file to execute
    file: Option<String>,

    /// Dump the AST instead of executing
    #[arg(long)]
    ast: bool,

    /// Enable debug/verbose output
    #[arg(long)]
    debug: bool,

    /// Start the REPL
    #[arg(long)]
    repl: bool,
}

fn main() {
    let cli = Cli::parse();

    // REPL mode: no file given or explicit --repl flag
    if cli.file.is_none() || cli.repl {
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

    let filename = cli.file.as_ref().unwrap();

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

    if cli.debug {
        eprintln!("[debug] File: {}", absolute_path.display());
        eprintln!("[debug] Source length: {} bytes", source.len());
    }

    // Tokenize
    let lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    if cli.debug {
        eprintln!("[debug] Tokens: {}", tokens.len());
    }

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

    if cli.debug {
        eprintln!("[debug] Statements: {}", program.len());
    }

    // AST dump mode
    if cli.ast {
        for stmt in &program {
            println!("{:#?}", stmt);
        }
        return;
    }

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
