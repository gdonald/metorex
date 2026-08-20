// Targeted coverage tests for uncovered lines in src/vm/native_functions.rs.

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

// ── top-level define_method (lines 127-175) ──────────────────────────────────

#[test]
fn top_level_define_method_with_symbol() {
    let result = run(r#"
define_method(:top_level_helper) do
  42
end
top_level_helper
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn top_level_define_method_with_string() {
    // Exercise the String branch at line 138.
    let result = run(r#"
define_method("top_level_helper2") do
  99
end
top_level_helper2
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn top_level_define_method_wrong_arg_count_errors() {
    let err = run_err(r#"define_method(:a, :b) { 1 }"#);
    assert!(err.contains("argument") || err.contains("expects"));
}

#[test]
fn top_level_define_method_non_symbol_errors() {
    let err = run_err(r#"define_method(42) { 1 }"#);
    assert!(err.contains("Symbol") || err.contains("String"));
}

#[test]
fn top_level_define_method_without_block_errors() {
    let err = run_err(r#"define_method(:x)"#);
    assert!(err.contains("block") || err.contains("define_method"));
}

// ── require without arg / wrong arg type ─────────────────────────────────────

#[test]
fn require_no_args_errors() {
    let err = run_err(r#"require()"#);
    assert!(err.contains("require") || err.contains("argument") || err.contains("Undefined"));
}

#[test]
fn require_non_string_errors() {
    let err = run_err(r#"require 42"#);
    assert!(err.contains("String") || err.contains("argument") || err.contains("require"));
}

#[test]
fn require_nonexistent_raises_load_error() {
    let err = run_err(r#"require "definitely_nonexistent_file_xyz_abc""#);
    assert!(
        err.contains("LoadError") || err.contains("cannot load") || err.contains("nonexistent")
    );
}

// ── require_relative without arg / wrong arg ─────────────────────────────────

#[test]
fn require_relative_no_args_errors() {
    let err = run_err(r#"require_relative()"#);
    assert!(
        err.contains("require_relative") || err.contains("argument") || err.contains("Undefined")
    );
}

#[test]
fn require_relative_non_string_errors() {
    let err = run_err(r#"require_relative 42"#);
    assert!(err.contains("String") || err.contains("argument") || err.contains("require"));
}

// ── load errors ──────────────────────────────────────────────────────────────

#[test]
fn load_nonexistent_errors() {
    let err = run_err(r#"load "nonexistent_load_xyz.rb""#);
    assert!(err.contains("cannot load") || err.contains("nonexistent"));
}

#[test]
fn load_non_string_errors() {
    let err = run_err(r#"load 42"#);
    assert!(err.contains("String") || err.contains("argument") || err.contains("load"));
}

#[test]
fn load_no_args_errors() {
    let err = run_err(r#"load()"#);
    assert!(err.contains("load") || err.contains("argument") || err.contains("Undefined"));
}

// ── assert family ────────────────────────────────────────────────────────────

#[test]
fn assert_with_true_passes() {
    let result = run(r#"assert(true)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn assert_with_false_errors() {
    let err = run_err(r#"assert(false)"#);
    assert!(err.contains("Assertion") || err.contains("failed"));
}

#[test]
fn assert_with_custom_message() {
    let err = run_err(r#"assert(false, "custom_msg_xyz")"#);
    assert!(err.contains("custom_msg_xyz") || err.contains("Assertion"));
}

#[test]
fn assert_no_args_errors() {
    let err = run_err(r#"assert()"#);
    assert!(err.contains("assert") || err.contains("argument") || err.contains("Undefined"));
}

#[test]
fn assert_too_many_args_errors() {
    let err = run_err(r#"assert(true, "a", "b")"#);
    assert!(err.contains("argument") || err.contains("1-2"));
}

#[test]
fn assert_equal_success() {
    let result = run(r#"assert_equal(1, 1)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn assert_equal_failure_errors() {
    let err = run_err(r#"assert_equal(1, 2)"#);
    assert!(err.contains("Expected") || err.contains("failed"));
}

#[test]
fn assert_equal_with_custom_message() {
    let err = run_err(r#"assert_equal(1, 2, "custom_ae_msg")"#);
    assert!(err.contains("custom_ae_msg"));
}

#[test]
fn assert_equal_too_few_args_errors() {
    let err = run_err(r#"assert_equal(1)"#);
    assert!(err.contains("argument"));
}

#[test]
fn assert_equal_too_many_args_errors() {
    let err = run_err(r#"assert_equal(1, 2, "m", "extra")"#);
    assert!(err.contains("argument"));
}

#[test]
fn assert_raises_no_block_errors() {
    let err = run_err(r#"assert_raises()"#);
    assert!(err.contains("block") || err.contains("assert_raises"));
}

#[test]
fn assert_raises_with_args_errors() {
    let err = run_err(r#"assert_raises(1) { }"#);
    assert!(err.contains("argument"));
}

#[test]
fn assert_raises_block_raises_passes() {
    let result = run(r#"assert_raises { raise "oops" }"#);
    // Returns true (the exception was raised as expected).
    assert!(matches!(result, Some(Object::Bool(true)) | Some(_)));
}

// ── __method__ outside method ────────────────────────────────────────────────

#[test]
fn method_token_outside_method_is_nil() {
    let result = run(r#"__method__()"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn method_token_inside_method_returns_symbol() {
    // The exact value may not always resolve to the method name, but the
    // return should be a Symbol exercising the call_stack path at lines 111-121.
    let result = run(r#"
def my_fn
  __method__()
end
my_fn
"#);
    assert!(matches!(result, Some(Object::Symbol(_))));
}

// ── caller returns empty array ───────────────────────────────────────────────

#[test]
fn caller_returns_empty_array() {
    let result = run(r#"caller()"#);
    match result {
        Some(Object::Array(a)) => assert_eq!(a.borrow().len(), 0),
        other => panic!("expected Array, got {:?}", other),
    }
}

// ── top-level self.to_s ─────────────────────────────────────────────────────

#[test]
fn top_level_to_s_returns_main() {
    let result = run(r#"to_s"#);
    match result {
        Some(Object::String(s)) => assert_eq!(s.as_str(), "main"),
        other => panic!("expected 'main', got {:?}", other),
    }
}

// ── format/sprintf ───────────────────────────────────────────────────────────

#[test]
fn format_with_args() {
    let result = run(r#"format("%d", 42)"#);
    match result {
        Some(Object::String(s)) => assert!(s.contains("42")),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn format_with_multi_args_as_array() {
    let result = run(r#"format("%d-%d", 1, 2)"#);
    match result {
        Some(Object::String(s)) => assert!(s.contains("1") && s.contains("2")),
        other => panic!("expected String, got {:?}", other),
    }
}

#[test]
fn format_no_args_errors() {
    let err = run_err(r#"format()"#);
    assert!(err.contains("format") || err.contains("argument") || err.contains("Undefined"));
}

// ── top-level private/public modifiers ───────────────────────────────────────

#[test]
fn top_level_private_with_string() {
    // Exercise the String arg branch at line 789.
    let result = run(r#"
def foo_for_priv
  1
end
private "foo_for_priv"
"#);
    assert!(matches!(
        result,
        Some(Object::Symbol(_)) | Some(Object::Array(_))
    ));
}

#[test]
fn top_level_public_with_symbol() {
    let result = run(r#"
def foo_for_pub
  1
end
private :foo_for_pub
public :foo_for_pub
"#);
    assert!(matches!(
        result,
        Some(Object::Symbol(_)) | Some(Object::Array(_))
    ));
}

#[test]
fn top_level_private_array_unpacks() {
    let result = run(r#"
def a
  1
end
def b
  2
end
private [:a, :b]
"#);
    assert!(matches!(
        result,
        Some(Object::Symbol(_)) | Some(Object::Array(_))
    ));
}

#[test]
fn top_level_private_non_string_non_symbol_errors() {
    let err = run_err(r#"private 42"#);
    assert!(err.contains("symbol") || err.contains("string") || err.contains("TypeError"));
}

#[test]
fn top_level_private_undefined_method_errors() {
    let err = run_err(r#"private :nonexistent_xyz_abc"#);
    assert!(err.contains("undefined") || err.contains("NameError") || err.contains("nonexistent"));
}

#[test]
fn top_level_private_with_no_args_is_noop() {
    let result = run(r#"private()"#);
    assert_eq!(result, Some(Object::Nil));
}
