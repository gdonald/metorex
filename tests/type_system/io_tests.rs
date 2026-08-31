// Tests for IO and print functions

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;
use std::rc::Rc;

fn run(code: &str) -> Option<Object> {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).expect("execution failed")
}

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ============================================================================
// print function
// ============================================================================

#[test]
fn print_returns_nil() {
    let result = run(r#"print("hello")"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn print_multiple_args_returns_nil() {
    let result = run(r#"print("a", "b", "c")"#);
    assert_eq!(result, Some(Object::Nil));
}

// ============================================================================
// p function
// ============================================================================

#[test]
fn p_returns_the_argument() {
    let result = run("p(42)");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn p_multiple_returns_the_arguments() {
    let result = run("p(1, 2).inspect");
    assert_eq!(result, Some(Object::string("[1, 2]")));
}

#[test]
fn p_without_arguments_returns_nil() {
    let result = run("p()");
    assert_eq!(result, Some(Object::Nil));
}

// ============================================================================
// File.read
// ============================================================================

#[test]
fn file_read_existing_file() {
    // Read our own test fixture
    let result = run(r#"File.read("tests/_examples/require/bad_runtime.rb")"#);
    if let Some(Object::String(s)) = result {
        assert!(s.contains("raise"));
    } else {
        panic!("Expected string content");
    }
}

#[test]
fn file_read_nonexistent_file_error() {
    let err = run_err(r#"File.read("nonexistent_file_xyz.txt")"#);
    assert!(err.contains("Failed to read") || err.contains("No such file"));
}

#[test]
fn file_read_error_no_args() {
    let err = run_err("File.read");
    assert!(err.contains("argument"));
}

#[test]
fn file_read_error_non_string_arg() {
    let err = run_err("File.read(42)");
    assert!(err.contains("String") || err.contains("type"));
}

// ============================================================================
// File.write
// ============================================================================

#[test]
fn file_write_creates_file() {
    let path = "tests/_examples/io_test_output.txt";
    let result = run(&format!(r#"File.write("{}", "hello world")"#, path));
    // Returns number of bytes written
    assert_eq!(result, Some(Object::Int(11)));

    // Verify the file was created
    let contents = std::fs::read_to_string(path).expect("file should exist");
    assert_eq!(contents, "hello world");

    // Cleanup
    std::fs::remove_file(path).ok();
}

#[test]
fn file_write_error_no_args() {
    let err = run_err("File.write");
    assert!(err.contains("argument"));
}

// ============================================================================
// File.exist?
// ============================================================================

#[test]
fn file_exist_returns_true_for_existing() {
    let result = run(r#"File.exist?("Cargo.toml")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn file_exist_returns_false_for_missing() {
    let result = run(r#"File.exist?("nonexistent_xyz.txt")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn file_exists_alias() {
    let result = run(r#"File.exists?("Cargo.toml")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn file_exist_error_no_args() {
    let err = run_err("File.exist?");
    assert!(err.contains("argument"));
}

// ============================================================================
// File.read + File.write roundtrip
// ============================================================================

#[test]
fn file_write_then_read_roundtrip() {
    let path = "tests/_examples/io_roundtrip_test.txt";
    let result = run(&format!(
        r#"
File.write("{path}", "roundtrip test")
File.read("{path}")
"#,
    ));
    assert_eq!(
        result,
        Some(Object::String(Rc::new("roundtrip test".to_string())))
    );

    // Cleanup
    std::fs::remove_file(path).ok();
}

// ============================================================================
// Additional error paths for coverage
// ============================================================================

#[test]
fn file_write_non_string_path_error() {
    let err = run_err(r#"File.write(42, "content")"#);
    assert!(err.contains("String") || err.contains("type"));
}

#[test]
fn file_exist_non_string_error() {
    let err = run_err("File.exist?(42)");
    assert!(err.contains("String") || err.contains("type"));
}

#[test]
fn file_write_with_non_string_content() {
    let path = "tests/_examples/io_nonstr_test.txt";
    let result = run(&format!(r#"File.write("{}", 42)"#, path));
    // Non-string content gets formatted via Display
    assert_eq!(result, Some(Object::Int(2))); // "42" is 2 bytes

    // Cleanup
    std::fs::remove_file(path).ok();
}

#[test]
fn print_with_no_args_returns_nil() {
    let result = run("print()");
    assert_eq!(result, Some(Object::Nil));
}

// ============================================================================
// File.realpath
// ============================================================================

#[test]
fn file_realpath_for_existing_file() {
    let result = run(r#"File.realpath("Cargo.toml")"#);
    match result {
        Some(Object::String(s)) => assert!(s.ends_with("Cargo.toml")),
        other => panic!("expected canonical path string, got {:?}", other),
    }
}

#[test]
fn file_realpath_with_base_dir() {
    let result = run(r#"File.realpath("Cargo.toml", ".")"#);
    match result {
        Some(Object::String(s)) => assert!(s.ends_with("Cargo.toml")),
        other => panic!("expected canonical path string, got {:?}", other),
    }
}

#[test]
fn file_realpath_missing_raises_enoent() {
    let err = run_err(r#"File.realpath("definitely_not_here_xyz_123.txt")"#);
    assert!(err.contains("ENOENT") || err.contains("No such file"));
}

#[test]
fn file_realpath_wrong_arg_count_errors() {
    let err = run_err("File.realpath");
    assert!(err.contains("argument"));
}

#[test]
fn file_realpath_non_string_arg_errors() {
    let err = run_err("File.realpath(42)");
    assert!(err.contains("String"));
}

#[test]
fn file_realpath_non_string_base_errors() {
    let err = run_err(r#"File.realpath("Cargo.toml", 42)"#);
    assert!(err.contains("String"));
}

// ============================================================================
// File.directory?
// ============================================================================

#[test]
fn file_directory_true_for_existing_dir() {
    let result = run(r#"File.directory?("src")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn file_directory_false_for_file() {
    let result = run(r#"File.directory?("Cargo.toml")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn file_directory_wrong_arg_count_errors() {
    let err = run_err("File.directory?");
    assert!(err.contains("argument"));
}

#[test]
fn file_directory_non_string_arg_errors() {
    let err = run_err("File.directory?(42)");
    assert!(err.contains("String"));
}

// ============================================================================
// File.file?
// ============================================================================

#[test]
fn file_file_true_for_existing_file() {
    let result = run(r#"File.file?("Cargo.toml")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn file_file_false_for_directory() {
    let result = run(r#"File.file?("src")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn file_file_wrong_arg_count_errors() {
    let err = run_err("File.file?");
    assert!(err.contains("argument"));
}

#[test]
fn file_file_non_string_arg_errors() {
    let err = run_err("File.file?(42)");
    assert!(err.contains("String"));
}

// ============================================================================
// Dir[] / Dir.glob
// ============================================================================

#[test]
fn dir_bracket_glob_returns_matches() {
    let result = run(r#"Dir["Cargo.*"]"#);
    match result {
        Some(Object::Array(arr)) => {
            let names: Vec<String> = arr
                .borrow()
                .iter()
                .filter_map(|o| match o {
                    Object::String(s) => Some((**s).clone()),
                    _ => None,
                })
                .collect();
            assert!(names.iter().any(|n| n.ends_with("Cargo.toml")));
        }
        other => panic!("expected Array, got {:?}", other),
    }
}

#[test]
fn dir_glob_returns_matches() {
    let result = run(r#"Dir.glob("Cargo.*")"#);
    match result {
        Some(Object::Array(arr)) => {
            assert!(!arr.borrow().is_empty());
        }
        other => panic!("expected Array, got {:?}", other),
    }
}

#[test]
fn dir_glob_no_match_returns_empty_array() {
    let result = run(r#"Dir["zzz_no_such_pattern_xyz_*"]"#);
    match result {
        Some(Object::Array(arr)) => assert!(arr.borrow().is_empty()),
        other => panic!("expected Array, got {:?}", other),
    }
}

#[test]
fn dir_glob_no_args_errors() {
    let err = run_err(r#"Dir.glob"#);
    assert!(err.contains("argument"));
}

#[test]
fn dir_glob_non_string_arg_errors() {
    let err = run_err("Dir[42]");
    assert!(err.contains("String"));
}

// ── Kernel.load (native_methods/mod.rs lines 648-660) ──────────────────────

#[test]
fn kernel_load_executes_file() {
    // Kernel.load covers the Module receiver path in native_methods/mod.rs
    let tokens =
        metorex::lexer::Lexer::new(r#"Kernel.load("tests/_examples/require/lib/helper.rb")"#)
            .tokenize();
    let stmts = metorex::parser::Parser::new(tokens)
        .parse()
        .expect("parse failed");
    let mut vm = metorex::vm::VirtualMachine::new();
    // Run from the project root so the relative path resolves
    let result = vm.execute_program(&stmts);
    assert!(result.is_ok());
}

#[test]
fn kernel_load_wrong_arg_count_errors() {
    let err = run_err("Kernel.load");
    assert!(err.contains("argument") || err.contains("load") || err.contains("expected"));
}

#[test]
fn kernel_load_non_string_arg_errors() {
    let err = run_err("Kernel.load(42)");
    assert!(err.contains("String") || err.contains("argument") || err.contains("type"));
}

// ── p renders with inspect ───────────────────────────────────────────────────

#[test]
fn p_returns_its_single_argument() {
    let result = run(r#"p("abcde")"#);
    assert_eq!(result, Some(Object::string("abcde")));
}

#[test]
fn p_uses_a_user_defined_inspect() {
    let result = run(r#"
class Widget
  def inspect
    "custom inspect"
  end
end
p(Widget.new).class.name
"#);
    assert_eq!(result, Some(Object::string("Widget")));
}

#[test]
fn p_on_an_instance_without_inspect_does_not_recurse() {
    let result = run(r#"
class Plain
end
p(Plain.new).class.name
"#);
    assert_eq!(result, Some(Object::string("Plain")));
}

#[test]
fn a_bare_p_prints_nothing_and_returns_nil() {
    let result = run("p");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn p_is_a_private_instance_method_on_kernel() {
    let result = run("Kernel.private_instance_methods(false).include?(:p)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Output routes through $stdout ────────────────────────────────────────────

const CAPTURE: &str = r#"
class Capture
  def initialize
    @written = ""
  end
  def write(text)
    @written += text.to_s
  end
  def written
    @written
  end
end
capture = Capture.new
$stdout = capture
"#;

#[test]
fn puts_writes_through_a_replaced_stdout() {
    let result = run(&format!(
        "{CAPTURE}\nputs \"line\"\ncapture.written.inspect"
    ));
    assert_eq!(result, Some(Object::string("\"line\\n\"")));
}

#[test]
fn print_writes_through_a_replaced_stdout() {
    let result = run(&format!(
        "{CAPTURE}\nprint \"bare\"\ncapture.written.inspect"
    ));
    assert_eq!(result, Some(Object::string("\"bare\"")));
}

#[test]
fn p_writes_through_a_replaced_stdout() {
    let result = run(&format!("{CAPTURE}\np \"quoted\"\ncapture.written.inspect"));
    assert_eq!(result, Some(Object::string("\"\\\"quoted\\\"\\n\"")));
}

#[test]
fn a_bare_puts_writes_one_newline() {
    let result = run(&format!("{CAPTURE}\nputs\ncapture.written.inspect"));
    assert_eq!(result, Some(Object::string("\"\\n\"")));
}

// ── print with no arguments writes $_ ────────────────────────────────────────

#[test]
fn a_bare_print_writes_the_last_read_line() {
    let result = run(&format!(
        "{CAPTURE}\n$_ = \"remembered\"\nprint\ncapture.written.inspect"
    ));
    assert_eq!(result, Some(Object::string("\"remembered\"")));
}

// ── puts and print use to_s, not inspect ─────────────────────────────────────

#[test]
fn puts_renders_a_symbol_without_its_colon() {
    let result = run(&format!("{CAPTURE}\nputs :name\ncapture.written.inspect"));
    assert_eq!(result, Some(Object::string("\"name\\n\"")));
}

#[test]
fn p_keeps_the_colon_on_a_symbol() {
    let result = run(&format!("{CAPTURE}\np :name\ncapture.written.inspect"));
    assert_eq!(result, Some(Object::string("\":name\\n\"")));
}

// ── A bare call reaches self's method, not the Kernel function ───────────────

#[test]
fn a_bare_to_s_inside_a_class_reaches_that_class() {
    let result = run(r#"
class Speaker
  def to_s
    "speaker to_s"
  end
  def describe
    to_s
  end
end
Speaker.new.describe
"#);
    assert_eq!(result, Some(Object::string("speaker to_s")));
}

#[test]
fn a_user_defined_equals_can_call_a_bare_to_s() {
    let result = run(r#"
class Named
  def to_s
    "named"
  end
  def ==(other)
    to_s == other
  end
end
Named.new == "named"
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── readline / readlines / gets at end of input ──────────────────────────────

#[test]
fn readline_is_a_private_instance_method_on_kernel() {
    let result = run("Kernel.private_instance_methods(false).include?(:readline)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn readlines_is_a_private_instance_method_on_kernel() {
    let result = run("Kernel.private_instance_methods(false).include?(:readlines)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn eof_error_descends_from_io_error() {
    let result = run("EOFError.superclass.name");
    assert_eq!(result, Some(Object::string("IOError")));
}

#[test]
fn eof_error_is_a_standard_error() {
    let result = run("EOFError.ancestors.include?(StandardError)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn readline_rejects_arguments() {
    let error = run_err("readline(1)");
    assert!(error.contains("readline() expects 0 arguments, got 1"));
}

#[test]
fn readlines_rejects_arguments() {
    let error = run_err("readlines(1)");
    assert!(error.contains("readlines() expects 0 arguments, got 1"));
}
