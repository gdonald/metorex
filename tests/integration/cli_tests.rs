// CLI integration tests

use std::process::Command;

fn metorex_cmd() -> Command {
    let binary = env!("CARGO_BIN_EXE_metorex");
    Command::new(binary)
}

// ============================================================================
// --version
// ============================================================================

#[test]
fn cli_version_flag() {
    let output = metorex_cmd()
        .arg("--version")
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("metorex"));
    assert!(stdout.contains("0.1.0"));
}

// ============================================================================
// --help
// ============================================================================

#[test]
fn cli_help_flag() {
    let output = metorex_cmd()
        .arg("--help")
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Metorex"));
    assert!(stdout.contains("--ast"));
    assert!(stdout.contains("--debug"));
    assert!(stdout.contains("--repl"));
}

// ============================================================================
// --ast
// ============================================================================

#[test]
fn cli_ast_flag_dumps_ast() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["--ast", "tests/_examples/basics/each_block.rb"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Expression"));
}

// ============================================================================
// --debug
// ============================================================================

#[test]
fn cli_debug_flag_prints_debug_info() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["--debug", "tests/_examples/basics/each_block.rb"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("[debug]"));
    assert!(stderr.contains("Tokens:"));
    assert!(stderr.contains("Statements:"));
}

// ============================================================================
// File execution
// ============================================================================

#[test]
fn cli_execute_file() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .arg("tests/_examples/basics/each_block.rb")
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
}

#[test]
fn cli_nonexistent_file_exits_with_error() {
    let output = metorex_cmd()
        .arg("nonexistent_file_xyz.rb")
        .output()
        .expect("failed to execute");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error"));
}

#[test]
fn cli_file_with_syntax_error_exits_with_error() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .arg("tests/_examples/execute_file/syntax_error.rb")
        .output()
        .expect("failed to execute");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Parse error"));
}

// ============================================================================
// --test (test discovery)
// ============================================================================

#[test]
fn cli_test_flag_discovers_and_runs_tests() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["--test", "tests/_examples/test_discovery/nested"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("1 passed"));
    assert!(stdout.contains("0 failed"));
}

#[test]
fn cli_test_flag_shows_discovery_count() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["--test", "tests/_examples/test_discovery/nested"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Discovered 1 test file(s)"));
}

#[test]
fn cli_test_flag_exits_nonzero_on_failure() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["--test", "tests/_examples/test_discovery/failing"])
        .output()
        .expect("failed to execute");
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("FAIL"));
    assert!(stdout.contains("2 failed"));
}

#[test]
fn cli_test_flag_nonexistent_dir_fails() {
    let output = metorex_cmd()
        .args(["--test", "nonexistent_dir_xyz"])
        .output()
        .expect("failed to execute");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error") || stderr.contains("Error"));
}

#[test]
fn cli_test_flag_appears_in_help() {
    let output = metorex_cmd()
        .arg("--help")
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--test"));
}
