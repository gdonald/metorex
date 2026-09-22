// Running a file, recording it among the features already loaded.

use super::*;

impl VirtualMachine {
    /// Run a file, recording it in `$LOADED_FEATURES` and skipping it when it
    /// is already there. `load` passes false: it runs the file every time and
    /// leaves the feature list alone.
    pub fn execute_file_recording(
        &mut self,
        path: &std::path::Path,
        record: bool,
    ) -> Result<Object, MetorexError> {
        use crate::file_loader::{find_file_path, load_file_source, parse_file};

        // Find the actual file path (with extension auto-detection)
        let actual_path = find_file_path(path)?;

        // Canonicalize the file path to absolute path for proper deduplication
        let canonical_path = actual_path.canonicalize().map_err(|e| {
            MetorexError::runtime_error(
                format!(
                    "Failed to canonicalize file path '{}': {}",
                    actual_path.display(),
                    e
                ),
                SourceLocation::new(0, 0, 0),
            )
        })?;

        // Deduplicate against `$LOADED_FEATURES` ($"). This is the Ruby-side
        // source of truth so spec helpers that snapshot and restore $"
        // across tests really do reset the load. The internal
        // `loaded_files` set still tracks the same canonical paths as a
        // convenience for non-Ruby callers, but mirroring to $" comes first.
        // The spelling the file was named by, which is the path it reports
        // of itself and the one the feature list holds. Ruby lists the path
        // as it expands, with any symlink along it left as written, so a
        // program that takes its own entry back out of the list names the
        // same string it required.
        let named_path = std::path::absolute(&actual_path)
            .map(|absolute| without_dot_components(&absolute))
            .unwrap_or_else(|_| canonical_path.clone());
        // A file another thread is part-way through loading is waited for,
        // so the second thread reads what the load defined rather than a
        // half-built file, and answers that it loaded nothing itself.
        if record {
            self.wait_for_the_file_being_loaded(&canonical_path);
        }
        let listed_str = named_path.to_string_lossy().into_owned();
        let already_in_features = if let Some(Object::Array(arr)) = self.globals().get("\"") {
            arr.borrow()
                .iter()
                .any(|o| matches!(o, Object::String(s) if *s.as_str() == *listed_str))
        } else {
            false
        };
        if record && already_in_features {
            // A file that asks for itself while it is still running gets
            // nothing back, and a verbose run says so.
            let in_progress = self
                .loading_paths
                .iter()
                .any(|(held, _)| held == &canonical_path.to_string_lossy());
            if in_progress && matches!(self.globals().get("VERBOSE"), Some(Object::Bool(true))) {
                let message = format!(
                    "warning: loading in progress, circular require considered harmful - {}",
                    named_path.display()
                );
                self.emit_warning_to_stderr(&message, crate::lexer::Position::new(0, 0, 0));
            }
            // Keep loaded_files in sync — once $" lists the path, the
            // internal set should agree.
            self.mark_file_loaded(canonical_path.clone());
            return Ok(Object::Nil);
        }
        // A file dropped from `$"` is loaded again, so the internal set is
        // not allowed to keep saying otherwise.
        if record {
            self.loaded_files.remove(&canonical_path);
        }

        // Mark eagerly in both stores BEFORE executing so a self-recursive
        // require during the file's own body short-circuits.
        if record {
            self.mark_file_loaded(canonical_path.clone());
            if let Some(Object::Array(arr)) = self.globals().get("\"") {
                arr.borrow_mut().push(Object::string(listed_str.clone()));
            }
        }

        // A symlink keeps the name it was reached through, while everything
        // that loads the file works from the resolved path.
        self.reported_files
            .insert(canonical_path.clone(), named_path.clone());
        // Save the current file path to restore later
        let previous_file = self.current_file.clone();
        // Code in the file being executed belongs to that file, so a block
        // written in it names it however far from the load it is called.
        let previous_source_file = self
            .current_source_file
            .replace(named_path.display().to_string());

        // Load file source with error context
        let source = load_file_source(&canonical_path).map_err(|e| {
            // A file that is there but cannot be read is one the program
            // cannot load, which Ruby reports as a LoadError rather than as
            // an error of its own.
            let message = format!("cannot load such file -- {}", canonical_path.display());
            if e.message().contains("Permission denied") {
                return MetorexError::UncaughtException {
                    exception: crate::vm::errors::load_error(
                        message.clone(),
                        &canonical_path.to_string_lossy(),
                    ),
                    location: SourceLocation::new(0, 0, 0),
                    message,
                };
            }
            MetorexError::runtime_error(
                format!("Failed to load file '{}': {}", canonical_path.display(), e),
                SourceLocation::new(0, 0, 0),
            )
        })?;
        // The file names the encoding it is written in, which is what
        // `__ENCODING__` answers while it runs.
        let named_encoding = crate::lexer::named_source_encoding(&source);
        if let Some(named) = &named_encoding {
            self.file_encodings
                .insert(named_path.display().to_string(), named.clone());
        }
        let previous_source_encoding =
            std::mem::replace(&mut self.current_source_encoding, named_encoding);
        // A `frozen_string_literal` comment written after code names nothing,
        // and a verbose run says so.
        if crate::lexer::frozen_string_literal_after_a_token(&source)
            && matches!(self.globals().get("VERBOSE"), Some(Object::Bool(true)))
        {
            self.emit_warning_to_stderr(
                "warning: `frozen_string_literal' is ignored after any tokens",
                crate::lexer::Position::new(0, 0, 0),
            );
        }

        // Parse file with error context
        let statements = parse_file(&source, &canonical_path.to_string_lossy()).map_err(|e| {
            crate::vm::errors::syntax_error(
                format!("Failed to parse file '{}': {}", canonical_path.display(), e),
                Some(&canonical_path.to_string_lossy()),
                crate::lexer::Position::new(0, 0, 0),
            )
        })?;

        // A file read while measurement is on is counted from here on, which
        // is why the file that turns measurement on is never in the report.
        self.coverage_note_file(&named_path.display().to_string(), &source, &statements);

        // Update current file path for require_relative calls within this file
        self.set_current_file(canonical_path.clone());

        // Mark this path as actively executing so autoload can tell
        // "file is mid-load" apart from "file already loaded".
        let loading_on = self.running_thread();
        self.loading_paths
            .push((canonical_path.to_string_lossy().into_owned(), loading_on));

        // A loaded file's statements run at top level, whatever method the
        // load was called from, so `Module.nesting` inside it follows the
        // file's own class and module bodies rather than the caller's frame.
        let caller_nesting = std::mem::take(&mut self.method_nesting_stack);
        // A `def` in the loaded file belongs to Object, not to whatever class
        // or module body the load was called from. A wrapped load names a
        // module for them to land on instead.
        let caller_def_scope = std::mem::take(&mut self.def_scope_stack);
        let wrapped = self.load_wrap_module.take();
        if let Some(wrapper) = &wrapped {
            self.def_scope_stack.push(Rc::clone(wrapper));
        }
        // A wrapped load runs with a copy of the top-level main as `self`,
        // which is what the file sees and what its `to_s` reports. It is
        // bound once the file's own scope is open, below, so the scope the
        // load was called from keeps the self it had.
        let stand_in = wrapped
            .as_ref()
            .and_then(|wrapper| self.a_main_of_its_own(wrapper));
        // What the scope the load was called from had as `self`, put back
        // once the file has run.
        let previous_self = stand_in.as_ref().map(|_| self.environment().get("self"));
        // The same goes for `__callee__` and `__method__`: a loaded file runs
        // at top level, so neither reports the method that ran the load.
        let load_site = self.load_call_site.take();
        self.call_stack_push(
            crate::vm::CallFrame::boundary(format!("<file:{}>", named_path.display()))
                .with_location(
                    load_site
                        .as_ref()
                        .map(|(_, at)| format!("{}:{}", at.line, at.column)),
                )
                .with_source_file(load_site.map(|(file, _)| file)),
        );
        // A file loaded from another one keeps its own top-level locals, so a
        // name it binds there is gone once the file has run and never shows
        // up among the locals of the file that loaded it. The script the
        // program was started from is the top level itself, and its locals
        // stay where they are.
        let own_locals = previous_file.is_some();
        if own_locals {
            self.environment_mut().push_isolated_scope();
        }
        if let Some(held) = stand_in {
            self.environment_mut().define("self".to_string(), held);
        }
        let result = self.execute_program(&statements);
        if own_locals {
            self.environment_mut().pop_scope();
        }
        self.call_stack_pop();
        self.method_nesting_stack = caller_nesting;
        self.def_scope_stack = caller_def_scope;
        if let Some(saved) = previous_self {
            match saved {
                Some(receiver) => self.environment_mut().define("self".to_string(), receiver),
                None => self
                    .environment_mut()
                    .define("self".to_string(), Object::Nil),
            }
        }
        self.current_source_file = previous_source_file;
        self.loading_paths.pop();
        self.current_file = previous_file;
        self.current_source_encoding = previous_source_encoding;
        // A file that fails part-way through was never loaded, so the
        // feature list is left as it stood before the attempt.
        if record && result.is_err() {
            self.loaded_files.remove(&canonical_path);
            if let Some(Object::Array(arr)) = self.globals().get("\"") {
                arr.borrow_mut().retain(
                    |held| !matches!(held, Object::String(name) if *name.as_str() == *listed_str),
                );
            }
        }
        let value = result.map_err(|e| {
            let rendered = e.to_string();
            keep_exception(e, |_| {
                MetorexError::runtime_error(
                    format!(
                        "Error executing file '{}': {}",
                        canonical_path.display(),
                        rendered
                    ),
                    SourceLocation::new(0, 0, 0),
                )
            })
        })?;

        // Return the result or Nil if no return value
        Ok(value.unwrap_or(Object::Nil))
    }

    /// Run a library held in the binary, recording it so a second `require`
    /// of the same name does nothing.
    pub(crate) fn run_embedded_library(
        &mut self,
        name: &str,
        source: &str,
    ) -> Result<bool, MetorexError> {
        let feature = format!("<metorex>/{}.rb", name);
        let marker = std::path::PathBuf::from(&feature);
        if self.is_file_loaded(&marker) {
            return Ok(false);
        }
        self.mark_file_loaded(marker);
        let tokens = crate::lexer::Lexer::for_embedded_library(source).tokenize();
        let statements = crate::parser::Parser::new(tokens)
            .parse()
            .map_err(|errors| {
                let first = errors
                    .first()
                    .map(|error| error.to_string())
                    .unwrap_or_default();
                MetorexError::runtime_error(
                    format!("require('{}') reports {}", name, first),
                    SourceLocation::new(0, 0, 0),
                )
            })?;
        // A library metorex carries runs at top level however deep the
        // `require` was written, so a `module Foo` at its top names ::Foo
        // rather than nesting inside the class or module body it was asked
        // for from.
        let caller_def_scope = std::mem::take(&mut self.def_scope_stack);
        let caller_nesting = std::mem::take(&mut self.method_nesting_stack);
        let result = self.execute_program(&statements);
        self.method_nesting_stack = caller_nesting;
        self.def_scope_stack = caller_def_scope;
        result?;
        Ok(true)
    }

    /// The encoding a string literal in the source running now is written in,
    /// or None when that is the default the plain constructor already gives.
    pub(crate) fn source_literal_encoding(&self) -> Option<String> {
        let named = crate::vm::native_methods::string_methods::canonical_encoding_name(
            self.current_source_encoding.as_deref()?,
        );
        if named == crate::object::string_value::DEFAULT_ENCODING {
            return None;
        }
        Some(named)
    }
}
