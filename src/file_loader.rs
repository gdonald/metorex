// File Loading Infrastructure for require_relative
//
// This module provides utilities for loading, parsing, and resolving file paths
// in the Metorex language. It supports Ruby's file loading conventions including
// automatic file extension detection (.rb, .mx, or no extension).

use crate::error::{MetorexError, SourceLocation};
use std::fs;
use std::path::Path;

/// Loads the source code from a file, with automatic file extension detection.
///
/// This function supports Ruby's file loading conventions:
/// - If the path has an extension (.rb, .mx, etc.), it tries that path first
/// - If no extension or file not found, it tries adding .rb
/// - If still not found, it tries adding .mx
/// - Returns an error if the file doesn't exist with any extension
///
/// # Arguments
/// * `path` - The file path to load (may or may not include extension)
///
/// # Returns
/// * `Ok(String)` - The file contents as a string
/// * `Err(MetorexError)` - If the file cannot be found or read
pub fn load_file_source(path: &Path) -> Result<String, MetorexError> {
    // Try the path as given first
    if path.exists() {
        return fs::read_to_string(path).map_err(|e| {
            MetorexError::runtime_error(
                format!("Failed to read file '{}': {}", path.display(), e),
                SourceLocation::new(0, 0, 0),
            )
        });
    }

    // If the path has an extension, don't try alternatives
    if path.extension().is_some() {
        return Err(MetorexError::runtime_error(
            format!("File not found: '{}'", path.display()),
            SourceLocation::new(0, 0, 0),
        ));
    }

    // Try with .rb extension
    let rb_path = path.with_extension("rb");
    if rb_path.exists() {
        return fs::read_to_string(&rb_path).map_err(|e| {
            MetorexError::runtime_error(
                format!("Failed to read file '{}': {}", rb_path.display(), e),
                SourceLocation::new(0, 0, 0),
            )
        });
    }

    // Try with .mx extension
    let mx_path = path.with_extension("mx");
    if mx_path.exists() {
        return fs::read_to_string(&mx_path).map_err(|e| {
            MetorexError::runtime_error(
                format!("Failed to read file '{}': {}", mx_path.display(), e),
                SourceLocation::new(0, 0, 0),
            )
        });
    }

    // File not found with any extension
    Err(MetorexError::runtime_error(
        format!(
            "File not found: '{}' (tried {}, {}.rb, {}.mx)",
            path.display(),
            path.display(),
            path.display(),
            path.display()
        ),
        SourceLocation::new(0, 0, 0),
    ))
}
