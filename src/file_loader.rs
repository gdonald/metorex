// File Loading Infrastructure for require_relative
//
// This module provides utilities for loading, parsing, and resolving file paths
// in the Metorex language. It supports Ruby's file loading conventions including
// automatic file extension detection (.rb, .mx, or no extension).

use crate::ast::Statement;
use crate::error::{MetorexError, SourceLocation};
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::fs;
use std::path::{Path, PathBuf};

/// Returns the absolute display form of a path (best-effort).
/// Uses canonicalize if possible, otherwise falls back to std::fs::canonicalize
/// or just returns the path as-is.
fn absolute_display(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| {
            // If the file doesn't exist we can't canonicalize, so try to
            // absolutize by joining with the current directory.
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                std::env::current_dir()
                    .map(|cwd| cwd.join(path))
                    .unwrap_or_else(|_| path.to_path_buf())
            }
        })
        .display()
        .to_string()
}

/// Suggest similar filenames from the same directory when a file is not found.
///
/// Looks at sibling files in the parent directory and returns names that are
/// similar to the requested filename (using a simple substring / edit-distance
/// heuristic).  Returns at most 3 suggestions.
pub fn suggest_similar_files(path: &Path) -> Vec<String> {
    let parent = match path.parent() {
        Some(p) if p.as_os_str().is_empty() => Path::new("."),
        Some(p) => p,
        None => return vec![],
    };

    let target_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    if target_stem.is_empty() {
        return vec![];
    }

    let entries = match fs::read_dir(parent) {
        Ok(entries) => entries,
        Err(_) => return vec![],
    };

    let mut candidates: Vec<(usize, String)> = Vec::new();

    for entry in entries.flatten() {
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }

        let entry_name = match entry_path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let entry_stem = entry_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Check if the extension is one we care about
        let ext = entry_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if !matches!(ext, "rb" | "mx" | "") {
            continue;
        }

        let distance = levenshtein_distance(&target_stem, &entry_stem);
        // Accept suggestions within a reasonable edit distance
        let threshold = if target_stem.len() <= 3 { 1 } else { 3 };
        if distance > 0 && distance <= threshold {
            candidates.push((distance, entry_name));
        }
    }

    candidates.sort_by_key(|(d, _)| *d);
    candidates
        .into_iter()
        .take(3)
        .map(|(_, name)| name)
        .collect()
}

/// Simple Levenshtein distance implementation for short strings.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    let mut prev = (0..=n).collect::<Vec<_>>();
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n]
}

/// Finds the actual file path with extension auto-detection.
///
/// This function supports Ruby's file loading conventions:
/// - If the path has an extension (.rb, .mx, etc.), it tries that path first
/// - If no extension or file not found, it tries adding .rb
/// - If still not found, it tries adding .mx
/// - Returns the path of the first file that exists
///
/// # Arguments
/// * `path` - The file path to find (may or may not include extension)
///
/// # Returns
/// * `Ok(PathBuf)` - The actual file path that exists
/// * `Err(MetorexError)` - If the file doesn't exist with any extension
pub fn find_file_path(path: &Path) -> Result<PathBuf, MetorexError> {
    // Try the path as given first
    if path.exists() {
        return Ok(path.to_path_buf());
    }

    // If the path has an extension, don't try alternatives
    if path.extension().is_some() {
        let abs = absolute_display(path);
        return Err(MetorexError::runtime_error(
            format!("File not found: '{}'", abs),
            SourceLocation::new(0, 0, 0),
        ));
    }

    // Try with .rb extension
    let rb_path = path.with_extension("rb");
    if rb_path.exists() {
        return Ok(rb_path);
    }

    // Try with .mx extension
    let mx_path = path.with_extension("mx");
    if mx_path.exists() {
        return Ok(mx_path);
    }

    // File not found with any extension — build helpful error message
    let abs = absolute_display(path);
    let abs_rb = format!("{}.rb", abs);
    let abs_mx = format!("{}.mx", abs);

    let mut msg = format!(
        "File not found: '{}' (tried {}, {}, {})",
        abs, abs, abs_rb, abs_mx
    );

    // Add "did you mean?" suggestions
    let suggestions = suggest_similar_files(path);
    if !suggestions.is_empty() {
        msg.push_str(&format!(". Did you mean: {}?", suggestions.join(", ")));
    }

    Err(MetorexError::runtime_error(
        msg,
        SourceLocation::new(0, 0, 0),
    ))
}

/// Loads the source code from a file, with automatic file extension detection.
///
/// This function supports Ruby's file loading conventions:
/// - If the path has an extension (.rb, .rb, etc.), it tries that path first
/// - If no extension or file not found, it tries adding .rb
/// - If still not found, it tries adding .rb
/// - Returns an error if the file doesn't exist with any extension
///
/// # Arguments
/// * `path` - The file path to load (may or may not include extension)
///
/// # Returns
/// * `Ok(String)` - The file contents as a string
/// * `Err(MetorexError)` - If the file cannot be found or read
pub fn load_file_source(path: &Path) -> Result<String, MetorexError> {
    let actual_path = find_file_path(path)?;
    fs::read_to_string(&actual_path).map_err(|e| {
        MetorexError::runtime_error(
            format!("Failed to read file '{}': {}", actual_path.display(), e),
            SourceLocation::new(0, 0, 0),
        )
    })
}

/// Resolves a relative path based on the location of a base file.
///
/// This function implements Ruby's `require_relative` path resolution logic:
/// - Gets the parent directory of the base file
/// - Joins the relative path to that directory
/// - Canonicalizes the result to resolve `..`, `.`, and symlinks
///
/// # Arguments
/// * `base_file` - The file from which the relative path is resolved
/// * `relative_path` - The relative path to resolve
///
/// # Returns
/// * `Ok(PathBuf)` - The absolute, canonicalized path
/// * `Err(MetorexError)` - If path resolution fails
///
/// # Examples
/// ```
/// // If base_file is "/home/user/project/lib/helper.rb"
/// // and relative_path is "../config/settings"
/// // Result will be "/home/user/project/config/settings"
/// ```
pub fn resolve_relative_path(
    base_file: &Path,
    relative_path: &str,
) -> Result<PathBuf, MetorexError> {
    // Get the parent directory of the base file
    let base_dir = base_file.parent().ok_or_else(|| {
        MetorexError::runtime_error(
            format!(
                "Cannot determine parent directory of '{}'",
                base_file.display()
            ),
            SourceLocation::new(0, 0, 0),
        )
    })?;

    // Join the relative path to the base directory
    // Note: We don't canonicalize here because the file may not exist yet (extension auto-detection)
    // Canonicalization will happen in execute_file after load_file_source finds the actual file
    let target_path = base_dir.join(relative_path);

    Ok(target_path)
}

/// Parses source code into an Abstract Syntax Tree (AST).
///
/// This function takes source code as a string and converts it into a vector of
/// Statement nodes that can be executed by the VM. It handles the lexing and parsing
/// process, converting any parse errors into MetorexError.
///
/// # Arguments
/// * `source` - The source code to parse
/// * `filename` - The name of the file being parsed (used in error messages)
///
/// # Returns
/// * `Ok(Vec<Statement>)` - The parsed AST
/// * `Err(MetorexError)` - If there are syntax errors in the source code
pub fn parse_file(source: &str, filename: &str) -> Result<Vec<Statement>, MetorexError> {
    // Create lexer from source
    let lexer = Lexer::new(source);

    // Tokenize source code
    let tokens = lexer.tokenize();

    // Create parser from tokens
    let mut parser = Parser::new(tokens);

    // Parse and return AST, converting parse errors to MetorexError
    parser.parse().map_err(|errors| {
        // If there are multiple parse errors, we'll return the first one
        // In the future, we might want to collect all errors
        if let Some(first_error) = errors.first() {
            // Create a new error with the filename context
            MetorexError::runtime_error(
                format!("Parse error in '{}': {}", filename, first_error),
                SourceLocation::new(0, 0, 0),
            )
        } else {
            // Shouldn't happen, but handle gracefully
            MetorexError::runtime_error(
                format!("Unknown parse error in '{}'", filename),
                SourceLocation::new(0, 0, 0),
            )
        }
    })
}
