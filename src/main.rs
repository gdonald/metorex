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

    /// Evaluate code from command line, once per `-e` written
    #[arg(short = 'e')]
    execute: Vec<String>,

    /// Ruby --backtrace-limit (how many frames a report writes out)
    #[arg(long = "backtrace-limit", hide = true)]
    backtrace_limit: Option<i64>,

    /// Ruby -x (skip everything before the line naming ruby)
    #[arg(short = 'x', hide = true, action = clap::ArgAction::SetTrue)]
    strip_leading_text: bool,

    /// Ruby -c (check the syntax and report it, without running anything)
    #[arg(short = 'c', hide = true, action = clap::ArgAction::SetTrue)]
    check_syntax: bool,

    /// Ruby -C (the directory to work from)
    #[arg(short = 'C', hide = true)]
    working_directory: Option<String>,

    /// Ruby -X (the directory to work from, spelled the other way)
    #[arg(short = 'X', hide = true)]
    working_directory_x: Option<String>,

    /// Ignored flags for Ruby compatibility
    #[arg(long = "disable", hide = true)]
    _disable: Option<String>,

    /// Ignored: Ruby --enable=<feature>
    #[arg(long = "enable", hide = true)]
    _enable: Option<String>,

    /// Ignored: Ruby --enable-frozen-string-literal, spelled either way.
    /// Metorex has no in-place String mutation, so its literals already
    /// behave as frozen ones do.
    #[arg(
        long = "enable-frozen-string-literal",
        alias = "enable-frozen_string_literal",
        alias = "disable-frozen-string-literal",
        alias = "disable-frozen_string_literal",
        hide = true,
        action = clap::ArgAction::SetTrue
    )]
    _frozen_string_literal: bool,

    /// Ignored: Ruby --enable-gems
    #[arg(long = "enable-gems", hide = true, action = clap::ArgAction::SetTrue)]
    _enable_gems: bool,

    /// Ignored: Ruby --enable-did_you_mean, spelled either way
    #[arg(
        long = "enable-did_you_mean",
        alias = "enable-did-you-mean",
        hide = true,
        action = clap::ArgAction::SetTrue
    )]
    _enable_did_you_mean: bool,

    /// Ignored: Ruby --enable-rubyopt
    #[arg(long = "enable-rubyopt", hide = true, action = clap::ArgAction::SetTrue)]
    _enable_rubyopt: bool,

    /// Ignored: Ruby --enable-all
    #[arg(long = "enable-all", hide = true, action = clap::ArgAction::SetTrue)]
    _enable_all: bool,

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

    /// Ruby -p (the -n loop, printing the line after each pass)
    #[arg(short = 'p', hide = true, action = clap::ArgAction::SetTrue)]
    print_loop: bool,

    /// Ruby -a (split each line into $F, alongside -n or -p)
    #[arg(short = 'a', hide = true, action = clap::ArgAction::SetTrue)]
    split_lines: bool,

    /// Ruby -F (the pattern -a splits on)
    #[arg(short = 'F', hide = true)]
    field_separator: Option<String>,

    /// Ruby -0 (the octal code of the line separator $/ reads by)
    #[arg(short = '0', hide = true)]
    line_separator: Option<String>,

    /// Ruby -r (require library before executing)
    #[arg(short = 'r', hide = true)]
    require_libs: Vec<String>,

    /// Ruby -I (prepend to $LOAD_PATH)
    #[arg(short = 'I', hide = true)]
    include_paths: Vec<String>,

    /// Ruby -n (run the program once per input line, with the line in `$_`)
    #[arg(short = 'n', hide = true, action = clap::ArgAction::SetTrue)]
    each_line: bool,

    /// Ruby -w (turn on the warnings a plain run keeps quiet)
    #[arg(short = 'w', hide = true, action = clap::ArgAction::SetTrue)]
    warnings: bool,

    /// Ignored: Ruby -W (warning level)
    #[arg(short = 'W', hide = true)]
    _warning_level: Option<String>,

    /// Ruby --external-encoding (the encoding text read and written is in)
    #[arg(long = "external-encoding", hide = true)]
    external_encoding: Option<String>,

    /// Ruby --internal-encoding (the encoding text is carried into)
    #[arg(long = "internal-encoding", hide = true)]
    internal_encoding: Option<String>,

    /// Ruby -E (the external and internal encodings, written `external:internal`)
    #[arg(short = 'E', long = "encoding", hide = true)]
    encoding_pair: Option<String>,

    /// Ruby -K (the source encoding, which metorex reads as UTF-8 whatever
    /// this names)
    #[arg(short = 'K', hide = true)]
    source_encoding: Option<String>,

    /// Ruby -U (read text as UTF-8 whatever it was written in)
    #[arg(short = 'U', hide = true, action = clap::ArgAction::Count)]
    utf8_internal: u8,

    /// Ruby -d (turn on `$DEBUG`, and the warnings with it)
    #[arg(short = 'd', hide = true, action = clap::ArgAction::SetTrue)]
    ruby_debug: bool,
}

/// How the line-reading options were set, which decides what the loop around
/// the program does with each line.
struct LineLoop {
    /// `-n` or `-p`: run the program once for each line.
    each_line: bool,
    /// `-p`: write the line out after each pass.
    printing: bool,
    /// `-a`: split each line into `$F`.
    splitting: bool,
    /// `-F`: the pattern `-a` splits on, where one was named.
    field_separator: Option<String>,
}

/// Run a program, either once or, under `-n` and `-p`, once for each line of
/// standard input with that line in `$_`.
fn run_program(
    vm: &mut VirtualMachine,
    program: &[metorex::ast::Statement],
    reading: &LineLoop,
) -> Result<(), metorex::error::MetorexError> {
    if !reading.each_line {
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
        if reading.splitting {
            vm.set_split_fields(&line, reading.field_separator.as_deref());
        }
        vm.execute_program(program)?;
        if reading.printing {
            vm.print_current_line();
        }
    }
    Ok(())
}

/// How the line-reading flags were written on the command line.
fn line_loop_from(cli: &Cli) -> LineLoop {
    LineLoop {
        each_line: cli.each_line || cli.print_loop,
        printing: cli.print_loop,
        splitting: cli.split_lines,
        field_separator: cli.field_separator.clone(),
    }
}

/// A `-I` path as `$LOAD_PATH` holds it: written out from the working
/// directory when it was named relative to it, with the symlinks along the
/// way left alone.
fn load_path_entry(written: &str) -> String {
    let path = Path::new(written);
    if path.is_absolute() {
        return written.to_string();
    }
    match std::env::current_dir() {
        Ok(here) => here.join(path).to_string_lossy().into_owned(),
        Err(_) => written.to_string(),
    }
}

/// Report whether source parses, the way `ruby -c` does, and end there.
/// The Ruby inside a file that opens with something else. `asked` says the
/// `-x` option was written; a first line naming another interpreter asks for
/// the same reading on its own.
fn embedded_script(source: &str, asked: bool) -> Result<String, String> {
    let names_ruby = |line: &str| line.starts_with("#!") && line.contains("ruby");
    let first = source.lines().next().unwrap_or_default();
    let other_launcher = first.starts_with("#!") && !first.contains("ruby");
    if !asked && !other_launcher {
        return Ok(source.to_string());
    }
    let mut lines = source.lines();
    let mut held = String::new();
    let mut found = false;
    for line in lines.by_ref() {
        if names_ruby(line) {
            found = true;
            break;
        }
    }
    if !found {
        return Err("no Ruby script found in input".to_string());
    }
    for line in lines {
        held.push_str(line);
        held.push('\n');
    }
    Ok(held)
}

fn check_syntax(source: &str, name: &str) -> ! {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer.tokenize());
    match parser.parse() {
        Ok(_) => {
            println!("Syntax OK");
            process::exit(0)
        }
        Err(errors) => {
            for err in errors {
                eprintln!("{}: {} (SyntaxError)", name, err);
            }
            process::exit(1)
        }
    }
}

/// The encoding a `-K` letter stands for. Ruby reads the first letter alone
/// and leaves the source encoding alone when it names none.
fn source_encoding_letter(written: &str) -> Option<&'static str> {
    match written.chars().next() {
        Some('E') | Some('e') => Some("EUC-JP"),
        Some('S') | Some('s') => Some("Windows-31J"),
        Some('U') | Some('u') => Some("UTF-8"),
        Some('N') | Some('n') | Some('A') | Some('a') => Some("ASCII-8BIT"),
        _ => None,
    }
}

/// Apply the flags naming the encodings a program reads and writes text in:
/// `-E`, `-U`, `--external-encoding` and `--internal-encoding`.
fn apply_encoding_flags(vm: &mut VirtualMachine, cli: &Cli) {
    let mut external = cli.external_encoding.clone();
    let mut internal = cli.internal_encoding.clone();
    if let Some(written) = &cli.encoding_pair {
        let (before, after) = match written.split_once(':') {
            Some((before, after)) => (before, after),
            None => (written.as_str(), ""),
        };
        if !before.is_empty() {
            external = Some(before.to_string());
        }
        if after.contains(':') {
            eprintln!("metorex: extra argument for -E: {}", after);
            process::exit(1);
        }
        if !after.is_empty() {
            if cli.utf8_internal > 0 {
                eprintln!(
                    "metorex: -U and -E option ({}) conflicts (RuntimeError)",
                    after
                );
                process::exit(1);
            }
            internal = Some(after.to_string());
        }
    }
    if cli.utf8_internal > 0 {
        internal = Some("UTF-8".to_string());
    }
    // `-K` names the source encoding by a single letter rather than by name,
    // and the text a program reads and writes is in that encoding too unless
    // a later flag names another.
    if let Some(written) = &cli.source_encoding
        && let Some(named) = source_encoding_letter(written)
    {
        vm.set_default_encoding("__source__=", named);
        external.get_or_insert_with(|| named.to_string());
    }
    for (named, wanted) in [
        ("default_external=", &external),
        ("default_internal=", &internal),
    ] {
        if let Some(held) = wanted {
            vm.set_default_encoding(named, held);
        }
    }
}

/// Apply `-I` (include paths), `-r` (require libraries) and `-w` (warnings)
/// flags to a VM.
fn apply_cli_flags(vm: &mut VirtualMachine, cli: &Cli) {
    // Ruby's `-w` turns on the deprecation warnings a plain run keeps quiet,
    // and `-d` and `-v` turn them on the same way.
    let verbose = cli.warnings || cli.ruby_debug || cli.ruby_version;
    if verbose {
        vm.enable_warning_category("deprecated");
    }
    // Ruby reports which of the line-reading flags were written, under the
    // name of the flag itself.
    // `--backtrace-limit` is read by the reports and by
    // `Thread::Backtrace.limit`.
    if let Some(limit) = cli.backtrace_limit {
        vm.set_backtrace_limit(limit);
    }
    vm.set_flag_global("a", cli.split_lines);
    vm.set_flag_global("n", cli.each_line);
    vm.set_flag_global("p", cli.print_loop);
    vm.set_flag_global("w", cli.warnings);
    vm.set_flag_global("d", cli.ruby_debug);
    // `-w`, `-v` and `-d` are the switches `$VERBOSE` reports.
    if verbose {
        vm.set_verbose(true);
    }
    if cli.ruby_debug {
        vm.set_debug(true);
    }

    // `-0` names the line separator by its octal code, and a bare `-0` means
    // paragraph mode, which reads a blank line as the separator.
    if let Some(written) = &cli.line_separator {
        vm.set_line_separator(written);
    }
    apply_encoding_flags(vm, cli);
    for path in &cli.include_paths {
        vm.prepend_load_path(load_path_entry(path));
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
        .flat_map(metorex::split_short_flags)
        .collect();
    let cli = Cli::parse_from(arguments);

    // `-C` and `-X` both name the directory the rest of the run works from.
    if let Some(directory) = cli
        .working_directory
        .as_ref()
        .or(cli.working_directory_x.as_ref())
        && let Err(err) = std::env::set_current_dir(directory)
    {
        eprintln!("Error changing directory to '{}': {}", directory, err);
        process::exit(1);
    }

    // Ruby-compatible version output. `-v` also turns `$VERBOSE` on, so a
    // script or `-e` written alongside it still runs.
    if cli.ruby_version {
        println!("{}", metorex::ruby_description());
        if cli.file.is_empty() && cli.execute.is_empty() && cli.test.is_none() && !cli.repl {
            return;
        }
    }

    // Evaluate inline code. Ruby joins the `-e` strings with newlines.
    if !cli.execute.is_empty() {
        let code = cli.execute.join("\n");
        if cli.check_syntax {
            check_syntax(&code, "-e");
        }
        let code = &code;
        let lexer = Lexer::new(code);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = match parser.parse() {
            Ok(prog) => prog,
            Err(errors) => {
                // Ruby names the class of the failure it reports, which is
                // what a caller reading the output looks for.
                for err in errors {
                    eprintln!("-e: {} (SyntaxError)", err);
                }
                process::exit(1);
            }
        };
        let mut vm = VirtualMachine::new();
        apply_cli_flags(&mut vm, &cli);
        // Code given on the command line is named `-e`, which is what a
        // report and `__FILE__` say of it.
        vm.set_current_file(std::path::PathBuf::from("-e"));
        vm.set_script_path(
            std::path::PathBuf::from("-e"),
            std::path::PathBuf::from("-e"),
        );
        if let Err(err) = run_program(&mut vm, &program, &line_loop_from(&cli)) {
            finish_with_error(&mut vm, &err);
        }
        process::exit(vm.run_at_exit_handlers(0, None));
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

    // `-x` runs the script inside another file, which opens at the first
    // line naming ruby. A file whose first line names another interpreter is
    // read the same way, whether `-x` was written or not.
    let source = match embedded_script(&source, cli.strip_leading_text) {
        Ok(held) => held,
        Err(message) => {
            eprintln!("{}: {}", filename, message);
            process::exit(1);
        }
    };

    if cli.check_syntax {
        check_syntax(&source, filename);
    }

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

    if let Err(err) = run_program(&mut vm, &program, &line_loop_from(&cli)) {
        // `abort` and `exit` raise SystemExit: it ends the program with the
        // status it carries, having already reported anything it wanted to.
        if let metorex::error::MetorexError::UncaughtException {
            exception: exception @ metorex::object::Object::Exception(exc),
            ..
        } = &err
            && exc.borrow().is_system_exit()
        {
            let status = exc.borrow().status.unwrap_or(0) as i32;
            let ending = exception.clone();
            process::exit(vm.run_at_exit_handlers(status, Some(ending)));
        }
        // A SignalException nothing caught ends the program the way the
        // signal itself would have, so the exit status names the signal
        // rather than a plain failure.
        if let metorex::error::MetorexError::UncaughtException {
            exception: metorex::object::Object::Exception(exc),
            ..
        } = &err
            && let Some(number) = metorex::vm::signals::signal_number_of(&exc.borrow())
        {
            let ending = err_exception(&err);
            vm.run_at_exit_handlers(1, ending);
            // SAFETY: the default disposition is restored and the signal is
            // sent to this process, which then ends before returning.
            unsafe {
                libc::signal(number, libc::SIG_DFL);
                libc::raise(number);
            }
        }
        // The `at_exit` handlers run before the error is reported, so one
        // that calls `exit!` replaces both the report and the status.
        let status = vm.run_at_exit_handlers(1, err_exception(&err));
        report_error(&mut vm, &err);
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

/// End the program the way the failure calls for: `exit` leaves with the
/// status it carries, a signal nothing caught ends the process the way the
/// signal itself would, and everything else is reported and leaves with 1.
fn finish_with_error(vm: &mut VirtualMachine, err: &metorex::error::MetorexError) -> ! {
    if let metorex::error::MetorexError::UncaughtException {
        exception: exception @ metorex::object::Object::Exception(exc),
        ..
    } = err
        && exc.borrow().is_system_exit()
    {
        let status = exc.borrow().status.unwrap_or(0) as i32;
        let ending = exception.clone();
        process::exit(vm.run_at_exit_handlers(status, Some(ending)));
    }
    let signalled = match err {
        metorex::error::MetorexError::UncaughtException {
            exception: metorex::object::Object::Exception(exc),
            ..
        } => metorex::vm::signals::signal_number_of(&exc.borrow()),
        _ => None,
    };
    if let Some(number) = signalled {
        let ending = err_exception(err);
        vm.run_at_exit_handlers(1, ending);
        report_error(vm, err);
        // SAFETY: the default disposition is restored and the signal is sent
        // to this process, which then ends before returning.
        unsafe {
            libc::signal(number, libc::SIG_DFL);
            libc::raise(number);
        }
    }
    let status = vm.run_at_exit_handlers(1, err_exception(err));
    report_error(vm, err);
    process::exit(status);
}

/// Report a failure the way Ruby reports one: an exception nothing rescued
/// names where it was raised, what it says, and the class it is, and every
/// other failure is reported as itself.
fn report_error(vm: &mut VirtualMachine, err: &metorex::error::MetorexError) {
    if let Some(exception) = err_exception(err)
        && let Some(report) = vm.uncaught_report(&exception)
    {
        eprint!("{}", report);
        return;
    }
    eprintln!("Runtime error: {}", err);
}

/// The exception an error carries, which the `at_exit` handlers are told
/// about so one of them can report or replace it.
fn err_exception(err: &metorex::error::MetorexError) -> Option<metorex::object::Object> {
    match err {
        metorex::error::MetorexError::UncaughtException { exception, .. } => {
            Some(exception.clone())
        }
        _ => None,
    }
}
