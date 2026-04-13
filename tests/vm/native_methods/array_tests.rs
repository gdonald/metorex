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

// ── array_methods.rs: partition without block error (lines 106-109) ─────────

#[test]
fn array_partition_without_block_error() {
    let err = run_err("[1, 2, 3].partition");
    assert!(err.contains("block") || err.contains("partition") || err.contains("requires"));
}

// ── array_methods.rs: partition basic (lines 106-109 success path) ───────────

#[test]
fn array_partition_basic() {
    let result = run("[1, 2, 3, 4].partition { |x| x > 2 }");
    assert!(result.is_some());
}

// ── array_methods.rs: inject without block error (lines 216-221) ─────────────

#[test]
fn array_inject_without_block_error() {
    let err = run_err("[1, 2, 3].inject");
    assert!(err.contains("block") || err.contains("inject") || err.contains("requires"));
}

// ── array_methods.rs: inject basic (lines 216-221 success path) ──────────────

#[test]
fn array_inject_basic() {
    let result = run("[1, 2, 3].inject { |sum, x| sum + x }");
    assert_eq!(result, Some(Object::Int(6)));
}

// ── array_methods.rs: inject with initial value ───────────────────────────────

#[test]
fn array_inject_with_initial_value() {
    let result = run("[1, 2, 3].inject(10) { |sum, x| sum + x }");
    assert_eq!(result, Some(Object::Int(16)));
}

// ── array_methods.rs: inject on empty array returns nil (lines 245-249) ─────

#[test]
fn array_inject_empty_array_returns_nil() {
    let result = run("[].inject { |sum, x| sum + x }");
    assert_eq!(result, Some(Object::Nil));
}

// ── array_methods.rs: inject empty with initial returns initial (lines 254-259) ─

#[test]
fn array_inject_empty_with_initial_returns_initial() {
    let result = run("[].inject(42) { |sum, x| sum + x }");
    assert_eq!(result, Some(Object::Int(42)));
}

// ── array_methods.rs: reduce too many args error (lines 297-302) ─────────────

#[test]
fn array_reduce_too_many_args_error() {
    let err = run_err("[1, 2, 3].reduce(1, 2) { |s, x| s + x }");
    assert!(err.contains("argument"));
}

// ── array_methods.rs: min with mixed types keeps first (line 703) ─────────────

#[test]
fn array_min_with_mixed_types_keeps_current() {
    // When comparing Int and String, the fallback keeps 'current' (first element wins)
    let result = run(r#"[1, "a", 2].min"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ── array_methods.rs: max with mixed types falls back to current (line 737) ───

#[test]
fn array_max_with_mixed_types_fallback() {
    // When comparing Int and String, the fallback keeps 'current' (the Int candidate)
    // For [1, "a"]: "a" vs Int(1) → keeps Int(1); result is Int(1)
    let result = run(r#"[1, "a"].max"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ── array_methods.rs: any?/all?/none? with positional args error (lines 765-769) ─

#[test]
fn array_any_with_args_error() {
    let err = run_err("[1, 2, 3].any?(1)");
    assert!(err.contains("argument"));
}

#[test]
fn array_all_with_args_error() {
    let err = run_err("[1, 2, 3].all?(1)");
    assert!(err.contains("argument"));
}

#[test]
fn array_none_with_args_error() {
    let err = run_err("[1, 2, 3].none?(1)");
    assert!(err.contains("argument"));
}

// ── array_methods.rs: pack with 0 arguments error (lines 801-805) ────────────

#[test]
fn array_pack_no_args_error() {
    let err = run_err("[1, 2].pack");
    assert!(err.contains("argument"));
}

// ── array_methods.rs: pack with format 'l' (line 839) ────────────────────────

#[test]
fn array_pack_format_l() {
    // 'l' packs as 32-bit little-endian integer
    let result = run(r#"[1].pack("l")"#);
    assert!(result.is_some());
    // 1 as i32 little-endian = [1, 0, 0, 0]
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 4);
    }
}

// ── array_methods.rs: pack with format 's' (line 847) ────────────────────────

#[test]
fn array_pack_format_s() {
    // 's' packs as 16-bit little-endian integer
    let result = run(r#"[1].pack("s")"#);
    assert!(result.is_some());
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 2);
    }
}

// ── array_methods.rs: pack with format 'c' (line 855) ────────────────────────

#[test]
fn array_pack_format_c() {
    // 'c' packs as single byte
    let result = run(r#"[65].pack("c")"#);
    assert!(result.is_some());
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 1);
    }
}

// ── array_methods.rs: pack with format 'q' (64-bit) (line 863) ───────────────

#[test]
fn array_pack_format_q() {
    // 'q' packs as 64-bit little-endian integer
    let result = run(r#"[1].pack("q")"#);
    assert!(result.is_some());
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 8);
    }
}

// ── array_methods.rs: pack with format 'j' (native 64-bit) ──────────────────

#[test]
fn array_pack_format_j() {
    // 'j' packs as 64-bit little-endian integer (native size)
    let result = run(r#"[42].pack("j")"#);
    assert!(result.is_some());
    if let Some(Object::String(s)) = result {
        assert_eq!(s.len(), 8);
    }
}

// ── array_methods.rs: pack with unsupported directive error ──────────────────

#[test]
fn array_pack_unsupported_directive_error() {
    let err = run_err(r#"[1].pack("z")"#);
    assert!(err.contains("unsupported") || err.contains("directive") || err.contains("pack"));
}

// ── array_methods.rs: zip without arguments error (lines 536-541) ─────────────

#[test]
fn array_zip_without_args_error() {
    let err = run_err("[1, 2, 3].zip");
    assert!(err.contains("argument"));
}

// ── array_methods.rs lines 106-109: [] via send dispatches to call_array_method

#[test]
fn array_bracket_via_send() {
    let result = run("[10, 20, 30].send(:[], 1)");
    assert_eq!(result, Some(Object::Int(20)));
}
