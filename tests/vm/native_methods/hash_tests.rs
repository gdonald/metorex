// Coverage tests for hash native methods

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

// ── Hash each without block ───────────────────────────────────────────────────

#[test]
fn hash_each_without_block_error() {
    let err = run_err(
        r#"
{"a" => 1}.each
"#,
    );
    assert!(err.contains("block") || err.contains("each") || err.contains("requires"));
}

// ── Hash map without block ─────────────────────────────────────────────────────

#[test]
fn hash_map_without_block_error() {
    let err = run_err(
        r#"
{"a" => 1}.map
"#,
    );
    assert!(err.contains("block") || err.contains("map") || err.contains("requires"));
}

// ── Hash select without block ──────────────────────────────────────────────────

#[test]
fn hash_select_without_block_error() {
    let err = run_err(
        r#"
{"a" => 1}.select
"#,
    );
    assert!(err.contains("block") || err.contains("select") || err.contains("requires"));
}

// ── Hash each with break ────────────────────────────────────────────────

#[test]
fn hash_each_with_break() {
    let result = run(
        "count = 0\n{\"a\" => 1, \"b\" => 2, \"c\" => 3}.each do |k, v|\n  count = count + 1\n  if count == 2\n    break\n  end\nend\ncount",
    );
    assert_eq!(result, Some(Object::Int(2)));
}

// ── hash_methods.rs: get/fetch with default values (lines 112-123) ──────────

#[test]
fn hash_get_with_existing_key() {
    let result = run(r#"
h = {"a" => 1, "b" => 2}
h.get("a")
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_get_with_missing_key_returns_nil() {
    let result = run(r#"
h = {"a" => 1}
h.get("z")
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn hash_get_with_default_value() {
    let result = run(r#"
h = {"a" => 1}
h.get("z", 99)
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn hash_fetch_with_existing_key() {
    let result = run(r#"
h = {"a" => 1}
h.fetch("a")
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_fetch_with_missing_key_raises_error() {
    let err = run_err(
        r#"
h = {"a" => 1}
h.fetch("z")
"#,
    );
    assert!(err.contains("not found") || err.contains("Key"));
}

#[test]
fn hash_fetch_with_default_value() {
    let result = run(r#"
h = {"a" => 1}
h.fetch("z", 42)
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── hash_methods.rs: each with return in block error (lines 200-205) ────────

#[test]
fn hash_each_return_in_block_error() {
    let err = run_err(
        r#"
{"a" => 1}.each do |k, v|
  return k
end
"#,
    );
    assert!(err.contains("return") || err.contains("control") || err.contains("loop"));
}

// ── hash_methods.rs: each with exception in block ───────────────────────────

#[test]
fn hash_each_exception_in_block_propagates() {
    let err = run_err(
        r#"
{"a" => 1}.each do |k, v|
  raise "hash block error"
end
"#,
    );
    assert!(
        err.contains("hash block error") || err.contains("Uncaught") || err.contains("exception")
    );
}

// ── hash_methods.rs: each with continue in block ────────────────────────────

#[test]
fn hash_each_with_continue_in_block() {
    let result = run(r#"
count = 0
{"a" => 1, "b" => 2}.each do |k, v|
  continue
end
count
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

// ── hash_methods.rs: merge operation (lines 228-230) ────────────────────────

#[test]
fn hash_merge_operation() {
    let result = run(r#"
h1 = {"a" => 1, "b" => 2}
h2 = {"b" => 3, "c" => 4}
h3 = h1.merge(h2)
h3.size
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn hash_merge_overwrites_existing_keys() {
    let result = run(r#"
h1 = {"a" => 1, "b" => 2}
h2 = {"b" => 99}
h3 = h1.merge(h2)
h3["b"]
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

// ── Error path tests for hash methods ──────────────────────────────────────

#[test]
fn hash_get_existing_key() {
    let result = run(r#"
h = {"a" => 1, "b" => 2}
h.get("a")
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_get_missing_key_returns_nil() {
    let result = run(r#"
h = {"a" => 1}
h.get("z")
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn hash_get_with_default() {
    let result = run(r#"
h = {"a" => 1}
h.get("z", 99)
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn hash_fetch_existing_key() {
    let result = run(r#"
h = {"a" => 1}
h.fetch("a")
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_fetch_missing_key_error() {
    let err = run_err(
        r#"
h = {"a" => 1}
h.fetch("z")
"#,
    );
    assert!(
        err.contains("not found") || err.contains("key"),
        "Error was: {}",
        err
    );
}

#[test]
fn hash_fetch_with_default() {
    let result = run(r#"
h = {"a" => 1}
h.fetch("z", 42)
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn hash_each_continue_skips_iteration() {
    let result = run(r#"
h = {"a" => 1}
h.each do |k, v|
  continue
end
"#);
    assert!(result.is_some());
}
