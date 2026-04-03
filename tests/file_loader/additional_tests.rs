// Additional coverage tests for file_loader.rs uncovered paths

use metorex::file_loader::{find_file_path, load_file_source, parse_file, resolve_relative_path};
use std::path::Path;

// ── file_loader.rs lines 43-71: suggest_similar_files edge cases ────────────

#[test]
fn find_file_path_with_existing_extension_not_found() {
    // Path with extension that doesn't exist triggers the "has extension" error
    let result = find_file_path(Path::new("tests/_examples/file_loader/nonexistent.rb"));
    assert!(result.is_err());
    let msg = result.unwrap_err().message().to_string();
    assert!(msg.contains("File not found"));
}

// ── file_loader.rs lines 211-213: load_file_source read error path ──────────

#[test]
fn load_file_source_nonexistent_with_extension() {
    let result = load_file_source(Path::new("does_not_exist.rb"));
    assert!(result.is_err());
}

// ── file_loader.rs lines 297-299: parse_file with empty error list ──────────
// Hard to trigger because the parser always produces at least one error.
// But we can test the normal error path.

#[test]
fn parse_file_with_syntax_error() {
    let result = parse_file("def foo\n  1 +\nend", "test.rb");
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("test.rb") || msg.contains("error") || msg.contains("Parse"));
}

#[test]
fn parse_file_valid_source() {
    let result = parse_file("x = 1 + 2", "test.rb");
    assert!(result.is_ok());
}

// ── file_loader.rs: resolve_relative_path ───────────────────────────────────

#[test]
fn resolve_relative_path_basic() {
    let base = Path::new("/home/user/project/lib/helper.rb");
    let result = resolve_relative_path(base, "utils");
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("utils"));
}

#[test]
fn resolve_relative_path_with_parent_dir() {
    let base = Path::new("/home/user/project/lib/helper.rb");
    let result = resolve_relative_path(base, "../config/settings");
    assert!(result.is_ok());
    let path = result.unwrap();
    assert!(path.to_string_lossy().contains("config"));
    assert!(path.to_string_lossy().contains("settings"));
}

// ── file_loader.rs: absolute_display fallback paths ─────────────────────────

#[test]
fn find_file_path_nonexistent_no_extension_has_absolute_path() {
    let result = find_file_path(Path::new("nonexistent_xyz_abc"));
    assert!(result.is_err());
    let msg = result.unwrap_err().message().to_string();
    // Should have an absolute path in the message
    assert!(msg.contains('/'), "Expected absolute path in: {}", msg);
}

// ── file_loader.rs: levenshtein_distance ────────────────────────────────────

#[test]
fn find_file_path_similar_name_suggestion() {
    // We know "test_file.rb" exists in file_loader examples dir
    let path = Path::new("tests/_examples/file_loader/test_fiel");
    let result = find_file_path(path);
    assert!(result.is_err());
    let msg = result.unwrap_err().message().to_string();
    // May or may not suggest - just ensure no crash
    assert!(msg.contains("File not found"));
}
