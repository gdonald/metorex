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

// ── 9.3.10: Error messages use absolute paths ────

#[test]
fn test_error_message_contains_absolute_path() {
    let path = Path::new(EXAMPLES_DIR).join("file_loader/nonexistent");
    let result = find_file_path(&path);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().message().to_string();
    // Error message should contain an absolute path (starts with /)
    assert!(
        err_msg.contains('/'),
        "Error should contain absolute path, got: {}",
        err_msg
    );
    assert!(
        err_msg.contains("File not found"),
        "Error should say 'File not found', got: {}",
        err_msg
    );
}

#[test]
fn test_error_message_shows_tried_extensions() {
    let path = Path::new(EXAMPLES_DIR).join("file_loader/nonexistent");
    let result = find_file_path(&path);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().message().to_string();
    assert!(
        err_msg.contains(".rb") && err_msg.contains(".mx"),
        "Error should list tried extensions (.rb, .mx), got: {}",
        err_msg
    );
}

// ── 9.3.11: "Did you mean?" suggestions ────

#[test]
fn test_error_message_suggests_similar_file() {
    // "test_fil" is close to "test_file.rb" — should suggest it
    let path = Path::new(EXAMPLES_DIR).join("file_loader/test_fil");
    let result = find_file_path(&path);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().message().to_string();
    assert!(
        err_msg.contains("Did you mean"),
        "Error should contain 'Did you mean' suggestion, got: {}",
        err_msg
    );
    assert!(
        err_msg.contains("test_file.rb"),
        "Error should suggest test_file.rb, got: {}",
        err_msg
    );
}

#[test]
fn test_error_message_no_suggestion_for_unrelated_name() {
    // "zzzzzzzzz" is totally unrelated — should not suggest anything
    let path = Path::new(EXAMPLES_DIR).join("file_loader/zzzzzzzzz");
    let result = find_file_path(&path);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().message().to_string();
    assert!(
        !err_msg.contains("Did you mean"),
        "Error should NOT contain suggestion for unrelated name, got: {}",
        err_msg
    );
}

// ── suggest_similar_files edge cases (file_loader.rs lines 43-86) ──────────

#[test]
fn test_suggest_similar_for_typo() {
    // Try a typo near existing files to trigger suggest_similar_files
    let path = Path::new(EXAMPLES_DIR).join("basics/intgers.rb");
    let result = find_file_path(&path);
    assert!(result.is_err());
}

#[test]
fn test_suggest_empty_stem() {
    let result = find_file_path(Path::new(".rb"));
    assert!(result.is_err());
}

// ── load_file_source read error (file_loader.rs lines 211-213) ─────────────

#[test]
fn test_load_nonexistent_file() {
    let result =
        metorex::file_loader::load_file_source(Path::new("this_file_does_not_exist_xyz.rb"));
    assert!(result.is_err());
}

// ── Additional load tests ───────────────────────────────────────────────────

#[test]
fn load_file_source_nonexistent_with_extension() {
    use metorex::file_loader::load_file_source;
    let result = load_file_source(std::path::Path::new("/nonexistent/file.rb"));
    assert!(result.is_err());
}
