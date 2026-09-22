// The globals a command line flag sets, and the line-reading loop
// that `-n` and `-p` add.

use super::*;

impl VirtualMachine {
    /// Set the current file being executed.
    /// Say what encoding the source running now is written in, which is what
    /// `__ENCODING__` answers.
    pub fn set_source_encoding(&mut self, named: Option<String>) {
        self.current_source_encoding = named;
    }

    pub fn set_current_file(&mut self, path: PathBuf) {
        self.current_file = Some(path);
    }

    /// The directories `$LOAD_PATH` names. An entry that is not a String is
    /// asked for its `to_path`, the way Ruby reads one.
    pub(crate) fn load_path_directories(&mut self) -> Vec<String> {
        let Some(Object::Array(entries)) = self.globals().get(":") else {
            return Vec::new();
        };
        let entries = entries.borrow().clone();
        let mut directories = Vec::with_capacity(entries.len());
        for entry in entries {
            match entry {
                Object::String(directory) => directories.push(directory.as_str().to_string()),
                other => {
                    let position = crate::lexer::Position::new(0, 0, 0);
                    if let Ok(path) = self.coerce_load_path(&other, position) {
                        directories.push(path);
                    }
                }
            }
        }
        directories
    }

    /// A path with a leading `~` expanded against HOME, which is read from
    /// `ENV` so a program that sets it there is followed.
    pub(crate) fn expand_home_path(&self, path: &str) -> String {
        let Some(rest) = path.strip_prefix("~/") else {
            return path.to_string();
        };
        let home = match self.globals().get("ENV") {
            Some(Object::Dict(entries)) => match entries.borrow().get("HOME") {
                Some(Object::String(home)) => Some(home.as_str().to_string()),
                _ => None,
            },
            _ => None,
        }
        .or_else(|| std::env::var("HOME").ok());
        match home {
            Some(home) => format!("{}/{}", home.trim_end_matches('/'), rest),
            None => path.to_string(),
        }
    }

    /// Set `$_`, the line `-n` last read.
    /// The files named on the command line, taken off ARGV the way Ruby's
    /// line-reading loop takes them.
    pub fn argv_paths(&mut self) -> Vec<String> {
        let Some(Object::Array(held)) = self.globals().get("ARGV") else {
            return Vec::new();
        };
        let named: Vec<String> = held
            .borrow()
            .iter()
            .filter_map(|entry| match entry {
                Object::String(text) => Some(text.as_str().to_string()),
                _ => None,
            })
            .collect();
        held.borrow_mut().clear();
        named
    }

    /// How many records the line-reading loop has read, which `$.` reports.
    pub fn set_records_read(&mut self, counted: i64) {
        self.globals_mut().set_variable(".", Object::Int(counted));
    }

    pub fn set_current_line(&mut self, line: String) {
        self.globals_mut().set_variable("_", Object::string(line));
    }

    /// Split the line just read into `$F`, which is what `-a` asks for. The
    /// pattern `-F` named does the splitting, and whitespace does it
    /// otherwise.
    pub fn set_split_fields(&mut self, line: &str, separator: Option<&str>) {
        let parts: Vec<Object> = match separator {
            Some(pattern) if !pattern.is_empty() => line
                .trim_end_matches('\n')
                .split(pattern)
                .map(Object::string)
                .collect(),
            _ => line.split_whitespace().map(Object::string).collect(),
        };
        self.globals_mut().set_variable("F", Object::array(parts));
    }

    /// Write out the line just read, which is what `-p` adds to the loop.
    pub fn print_current_line(&mut self) {
        // `print` writes whatever `$_` holds, which the program may have
        // replaced with something that is not a string.
        match self.globals().get("_") {
            Some(Object::Nil) | None => {}
            Some(Object::String(line)) => print!("{}", line.as_str()),
            Some(other) => print!("{other}"),
        }
    }

    /// Whether the run was asked to be verbose, which `$VERBOSE` reports.
    pub fn set_verbose(&mut self, verbose: bool) {
        self.globals_mut()
            .set_variable("VERBOSE", Object::Bool(verbose));
    }

    /// Say that the run reports nothing at all, which `-W0` asks for and
    /// `$VERBOSE` reports as nil rather than as false.
    pub fn set_verbose_nil(&mut self) {
        self.globals_mut().set_variable("VERBOSE", Object::Nil);
    }

    /// Set `Encoding.default_external` or `Encoding.default_internal` to the
    /// encoding a command line option named.
    pub fn set_default_encoding(&mut self, setter: &str, named: &str) {
        let Some(encoding) = self.globals().get("Encoding") else {
            return;
        };
        let _ = self.send_to_object(
            encoding,
            setter,
            vec![Object::string(named.to_string())],
            crate::lexer::Position::new(0, 0, 0),
        );
    }

    /// `-d`, which Ruby reports as `$DEBUG`.
    pub fn set_debug(&mut self, debug: bool) {
        self.globals_mut()
            .set_variable("DEBUG", Object::Bool(debug));
    }

    /// Say that `--debug-frozen-string-literal` was written, so every string
    /// literal remembers where it was written.
    pub fn set_debug_frozen_string_literal(&mut self, debug: bool) {
        self.debug_frozen_string_literal = debug;
    }

    /// Where a literal being built now is written, as `file:line`, for a run
    /// that asked to be told. Nothing is recorded otherwise.
    pub(crate) fn literal_birthplace(&self, position: crate::lexer::Position) -> Option<String> {
        if !self.debug_frozen_string_literal && !self.tracing_allocations {
            return None;
        }
        // The place is named the way `__FILE__` names it, so a script run by
        // a relative path is reported by that path.
        let file = self.current_source_file.clone().or_else(|| {
            self.reported_current_file()
                .map(|path| path.display().to_string())
        })?;
        Some(format!("{}:{}", file, position.line))
    }

    /// Record whether a command line flag was written, under the name Ruby
    /// reports it by: `-a` reads back as `$-a`.
    pub fn set_flag_global(&mut self, flag: &str, written: bool) {
        self.globals_mut()
            .set_variable(format!("-{flag}"), Object::Bool(written));
    }

    /// A global `-s` bound from a switch written among the program's own
    /// arguments.
    pub fn set_switch_global(&mut self, name: &str, value: Object) {
        self.globals_mut().set_variable(name.to_string(), value);
        self.seeded_global_names.insert(name.to_string());
    }

    /// The extension `-i` names a backup by, which ARGF reads to decide
    /// whether to edit the files it opens in place.
    pub fn set_in_place_extension(&mut self, extension: &str) {
        self.globals_mut()
            .set_variable("-i", Object::string(extension.to_string()));
    }

    /// The separator `$/` reads lines by, named by its octal code. A bare
    /// `-0` names the null byte, and `-00` asks for paragraph mode.
    pub fn set_line_separator(&mut self, written: &str) {
        let separator = if written.is_empty() {
            "\0".to_string()
        } else if written == "0" {
            "\n\n".to_string()
        } else {
            match u32::from_str_radix(written, 8) {
                Ok(code) => char::from_u32(code).map(String::from).unwrap_or_default(),
                Err(_) => "\n".to_string(),
            }
        };
        // The separator the flag named cannot be changed afterwards.
        let held = Object::string(separator);
        if let Object::String(text) = &held {
            text.freeze();
        }
        self.globals_mut().set_variable("/", held.clone());
        // Ruby reports the separator the flag named under the flag's own name.
        self.globals_mut().set_variable("-0", held);
    }

    /// The separator `$/` names now, which a program may have changed.
    pub fn line_separator(&self) -> String {
        match self.globals().get("/") {
            Some(Object::String(text)) => text.as_str().to_string(),
            _ => "\n".to_string(),
        }
    }

    /// `-l` chomps each line and writes the separator back out, which Ruby
    /// records as `$\` alongside the flag itself.
    pub fn set_chomping_lines(&mut self) {
        self.set_flag_global("l", true);
        let separator = Object::string(self.line_separator());
        self.globals_mut().set_variable("\\", separator);
    }

    /// The name the main script was run under, which is what `Process.argv0`
    /// and `__FILE__` report for it.
    pub(crate) fn script_name(&self) -> Option<String> {
        self.script_path
            .as_ref()
            .map(|(_, as_given)| as_given.display().to_string())
    }

    /// Record the main script's canonical path and the path it was named by.
    pub fn set_script_path(&mut self, canonical: PathBuf, as_given: PathBuf) {
        // `$0` names the program the way the command line spelled it, which
        // is what a script reports of itself.
        let named = Object::string(as_given.display().to_string());
        self.globals_mut().set_variable("0", named.clone());
        self.globals_mut().set_variable("PROGRAM_NAME", named);
        self.script_path = Some((canonical, as_given));
        // The scope in force here is the one the script's own top level runs
        // in, which is what TOPLEVEL_BINDING stands over.
        self.main_script_scope = Some(self.environment().current_scope());
    }

    /// The real path a backtrace entry names, resolved once when the file was
    /// loaded rather than when the entry is read, so it holds even after the
    /// name it was reached by is gone. A name no file answers to has none,
    /// which is what a location eval'd under a made-up filename reports.
    pub(crate) fn absolute_path_for(&self, named: &str) -> Option<String> {
        if let Some((canonical, as_given)) = &self.script_path
            && as_given.as_os_str() == named
        {
            return Some(canonical.display().to_string());
        }
        for (canonical, reported) in &self.reported_files {
            if reported.as_os_str() == named {
                return Some(canonical.display().to_string());
            }
        }
        std::fs::canonicalize(named)
            .ok()
            .map(|resolved| resolved.display().to_string())
    }

    /// Bring TOPLEVEL_BINDING up to date with the main script's own top-level
    /// scope. The binding shares the scope's cells, so a value assigned after
    /// it was read is the value it answers.
    pub(crate) fn refresh_toplevel_binding(&mut self) {
        let Some(scope) = self.main_script_scope.clone() else {
            return;
        };
        let Some(Object::Binding(held)) = self.globals().get("TOPLEVEL_BINDING") else {
            return;
        };
        let named: Vec<String> = scope.borrow().own_variable_names();
        for name in named {
            // The scope the script runs in also holds the builtins, which are
            // not locals of the program, and a constant is not one either.
            if !name.starts_with(|first: char| first == '_' || first.is_lowercase()) {
                continue;
            }
            if self.seeded_global_names.contains(&name) {
                continue;
            }
            let Some(cell) = scope.borrow().own_var_ref(&name) else {
                continue;
            };
            if crate::scope::names_a_definition(&cell.borrow()) {
                continue;
            }
            held.set(&name, cell);
        }
    }
}

/// An absolute path with `.` and `..` components resolved the way
/// `File.expand_path` resolves them, without touching the filesystem. A
/// symlink therefore keeps the name it was reached through.
pub(crate) fn without_dot_components(path: &std::path::Path) -> std::path::PathBuf {
    use std::path::Component;
    let mut built = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !built.pop() {
                    built.push(component.as_os_str());
                }
            }
            other => built.push(other.as_os_str()),
        }
    }
    built
}
