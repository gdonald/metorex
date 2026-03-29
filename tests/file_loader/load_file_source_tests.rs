use metorex::file_loader::{find_file_path, load_file_source};
use std::path::Path;

use crate::common::EXAMPLES_DIR;

fn load_example(filename: &str) -> Result<String, metorex::error::MetorexError> {
    let path = Path::new(EXAMPLES_DIR).join("file_loader").join(filename);
    load_file_source(&path)
}

#[test]
fn test_load_file_with_rb_extension() {
    let result = load_example("test_file.rb");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "puts \"Hello from .rb file\"\n");
}

#[test]
fn test_load_file_with_mx_extension() {
    let result = load_example("test_file.rb");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "puts \"Hello from .rb file\"\n");
}

#[test]
fn test_load_file_no_extension() {
    let result = load_example("no_extension");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "puts \"File with no extension\"\n");
}

#[test]
fn test_load_file_auto_detect_rb() {
    // When no extension is provided, should try .rb first
    let result = load_example("test_file");
    assert!(result.is_ok());
    // Should load the .rb version since it's tried first
    assert_eq!(result.unwrap(), "puts \"Hello from .rb file\"\n");
}

#[test]
fn test_load_file_explicit_rb_extension() {
    let result = load_example("explicit.rb");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "puts \"File with explicit .rb extension\"\n"
    );
}

#[test]
fn test_load_file_explicit_mx_extension() {
    let result = load_example("explicit.rb");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "puts \"File with explicit .rb extension\"\n"
    );
}

#[test]
fn test_load_file_not_found() {
    let result = load_example("nonexistent");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("File not found"));
    assert!(err_msg.contains("nonexistent"));
}

#[test]
fn test_load_file_not_found_with_extension() {
    let result = load_example("nonexistent.rb");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("File not found"));
    assert!(err_msg.contains("nonexistent.rb"));
}

#[test]
fn test_load_file_invalid_path() {
    let result = load_example("does/not/exist.rb");
    assert!(result.is_err());
}

// ── find_file_path: .mx extension auto-detection (file_loader.rs line 51) ────

#[test]
fn test_find_file_path_mx_extension_auto_detect() {
    // mx_only.mx exists but mx_only.rb does not — should find the .mx file
    let path = Path::new(EXAMPLES_DIR).join("file_loader/mx_only");
    let result = find_file_path(&path);
    assert!(result.is_ok(), "Expected Ok but got: {:?}", result);
    let found = result.unwrap();
    assert!(found.to_string_lossy().ends_with(".mx"));
}
