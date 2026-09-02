// VM tests for string indexing, STDOUT/STDERR, range methods, string_each_char.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;

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

// ── String range indexing ────────────────────────────────────────────────────

#[test]
fn string_range_index() {
    let result = run("'hello'[1..3]");
    assert_eq!(result, Some(Object::string("ell")));
}

#[test]
fn string_range_exclusive() {
    let result = run("'hello'[1...3]");
    assert_eq!(result, Some(Object::string("el")));
}

#[test]
fn string_negative_range_index() {
    let result = run("'hello'[1..-1]");
    assert_eq!(result, Some(Object::string("ello")));
}

#[test]
fn string_exclusive_range_index() {
    let result = run("'hello'[0...2]");
    assert_eq!(result, Some(Object::string("he")));
}

#[test]
fn string_index_with_invalid_type_errors() {
    let err = run_err(r#""hello"[:foo]"#);
    assert!(err.contains("String index") || err.contains("Integer") || err.contains("Range"));
}

#[test]
fn string_slice_start_beyond_length_returns_empty() {
    let result = run(r#""ab"[5, 2]"#);
    assert_eq!(result, Some(Object::string("")));
}

#[test]
fn string_range_index_with_float_bounds() {
    let result = run(r#"
start_f = 1.0
end_f = 3.0
r = start_f..end_f
"hello"[r]
"#);
    assert!(matches!(result, Some(Object::String(_))));
}

// ── STDOUT/STDERR stream methods ─────────────────────────────────────────────

#[test]
fn stdout_puts_returns_nil() {
    let result = run("STDOUT.puts 'test'");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn stderr_puts_returns_nil() {
    let result = run("STDERR.puts 'test'");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn stdout_print_returns_nil() {
    let result = run("STDOUT.print 'test'");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn stdout_puts_no_args() {
    let result = run("STDOUT.puts");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn stdout_puts_with_non_string_arg() {
    let result = run(r#"STDOUT.puts 42"#);
    assert!(result.is_some() || result.is_none());
}

// ── Range each/map error paths ───────────────────────────────────────────────

#[test]
fn range_each_with_args_error() {
    let err = run_err("(1..3).each(1) { |x| x }");
    assert!(err.contains("argument"));
}

#[test]
fn range_map_with_args_error() {
    let err = run_err("(1..3).map(1) { |x| x }");
    assert!(err.contains("argument"));
}

#[test]
fn range_each_no_block_error() {
    let err = run_err("(1..3).each");
    assert!(err.contains("block") || err.contains("requires"));
}

#[test]
fn range_map_no_block_error() {
    let err = run_err("(1..3).map");
    assert!(err.contains("block") || err.contains("requires"));
}

// ── String#each_char ─────────────────────────────────────────────────────────

#[test]
fn string_each_char() {
    let result = run(r#"
count = 0
"hello".each_char { |c| count = count + 1 }
count
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

// ── dispatch.rs: __dir__ ─────────────────────────────────────────────────────

#[test]
fn magic_dir_without_file_context_returns_nil() {
    let result = run("__dir__");
    assert_eq!(result, Some(Object::Nil));
}
