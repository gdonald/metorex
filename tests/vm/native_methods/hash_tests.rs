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

// ── From edge_tests and misc_tests ──────────────────────────────────────────

#[test]
fn hash_merge_method_edge() {
    let result = run(r#"a = {"x" => 1}; b = {"y" => 2}; c = a.merge(b); c["y"]"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn int_as_hash_key_misc() {
    let result = run("h = { 1 => \"one\" }; h[1]");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("one".to_string())))
    );
}

#[test]
fn bool_as_hash_key_misc() {
    let result = run("h = { true => \"yes\" }; h[true]");
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("yes".to_string())))
    );
}

// ── compare_by_identity ─────────────────────────────────────────────────────

#[test]
fn hash_compare_by_identity_returns_self() {
    let result = run(r#"
h = {"a" => 1}
h.compare_by_identity
h["a"]
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_compare_by_identity_query_returns_false() {
    let result = run(r#"
h = {"a" => 1}
h.compare_by_identity?
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn hash_compare_by_identity_with_args_error() {
    let err = run_err(r#"{"a" => 1}.compare_by_identity(1)"#);
    assert!(err.contains("argument"));
}

#[test]
fn hash_compare_by_identity_query_with_args_error() {
    let err = run_err(r#"{"a" => 1}.compare_by_identity?(1)"#);
    assert!(err.contains("argument"));
}

// ── Hash#[] error ───────────────────────────────────────────────────────────

#[test]
fn hash_index_wrong_arg_count_error() {
    let err = run_err(r#"{"a" => 1}.[](1, 2)"#);
    assert!(err.contains("argument"));
}

// ── Hash#each ───────────────────────────────────────────────────────────────

#[test]
fn hash_each_iterates_coverage() {
    let result = run(r#"
h = {"a" => 1, "b" => 2}
sum = 0
h.each { |k, v| sum = sum + v }
sum
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── hash_methods.rs: merge with non-Hash argument error (lines 144-147) ─────

#[test]
fn hash_merge_non_hash_arg_error() {
    let err = run_err(
        r#"
h = {"a" => 1}
h.merge(42)
"#,
    );
    assert!(
        err.contains("Hash") || err.contains("type") || err.contains("argument"),
        "Error was: {}",
        err
    );
}

#[test]
fn hash_merge_string_arg_error() {
    let err = run_err(
        r#"
h = {"a" => 1}
h.merge("not a hash")
"#,
    );
    assert!(
        err.contains("Hash") || err.contains("type") || err.contains("argument"),
        "Error was: {}",
        err
    );
}

// ── hash_methods.rs: each with block (lines 224-229 – the None branch when no block) ─

#[test]
fn hash_each_without_block_raises_error() {
    let err = run_err(r#"{"a" => 1, "b" => 2}.each"#);
    assert!(
        err.contains("block") || err.contains("each") || err.contains("requires"),
        "Error was: {}",
        err
    );
}

// ── hash_methods.rs lines 144-147: Hash#[] via send ──────────────────────────

#[test]
fn hash_bracket_via_send() {
    let result = run(r#"
h = {"x" => 10, "y" => 20}
h.send(:[], "x")
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

// ── hash delete ─────────────────────────────────────────────────────────────

#[test]
fn hash_delete_existing() {
    let result = run(r#"
h = {"a" => 1, "b" => 2}
h.delete("a")
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn hash_delete_missing() {
    assert_eq!(run(r#"{"a" => 1}.delete("z")"#), Some(Object::Nil));
}

#[test]
fn hash_delete_wrong_args() {
    let err = run_err(r#"{"a" => 1}.delete"#);
    assert!(err.contains("argument"));
}

// ── hash with non-primitive key (reconstruct_key via KEY_OBJECTS) ───────────

#[test]
fn hash_non_primitive_key_reconstruct() {
    let result = run(r#"
a = [1, 2]
h = {}
h[a] = "val"
h.keys.length
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ── hash fetch with default / block ─────────────────────────────────────────

#[test]
fn hash_fetch_with_default_cov() {
    assert_eq!(run(r#"{"a" => 1}.fetch("z", 99)"#), Some(Object::Int(99)));
}

#[test]
fn hash_fetch_missing_no_default_errors() {
    let err = run_err(r#"{"a" => 1}.fetch("z")"#);
    assert!(err.contains("key") || err.contains("KeyError") || err.contains("not found"));
}

// ── hash_methods.rs uncovered paths ─────────────────────────────────────

#[test]
fn hash_default_returns_nil() {
    let result = run(r#"
h = { a: 1 }
h.default
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn hash_delete_removes_non_primitive_key_from_key_objects() {
    // Non-primitive key is tracked in KEY_OBJECTS_KEY. Deleting also removes
    // from the tracking dict (hash_methods.rs line 155).
    let result = run(r#"
a = [1, 2]
h = {}
h[a] = "v"
h.delete(a)
h.keys.length
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

// ── Insertion order ──────────────────────────────────────────────────────────

#[test]
fn hash_keys_come_back_in_insertion_order() {
    let result = run(r#"{ "c" => 1, "a" => 2, "b" => 3 }.keys.inspect"#);
    assert_eq!(result, Some(Object::string("[\"c\", \"a\", \"b\"]")));
}

#[test]
fn hash_values_come_back_in_insertion_order() {
    let result = run(r#"{ "c" => 1, "a" => 2, "b" => 3 }.values.inspect"#);
    assert_eq!(result, Some(Object::string("[1, 2, 3]")));
}

#[test]
fn a_new_key_goes_to_the_end() {
    let result = run(r#"
h = { "a" => 1, "b" => 2 }
h["c"] = 3
h.keys.inspect
"#);
    assert_eq!(result, Some(Object::string("[\"a\", \"b\", \"c\"]")));
}

#[test]
fn reassigning_a_key_keeps_its_position() {
    let result = run(r#"
h = { "a" => 1, "b" => 2, "c" => 3 }
h["a"] = 99
h.keys.inspect
"#);
    assert_eq!(result, Some(Object::string("[\"a\", \"b\", \"c\"]")));
}

#[test]
fn deleting_a_key_keeps_the_rest_in_order() {
    let result = run(r#"
h = { "a" => 1, "b" => 2, "c" => 3 }
h.delete("b")
h.keys.inspect
"#);
    assert_eq!(result, Some(Object::string("[\"a\", \"c\"]")));
}

#[test]
fn each_walks_a_hash_in_insertion_order() {
    let result = run(r#"
seen = []
{ "z" => 1, "y" => 2, "x" => 3 }.each do |key, value|
  seen << key
end
seen.inspect
"#);
    assert_eq!(result, Some(Object::string("[\"z\", \"y\", \"x\"]")));
}

// ── An implicit hash at the end of an array literal ──────────────────────────

#[test]
fn an_array_literal_gathers_trailing_keyword_pairs_into_a_hash() {
    let result = run(r#"[1, 2, first: "a", second: "b"].length"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn the_gathered_hash_keeps_its_keys_in_order() {
    let result = run(r#"[1, first: "a", second: "b"].last.keys.inspect"#);
    assert_eq!(result, Some(Object::string("[:first, :second]")));
}

#[test]
fn an_array_literal_gathers_arrow_pairs_too() {
    let result = run(r#"["x" => 1, "y" => 2].first.keys.inspect"#);
    assert_eq!(result, Some(Object::string("[\"x\", \"y\"]")));
}

#[test]
fn an_array_of_only_keyword_pairs_holds_one_hash() {
    let result = run("[alpha: 1, beta: 2].length");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn an_array_literal_without_pairs_is_unchanged() {
    let result = run(r#"[1, "two", :three].length"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn each_pair_walks_a_hash_like_each() {
    let result = run(r##"
seen = []
{ "a" => 1, "b" => 2 }.each_pair do |key, value|
  seen << "#{key}=#{value}"
end
seen.inspect
"##);
    assert_eq!(result, Some(Object::string("[\"a=1\", \"b=2\"]")));
}
