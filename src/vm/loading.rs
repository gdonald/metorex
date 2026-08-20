// File loading and `require` support for the VirtualMachine.
//
// This handles file path tracking, the `$LOAD_PATH` search, and the
// deduplicated `execute_file` pipeline used by both the CLI and the
// `require` / `load` builtins.

use std::path::PathBuf;

use super::core::VirtualMachine;
use crate::error::{MetorexError, SourceLocation};
use crate::object::Object;
use std::rc::Rc;

impl VirtualMachine {
    /// Set the current file being executed.
    pub fn set_current_file(&mut self, path: PathBuf) {
        self.current_file = Some(path);
    }

    /// Get the current file being executed.
    pub fn get_current_file(&self) -> Option<&PathBuf> {
        self.current_file.as_ref()
    }

    /// Mark a file as loaded in the registry.
    pub fn mark_file_loaded(&mut self, path: PathBuf) {
        self.loaded_files.insert(path);
    }

    /// Check if a file has already been loaded.
    pub fn is_file_loaded(&self, path: &PathBuf) -> bool {
        self.loaded_files.contains(path)
    }

    /// Drop the loaded-files entry for `path`. Used by autoload's error path
    /// so a subsequent constant access re-runs the file rather than skipping
    /// it via deduplication. Also drops a matching string from
    /// `$LOADED_FEATURES` so `autoload?` / `defined?` don't observe a stale
    /// "loaded" status after a failed load.
    pub fn unmark_file_loaded(&mut self, path: &PathBuf) {
        self.loaded_files.remove(path);
        let path_str = path.to_string_lossy().into_owned();
        if let Some(Object::Array(arr)) = self.globals().get("\"") {
            arr.borrow_mut()
                .retain(|o| !matches!(o, Object::String(s) if **s == path_str));
        }
    }

    /// Return the autoload path registered for `name` on `class_rc` (walking
    /// ancestors), unless that file appears in `$LOADED_FEATURES` ($") —
    /// in which case the autoload registration is silently dropped and
    /// `None` is returned. Per MRI: `autoload?` and `defined?` treat an
    /// autoload as cleared once its target file has been required directly.
    /// `$LOADED_FEATURES` (rather than the internal `loaded_files` set) is
    /// the source of truth here so spec helpers that snapshot/restore `$"`
    /// across tests really do reset autoload visibility. When the direct
    /// require finished without actually defining the constant, the name
    /// is also moved into the "unrealized autoload" registry so it stays
    /// in `Module#constants` (per MRI's behavior for the direct-require
    /// path, which differs from the const-access trigger path).
    pub(crate) fn effective_autoload(
        &mut self,
        class_rc: &Rc<crate::class::Class>,
        name: &str,
    ) -> Option<String> {
        let path = class_rc.lookup_autoload(name)?;
        // If this autoload is currently loading, behaviour depends on
        // which thread is asking. The loading thread itself observes
        // the autoload as cleared (Ruby treats the entry as already
        // satisfied from inside its own load). Other threads still see
        // the registered path — so a concurrent `autoload?` returns the
        // path while the loading thread's `autoload?` returns nil.
        let current = self
            .thread_current_stack
            .last()
            .cloned()
            .unwrap_or(Object::Nil);
        for (cls, n, loader) in &self.autoload_loading {
            if Rc::ptr_eq(cls, class_rc) && n == name {
                let same_thread = match (loader, &current) {
                    (Object::Nil, Object::Nil) => true,
                    (Object::Instance(a), Object::Instance(b)) => Rc::ptr_eq(a, b),
                    _ => false,
                };
                if same_thread {
                    return None;
                } else {
                    return Some(path);
                }
            }
        }
        if self.path_in_loaded_features(&path) {
            class_rc.remove_autoload(name);
            // Only the direct-require path keeps the cleared name in
            // `Module#constants` (via the unrealized list). When clearing
            // happens inside a const-access autoload trigger, the trigger
            // itself is responsible for the post-load bookkeeping.
            if self.autoload_const_access_depth == 0 && class_rc.get_class_var(name).is_none() {
                class_rc.mark_unrealized_autoload(name);
            }
            return None;
        }
        Some(path)
    }

    /// Read-only variant of `effective_autoload` for constant-presence
    /// checks (`const_defined?` / `const_get` lookup): whether `class_rc`
    /// itself has a pending autoload registration for `name`. Unlike
    /// `effective_autoload` it never clears the registration, checks only
    /// the receiver (callers walk ancestors themselves), and treats a
    /// same-thread in-progress load or an already-loaded file as not
    /// pending.
    pub(crate) fn autoload_pending(
        &mut self,
        class_rc: &Rc<crate::class::Class>,
        name: &str,
    ) -> bool {
        let Some(path) = class_rc.get_autoload(name) else {
            return false;
        };
        let current = self
            .thread_current_stack
            .last()
            .cloned()
            .unwrap_or(Object::Nil);
        for (cls, n, loader) in &self.autoload_loading {
            if Rc::ptr_eq(cls, class_rc) && n == name {
                let same_thread = match (loader, &current) {
                    (Object::Nil, Object::Nil) => true,
                    (Object::Instance(a), Object::Instance(b)) => Rc::ptr_eq(a, b),
                    _ => false,
                };
                return !same_thread;
            }
        }
        !self.path_in_loaded_features(&path)
    }

    /// Whether `path` (after canonicalization) is currently listed in
    /// `$LOADED_FEATURES`. Used by `effective_autoload` and by the
    /// `autoload?` natives.
    pub(crate) fn path_in_loaded_features(&self, path: &str) -> bool {
        let abs = std::path::Path::new(path);
        let canonical = match abs.canonicalize() {
            Ok(p) => p.to_string_lossy().into_owned(),
            Err(_) => return false,
        };
        if let Some(Object::Array(arr)) = self.globals().get("\"") {
            arr.borrow()
                .iter()
                .any(|o| matches!(o, Object::String(s) if **s == canonical))
        } else {
            false
        }
    }

    /// If `class_rc` (or an ancestor) has an autoload registration for
    /// `name`, fire it: load the file, drop the registration on success, and
    /// return the now-defined constant. On load failure, restore the
    /// registration and unmark the file so a retry can re-execute. Returns
    /// `Ok(None)` when there is no autoload and the caller should fall
    /// through to a NameError or other fallback.
    pub(crate) fn try_autoload_constant(
        &mut self,
        class_rc: &Rc<crate::class::Class>,
        name: &str,
    ) -> Result<Option<Object>, MetorexError> {
        let Some(path) = class_rc.lookup_autoload(name) else {
            return Ok(None);
        };
        // If this autoload is currently loading on this thread, just
        // return whatever class_var the load has deposited so far —
        // keeping the entry alive for other threads' visibility. The
        // `try_autoload_constant` invocation that started the load is
        // responsible for cleaning up after the load completes.
        let already_loading = self
            .autoload_loading
            .iter()
            .any(|(cls, n, _)| Rc::ptr_eq(cls, class_rc) && n == name);
        if already_loading {
            return Ok(class_rc.get_class_var(name));
        }
        // If the registered file has already been loaded *and* it
        // deposited a value for this constant, the autoload is
        // satisfied: drop the entry and return the stored value.
        //
        // If the constant is still missing, two cases apply:
        //  - The file is *currently* mid-execution (a class body inside
        //    that file is reopening this scope and asking for the
        //    constant before the assignment line has run). Skip
        //    re-loading — the load that's already running will get
        //    there. Returning None lets the caller fall through to a
        //    NameError, which the in-flight body either ignores or
        //    handles (typical Ruby autoload-during-require pattern).
        //  - The file finished and didn't define this constant. That
        //    means several autoloads point at the same path and the
        //    finished load only defined sibling names. Re-load so the
        //    body re-runs and defines this one too. Defeat
        //    `execute_file`'s dedup by clearing the path from `$"` and
        //    from the internal `loaded_files` set.
        let mut reloading = false;
        if self.path_in_loaded_features(&path) {
            if let Some(val) = class_rc.get_class_var(name) {
                class_rc.remove_autoload(name);
                return Ok(Some(val));
            }
            // Top-level `module Foo` / `class Foo` lands on globals
            // rather than on Object's class_vars in our model. When
            // the autoload was registered on Object, peek there too
            // before deciding to re-load.
            if class_rc.name() == "Object"
                && let Some(val) = self.globals().get(name)
            {
                class_rc.remove_autoload(name);
                return Ok(Some(val));
            }
            let canonical_str = std::path::Path::new(&path)
                .canonicalize()
                .ok()
                .map(|p| p.to_string_lossy().into_owned());
            let in_progress = canonical_str
                .as_ref()
                .is_some_and(|c| self.loading_paths.iter().any(|p| p == c))
                || self.loading_paths.iter().any(|p| p == &path);
            if in_progress {
                return Ok(None);
            }
            if let Some(canonical) = canonical_str {
                self.unmark_file_loaded(&PathBuf::from(canonical));
            }
            if let Some(Object::Array(arr)) = self.globals().get("\"") {
                arr.borrow_mut()
                    .retain(|o| !matches!(o, Object::String(s) if **s == path));
            }
            reloading = true;
        }
        // Don't remove the registration up-front. MRI keeps the constant
        // visible in `Module#constants` for the duration of the load; if
        // the load body asks `defined?` or `autoload?` about the same
        // name, `effective_autoload` (consulted by those methods) will
        // see the file in `$LOADED_FEATURES` and clear the entry on its
        // own.
        let p = std::path::Path::new(&path);
        // Autoload load runs at top level (like `require`): a `module Foo`
        // at the file's top should reopen ::Foo, not nest inside the
        // currently-executing class/module body. Save and clear the
        // def-scope stack for the duration of the load so lexical-nesting
        // logic in execute_class_def / execute_module_def sees an empty
        // outer scope.
        let saved_def_scope = std::mem::take(&mut self.def_scope_stack);
        self.autoload_const_access_depth += 1;
        if reloading {
            self.autoload_reload_depth += 1;
        }
        let loader_thread = self
            .thread_current_stack
            .last()
            .cloned()
            .unwrap_or(Object::Nil);
        self.autoload_loading
            .push((Rc::clone(class_rc), name.to_string(), loader_thread));
        // MRI dispatches autoload's load through `main.require(path)` so
        // singleton-method mocks on `main` (mspec's `main.should_receive
        // (:require)`) catch the call. Try that path first; fall back to
        // the internal loader when `main` doesn't have a `require` method
        // (e.g. when the singleton method has been removed or never set
        // up via Kernel inclusion).
        let main_require = self
            .globals()
            .get("TOPLEVEL_BINDING")
            .and_then(|tb| match tb {
                Object::Binding(b) => b.receiver.clone(),
                _ => None,
            })
            .and_then(|main| {
                self.lookup_method(&main, "require")
                    .map(|(cls, m)| (main, cls, m))
            });
        let load_result: Result<(), MetorexError> = if let Some((main, cls, method)) = main_require
        {
            self.invoke_method(
                cls,
                method,
                main,
                vec![Object::String(Rc::new(path.clone()))],
                crate::lexer::Position::new(0, 0, 0),
            )
            .map(|_| ())
        } else if p.is_absolute() {
            self.execute_file(p).map(|_| ())
        } else {
            self.require_library(&path)
        };
        self.autoload_const_access_depth -= 1;
        if reloading {
            self.autoload_reload_depth -= 1;
        }
        self.autoload_loading
            .retain(|(cls, n, _)| !(Rc::ptr_eq(cls, class_rc) && n == name));
        self.def_scope_stack = saved_def_scope;
        if let Err(err) = load_result {
            class_rc.set_autoload(name, &path);
            if let Ok(canonical) = p.canonicalize() {
                self.unmark_file_loaded(&canonical);
            }
            // Translate "file missing" runtime errors into a Ruby-level
            // LoadError so `rescue LoadError` works the way MRI's autoload
            // does. Autoload's other errors (RuntimeError, NameError, etc.)
            // pass through unchanged.
            let msg = err.message().to_string();
            if msg.contains("File not found") || msg.contains("cannot load such file") {
                let exc = Object::exception("LoadError", msg.clone());
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: SourceLocation::new(0, 0, 0),
                    message: msg,
                });
            }
            return Err(err);
        }
        // Const-access path: load completed. Drop the registration now —
        // either the constant was defined (a real class_var assignment
        // already cleared the autoload via set_class_var, but a no-op
        // remove is harmless) or it wasn't (MRI fully drops the name from
        // `#constants` for the const-access trigger, distinct from the
        // direct-require path which retains an unrealized marker).
        class_rc.remove_autoload(name);
        let mut value = class_rc.get_class_var(name);
        // Top-level `module Foo` / `class Foo` lands on globals rather
        // than on Object's class_vars in our model. When the autoload
        // was registered on Object, fall back to globals so the lookup
        // surfaces the freshly-defined constant.
        if value.is_none() && class_rc.name() == "Object" {
            value = self.globals().get(name);
        }
        // MRI's verbose-mode warning when an autoload-triggered file
        // completed but didn't define the named constant. Only emitted
        // under `$VERBOSE = true`; routed through `$stderr` so the
        // mspec `complain` matcher (which swaps `$stderr` for an
        // `IOStub`) captures it.
        if value.is_none() && matches!(self.globals().get("VERBOSE"), Some(Object::Bool(true))) {
            let mod_name = if class_rc.name().is_empty() {
                "main".to_string()
            } else {
                class_rc.ruby_name()
            };
            let msg = format!(
                "Expected {} to define {}::{} but it didn't",
                path, mod_name, name,
            );
            self.emit_warning_to_stderr(&msg, crate::lexer::Position::new(0, 0, 0));
        }
        Ok(value)
    }

    /// Prepend a path to the `$LOAD_PATH` (`$:`) global array.
    pub fn prepend_load_path(&mut self, path: String) {
        if let Some(Object::Array(arr)) = self.globals.get(":") {
            arr.borrow_mut().insert(0, Object::String(Rc::new(path)));
        }
    }

    /// Require a library by name, searching `$LOAD_PATH` just like the `require` builtin.
    pub fn require_library(&mut self, name: &str) -> Result<(), MetorexError> {
        let load_path = self.globals().get(":").unwrap_or(Object::Nil);
        let search_dirs: Vec<String> = match &load_path {
            Object::Array(arr) => arr
                .borrow()
                .iter()
                .filter_map(|obj| match obj {
                    Object::String(s) => Some(s.as_ref().clone()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };

        let mut found_path = None;
        for dir in &search_dirs {
            let base = std::path::PathBuf::from(dir);
            // Try `.rb` first so a matching .rb file wins over a sibling directory.
            let candidates = [base.join(format!("{}.rb", name)), base.join(name)];
            for candidate in &candidates {
                if candidate.is_file() {
                    found_path = Some(candidate.clone());
                    break;
                }
            }
            if found_path.is_some() {
                break;
            }
        }

        let resolved = found_path.ok_or_else(|| {
            MetorexError::runtime_error(
                format!(
                    "cannot load such file -- {} (searched in $LOAD_PATH: {:?})",
                    name, search_dirs
                ),
                SourceLocation::new(0, 0, 0),
            )
        })?;

        self.execute_file(&resolved).map_err(|e| {
            MetorexError::runtime_error(
                format!("require('{}') — {}", name, e.message()),
                SourceLocation::new(0, 0, 0),
            )
        })?;

        Ok(())
    }

    /// Execute a file with automatic deduplication and path tracking.
    ///
    /// This method loads and executes a file, handling:
    /// - File deduplication (files are only executed once)
    /// - Current file path tracking (for require_relative)
    /// - Automatic path canonicalization
    /// - Proper restoration of the previous current file
    pub fn execute_file(&mut self, path: &std::path::Path) -> Result<Object, MetorexError> {
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
        let canonical_str = canonical_path.to_string_lossy().into_owned();
        let already_in_features = if let Some(Object::Array(arr)) = self.globals().get("\"") {
            arr.borrow()
                .iter()
                .any(|o| matches!(o, Object::String(s) if **s == canonical_str))
        } else {
            false
        };
        if already_in_features {
            // Keep loaded_files in sync — once $" lists the path, the
            // internal set should agree.
            self.mark_file_loaded(canonical_path.clone());
            return Ok(Object::Nil);
        }

        // Mark eagerly in both stores BEFORE executing so a self-recursive
        // require during the file's own body short-circuits.
        self.mark_file_loaded(canonical_path.clone());
        if let Some(Object::Array(arr)) = self.globals().get("\"") {
            arr.borrow_mut()
                .push(Object::String(Rc::new(canonical_str.clone())));
        }

        // Save the current file path to restore later
        let previous_file = self.current_file.clone();

        // Load file source with error context
        let source = load_file_source(&canonical_path).map_err(|e| {
            MetorexError::runtime_error(
                format!("Failed to load file '{}': {}", canonical_path.display(), e),
                SourceLocation::new(0, 0, 0),
            )
        })?;

        // Parse file with error context
        let statements = parse_file(&source, &canonical_path.to_string_lossy()).map_err(|e| {
            MetorexError::runtime_error(
                format!("Failed to parse file '{}': {}", canonical_path.display(), e),
                SourceLocation::new(0, 0, 0),
            )
        })?;

        // Update current file path for require_relative calls within this file
        self.set_current_file(canonical_path.clone());

        // Mark this path as actively executing so autoload can tell
        // "file is mid-load" apart from "file already loaded".
        self.loading_paths.push(canonical_str.clone());

        // A loaded file's statements run at top level, whatever method the
        // load was called from, so `Module.nesting` inside it follows the
        // file's own class and module bodies rather than the caller's frame.
        let caller_nesting = std::mem::take(&mut self.method_nesting_stack);
        // Execute the parsed statements (always restore current_file, even on error).
        let result = self.execute_program(&statements);
        self.method_nesting_stack = caller_nesting;
        self.loading_paths.pop();
        self.current_file = previous_file;
        let value = result.map_err(|e| {
            MetorexError::runtime_error(
                format!("Error executing file '{}': {}", canonical_path.display(), e),
                SourceLocation::new(0, 0, 0),
            )
        })?;

        // Return the result or Nil if no return value
        Ok(value.unwrap_or(Object::Nil))
    }
}
