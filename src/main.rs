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

    /// Ruby -S (look the script up in RUBYPATH, then in PATH)
    #[arg(short = 'S', hide = true, action = clap::ArgAction::SetTrue)]
    script_search: bool,

    /// Ruby -l (chomp each line the loop reads, and write the separator back)
    #[arg(short = 'l', hide = true, action = clap::ArgAction::SetTrue)]
    chomp_lines: bool,

    /// Ruby -c (check the syntax and report it, without running anything)
    #[arg(short = 'c', hide = true, action = clap::ArgAction::SetTrue)]
    check_syntax: bool,

    /// Ruby -C (the directory to work from)
    #[arg(short = 'C', hide = true)]
    working_directory: Option<String>,

    /// Ruby -X (the directory to work from, spelled the other way)
    #[arg(short = 'X', hide = true)]
    working_directory_x: Option<String>,

    /// Ruby --disable=<feature>, which `--disable-<feature>` is rewritten to
    /// before the arguments are read.
    #[arg(long = "disable", hide = true)]
    disabled_features: Vec<String>,

    /// Ruby --enable=<feature>, which `--enable-<feature>` is rewritten to
    /// before the arguments are read.
    #[arg(long = "enable", hide = true)]
    enabled_features: Vec<String>,

    /// Ruby --debug-frozen-string-literal: a literal remembers where it was
    /// written, which is named when something tries to change it.
    #[arg(
        long = "debug-frozen-string-literal",
        alias = "debug-frozen_string_literal",
        hide = true,
        action = clap::ArgAction::SetTrue
    )]
    debug_frozen_string_literal: bool,

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

    /// Ruby -W: a level from 0 to 2, or a warning category to turn on or
    /// off, such as `-W:deprecated` and `-W:no-experimental`. Written bare it
    /// means the loudest level, and it may be written more than once.
    #[arg(short = 'W', hide = true, action = clap::ArgAction::Append)]
    warning_levels: Vec<String>,

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

    /// Ruby -i (edit the files ARGF reads in place, keeping a backup under
    /// the extension written after the flag)
    #[arg(long = "in-place", hide = true)]
    in_place: Option<String>,

    /// Ruby -d (turn on `$DEBUG`, and the warnings with it)
    #[arg(short = 'd', hide = true, action = clap::ArgAction::SetTrue)]
    ruby_debug: bool,

    /// Ruby -s (a switch written among the program's own arguments becomes a
    /// global of the same name)
    #[arg(short = 's', hide = true, action = clap::ArgAction::SetTrue)]
    switch_globals: bool,
}

/// Open the main script's data section as the `DATA` constant, standing where
/// the text after `__END__` begins.
fn define_data_constant(vm: &mut metorex::vm::VirtualMachine, path: &Path, offset: usize) {
    let setup = format!(
        "DATA = File.open({:?}, \"rb\")\nDATA.seek({})\n",
        path.display().to_string(),
        offset
    );
    let tokens = Lexer::new(&setup).tokenize();
    let Ok(program) = Parser::new(tokens).parse() else {
        return;
    };
    let _ = vm.execute_program(&program);
}

/// Take the leading switches off the program's arguments and bind each one as
/// a global, which is what `-s` asks for. A switch with no value binds true,
/// and the dashes in its name become underscores.
fn take_switch_globals(
    vm: &mut metorex::vm::VirtualMachine,
    arguments: Vec<String>,
) -> Vec<String> {
    let mut rest = Vec::new();
    let mut reading = true;
    for argument in arguments {
        if !reading {
            rest.push(argument);
            continue;
        }
        if argument == "--" {
            reading = false;
            continue;
        }
        let Some(body) = argument.strip_prefix('-').filter(|held| !held.is_empty()) else {
            reading = false;
            rest.push(argument);
            continue;
        };
        let (name, value) = match body.split_once('=') {
            Some((named, held)) => (named, metorex::object::Object::string(held.to_string())),
            None => (body, metorex::object::Object::Bool(true)),
        };
        vm.set_switch_global(&name.replace('-', "_"), value);
    }
    rest
}

/// How the line-reading options were set, which decides what the loop around
/// the program does with each line.
struct LineLoop {
    /// `-n` or `-p`: run the program once for each line.
    each_line: bool,
    /// `-l`: take the separator off each line before the program sees it.
    chomping: bool,
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
    // `BEGIN` runs before the loop starts, so a separator it names is in
    // force for the first record too.
    let opening: Vec<metorex::ast::Statement> = program
        .iter()
        .filter(|statement| names_begin_block(statement))
        .cloned()
        .collect();
    if !opening.is_empty() {
        vm.execute_program(&opening)?;
    }
    // Ruby reads the files named on the command line, and standard input
    // when none was. The names are taken off ARGV as they are opened.
    let named = vm.argv_paths();
    let mut records = Records::of(named);
    let mut counted = 0i64;
    loop {
        // The separator may have been changed by the program itself, as a
        // `BEGIN` block does, so it is read again for each record.
        let separator = vm.line_separator();
        let Some(line) = records.next(&separator) else {
            break;
        };
        counted += 1;
        vm.set_records_read(counted);
        let held = if reading.chomping {
            line.strip_suffix(&separator).unwrap_or(&line).to_string()
        } else {
            line
        };
        vm.set_current_line(held.clone());
        let line = held;
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

/// The script `-S` names, looked for in RUBYPATH and then in PATH. A name
/// that is already a path, or one nothing answers, is left as it was.
fn script_on_search_path(named: &str) -> Option<String> {
    if named.contains('/') {
        return None;
    }
    let searched = ["RUBYPATH", "PATH"];
    for variable in searched {
        let Ok(written) = std::env::var(variable) else {
            continue;
        };
        for directory in written.split(':').filter(|part| !part.is_empty()) {
            let candidate = Path::new(directory).join(named);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
    }
    None
}

/// Whether a statement is a `BEGIN { ... }` block, which runs once before
/// anything else the program does.
fn names_begin_block(statement: &metorex::ast::Statement) -> bool {
    matches!(statement, metorex::ast::Statement::BeginBlock { .. })
}

/// One record from standard input, read up to and including `separator`.
/// None once the input is spent.
/// The records the `-n` and `-p` loops read: the contents of every file named
/// on the command line in turn, and standard input when none was.
struct Records {
    /// The files still to be read, in the order they were named.
    waiting: Vec<String>,
    /// The bytes of the file being read, and how far through it the reading
    /// has got.
    held: Option<(Vec<u8>, usize)>,
    /// Whether standard input has been read, for a run that names no file.
    read_stdin: bool,
    /// Whether the command line named a file at all. A run that did reads
    /// those and nothing else.
    named_a_file: bool,
}

impl Records {
    fn of(files: Vec<String>) -> Self {
        Self {
            named_a_file: !files.is_empty(),
            waiting: files,
            held: None,
            read_stdin: false,
        }
    }

    /// The next record, or None once every file has been read through.
    fn next(&mut self, separator: &str) -> Option<String> {
        loop {
            if self.held.is_none() && !self.open_the_next()? {
                continue;
            }
            let (bytes, at) = self.held.as_mut()?;
            if *at >= bytes.len() {
                self.held = None;
                continue;
            }
            let ending = separator.as_bytes();
            let stop = if ending.is_empty() {
                bytes.len()
            } else {
                match bytes[*at..]
                    .windows(ending.len())
                    .position(|held| held == ending)
                {
                    Some(found) => *at + found + ending.len(),
                    None => bytes.len(),
                }
            };
            let record = String::from_utf8_lossy(&bytes[*at..stop]).into_owned();
            *at = stop;
            return Some(record);
        }
    }

    /// Open the next file, answering whether it holds anything to read. None
    /// once there is nothing left to open at all.
    fn open_the_next(&mut self) -> Option<bool> {
        use std::io::Read;
        if let Some(named) = self.waiting.first().cloned() {
            self.waiting.remove(0);
            match std::fs::read(&named) {
                Ok(bytes) => {
                    self.held = Some((bytes, 0));
                    return Some(true);
                }
                Err(trouble) => {
                    eprintln!("metorex: No such file or directory -- {named} ({trouble})");
                    return Some(false);
                }
            }
        }
        if self.read_stdin || self.named_a_file {
            return None;
        }
        self.read_stdin = true;
        let mut bytes = Vec::new();
        let _ = std::io::stdin().lock().read_to_end(&mut bytes);
        self.held = Some((bytes, 0));
        Some(true)
    }
}

/// How the line-reading flags were written on the command line.
fn line_loop_from(cli: &Cli) -> LineLoop {
    LineLoop {
        each_line: cli.each_line || cli.print_loop,
        chomping: cli.chomp_lines,
        printing: cli.print_loop,
        splitting: cli.split_lines,
        field_separator: cli.field_separator.clone(),
    }
}

/// A `-I` path as `$LOAD_PATH` holds it: written out from the working
/// directory when it was named relative to it, with the symlinks along the
/// way left alone.
/// Where a library installed alongside metorex is looked for: the `lib`
/// directory under the prefix the program was installed into, named by
/// version and then by platform the way Ruby names its own.
fn installed_library_paths() -> Vec<String> {
    let Ok(binary) = std::env::current_exe() else {
        return Vec::new();
    };
    let Some(prefix) = binary.parent().and_then(|held| held.parent()) else {
        return Vec::new();
    };
    let versioned = prefix
        .join("lib")
        .join("metorex")
        .join(metorex::reported_ruby_version());
    let platformed = versioned.join(metorex::reported_ruby_platform());
    vec![
        versioned.to_string_lossy().into_owned(),
        platformed.to_string_lossy().into_owned(),
    ]
}

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
        if let Some((_, extra)) = after.split_once(':') {
            // The complaint names the option the way it was written and the
            // part past the pair of encodings it takes.
            let named = if std::env::args().any(|held| held.starts_with("--encoding")) {
                "--encoding"
            } else {
                "-E"
            };
            eprintln!("metorex: extra argument for {}: {}", named, extra);
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
    // A feature turned on leaves a module behind under the name Ruby gives
    // it, which is what `defined?` finds.
    let features = Features::read(cli);
    if features.gems {
        vm.define_feature_module("Gem");
    }
    if features.did_you_mean {
        vm.define_feature_module("DidYouMean");
    }
    // Ruby's `-w` turns on the deprecation warnings a plain run keeps quiet,
    // and `-d` and `-v` turn them on the same way.
    // `-W` with a number says how loud a run is: 0 quiet, 1 the default, and
    // 2 as loud as `-w`. With a name it turns one category on or off.
    let mut named_categories: Vec<(String, bool)> = Vec::new();
    let mut warning_level = None;
    for written in &cli.warning_levels {
        match written.strip_prefix(':') {
            Some(category) => match category.strip_prefix("no-") {
                Some(category) => named_categories.push((category.to_string(), false)),
                None => named_categories.push((category.to_string(), true)),
            },
            None => warning_level = written.parse::<u8>().ok(),
        }
    }
    // Ruby's `--debug` is `-d` with the frozen-literal reporting turned on
    // alongside it, so everything `-d` settles reads it too.
    let ruby_debug = cli.ruby_debug || cli.debug;
    let verbose = cli.warnings || ruby_debug || cli.ruby_version || warning_level == Some(2);
    if verbose {
        vm.enable_warning_category("deprecated");
    }
    for (category, enabled) in named_categories {
        vm.set_warning_category(&category, enabled);
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
    if cli.chomp_lines {
        vm.set_chomping_lines();
    }
    // `-i` names the extension a backup is kept under, and ARGF reads it to
    // decide whether the files it opens are edited in place.
    if let Some(extension) = &cli.in_place {
        vm.set_in_place_extension(extension);
    }
    vm.set_flag_global("w", cli.warnings);
    vm.set_flag_global("d", ruby_debug);
    // `-w`, `-v` and `-d` are the switches `$VERBOSE` reports, and `-W0`
    // turns it off altogether.
    if verbose {
        vm.set_verbose(true);
    }
    if warning_level == Some(0) {
        vm.set_verbose_nil();
    }
    if ruby_debug {
        vm.set_debug(true);
    }
    if cli.debug_frozen_string_literal || cli.debug {
        vm.set_debug_frozen_string_literal(true);
    }

    // `-0` names the line separator by its octal code, and a bare `-0` means
    // paragraph mode, which reads a blank line as the separator.
    if let Some(written) = &cli.line_separator {
        vm.set_line_separator(written);
    }
    apply_encoding_flags(vm, cli);
    let mut opened: Vec<String> = cli.include_paths.clone();
    // RUBYLIB names directories of its own, which stand after everything a
    // `-I` asked for and before the rest of the path.
    if let Ok(written) = std::env::var("RUBYLIB") {
        opened.extend(
            written
                .split(':')
                .filter(|part| !part.is_empty())
                .map(|part| part.to_string()),
        );
    }
    for path in opened.iter().rev() {
        vm.prepend_load_path(load_path_entry(path));
    }
    // The directories a library installed alongside metorex sits in stand
    // after everything the command line and the environment named, the way
    // Ruby's own standard library directories do.
    for path in installed_library_paths() {
        vm.append_load_path(path);
    }
    for lib in &cli.require_libs {
        if let Err(err) = vm.require_startup_library(lib) {
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
    let mut arguments: Vec<String> = std::env::args()
        .flat_map(spelled_out_flag)
        .flat_map(metorex::split_short_flags)
        .collect();
    // RUBYOPT is read as though its words had been written on the command
    // line, ahead of what was, so a flag written there is overridden by the
    // same flag on the line itself.
    let from_rubyopt = rubyopt_arguments(&arguments);
    arguments.splice(1..1, from_rubyopt);
    let cli = Cli::parse_from(arguments);
    let features = Features::read(&cli);

    // Without a magic comment of its own, a source takes the setting the
    // program was started with. Said here rather than with the rest of the
    // flags, since a source is read before a VM is built for it.
    match features.frozen_string_literal {
        Some(true) => metorex::lexer::set_literal_default(metorex::lexer::LiteralDefault::Frozen),
        Some(false) => metorex::lexer::set_literal_default(metorex::lexer::LiteralDefault::Mutable),
        None => {}
    }
    // `-K` names the encoding every source is read as, so it has to be known
    // before any of them is read.
    if let Some(written) = &cli.source_encoding
        && let Some(named) = source_encoding_letter(written)
    {
        metorex::lexer::set_default_source_encoding(named);
    }

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
        vm.set_source_encoding(None);
        vm.set_script_path(
            std::path::PathBuf::from("-e"),
            std::path::PathBuf::from("-e"),
        );
        // Names written after the code are the program's arguments, which is
        // where ARGF looks for the files to read.
        let named = if cli.switch_globals {
            take_switch_globals(&mut vm, cli.file.clone())
        } else {
            cli.file.clone()
        };
        vm.set_argv(named);
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

    let named = &cli.file[0];
    let found = if cli.script_search {
        script_on_search_path(named)
    } else {
        None
    };
    let filename = found.as_ref().unwrap_or(named);
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
        // Bytes that spell no character at all are what Ruby reports as an
        // invalid multibyte char, naming the file and the first line.
        Err(err) if err.kind() == std::io::ErrorKind::InvalidData => {
            eprintln!(
                "{}:1: invalid multibyte char (UTF-8)",
                absolute_path.display()
            );
            process::exit(1);
        }
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

    // A line reading `__END__` closes the code, and the main script's data
    // after it is what the `DATA` constant reads.
    let (code, data_offset) = metorex::lexer::source_before_data_section(&source);
    let source = code.to_string();

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
    vm.set_source_encoding(metorex::lexer::named_source_encoding(&source));
    // `__FILE__` reports the path the script was named by on the command
    // line, while everything that resolves a path uses the canonical one.
    vm.set_script_path(absolute_path.clone(), std::path::PathBuf::from(filename));
    vm.mark_file_loaded(absolute_path.clone());
    let script_args = if cli.switch_globals {
        take_switch_globals(&mut vm, script_args)
    } else {
        script_args
    };
    vm.set_argv(script_args);
    if let Some(offset) = data_offset {
        define_data_constant(&mut vm, &absolute_path, offset);
    }

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

/// The features `--enable` and `--disable` name, resolved from the command
/// line. Ruby turns gems, did_you_mean, and RUBYOPT on by default and leaves
/// frozen string literals to each source unless a flag says otherwise.
struct Features {
    gems: bool,
    did_you_mean: bool,
    frozen_string_literal: Option<bool>,
}

/// The feature names Ruby answers to, as `--enable` and `--disable` spell
/// them. `gem` is the singular Ruby also accepts for `gems`.
const FEATURE_NAMES: [&str; 5] = [
    "gems",
    "gem",
    "did_you_mean",
    "rubyopt",
    "frozen_string_literal",
];

/// A feature name with the spellings Ruby accepts folded together: a hyphen
/// reads as an underscore, so `--enable=frozen-string-literal` is the same
/// flag as `--enable=frozen_string_literal`.
fn feature_key(written: &str) -> String {
    written.trim().replace('-', "_")
}

impl Features {
    fn read(cli: &Cli) -> Self {
        let mut held = Features {
            gems: true,
            did_you_mean: true,
            frozen_string_literal: None,
        };
        for written in &cli.enabled_features {
            held.apply(written, true);
        }
        for written in &cli.disabled_features {
            held.apply(written, false);
        }
        held
    }

    /// Turn every feature a `--enable` or `--disable` word names on or off.
    /// Ruby takes several at once, separated by commas.
    fn apply(&mut self, written: &str, on: bool) {
        for part in written.split(',') {
            match feature_key(part).as_str() {
                "all" => {
                    self.gems = on;
                    self.did_you_mean = on;
                    self.frozen_string_literal = Some(on);
                }
                "gems" | "gem" => self.gems = on,
                "did_you_mean" => self.did_you_mean = on,
                "frozen_string_literal" => self.frozen_string_literal = Some(on),
                // Whether RUBYOPT is read is settled before the arguments
                // are, so the name is taken here and nothing more is done.
                "rubyopt" => {}
                other => {
                    let flag = if on { "--enable" } else { "--disable" };
                    eprintln!(
                        "metorex: warning: unknown argument for {}: '{}'",
                        flag, other
                    );
                    eprintln!(
                        "metorex: warning: features are [{}].",
                        FEATURE_NAMES.join(", ")
                    );
                }
            }
        }
    }
}

/// `--enable-<feature>` and `--disable-<feature>` written as one word, read
/// as the two the argument parser understands. `--debug-frozen-string-literal`
/// is a flag of its own rather than a feature, so it stands as it is.
fn spelled_out_flag(argument: String) -> Vec<String> {
    // A bare `-W` is the loudest level, which the argument parser reads as a
    // level written out.
    if argument == "-W" {
        return vec!["-W2".to_string()];
    }
    for flag in ["--enable", "--disable"] {
        if let Some(named) = argument.strip_prefix(&format!("{flag}-")) {
            return vec![flag.to_string(), named.to_string()];
        }
    }
    vec![argument]
}

/// Whether RUBYOPT is read, which `--disable=rubyopt` and `--disable=all`
/// turn off. Settled by reading the arguments directly, since RUBYOPT's own
/// words join them before they are parsed.
fn rubyopt_is_read(arguments: &[String]) -> bool {
    let mut read = true;
    let mut words = arguments.iter();
    while let Some(word) = words.next() {
        let named = match word.split_once('=') {
            Some(("--disable", named)) => named.to_string(),
            _ if word == "--disable" => match words.next() {
                Some(next) => next.clone(),
                None => break,
            },
            _ => continue,
        };
        for part in named.split(',') {
            if matches!(feature_key(part).as_str(), "rubyopt" | "all") {
                read = false;
            }
        }
    }
    read
}

/// The switches Ruby lets RUBYOPT carry. Anything else there ends the run.
fn allowed_in_rubyopt(word: &str) -> bool {
    if let Some(long) = word.strip_prefix("--") {
        let name = long.split('=').next().unwrap_or("");
        return matches!(
            name,
            "debug"
                | "disable"
                | "enable"
                | "external-encoding"
                | "internal-encoding"
                | "verbose"
                | "backtrace-limit"
        );
    }
    match word.strip_prefix('-').and_then(|rest| rest.chars().next()) {
        Some(letter) => "dEIKrTUvwW".contains(letter),
        None => false,
    }
}

/// The arguments RUBYOPT stands for, read as though they had been written on
/// the command line. A switch Ruby does not allow there ends the run.
fn rubyopt_arguments(arguments: &[String]) -> Vec<String> {
    if !rubyopt_is_read(arguments) {
        return Vec::new();
    }
    let Ok(written) = std::env::var("RUBYOPT") else {
        return Vec::new();
    };
    let mut read = Vec::new();
    for word in written.split_whitespace() {
        if !allowed_in_rubyopt(word) {
            eprintln!(
                "metorex: invalid switch in RUBYOPT: {} (RuntimeError)",
                word
            );
            process::exit(1);
        }
        read.extend(
            spelled_out_flag(word.to_string())
                .into_iter()
                .flat_map(metorex::split_short_flags),
        );
    }
    read
}
