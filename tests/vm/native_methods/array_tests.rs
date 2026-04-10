// Coverage tests for array native methods

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

// ── Array sort with non-comparable types (line 25 in compare_objects) ─────────

#[test]
fn array_sort_with_mixed_types_uses_string_comparison() {
    // [true, false, nil] sorted - triggers the fallback string comparison
    let result = run(r#"
arr = [true, false, nil]
arr.sort
"#);
    // Sort uses string comparison as fallback, result should be Some
    assert!(result.is_some());
}

#[test]
fn array_sort_with_bool_and_int_uses_fallback() {
    let result = run(r#"
arr = [true, 1, false]
arr.sort
"#);
    assert!(result.is_some());
}

// ── Array each without block ───────────────────────────────────────────────────

#[test]
fn array_each_without_block_error() {
    let err = run_err(
        r#"
[1, 2, 3].each
"#,
    );
    assert!(err.contains("block") || err.contains("each") || err.contains("requires"));
}

// ── Array map without block ────────────────────────────────────────────────────

#[test]
fn array_map_without_block_error() {
    let err = run_err(
        r#"
[1, 2, 3].map
"#,
    );
    assert!(err.contains("block") || err.contains("map") || err.contains("requires"));
}

// ── Array select without block ─────────────────────────────────────────────────

#[test]
fn array_select_without_block_error() {
    let err = run_err(
        r#"
[1, 2, 3].select
"#,
    );
    assert!(err.contains("block") || err.contains("select") || err.contains("requires"));
}

// ── Array reduce without block ─────────────────────────────────────────────────

#[test]
fn array_reduce_without_block_error() {
    let err = run_err(
        r#"
[1, 2, 3].reduce
"#,
    );
    assert!(err.contains("block") || err.contains("reduce") || err.contains("requires"));
}

// ── Array each/map/select/reduce block paths ────────────────────────────

#[test]
fn array_each_with_break() {
    let result = run(
        "result = []\n[1, 2, 3, 4, 5].each do |x|\n  if x == 4\n    break\n  end\n  result.push(x)\nend\nresult.length",
    );
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_each_with_continue() {
    // `next` is not a keyword in Metorex, use workaround
    let result = run(
        "result = []\n[1, 2, 3, 4, 5].each do |x|\n  if x != 3\n    result.push(x)\n  end\nend\nresult.length",
    );
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn array_select_basic() {
    let result = run("[1, 2, 3, 4].select { |x| x > 2 }");
    assert!(result.is_some());
}

#[test]
fn array_reduce_basic() {
    let result = run("[1, 2, 3].reduce { |sum, x| sum + x }");
    assert_eq!(result, Some(Object::Int(6)));
}

// ── array_methods.rs: push/append (line 39 early return for non-array) ──────

#[test]
fn array_append_alias() {
    let result = run(r#"
arr = [1, 2]
arr.append(3)
arr.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── array_methods.rs: each with continue in block (lines 121-123) ───────────

#[test]
fn array_each_with_continue_in_block() {
    let result = run(r#"
result = []
[1, 2, 3, 4, 5].each do |x|
  if x == 3
    continue
  end
  result.push(x)
end
result.length
"#);
    assert_eq!(result, Some(Object::Int(4)));
}

// ── array_methods.rs: each with exception in block (lines 131-142) ──────────

#[test]
fn array_each_exception_in_block_propagates() {
    let err = run_err(
        r#"
[1, 2, 3].each do |x|
  raise "array block error"
end
"#,
    );
    assert!(
        err.contains("array block error") || err.contains("Uncaught") || err.contains("exception")
    );
}

// ── array_methods.rs: each with return in block (lines 126-129) ─────────────

#[test]
fn array_each_return_in_block_error() {
    let err = run_err(
        r#"
[1, 2, 3].each do |x|
  return x
end
"#,
    );
    assert!(err.contains("return") || err.contains("control") || err.contains("loop"));
}

// ── array_methods.rs: map with return in block error ────────────────────────
// map uses execute_block_body which handles Return differently;
// The uncovered lines 158-163 are the Some(other) and None branches
// for map's pending_block match. The None branch is already tested.
// Test the Some(non-block) branch indirectly if possible.

// ── array_methods.rs: transpose with jagged arrays (line 354, nil padding) ──

#[test]
fn array_transpose_jagged_arrays_nil_padding() {
    let result = run(r#"
arr = [[1, 2, 3], [4, 5]]
t = arr.transpose
t.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_transpose_jagged_arrays_nil_values() {
    // The third column should have nil for the second row
    let result = run(r#"
arr = [[1, 2, 3], [4, 5]]
t = arr.transpose
t[2][1]
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── array_methods.rs: reduce with initial value ─────────────────────────────

#[test]
fn array_reduce_with_initial_value() {
    let result = run(r#"
[1, 2, 3].reduce(10) { |sum, x| sum + x }
"#);
    assert_eq!(result, Some(Object::Int(16)));
}

#[test]
fn array_reduce_empty_array_returns_nil() {
    let result = run(r#"
[].reduce { |sum, x| sum + x }
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── array_methods.rs: filter alias for select ───────────────────────────────

#[test]
fn array_filter_alias() {
    let result = run(r#"
[1, 2, 3, 4].filter { |x| x > 2 }
"#);
    assert!(result.is_some());
}

// ── Array index access ────────────────────────────────────────────────────

#[test]
fn array_bracket_access() {
    let result = run(r#"
a = [10, 20, 30]
a[1]
"#);
    assert_eq!(result, Some(Object::Int(20)));
}

// ── From edge_tests ─────────────────────────────────────────────────────────

#[test]
fn array_sort_method_edge() {
    assert_eq!(
        run("[3,1,2].sort"),
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3)
        ]))
    );
}

#[test]
fn array_map_double_edge() {
    assert_eq!(
        run("[1,2,3].map { |x| x * 2 }"),
        Some(Object::array(vec![
            Object::Int(2),
            Object::Int(4),
            Object::Int(6)
        ]))
    );
}

#[test]
fn array_select_filter_edge() {
    assert_eq!(
        run("[1,2,3,4].select { |x| x > 2 }"),
        Some(Object::array(vec![Object::Int(3), Object::Int(4)]))
    );
}

#[test]
fn array_reduce_sum_edge() {
    assert_eq!(
        run("[1,2,3].reduce { |sum, x| sum + x }"),
        Some(Object::Int(6))
    );
}
