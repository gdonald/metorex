// Hash error/edge coverage tests (split from native_methods_error_coverage_tests.rs)

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

// ══════════════════════════════════════════════════════════════════════════════
// Hash methods - get/fetch with defaults (lines 112-123)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn hash_get_missing_with_default_returns_default() {
    let result = run(r#"
h = {"x" => 10}
h.get("missing", "fallback")
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("fallback".to_string())))
    );
}

#[test]
fn hash_fetch_missing_without_default_errors() {
    let err = run_err(
        r#"
h = {"a" => 1}
h.fetch("nonexistent")
"#,
    );
    assert!(
        err.contains("not found") || err.contains("Key"),
        "Error was: {}",
        err
    );
}

#[test]
fn hash_fetch_missing_with_default_returns_default() {
    let result = run(r#"
h = {"a" => 1}
h.fetch("missing", "default_val")
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("default_val".to_string())))
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Hash methods - each with return/raise in block (lines 200-205)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn hash_each_return_in_method_errors() {
    let err = run_err(
        r#"
def test_hash_return
  {"a" => 1, "b" => 2}.each do |k, v|
    return k
  end
end
test_hash_return
"#,
    );
    assert!(
        err.contains("return") || err.contains("control") || err.contains("loop"),
        "Error was: {}",
        err
    );
}

#[test]
fn hash_each_raise_propagates() {
    let err = run_err(
        r#"
{"a" => 1}.each do |k, v|
  raise "hash boom"
end
"#,
    );
    assert!(
        err.contains("hash boom") || err.contains("Uncaught"),
        "Error was: {}",
        err
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Hash index access via [] method (lines 112-123)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn hash_bracket_access() {
    let result = run(r#"
h = {"name" => "Alice", "age" => 30}
h["name"]
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Alice".to_string()))));
}

#[test]
fn hash_bracket_missing_key() {
    let err = run_err(
        r#"
h = {"a" => 1}
h["missing"]
"#,
    );
    assert!(
        err.contains("not found") || err.contains("Key") || err.contains("missing"),
        "Error was: {}",
        err
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Hash keys, values, entries
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn hash_keys_method() {
    let result = run(r#"
h = {"a" => 1, "b" => 2}
h.keys.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn hash_values_method() {
    let result = run(r#"
h = {"a" => 1, "b" => 2}
h.values.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn hash_entries_method() {
    let result = run(r#"
h = {"a" => 1}
h.entries.length
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_has_key_method() {
    let result = run(r#"
h = {"a" => 1}
h.has_key?("a")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn hash_has_key_missing() {
    let result = run(r#"
h = {"a" => 1}
h.has_key?("z")
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}
