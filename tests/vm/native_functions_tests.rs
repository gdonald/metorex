// Coverage tests for vm/native_functions.rs — method() and require_relative error paths.

use metorex::lexer::Lexer;
use metorex::object::Object;
use metorex::parser::Parser;
use metorex::vm::VirtualMachine;
use std::path::Path;

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

// ── method() error paths ──────────────────────────────────────────────────────

#[test]
fn method_with_no_args_error() {
    let err = run_err("method()");
    // method() with 0 arguments
    assert!(err.contains("argument") || err.contains("method"));
}

#[test]
fn method_with_too_many_args_error() {
    let err = run_err("method(:foo, :bar)");
    assert!(err.contains("argument") || err.contains("method"));
}

#[test]
fn method_with_non_symbol_arg_error() {
    let err = run_err("method(42)");
    assert!(err.contains("Symbol") || err.contains("argument"));
}

#[test]
fn method_with_non_method_variable_error() {
    let err = run_err(
        r#"
x = 42
method(:x)
"#,
    );
    assert!(err.contains("not a method") || err.contains("method"));
}

#[test]
fn method_with_undefined_name_error() {
    let err = run_err("method(:nonexistent_xyz_abc)");
    assert!(err.contains("undefined") || err.contains("method"));
}

// ── method() happy path ───────────────────────────────────────────────────────

#[test]
fn method_returns_method_object() {
    let result = run(r#"
def greet
  "hello"
end
m = method(:greet)
m.nil?
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── require_relative without file context ─────────────────────────────────────

#[test]
fn require_relative_without_file_context_error() {
    let err = run_err(r#"require_relative("./nonexistent")"#);
    // No current file context when running via execute_program directly
    assert!(
        err.contains("require_relative")
            || err.contains("file")
            || err.contains("context")
            || err.contains("REPL")
    );
}

#[test]
fn require_relative_with_wrong_arg_count_error() {
    let err = run_err("require_relative()");
    assert!(err.contains("argument") || err.contains("require_relative"));
}

#[test]
fn require_relative_with_non_string_error() {
    let err = run_err("require_relative(42)");
    assert!(err.contains("String") || err.contains("argument"));
}

// ── require_relative execute_file error path (lines 136-138) ─────────────────

#[test]
fn require_relative_execute_file_error_propagates() {
    // Execute a file that require_relatives a helper which raises a runtime error.
    // This covers the map_err at lines 136-138 of native_functions.rs.
    let main_path = Path::new("tests/_examples/require/main_with_bad_require.rb");
    let mut vm = VirtualMachine::new();
    let result = vm.execute_file(main_path);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("require_relative") || msg.contains("error") || msg.contains("Error"));
}

// ── require_relative deduplication (lines 104-131) ──────────────────────

#[test]
fn require_relative_deduplication() {
    let code = "a = require_relative(\"lib/helper\")\nb = require_relative(\"lib/helper\")\nb";
    let tokens = metorex::lexer::Lexer::new(code).tokenize();
    let stmts = metorex::parser::Parser::new(tokens).parse().expect("parse");
    let mut vm = VirtualMachine::new();
    let base = std::fs::canonicalize("tests/_examples/require/basic.rb").unwrap();
    vm.set_current_file(base.clone());
    vm.mark_file_loaded(base);
    let result = vm.execute_program(&stmts);
    assert!(result.is_ok());
}

// ── assert_equal (lines 178-192) ────────────────────────────────────────

#[test]
fn assert_equal_success() {
    let result = run("assert_equal(1, 1)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn assert_equal_failure() {
    let err = run_err("assert_equal(1, 2)");
    assert!(
        err.contains("Expected") || err.contains("got"),
        "Error was: {}",
        err
    );
}

#[test]
fn assert_equal_with_message() {
    let err = run_err("assert_equal(1, 2, \"custom msg\")");
    assert!(err.contains("custom msg"), "Error was: {}", err);
}

#[test]
fn assert_equal_wrong_arg_count() {
    let err = run_err("assert_equal(1)");
    assert!(err.contains("argument"), "Error was: {}", err);
}

// ── assert_raises (lines 192-273) ───────────────────────────────────────

#[test]
fn assert_raises_with_args() {
    let err = run_err("assert_raises(1) { raise \"boom\" }");
    assert!(err.contains("argument"), "Error was: {}", err);
}

#[test]
fn assert_raises_no_block_error() {
    // assert_raises() called with no block and no parens returns function ref
    // Test with parens to trigger the error path
    let err = run_err("assert_raises()");
    assert!(
        err.contains("block") || err.contains("Block") || err.contains("requires"),
        "Error was: {}",
        err
    );
}
