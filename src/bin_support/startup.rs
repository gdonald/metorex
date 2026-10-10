// What the flags set up before the program runs.

use super::*;

/// The Ruby inside a file that opens with something else. `asked` says the
/// `-x` option was written; a first line naming another interpreter asks for
/// the same reading on its own.
pub(crate) fn embedded_script(source: &str, asked: bool) -> Result<String, String> {
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

/// Report whether source parses, the way `ruby -c` does, and end there.
pub(crate) fn check_syntax(source: &str, name: &str, cli: &Cli) -> ! {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer.tokenize());
    let errors = parser.parse().err().unwrap_or_default();
    let mut vm = VirtualMachine::new();
    match refused_program_messages(&mut vm, source, name, cli, &errors) {
        None => {
            println!("Syntax OK");
            process::exit(0)
        }
        Some(messages) => {
            for message in messages {
                let (first, rest) = message.split_once('\n').unwrap_or((&message, ""));
                eprintln!("{}: {} (SyntaxError)\n{}", program_name(), first, rest);
            }
            process::exit(1)
        }
    }
}

/// The switches that wrap a program in a loop, spelled as prism's options
/// name them.
fn wrapping_switches(cli: &Cli) -> String {
    let mut switches = String::new();
    if cli.split_lines {
        switches.push('a');
    }
    if cli.chomp_lines {
        switches.push('l');
    }
    if cli.each_line {
        switches.push('n');
    }
    if cli.print_loop {
        switches.push('p');
    }
    switches
}

/// The SyntaxError messages for a program MRI refuses, given the errors the
/// interpreter's own parser found, or None when the program reads.
pub(crate) fn refused_program_messages<E: ToString>(
    vm: &mut VirtualMachine,
    source: &str,
    named: &str,
    cli: &Cli,
    parse_errors: &[E],
) -> Option<Vec<String>> {
    if parse_errors.is_empty() && !vm.prism_refuses_program(source, &wrapping_switches(cli)) {
        return None;
    }
    if let Some(message) = vm.program_syntax_message(source, named, 1) {
        return Some(vec![message]);
    }
    if parse_errors.is_empty() {
        return None;
    }
    Some(parse_errors.iter().map(ToString::to_string).collect())
}

/// The encoding a `-K` letter stands for. Ruby reads the first letter alone
/// and leaves the source encoding alone when it names none.
pub(crate) fn source_encoding_letter(written: &str) -> Option<&'static str> {
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
pub(crate) fn apply_encoding_flags(vm: &mut VirtualMachine, cli: &Cli) {
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
            eprintln!(
                "{}: extra argument for {}: {}",
                program_name(),
                named,
                extra
            );
            process::exit(1);
        }
        if !after.is_empty() {
            if cli.utf8_internal > 0 {
                eprintln!(
                    "{}: -U and -E option ({}) conflicts (RuntimeError)",
                    program_name(),
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
pub(crate) fn apply_cli_flags(vm: &mut VirtualMachine, cli: &Cli) {
    // A feature turned on leaves a module behind under the name Ruby gives
    // it, which is what `defined?` finds.
    let features = Features::read(cli);
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
        vm.append_installed_load_path(path);
    }
    // Ruby loads RubyGems, error_highlight, did_you_mean and syntax_suggest
    // before the libraries `-r` names, once the load path did_you_mean
    // suggests features from is settled.
    let rubygems = features.gems.then_some("rubygems");
    let error_highlight = features.error_highlight.then_some("error_highlight");
    let did_you_mean = features.did_you_mean.then_some("did_you_mean");
    // Ruby loads syntax_suggest's hook through RubyGems, so turning gems off
    // leaves it out too. The rest of the library loads when a SyntaxError is
    // reported.
    let syntax_suggest =
        (features.gems && features.syntax_suggest).then_some("syntax_suggest/core_ext");
    for lib in rubygems
        .into_iter()
        .chain(error_highlight)
        .chain(did_you_mean)
        .chain(syntax_suggest)
        .chain(cli.require_libs.iter().map(String::as_str))
    {
        if let Err(err) = vm.require_startup_library(lib) {
            eprintln!("Runtime error: {}", err);
            process::exit(1);
        }
    }
}
