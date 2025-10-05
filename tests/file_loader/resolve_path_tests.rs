use metorex::file_loader::resolve_relative_path;
use std::path::Path;

use crate::common::EXAMPLES_DIR;

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
