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

        // Check if file is already loaded (deduplication)
        if self.is_file_loaded(&canonical_path) {
            return Ok(Object::Nil);
        }

        // Mark file as loaded before executing to prevent circular dependencies
        self.mark_file_loaded(canonical_path.clone());

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

        // Execute the parsed statements (always restore current_file, even on error).
        let result = self.execute_program(&statements);
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
