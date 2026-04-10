use metorex::file_loader::{find_file_path, resolve_relative_path};
use std::path::Path;

use crate::common::EXAMPLES_DIR;

#[test]
fn test_resolve_path_no_parent_directory_error() {
    // A path with no parent (empty or root) causes "Cannot determine parent directory"
    // On Unix, Path::new("") has no parent, triggering file_loader.rs lines 118-125
    let base_file = Path::new("");
    let result = resolve_relative_path(base_file, "test.rb");
    assert!(
        result.is_err(),
        "Expected error for empty base path but got Ok"
    );
    let err = result.unwrap_err().to_string();
    assert!(err.contains("parent") || err.contains("directory") || err.contains("Cannot"));
}

#[test]
fn test_resolve_path_same_directory() {
    // Base file: tests/_examples/file_loader/test_file.rb
    // Relative path: explicit.rb (same directory)
    let base_file = Path::new(EXAMPLES_DIR).join("file_loader/test_file.rb");
    let result = resolve_relative_path(&base_file, "explicit.rb");

    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert!(resolved.ends_with("file_loader/explicit.rb"));
}

#[test]
fn test_resolve_path_same_directory_no_extension() {
    // Base file: tests/_examples/file_loader/test_file.rb
    // Relative path: no_extension (same directory, no extension)
    let base_file = Path::new(EXAMPLES_DIR).join("file_loader/test_file.rb");
    let result = resolve_relative_path(&base_file, "no_extension");

    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert!(resolved.ends_with("file_loader/no_extension"));
}

#[test]
fn test_resolve_path_subdirectory() {
    // Base file: tests/_examples/file_loader/test_file.rb
    // Relative path: subdir/nested.rb
    let base_file = Path::new(EXAMPLES_DIR).join("file_loader/test_file.rb");
    let result = resolve_relative_path(&base_file, "subdir/nested.rb");

    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert!(resolved.ends_with("file_loader/subdir/nested.rb"));
}

#[test]
fn test_resolve_path_subdirectory_relative() {
    // Base file: tests/_examples/file_loader/test_file.rb
    // Relative path: ./subdir/nested.rb
    let base_file = Path::new(EXAMPLES_DIR).join("file_loader/test_file.rb");
    let result = resolve_relative_path(&base_file, "./subdir/nested.rb");

    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert!(resolved.ends_with("file_loader/subdir/nested.rb"));
}

#[test]
fn test_resolve_path_parent_directory() {
    // Base file: tests/_examples/file_loader/parent_test/child.rb
    // Relative path: ../test_file.rb (go up one level)
    let base_file = Path::new(EXAMPLES_DIR).join("file_loader/parent_test/child.rb");
    let result = resolve_relative_path(&base_file, "../test_file.rb");

    assert!(result.is_ok());
    let resolved = result.unwrap();
    // Path will contain ../ since we don't canonicalize anymore
    assert!(
        resolved
            .to_string_lossy()
            .contains("parent_test/../test_file.rb")
            || resolved.ends_with("file_loader/test_file.rb")
    );
}

#[test]
fn test_resolve_path_parent_then_subdirectory() {
    // Base file: tests/_examples/file_loader/parent_test/child.rb
    // Relative path: ../subdir/nested.rb (go up, then down into subdir)
    let base_file = Path::new(EXAMPLES_DIR).join("file_loader/parent_test/child.rb");
    let result = resolve_relative_path(&base_file, "../subdir/nested.rb");

    assert!(result.is_ok());
    let resolved = result.unwrap();
    // Path will contain ../ since we don't canonicalize anymore
    assert!(
        resolved
            .to_string_lossy()
            .contains("parent_test/../subdir/nested.rb")
            || resolved.ends_with("file_loader/subdir/nested.rb")
    );
}

#[test]
fn test_resolve_path_multiple_parent_levels() {
    // Base file: tests/_examples/file_loader/subdir/nested.rb
    // Relative path: ../../file_loader/test_file.rb (go up two levels, then down)
    let base_file = Path::new(EXAMPLES_DIR).join("file_loader/subdir/nested.rb");
    let result = resolve_relative_path(&base_file, "../../file_loader/test_file.rb");

    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert!(resolved.ends_with("file_loader/test_file.rb"));
}

#[test]
fn test_resolve_path_nonexistent_file() {
    // Base file: tests/_examples/file_loader/test_file.rb
    // Relative path: nonexistent.rb (file doesn't exist)
    let base_file = Path::new(EXAMPLES_DIR).join("file_loader/test_file.rb");
    let result = resolve_relative_path(&base_file, "nonexistent.rb");

    // Should succeed (we don't canonicalize anymore, just join paths)
    // The file existence check happens later in find_file_path/execute_file
    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert!(resolved.ends_with("file_loader/nonexistent.rb"));
}

#[test]
fn test_resolve_path_nonexistent_base_file() {
    // Base file doesn't exist
    let base_file = Path::new(EXAMPLES_DIR).join("file_loader/does_not_exist.rb");
    let result = resolve_relative_path(&base_file, "test_file.rb");

    // Should work because we only need the parent directory, not the base file itself
    // However, canonicalize will fail if the target doesn't exist
    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert!(resolved.ends_with("file_loader/test_file.rb"));
}

#[test]
fn test_resolve_path_dot_current_directory() {
    // Base file: tests/_examples/file_loader/test_file.rb
    // Relative path: ./explicit.rb
    let base_file = Path::new(EXAMPLES_DIR).join("file_loader/test_file.rb");
    let result = resolve_relative_path(&base_file, "./explicit.rb");

    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert!(resolved.ends_with("file_loader/explicit.rb"));
}

#[test]
fn find_file_path_with_existing_extension_not_found() {
    // Path with extension that doesn't exist triggers the "has extension" error
    let result = find_file_path(Path::new("tests/_examples/file_loader/nonexistent.rb"));
    assert!(result.is_err());
    let msg = result.unwrap_err().message().to_string();
    assert!(msg.contains("File not found"));
}

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

#[test]
fn find_file_path_nonexistent_no_extension_has_absolute_path() {
    let result = find_file_path(Path::new("nonexistent_xyz_abc"));
    assert!(result.is_err());
    let msg = result.unwrap_err().message().to_string();
    // Should have an absolute path in the message
    assert!(msg.contains('/'), "Expected absolute path in: {}", msg);
}

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
