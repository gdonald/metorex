// Tests for the test discovery module

use metorex::test_discovery::{discover_test_files, is_test_file, run_test_discovery};
use std::path::Path;

use crate::common::EXAMPLES_DIR;

// ============================================================================
// is_test_file
// ============================================================================

#[test]
fn is_test_file_suffix_test() {
    assert!(is_test_file("math_test.rb"));
    assert!(is_test_file("foo_bar_test.rb"));
}

#[test]
fn is_test_file_prefix_test() {
    assert!(is_test_file("test_strings.rb"));
    assert!(is_test_file("test_foo_bar.rb"));
}

#[test]
fn is_test_file_suffix_spec() {
    assert!(is_test_file("array_spec.rb"));
    assert!(is_test_file("foo_bar_spec.rb"));
}

#[test]
fn is_test_file_rejects_non_test_names() {
    assert!(!is_test_file("helper.rb"));
    assert!(!is_test_file("main.rb"));
    assert!(!is_test_file("testing.rb"));
    assert!(!is_test_file("my_tests.rb"));
    assert!(!is_test_file("specials.rb"));
}

#[test]
fn is_test_file_rejects_non_rb_files() {
    assert!(!is_test_file("math_test.txt"));
    assert!(!is_test_file("test_strings.py"));
    assert!(!is_test_file("array_spec.rs"));
}

#[test]
fn is_test_file_case_insensitive() {
    assert!(is_test_file("Math_Test.rb"));
    assert!(is_test_file("TEST_strings.rb"));
    assert!(is_test_file("Array_Spec.rb"));
}

// ============================================================================
// discover_test_files
// ============================================================================

#[test]
fn discover_finds_all_test_files() {
    let dir = Path::new(EXAMPLES_DIR).join("test_discovery");
    let files = discover_test_files(&dir).unwrap();

    let names: Vec<String> = files
        .iter()
        .map(|p| p.strip_prefix(&dir).unwrap().to_string_lossy().to_string())
        .collect();

    assert!(names.contains(&"array_spec.rb".to_string()));
    assert!(names.contains(&"math_test.rb".to_string()));
    assert!(names.contains(&"test_strings.rb".to_string()));
}

#[test]
fn discover_finds_nested_test_files() {
    let dir = Path::new(EXAMPLES_DIR).join("test_discovery");
    let files = discover_test_files(&dir).unwrap();

    let has_nested = files.iter().any(|p| {
        p.strip_prefix(&dir)
            .unwrap()
            .to_string_lossy()
            .contains("nested")
    });
    assert!(has_nested, "Should find test files in nested directories");
}

#[test]
fn discover_excludes_non_test_files() {
    let dir = Path::new(EXAMPLES_DIR).join("test_discovery");
    let files = discover_test_files(&dir).unwrap();

    let has_helper = files
        .iter()
        .any(|p| p.file_name().unwrap().to_string_lossy() == "helper.rb");
    assert!(
        !has_helper,
        "Should not include files that don't match test patterns"
    );
}

#[test]
fn discover_returns_sorted_paths() {
    let dir = Path::new(EXAMPLES_DIR).join("test_discovery");
    let files = discover_test_files(&dir).unwrap();
    let paths: Vec<String> = files.iter().map(|p| p.display().to_string()).collect();
    let mut sorted = paths.clone();
    sorted.sort();
    assert_eq!(paths, sorted);
}

#[test]
fn discover_errors_on_nonexistent_directory() {
    let dir = Path::new("nonexistent_dir_xyz");
    let result = discover_test_files(dir);
    assert!(result.is_err());
}

#[test]
fn discover_returns_correct_count() {
    let dir = Path::new(EXAMPLES_DIR).join("test_discovery");
    let files = discover_test_files(&dir).unwrap();
    // 4 passing + 2 failing = 6 total
    assert_eq!(files.len(), 6, "Should find exactly 6 test files");
}

// ============================================================================
// run_test_discovery
// ============================================================================

#[test]
fn run_discovery_passing_dir() {
    // Run only against the top-level passing files + nested passing file
    // Use the nested directory which has only 1 passing file
    let dir = Path::new(EXAMPLES_DIR).join("test_discovery/nested");
    let result = run_test_discovery(&dir).unwrap();
    assert!(result.all_passed(), "All test files should pass");
    assert_eq!(result.passed_count(), 1);
    assert_eq!(result.failed_count(), 0);
}

#[test]
fn run_discovery_empty_directory() {
    let dir = std::env::temp_dir().join("metorex_test_empty_dir");
    let _ = std::fs::create_dir_all(&dir);
    let result = run_test_discovery(&dir).unwrap();
    assert_eq!(result.results.len(), 0);
    assert!(result.all_passed());
    let _ = std::fs::remove_dir(&dir);
}

#[test]
fn run_discovery_nonexistent_directory() {
    let dir = Path::new("nonexistent_dir_xyz_test");
    let result = run_test_discovery(dir);
    assert!(result.is_err());
}

// ============================================================================
// Failure paths
// ============================================================================

#[test]
fn run_discovery_with_failing_tests() {
    let dir = Path::new(EXAMPLES_DIR).join("test_discovery/failing");
    let result = run_test_discovery(&dir).unwrap();
    assert!(!result.all_passed());
    assert_eq!(result.failed_count(), 2);
    assert_eq!(result.passed_count(), 0);
}

#[test]
fn run_discovery_failing_has_error_messages() {
    let dir = Path::new(EXAMPLES_DIR).join("test_discovery/failing");
    let result = run_test_discovery(&dir).unwrap();
    for r in &result.results {
        assert!(!r.success);
        assert!(r.error.is_some());
    }
}

#[test]
fn run_discovery_syntax_error_detected() {
    let dir = Path::new(EXAMPLES_DIR).join("test_discovery/failing");
    let result = run_test_discovery(&dir).unwrap();
    let syntax_err = result
        .results
        .iter()
        .find(|r| r.path.to_string_lossy().contains("syntax_error"))
        .unwrap();
    assert!(!syntax_err.success);
    assert!(syntax_err.error.as_ref().unwrap().contains("Parse error"));
}

#[test]
fn run_discovery_runtime_error_detected() {
    let dir = Path::new(EXAMPLES_DIR).join("test_discovery/failing");
    let result = run_test_discovery(&dir).unwrap();
    let runtime_err = result
        .results
        .iter()
        .find(|r| r.path.to_string_lossy().contains("runtime_error"))
        .unwrap();
    assert!(!runtime_err.success);
    assert!(runtime_err.error.is_some());
}

#[test]
fn test_file_result_has_duration() {
    let dir = Path::new(EXAMPLES_DIR).join("test_discovery");
    let result = run_test_discovery(&dir).unwrap();
    assert!(result.total_duration_ms < 10000); // should complete quickly
    for r in &result.results {
        // duration should be a reasonable value (not overflow or negative)
        assert!(r.duration_ms < 10000);
    }
}
