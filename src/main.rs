// Metorex CLI
// Command-line interface for the Metorex programming language

use clap::Parser as ClapParser;
use metorex::lexer::Lexer;
use metorex::parser::Parser;
use metorex::repl::Repl;
use metorex::test_discovery;
use metorex::vm::VirtualMachine;
use std::fs;
use std::path::Path;
use std::process;

const RUBY_VERSION: &str = "4.0.2";
const METOREX_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(ClapParser)]
#[command(name = "metorex", version, about = "The Metorex programming language")]
struct Cli {
    /// Source file to execute, followed by arguments for the script
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    file: Vec<String>,

    /// Dump the AST instead of executing
    #[arg(long)]
    ast: bool,

    /// Enable debug/verbose output
    #[arg(long)]
    debug: bool,

    /// Start the REPL
    #[arg(long)]
    repl: bool,

    /// Discover and run test files in a directory
    /// (matches *_test.rb, test_*.rb, *_spec.rb)
    #[arg(long)]
    test: Option<String>,

    /// Print Ruby-compatible version string
    #[arg(short = 'v', long = "verbose")]
    ruby_version: bool,

    /// Evaluate code from command line
    #[arg(short = 'e')]
    execute: Option<String>,

    /// Ignored flags for Ruby compatibility
    #[arg(long = "disable", hide = true)]
    _disable: Option<String>,

    /// Ignored: Ruby --disable-gems
    #[arg(long = "disable-gems", hide = true, action = clap::ArgAction::SetTrue)]
    _disable_gems: bool,

    /// Ignored: Ruby --disable-did_you_mean, spelled either way
    #[arg(
        long = "disable-did_you_mean",
        alias = "disable-did-you-mean",
        hide = true,
        action = clap::ArgAction::SetTrue
    )]
    _disable_did_you_mean: bool,

    /// Ignored: Ruby --disable-rubyopt
    #[arg(long = "disable-rubyopt", hide = true, action = clap::ArgAction::SetTrue)]
    _disable_rubyopt: bool,

    /// Ignored: Ruby --disable-all
    #[arg(long = "disable-all", hide = true, action = clap::ArgAction::SetTrue)]
    _disable_all: bool,

    /// Ruby -r (require library before executing)
    #[arg(short = 'r', hide = true)]
    require_libs: Vec<String>,

    /// Ruby -I (prepend to $LOAD_PATH)
    #[arg(short = 'I', hide = true)]
    include_paths: Vec<String>,

    /// Ruby -n (run the program once per input line, with the line in `$_`)
    #[arg(short = 'n', hide = true, action = clap::ArgAction::SetTrue)]
    each_line: bool,

    /// Ignored: Ruby -w (warnings)
    #[arg(short = 'w', hide = true, action = clap::ArgAction::SetTrue)]
    _warnings: bool,

    /// Ignored: Ruby -W (warning level)
    #[arg(short = 'W', hide = true)]
    _warning_level: Option<String>,

    /// Ignored: Ruby -d (debug mode)
    #[arg(short = 'd', hide = true, action = clap::ArgAction::SetTrue)]
    _ruby_debug: bool,
}

/// Run a program, either once or, under `-n`, once for each line of standard
/// input with that line in `$_`.
fn run_program(
    vm: &mut VirtualMachine,
    program: &[metorex::ast::Statement],
    each_line: bool,
) -> Result<(), metorex::error::MetorexError> {
    if !each_line {
        vm.execute_program(program)?;
        return Ok(());
    }
    let mut line = String::new();
    loop {
        line.clear();
        match std::io::stdin().read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        vm.set_current_line(line.clone());
        vm.execute_program(program)?;
    }
    Ok(())
}

/// Apply `-I` (include paths) and `-r` (require libraries) flags to a VM.
fn apply_cli_flags(vm: &mut VirtualMachine, cli: &Cli) {
    for path in &cli.include_paths {
        vm.prepend_load_path(path.clone());
    }
    for lib in &cli.require_libs {
        if let Err(err) = vm.require_library(lib) {
            eprintln!("Runtime error: {}", err);
            process::exit(1);
        }
    }
}

fn main() {
    // Use a larger stack for deeply nested Ruby programs (mspec, etc.)
    let builder = std::thread::Builder::new().stack_size(64 * 1024 * 1024); // 64 MB
    let handler = builder
        .spawn(move || {
            real_main();
        })
        .expect("Failed to spawn main thread");
    handler.join().expect("Main thread panicked");
}

fn real_main() {
    // Ruby lets `-r`, `-I`, and `-W` carry their value attached (`-rfoo`),
    // which the argument parser only understands as two words.
    let arguments: Vec<String> = std::env::args()
        .flat_map(|argument| match argument.as_str() {
            attached
                if attached.len() > 2
                    && (attached.starts_with("-r")
                        || attached.starts_with("-I")
                        || attached.starts_with("-W")) =>
            {
                let (flag, value) = attached.split_at(2);
                vec![flag.to_string(), value.to_string()]
            }
            _ => vec![argument],
        })
        .collect();
    let cli = Cli::parse_from(arguments);

    // Ruby-compatible version output
    if cli.ruby_version {
        println!("ruby {} (metorex {})", RUBY_VERSION, METOREX_VERSION);
        return;
    }

    // Evaluate inline code
    if let Some(ref code) = cli.execute {
        let lexer = Lexer::new(code);
        let tokens = lexer.tokenize();
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
        let mut vm = VirtualMachine::new();
        apply_cli_flags(&mut vm, &cli);
        if let Err(err) = run_program(&mut vm, &program, cli.each_line) {
            eprintln!("Runtime error: {}", err);
            process::exit(1);
        }
        return;
    }

    // Test discovery mode
    if let Some(ref test_dir) = cli.test {
        let dir = Path::new(test_dir);
        match test_discovery::run_test_discovery(dir) {
            Ok(result) => {
                if !result.all_passed() {
                    process::exit(1);
                }
            }
            Err(err) => {
                eprintln!("Test discovery error: {}", err);
                process::exit(1);
            }
        }
        return;
    }

    // REPL mode: no file given or explicit --repl flag
    if cli.file.is_empty() || cli.repl {
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

    let filename = &cli.file[0];
    let script_args: Vec<String> = cli.file[1..].to_vec();

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

    // The VM comes up before the script is parsed so that a library named by
    // `-r` is loaded either way, and the `at_exit` handlers it registered run
    // even when the script itself does not parse.
    let mut vm = VirtualMachine::new();
    apply_cli_flags(&mut vm, &cli);

    // Parse
    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(prog) => prog,
        Err(errors) => {
            for err in errors {
                eprintln!("{}: {} (SyntaxError)", filename, err);
            }
            process::exit(vm.run_at_exit_handlers(1, None));
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

    // Set the current file path and mark it as loaded
    vm.set_current_file(absolute_path.clone());
    // `__FILE__` reports the path the script was named by on the command
    // line, while everything that resolves a path uses the canonical one.
    vm.set_script_path(absolute_path.clone(), std::path::PathBuf::from(filename));
    vm.mark_file_loaded(absolute_path);
    vm.set_argv(script_args);

    if let Err(err) = run_program(&mut vm, &program, cli.each_line) {
        // `abort` and `exit` raise SystemExit: it ends the program with the
        // status it carries, having already reported anything it wanted to.
        if let metorex::error::MetorexError::UncaughtException {
            exception: exception @ metorex::object::Object::Exception(exc),
            ..
        } = &err
            && exc.borrow().exception_type == "SystemExit"
        {
            let status = exc.borrow().status.unwrap_or(0) as i32;
            let ending = exception.clone();
            process::exit(vm.run_at_exit_handlers(status, Some(ending)));
        }
        // The `at_exit` handlers run before the error is reported, so one
        // that calls `exit!` replaces both the report and the status.
        let ending = match &err {
            metorex::error::MetorexError::UncaughtException { exception, .. } => {
                Some(exception.clone())
            }
            _ => None,
        };
        let status = vm.run_at_exit_handlers(1, ending);
        eprintln!("Runtime error: {}", err);
        if let metorex::error::MetorexError::RuntimeError { stack_trace, .. } = &err
            && !stack_trace.is_empty()
        {
            eprintln!("Stack trace:");
            for frame in stack_trace {
                eprintln!("{}", frame);
            }
        }
        process::exit(status);
    }
    process::exit(vm.run_at_exit_handlers(0, None));
}
