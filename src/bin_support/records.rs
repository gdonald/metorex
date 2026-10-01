// The records the line-reading loop reads, one per turn.

use super::*;

/// Run a program, either once or, under `-n` and `-p`, once for each line of
/// standard input with that line in `$_`.
pub(crate) fn run_program(
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
pub(crate) fn script_on_search_path(named: &str) -> Option<String> {
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
pub(crate) fn names_begin_block(statement: &metorex::ast::Statement) -> bool {
    matches!(statement, metorex::ast::Statement::BeginBlock { .. })
}

/// One record from standard input, read up to and including `separator`.
/// None once the input is spent.
/// The records the `-n` and `-p` loops read: the contents of every file named
/// on the command line in turn, and standard input when none was.
pub(crate) struct Records {
    /// The files still to be read, in the order they were named.
    pub(crate) waiting: Vec<String>,
    /// The bytes of the file being read, and how far through it the reading
    /// has got.
    pub(crate) held: Option<(Vec<u8>, usize)>,
    /// Whether standard input has been read, for a run that names no file.
    pub(crate) read_stdin: bool,
    /// Whether the command line named a file at all. A run that did reads
    /// those and nothing else.
    pub(crate) named_a_file: bool,
}

impl Records {
    pub(crate) fn of(files: Vec<String>) -> Self {
        Self {
            named_a_file: !files.is_empty(),
            waiting: files,
            held: None,
            read_stdin: false,
        }
    }

    /// The next record, or None once every file has been read through.
    pub(crate) fn next(&mut self, separator: &str) -> Option<String> {
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
    pub(crate) fn open_the_next(&mut self) -> Option<bool> {
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
pub(crate) fn line_loop_from(cli: &Cli) -> LineLoop {
    LineLoop {
        each_line: cli.each_line || cli.print_loop,
        chomping: cli.chomp_lines,
        printing: cli.print_loop,
        splitting: cli.split_lines,
        field_separator: cli.field_separator.clone(),
    }
}

/// Where a library installed alongside metorex is looked for: the `lib`
/// directory under the prefix the program was installed into, named by
/// version and then by platform the way Ruby names its own.
pub(crate) fn installed_library_paths() -> Vec<String> {
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

/// A `-I` path as `$LOAD_PATH` holds it: written out from the working
/// directory when it was named relative to it, with the symlinks along the
/// way left alone.
pub(crate) fn load_path_entry(written: &str) -> String {
    let path = Path::new(written);
    if path.is_absolute() {
        return written.to_string();
    }
    match std::env::current_dir() {
        Ok(here) => here.join(path).to_string_lossy().into_owned(),
        Err(_) => written.to_string(),
    }
}
