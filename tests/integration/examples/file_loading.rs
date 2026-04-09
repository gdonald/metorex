use crate::common::EXAMPLES_DIR;
use std::process::Command;

use super::run_example;

/// Run an example that is expected to fail, returning stderr.
fn run_example_expect_error(path: &str) -> String {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/{}", EXAMPLES_DIR, path);
    let mut cmd = Command::new(binary);
    cmd.current_dir(manifest_dir).arg(&full_path);

    let output = cmd.output().expect("failed to execute example");
    assert!(
        !output.status.success(),
        "expected example {} to fail but it succeeded",
        path,
    );

    String::from_utf8(output.stderr).expect("stderr was not utf8")
}

#[test]
fn test_file_tracking_simple_execution() {
    let output = run_example("file_tracking/simple.rb");
    assert_eq!(output, "File tracking works\n");
}

#[test]
fn test_require_basic_execution() {
    let expected = "from helper\nhelper method called\n";
    let output = run_example("require/basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_require_deduplication_execution() {
    let expected = "counter loaded\n";
    let output = run_example("require/deduplication.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_require_nested_execution() {
    let expected = "util_a loaded\nutil_b loaded\nmain file\n";
    let output = run_example("require/nested.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_require_return_values_execution() {
    let expected = "true\nfalse\n";
    let output = run_example("require/return_values.rb");
    assert_eq!(output, expected);
}

// 9.3.5 — Extension auto-detection tests

#[test]
fn test_require_extension_auto_detect() {
    // require_relative "lib/helper" should find lib/helper.rb
    let expected = "from helper\nhelper method called\n";
    let output = run_example("require/extension_auto_detect.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_require_extension_explicit() {
    // require_relative "lib/helper.rb" should work with explicit extension
    let expected = "from helper\nhelper method called\n";
    let output = run_example("require/extension_explicit.rb");
    assert_eq!(output, expected);
}

// 9.3.6 — Scope/variable sharing tests

#[test]
fn test_require_scope_sharing() {
    // Variables, functions, and classes defined in required files are accessible
    let expected = "shared value\nshared function result\nSharedClass instance\n";
    let output = run_example("require/scope_sharing.rb");
    assert_eq!(output, expected);
}

// 9.3.8 — Circular requires handled gracefully

#[test]
fn test_require_circular() {
    // circular_a requires circular_b, circular_b requires circular_a
    // Should not infinite loop; the second require is a no-op
    let expected =
        "circular_a loading\ncircular_b loading\ncircular_b done\ncircular_a done\nmain done\n";
    let output = run_example("require/circular.rb");
    assert_eq!(output, expected);
}

// 9.3.9 — Diamond dependency (D loads only once)

#[test]
fn test_require_diamond_dependency() {
    // B and C both require D; D should only load once
    let expected = "diamond_d loaded\ndiamond_b loaded\ndiamond_c loaded\nmain done\n";
    let output = run_example("require/diamond.rb");
    assert_eq!(output, expected);
}

// 9.3.10 — Error messages use absolute paths

#[test]
fn test_require_error_shows_absolute_path() {
    let stderr = run_example_expect_error("require/bad_path.rb");
    // Error should contain an absolute path (starts with /)
    assert!(
        stderr.contains("/tests/_examples/require/lib/nonexistent_file_xyz"),
        "Error should contain absolute path, got: {}",
        stderr
    );
}

// 9.3.11 — "Did you mean?" suggestions

#[test]
fn test_require_error_suggests_similar_file() {
    let stderr = run_example_expect_error("require/typo_path.rb");
    assert!(
        stderr.contains("Did you mean"),
        "Error should contain 'Did you mean' suggestion, got: {}",
        stderr
    );
    assert!(
        stderr.contains("helper.rb"),
        "Error should suggest helper.rb, got: {}",
        stderr
    );
}

#[test]
fn test_require_error_no_suggestion_for_unrelated_name() {
    let stderr = run_example_expect_error("require/bad_path.rb");
    assert!(
        !stderr.contains("Did you mean"),
        "Error should NOT suggest for totally unrelated name, got: {}",
        stderr
    );
}

#[test]
fn test_require_runtime_error_in_required_file() {
    let stderr = run_example_expect_error("require/main_with_bad_require.rb");
    assert!(
        stderr.contains("intentional error"),
        "Error should mention the error from the required file, got: {}",
        stderr
    );
}
