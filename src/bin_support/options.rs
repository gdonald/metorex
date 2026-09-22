// The command line as clap reads it, and the switches a program
// carries in its own arguments.

use super::*;

#[derive(ClapParser)]
#[command(name = "metorex", version, about = "The Metorex programming language")]
pub(crate) struct Cli {
    /// Source file to execute, followed by arguments for the script
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub(crate) file: Vec<String>,

    /// Dump the AST instead of executing
    #[arg(long)]
    pub(crate) ast: bool,

    /// Enable debug/verbose output
    #[arg(long)]
    pub(crate) debug: bool,

    /// Start the REPL
    #[arg(long)]
    pub(crate) repl: bool,

    /// Discover and run test files in a directory
    /// (matches *_test.rb, test_*.rb, *_spec.rb)
    #[arg(long)]
    pub(crate) test: Option<String>,

    /// Print Ruby-compatible version string
    #[arg(short = 'v', long = "verbose")]
    pub(crate) ruby_version: bool,

    /// Evaluate code from command line, once per `-e` written
    #[arg(short = 'e')]
    pub(crate) execute: Vec<String>,

    /// Ruby --backtrace-limit (how many frames a report writes out)
    #[arg(long = "backtrace-limit", hide = true)]
    pub(crate) backtrace_limit: Option<i64>,

    /// Ruby -x (skip everything before the line naming ruby)
    #[arg(short = 'x', hide = true, action = clap::ArgAction::SetTrue)]
    pub(crate) strip_leading_text: bool,

    /// Ruby -S (look the script up in RUBYPATH, then in PATH)
    #[arg(short = 'S', hide = true, action = clap::ArgAction::SetTrue)]
    pub(crate) script_search: bool,

    /// Ruby -l (chomp each line the loop reads, and write the separator back)
    #[arg(short = 'l', hide = true, action = clap::ArgAction::SetTrue)]
    pub(crate) chomp_lines: bool,

    /// Ruby -c (check the syntax and report it, without running anything)
    #[arg(short = 'c', hide = true, action = clap::ArgAction::SetTrue)]
    pub(crate) check_syntax: bool,

    /// Ruby -C (the directory to work from)
    #[arg(short = 'C', hide = true)]
    pub(crate) working_directory: Option<String>,

    /// Ruby -X (the directory to work from, spelled the other way)
    #[arg(short = 'X', hide = true)]
    pub(crate) working_directory_x: Option<String>,

    /// Ruby --disable=<feature>, which `--disable-<feature>` is rewritten to
    /// before the arguments are read.
    #[arg(long = "disable", hide = true)]
    pub(crate) disabled_features: Vec<String>,

    /// Ruby --enable=<feature>, which `--enable-<feature>` is rewritten to
    /// before the arguments are read.
    #[arg(long = "enable", hide = true)]
    pub(crate) enabled_features: Vec<String>,

    /// Ruby --debug-frozen-string-literal: a literal remembers where it was
    /// written, which is named when something tries to change it.
    #[arg(
        long = "debug-frozen-string-literal",
        alias = "debug-frozen_string_literal",
        hide = true,
        action = clap::ArgAction::SetTrue
    )]
    pub(crate) debug_frozen_string_literal: bool,

    /// Ruby -p (the -n loop, printing the line after each pass)
    #[arg(short = 'p', hide = true, action = clap::ArgAction::SetTrue)]
    pub(crate) print_loop: bool,

    /// Ruby -a (split each line into $F, alongside -n or -p)
    #[arg(short = 'a', hide = true, action = clap::ArgAction::SetTrue)]
    pub(crate) split_lines: bool,

    /// Ruby -F (the pattern -a splits on)
    #[arg(short = 'F', hide = true)]
    pub(crate) field_separator: Option<String>,

    /// Ruby -0 (the octal code of the line separator $/ reads by)
    #[arg(short = '0', hide = true)]
    pub(crate) line_separator: Option<String>,

    /// Ruby -r (require library before executing)
    #[arg(short = 'r', hide = true)]
    pub(crate) require_libs: Vec<String>,

    /// Ruby -I (prepend to $LOAD_PATH)
    #[arg(short = 'I', hide = true)]
    pub(crate) include_paths: Vec<String>,

    /// Ruby -n (run the program once per input line, with the line in `$_`)
    #[arg(short = 'n', hide = true, action = clap::ArgAction::SetTrue)]
    pub(crate) each_line: bool,

    /// Ruby -w (turn on the warnings a plain run keeps quiet)
    #[arg(short = 'w', hide = true, action = clap::ArgAction::SetTrue)]
    pub(crate) warnings: bool,

    /// Ruby -W: a level from 0 to 2, or a warning category to turn on or
    /// off, such as `-W:deprecated` and `-W:no-experimental`. Written bare it
    /// means the loudest level, and it may be written more than once.
    #[arg(short = 'W', hide = true, action = clap::ArgAction::Append)]
    pub(crate) warning_levels: Vec<String>,

    /// Ruby --external-encoding (the encoding text read and written is in)
    #[arg(long = "external-encoding", hide = true)]
    pub(crate) external_encoding: Option<String>,

    /// Ruby --internal-encoding (the encoding text is carried into)
    #[arg(long = "internal-encoding", hide = true)]
    pub(crate) internal_encoding: Option<String>,

    /// Ruby -E (the external and internal encodings, written `external:internal`)
    #[arg(short = 'E', long = "encoding", hide = true)]
    pub(crate) encoding_pair: Option<String>,

    /// Ruby -K (the source encoding, which metorex reads as UTF-8 whatever
    /// this names)
    #[arg(short = 'K', hide = true)]
    pub(crate) source_encoding: Option<String>,

    /// Ruby -U (read text as UTF-8 whatever it was written in)
    #[arg(short = 'U', hide = true, action = clap::ArgAction::Count)]
    pub(crate) utf8_internal: u8,

    /// Ruby -i (edit the files ARGF reads in place, keeping a backup under
    /// the extension written after the flag)
    #[arg(long = "in-place", hide = true)]
    pub(crate) in_place: Option<String>,

    /// Ruby -d (turn on `$DEBUG`, and the warnings with it)
    #[arg(short = 'd', hide = true, action = clap::ArgAction::SetTrue)]
    pub(crate) ruby_debug: bool,

    /// Ruby -s (a switch written among the program's own arguments becomes a
    /// global of the same name)
    #[arg(short = 's', hide = true, action = clap::ArgAction::SetTrue)]
    pub(crate) switch_globals: bool,
}

/// Open the main script's data section as the `DATA` constant, standing where
/// the text after `__END__` begins.
pub(crate) fn define_data_constant(
    vm: &mut metorex::vm::VirtualMachine,
    path: &Path,
    offset: usize,
) {
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
pub(crate) fn take_switch_globals(
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
pub(crate) struct LineLoop {
    /// `-n` or `-p`: run the program once for each line.
    pub(crate) each_line: bool,
    /// `-l`: take the separator off each line before the program sees it.
    pub(crate) chomping: bool,
    /// `-p`: write the line out after each pass.
    pub(crate) printing: bool,
    /// `-a`: split each line into `$F`.
    pub(crate) splitting: bool,
    /// `-F`: the pattern `-a` splits on, where one was named.
    pub(crate) field_separator: Option<String>,
}

/// The features `--enable` and `--disable` name, resolved from the command
/// line. Ruby turns gems, did_you_mean, and RUBYOPT on by default and leaves
/// frozen string literals to each source unless a flag says otherwise.
pub(crate) struct Features {
    pub(crate) gems: bool,
    pub(crate) did_you_mean: bool,
    pub(crate) frozen_string_literal: Option<bool>,
}

/// The feature names Ruby answers to, as `--enable` and `--disable` spell
/// them. `gem` is the singular Ruby also accepts for `gems`.
pub(crate) const FEATURE_NAMES: [&str; 5] = [
    "gems",
    "gem",
    "did_you_mean",
    "rubyopt",
    "frozen_string_literal",
];

/// A feature name with the spellings Ruby accepts folded together: a hyphen
/// reads as an underscore, so `--enable=frozen-string-literal` is the same
/// flag as `--enable=frozen_string_literal`.
pub(crate) fn feature_key(written: &str) -> String {
    written.trim().replace('-', "_")
}

impl Features {
    pub(crate) fn read(cli: &Cli) -> Self {
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
    pub(crate) fn apply(&mut self, written: &str, on: bool) {
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
pub(crate) fn spelled_out_flag(argument: String) -> Vec<String> {
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
pub(crate) fn rubyopt_is_read(arguments: &[String]) -> bool {
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
pub(crate) fn allowed_in_rubyopt(word: &str) -> bool {
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
pub(crate) fn rubyopt_arguments(arguments: &[String]) -> Vec<String> {
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
