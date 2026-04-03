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

// ============================================================================
// -v (Ruby-compatible version)
// ============================================================================

#[test]
fn cli_v_flag_prints_ruby_version() {
    let output = metorex_cmd().arg("-v").output().expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ruby 4.0.2"));
    assert!(stdout.contains("metorex"));
}

// ============================================================================
// -e (evaluate inline code)
// ============================================================================

#[test]
fn cli_e_flag_evaluates_code() {
    let output = metorex_cmd()
        .args(["-e", "puts 1 + 2"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "3");
}

#[test]
fn cli_e_flag_syntax_error_exits_nonzero() {
    let output = metorex_cmd()
        .args(["-e", "def"])
        .output()
        .expect("failed to execute");
    assert!(!output.status.success());
}

// ============================================================================
// --disable (ignored Ruby flag)
// ============================================================================

#[test]
fn cli_disable_flag_ignored() {
    let output = metorex_cmd()
        .args(["--disable=gems", "-e", "puts 42"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "42");
}

// ============================================================================
// -r, -I, -w, -d (ignored Ruby flags)
// ============================================================================

#[test]
fn cli_r_flag_ignored() {
    let output = metorex_cmd()
        .args(["-r", "somefile", "-e", "puts 1"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "1");
}

#[test]
fn cli_i_flag_ignored() {
    let output = metorex_cmd()
        .args(["-I", "somepath", "-e", "puts 2"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "2");
}

#[test]
fn cli_w_flag_ignored() {
    let output = metorex_cmd()
        .args(["-w", "-e", "puts 3"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "3");
}

#[test]
fn cli_d_flag_ignored() {
    let output = metorex_cmd()
        .args(["-d", "-e", "puts 4"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "4");
}

// ============================================================================
// ARGV (script arguments)
// ============================================================================

#[test]
fn cli_argv_passed_to_script() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["tests/_examples/argv/argv_basic.rb", "foo", "bar", "baz"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "3\nfoo\nbar\nbaz");
}

#[test]
fn cli_argv_empty_when_no_args() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["tests/_examples/argv/argv_basic.rb"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "0");
}
