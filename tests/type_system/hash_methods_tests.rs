// Tests for Hash native method coverage

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

// ── keys ────────────────────────────────────────────────────────────────────

#[test]
fn hash_keys_returns_array() {
    let result = run(r#"{"a" => 1, "b" => 2}.keys.length"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn hash_keys_error_with_args() {
    let err = run_err(r#"{"a" => 1}.keys(1)"#);
    assert!(err.contains("argument"));
}

// ── values ──────────────────────────────────────────────────────────────────

#[test]
fn hash_values_returns_array() {
    let result = run(r#"{"a" => 1, "b" => 2}.values.length"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn hash_values_error_with_args() {
    let err = run_err(r#"{"a" => 1}.values(1)"#);
    assert!(err.contains("argument"));
}

// ── has_key? ─────────────────────────────────────────────────────────────────

#[test]
fn hash_has_key_string() {
    let result = run(r#"{"a" => 1}.has_key?("a")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn hash_has_key_missing() {
    let result = run(r#"{"a" => 1}.has_key?("z")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn hash_has_key_float_key() {
    let result = run(r#"{1.5 => "x"}.has_key?(1.5)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn hash_has_key_bool_key() {
    let result = run(r#"{true => "x"}.has_key?(true)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn hash_has_key_nil_key() {
    let result = run(r#"{nil => "x"}.has_key?(nil)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn hash_has_key_error_no_args() {
    let err = run_err(r#"{"a" => 1}.has_key?"#);
    assert!(err.contains("argument"));
}

#[test]
fn hash_has_key_error_wrong_type() {
    let err = run_err(r#"{"a" => 1}.has_key?([1, 2])"#);
    assert!(err.contains("String"));
}

#[test]
fn hash_key_alias() {
    let result = run(r#"{"a" => 1}.key?("a")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── entries / to_a ──────────────────────────────────────────────────────────

#[test]
fn hash_entries_returns_pairs() {
    let result = run(r#"{"a" => 1}.entries.length"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_to_a_alias() {
    let result = run(r#"{"a" => 1}.to_a.length"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_entries_error_with_args() {
    let err = run_err(r#"{"a" => 1}.entries(1)"#);
    assert!(err.contains("argument"));
}

// ── length / size ────────────────────────────────────────────────────────────

#[test]
fn hash_length() {
    let result = run(r#"{"a" => 1, "b" => 2, "c" => 3}.length"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn hash_size_alias() {
    let result = run(r#"{"a" => 1}.size"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_length_error_with_args() {
    let err = run_err(r#"{"a" => 1}.length(1)"#);
    assert!(err.contains("argument"));
}

#[test]
fn hash_size_error_with_args() {
    let err = run_err(r#"{"a" => 1}.size(1)"#);
    assert!(err.contains("argument"));
}

// ── has_key? with integer key ────────────────────────────────────────────────

#[test]
fn hash_has_key_integer_key_true() {
    let result = run(r#"{1 => "x"}.has_key?(1)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn hash_has_key_integer_key_false() {
    let result = run(r#"{1 => "x"}.has_key?(2)"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── get ─────────────────────────────────────────────────────────────────────

#[test]
fn hash_get_existing_key() {
    let result = run(r#"{"a" => 1}.get("a")"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_get_missing_key_returns_nil() {
    let result = run(r#"{"a" => 1}.get("z")"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn hash_get_missing_key_with_default() {
    let result = run(r#"{"a" => 1}.get("z", 99)"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn hash_get_existing_key_ignores_default() {
    let result = run(r#"{"a" => 1}.get("a", 99)"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_get_error_no_args() {
    let err = run_err(r#"{"a" => 1}.get"#);
    assert!(err.contains("argument"));
}

#[test]
fn hash_get_error_too_many_args() {
    let err = run_err(r#"{"a" => 1}.get("a", 1, 2)"#);
    assert!(err.contains("argument"));
}

#[test]
fn hash_get_with_integer_key() {
    let result = run(r#"{1 => "one"}.get(1)"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("one".to_string())))
    );
}

// ── fetch ───────────────────────────────────────────────────────────────────

#[test]
fn hash_fetch_existing_key() {
    let result = run(r#"{"a" => 1}.fetch("a")"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_fetch_missing_key_raises_error() {
    let err = run_err(r#"{"a" => 1}.fetch("z")"#);
    assert!(err.contains("not found") || err.contains("Key"));
}

#[test]
fn hash_fetch_missing_key_with_default() {
    let result = run(r#"{"a" => 1}.fetch("z", 42)"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── merge ───────────────────────────────────────────────────────────────────

#[test]
fn hash_merge_two_hashes() {
    let result = run(r#"{"a" => 1}.merge({"b" => 2}).size"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn hash_merge_overwrites_existing_keys() {
    let result = run(r#"{"a" => 1, "b" => 2}.merge({"b" => 99})["b"]"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn hash_merge_does_not_mutate_original() {
    let result = run(r#"
h = {"a" => 1}
h.merge({"b" => 2})
h.size
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_merge_error_no_args() {
    let err = run_err(r#"{"a" => 1}.merge"#);
    assert!(err.contains("argument"));
}

#[test]
fn hash_merge_error_non_hash_arg() {
    let err = run_err(r#"{"a" => 1}.merge(42)"#);
    assert!(err.contains("Hash") || err.contains("type"));
}

// ── each ────────────────────────────────────────────────────────────────────

#[test]
fn hash_each_iterates_pairs() {
    let result = run(r#"
result = []
{"a" => 1}.each { |k, v| result.push(k) }
result.length
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_each_receives_key_and_value() {
    let result = run(r#"
total = 0
{"x" => 10, "y" => 20}.each { |k, v| total += v }
total
"#);
    assert_eq!(result, Some(Object::Int(30)));
}

#[test]
fn hash_each_returns_original_hash() {
    let result = run(r#"
h = {"a" => 1}
r = h.each { |k, v| v }
r.size
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_each_error_no_block() {
    let err = run_err(r#"{"a" => 1}.each"#);
    assert!(err.contains("block") || err.contains("Block"));
}

#[test]
fn hash_each_error_with_args() {
    let err = run_err(r#"{"a" => 1}.each(1) { |k, v| v }"#);
    assert!(err.contains("argument"));
}

#[test]
fn hash_each_with_break() {
    let result = run(r#"
count = 0
{"a" => 1, "b" => 2, "c" => 3}.each do |k, v|
  count += 1
  if count == 2
    break
  end
end
count
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ── additional coverage: get/fetch error paths ──────────────────────────────

#[test]
fn hash_each_with_exception_in_block() {
    let err = run_err(
        r#"
{"a" => 1}.each do |k, v|
  raise "error in each"
end
"#,
    );
    assert!(err.contains("error in each") || err.contains("exception"));
}

#[test]
fn hash_get_error_non_hashable_key() {
    let err = run_err(r#"{"a" => 1}.get([1, 2])"#);
    assert!(err.contains("String") || err.contains("type"));
}

#[test]
fn hash_fetch_error_non_hashable_key() {
    let err = run_err(r#"{"a" => 1}.fetch([1, 2])"#);
    assert!(err.contains("String") || err.contains("type"));
}

#[test]
fn hash_merge_error_too_many_args() {
    let err = run_err(r#"{"a" => 1}.merge({"b" => 2}, {"c" => 3})"#);
    assert!(err.contains("argument"));
}

#[test]
fn hash_bracket_access() {
    let result = run(r#"h = { "a" => 1, "b" => 2 }; h["a"]"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_bracket_missing_key() {
    // Ruby semantics: missing hash key returns nil (not an error).
    let result = run(r#"h = { "a" => 1 }; h["missing"]"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── Hash[] index assignment with various key types ────────────────────────

#[test]
fn hash_index_assign_symbol_key() {
    let result = run(r#"h = {}; h[:foo] = 1; h[:foo]"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_index_assign_integer_key() {
    let result = run(r#"h = {}; h[42] = "answer"; h[42]"#);
    if let Some(Object::String(s)) = result {
        assert_eq!(s.as_str(), "answer");
    } else {
        panic!("expected string, got {:?}", result);
    }
}

#[test]
fn hash_index_assign_bool_key() {
    let result = run(r#"h = {}; h[true] = "yes"; h[true]"#);
    if let Some(Object::String(s)) = result {
        assert_eq!(s.as_str(), "yes");
    } else {
        panic!("expected string, got {:?}", result);
    }
}

#[test]
fn hash_index_assign_nil_key() {
    let result = run(r#"h = {}; h[nil] = "nothing"; h[nil]"#);
    if let Some(Object::String(s)) = result {
        assert_eq!(s.as_str(), "nothing");
    } else {
        panic!("expected string, got {:?}", result);
    }
}

#[test]
fn hash_index_assign_float_key() {
    let result = run(r#"h = {}; h[1.5] = "f"; h[1.5]"#);
    if let Some(Object::String(s)) = result {
        assert_eq!(s.as_str(), "f");
    } else {
        panic!("expected string, got {:?}", result);
    }
}

#[test]
fn hash_each_basic() {
    let result = run(
        r#"h = { "a" => 1, "b" => 2 }; total = 0; h.each do |k, v|; total = total + v; end; total"#,
    );
    assert_eq!(result, Some(Object::Int(3)));
}
