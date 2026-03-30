// 10.3.3 — Full Compilation Pipeline Tests
//
// End-to-end tests: source string → lexer → parser → VM → verified output.
// Uses the metorex binary to test the complete pipeline including stdout capture.

use crate::common::EXAMPLES_DIR;
use std::fs;
use std::process::Command;

/// Create a unique temp file path for this thread.
fn temp_rb_file(prefix: &str) -> std::path::PathBuf {
    let id = std::thread::current().id();
    std::env::temp_dir().join(format!("metorex_pipeline_{}{:?}.rb", prefix, id))
}

/// Write source to a temp file, run metorex on it, return stdout.
fn run_source(code: &str) -> String {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let tmp_path = temp_rb_file("ok_");
    fs::write(&tmp_path, code).expect("failed to write temp file");

    let output = Command::new(binary)
        .arg(&tmp_path)
        .output()
        .expect("failed to execute");

    let _ = fs::remove_file(&tmp_path);
    assert!(
        output.status.success(),
        "Program failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout not utf8")
}

/// Write source to a temp file, run metorex expecting failure, return stderr.
fn run_source_err(code: &str) -> String {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let tmp_path = temp_rb_file("err_");
    fs::write(&tmp_path, code).expect("failed to write temp file");

    let output = Command::new(binary)
        .arg(&tmp_path)
        .output()
        .expect("failed to execute");

    let _ = fs::remove_file(&tmp_path);
    assert!(!output.status.success(), "Expected program to fail");
    String::from_utf8(output.stderr).expect("stderr not utf8")
}

fn run_file(path: &str) -> String {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/{}", EXAMPLES_DIR, path);
    let output = Command::new(binary)
        .current_dir(manifest_dir)
        .arg(&full_path)
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "File {} failed: {}",
        path,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout not utf8")
}

// ── Full pipeline: source → stdout ──

#[test]
fn puts_integer() {
    assert_eq!(run_source("puts 42"), "42\n");
}

#[test]
fn puts_string() {
    assert_eq!(run_source("puts \"hello\""), "hello\n");
}

#[test]
fn puts_expression() {
    assert_eq!(run_source("puts 2 + 3"), "5\n");
}

#[test]
fn puts_multiple_lines() {
    assert_eq!(run_source("puts 1\nputs 2\nputs 3"), "1\n2\n3\n");
}

#[test]
fn string_interpolation_output() {
    assert_eq!(
        run_source("name = \"world\"\nputs \"hello #{name}\""),
        "hello world\n"
    );
}

#[test]
fn function_call_output() {
    let code = "def double(x)\n  x * 2\nend\nputs double(21)";
    assert_eq!(run_source(code), "42\n");
}

#[test]
fn class_method_output() {
    let code = r#"class Greeter
  def initialize(name)
    @name = name
  end
  def greet
    "Hello, #{@name}!"
  end
end
puts Greeter.new("World").greet"#;
    assert_eq!(run_source(code), "Hello, World!\n");
}

#[test]
fn array_iteration_output() {
    let code = "[10, 20, 30].each do |n|\n  puts n\nend";
    assert_eq!(run_source(code), "10\n20\n30\n");
}

#[test]
fn control_flow_output() {
    let code = r#"x = 15
if x > 10
  puts "big"
else
  puts "small"
end"#;
    assert_eq!(run_source(code), "big\n");
}

#[test]
fn exception_handling_output() {
    let code = r#"begin
  raise "oops"
rescue => e
  puts "caught"
end"#;
    assert_eq!(run_source(code), "caught\n");
}

// ── Error pipeline: source → stderr ──

#[test]
fn syntax_error_to_stderr() {
    let stderr = run_source_err("def foo\n  1");
    assert!(
        stderr.contains("error") || stderr.contains("Error"),
        "Expected error in stderr, got: {}",
        stderr
    );
}

#[test]
fn runtime_error_to_stderr() {
    let stderr = run_source_err("undefined_var_xyz");
    assert!(stderr.contains("Undefined variable"));
}

// ── File-based pipeline tests ──

#[test]
fn file_pipeline_fizzbuzz() {
    let output = run_file("programs/fizzbuzz.rb");
    assert!(output.contains("FizzBuzz"));
    assert!(output.contains("Fizz"));
    assert!(output.contains("Buzz"));
}

#[test]
fn file_pipeline_fibonacci() {
    let output = run_file("programs/fibonacci.rb");
    assert!(output.contains("34"));
    assert!(output.contains("Iterative:"));
    assert!(output.contains("Recursive:"));
}

#[test]
fn file_pipeline_oop_patterns() {
    let output = run_file("programs/oop_patterns.rb");
    assert!(output.contains("Strategy Pattern"));
    assert!(output.contains("Decorator Pattern"));
}

// ── AST dump pipeline (--ast flag) ──

#[test]
fn ast_flag_produces_output() {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = Command::new(binary)
        .current_dir(manifest_dir)
        .args(["--ast", &format!("{}/programs/fizzbuzz.rb", EXAMPLES_DIR)])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.is_empty(), "AST output should not be empty");
}
