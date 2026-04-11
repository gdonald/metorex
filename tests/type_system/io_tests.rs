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
fn p_multiple_returns_nil() {
    let result = run("p(1, 2)");
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
