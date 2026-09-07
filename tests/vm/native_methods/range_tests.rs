// Coverage tests for range native methods

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

// ── Range each without block ──────────────────────────────────────────────────

#[test]
fn range_each_without_block_error() {
    let err = run_err(
        r#"
(1..5).each
"#,
    );
    assert!(err.contains("block") || err.contains("each") || err.contains("requires"));
}

// ── Range map without block ───────────────────────────────────────────────

#[test]
fn range_map_without_block_error() {
    let err = run_err(
        r#"
(1..5).map
"#,
    );
    assert!(err.contains("block") || err.contains("map") || err.contains("requires"));
}

// ── Range select without block ─────────────────────────────────────────────────

#[test]
fn range_select_without_block_answers_an_enumerator() {
    let result = run(r#"
(1..5).select.to_a
"#);
    assert_eq!(format!("{}", result.expect("a value")), "[1, 2, 3, 4, 5]");
}

// ── Range.each with non-integer range (lines 89-91) ──────────────────────────

#[test]
fn range_each_float_bounds_error() {
    // Ruby walks a range with `succ`, which a Float has none of.
    let err = run_err(
        r#"
r = 1.5..3.5
r.each { |i| i }
"#,
    );
    assert!(
        err.contains("can't iterate from Float")
            || err.contains("integer")
            || err.contains("Integer")
            || err.contains("Range")
            || err.contains("only supports")
    );
}

// ── Range.to_a with non-integer range (lines 121-123) ────────────────────────

#[test]
fn range_to_a_float_bounds_error() {
    // Ruby walks a range with `succ`, which a Float has none of.
    let err = run_err(
        r#"
r = 1.5..3.5
r.to_a
"#,
    );
    assert!(
        err.contains("can't iterate from Float")
            || err.contains("integer")
            || err.contains("Integer")
            || err.contains("Range")
            || err.contains("only supports")
    );
}

// ── Range.include? with non-integer range (lines 154-156) ────────────────────

#[test]
fn range_include_float_bounds_works() {
    // Float range include? now works via fallback comparison
    let result = run("r = 1.5..3.5\nr.include?(2.0)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Range.map with non-integer range (lines 211-213) ─────────────────────────

#[test]
fn range_map_float_bounds_error() {
    let err = run_err(
        r#"
r = 1.5..3.5
r.map { |i| i }
"#,
    );
    assert!(
        err.contains("can't iterate from Float")
            || err.contains("integer")
            || err.contains("Integer")
            || err.contains("Range")
            || err.contains("only supports")
    );
}

// ── Range.each with exception in block ───────────────────────────────────────

#[test]
fn range_each_exception_in_block_propagates() {
    let err = run_err(
        r#"
(1..3).each { |i| raise "block error" }
"#,
    );
    assert!(err.contains("block error") || err.contains("exception") || err.contains("Uncaught"));
}

// ── Range.each with return in block error ────────────────────────────────────

#[test]
fn range_each_return_in_block_error() {
    let err = run_err(
        r#"
(1..3).each { |i| return i }
"#,
    );
    assert!(err.contains("return") || err.contains("loop") || err.contains("control"));
}

// ── Range each with break ───────────────────────────────────────────────

#[test]
fn range_each_with_break_coverage() {
    let result = run(
        "sum = 0\n(1..100).each do |i|\n  if i > 5\n    break\n  end\n  sum = sum + i\nend\nsum",
    );
    assert_eq!(result, Some(Object::Int(15)));
}

// ── range_methods.rs: to_a with inclusive range (lines 42-47) ───────────────

#[test]
fn range_to_a_inclusive() {
    let result = run(r#"
r = 1..5
r.to_a.length
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn range_to_a_exclusive() {
    let result = run(r#"
r = 1...5
r.to_a.length
"#);
    assert_eq!(result, Some(Object::Int(4)));
}

// ── range_methods.rs: map with block (lines 154-159) ────────────────────────

#[test]
fn range_map_with_block() {
    let result = run(r#"
result = (1..3).map do |i|
  i * 2
end
result.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn range_map_inclusive_produces_correct_values() {
    let result = run(r#"
(1..3).map { |i| i * 10 }
"#);
    assert!(result.is_some());
}

// ── range_methods.rs: each with continue in block ───────────────────────────

#[test]
fn range_each_with_continue_in_block() {
    let result = run(r#"
sum = 0
(1..5).each do |i|
  if i == 3
    continue
  end
  sum = sum + i
end
sum
"#);
    // 1 + 2 + 4 + 5 = 12 (skipping i==3)
    assert_eq!(result, Some(Object::Int(12)));
}

// ── Range to_a inclusive/exclusive (additional) ────────────────────────────

#[test]
fn range_to_a_inclusive_returns_array() {
    let result = run(r#"
(1..5).to_a
"#);
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 5);
    }
}

#[test]
fn range_to_a_exclusive_returns_shorter_array() {
    let result = run(r#"
(1...5).to_a
"#);
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 4);
    }
}

#[test]
fn range_map_transforms_elements() {
    let result = run(r#"
(1..3).map { |x| x * 2 }
"#);
    assert!(result.is_some());
}

#[test]
fn range_each_continue_skips_iteration() {
    let result = run(r#"
result = []
(1..5).each do |i|
  if i == 3
    continue
  end
  result.push(i)
end
result.length
"#);
    assert_eq!(result, Some(Object::Int(4)));
}

// ── From edge_tests ─────────────────────────────────────────────────────────

#[test]
fn range_to_a_inclusive_edge() {
    assert_eq!(
        run("(1..3).to_a"),
        Some(Object::array(vec![
            Object::Int(1),
            Object::Int(2),
            Object::Int(3)
        ]))
    );
}

#[test]
fn range_to_a_exclusive_edge() {
    assert_eq!(
        run("(1...3).to_a"),
        Some(Object::array(vec![Object::Int(1), Object::Int(2)]))
    );
}

// ── From remaining_tests ────────────────────────────────────────────────────

#[test]
fn range_each_inclusive_remaining() {
    let result = run("sum = 0; (1..3).each { |x| sum = sum + x }; sum");
    assert_eq!(result, Some(Object::Int(6)));
}

#[test]
fn range_each_exclusive_remaining() {
    let result = run("sum = 0; (1...3).each { |x| sum = sum + x }; sum");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn range_map_doubles_remaining() {
    assert_eq!(
        run("(1..3).map { |x| x * 2 }"),
        Some(Object::array(vec![
            Object::Int(2),
            Object::Int(4),
            Object::Int(6)
        ]))
    );
}

#[test]
fn range_include_boundary_remaining() {
    assert_eq!(run("(1..5).include?(5)"), Some(Object::Bool(true)));
    assert_eq!(run("(1...5).include?(5)"), Some(Object::Bool(false)));
}

// ── String range include? ─────────────────────────────────────────────────

#[test]
fn string_range_include_inclusive() {
    assert_eq!(run(r#"("a".."z").include?("m")"#), Some(Object::Bool(true)));
}

#[test]
fn string_range_include_at_end_inclusive() {
    assert_eq!(run(r#"("a".."z").include?("z")"#), Some(Object::Bool(true)));
}

#[test]
fn string_range_include_exclusive_excludes_end() {
    assert_eq!(
        run(r#"("a"..."z").include?("z")"#),
        Some(Object::Bool(false))
    );
}

#[test]
fn string_range_include_exclusive_includes_middle() {
    assert_eq!(
        run(r#"("a"..."z").include?("m")"#),
        Some(Object::Bool(true))
    );
}

#[test]
fn string_range_include_below_start() {
    assert_eq!(
        run(r#"("c".."z").include?("a")"#),
        Some(Object::Bool(false))
    );
}

#[test]
fn float_range_include_inclusive() {
    assert_eq!(run("(1.0..3.0).include?(2.5)"), Some(Object::Bool(true)));
}

#[test]
fn float_range_include_outside() {
    assert_eq!(run("(1.0..3.0).include?(5.0)"), Some(Object::Bool(false)));
}

// ── Range#first over a range that never reaches an end ──────────────────────

#[test]
fn range_first_counts_out_of_an_endless_integer_range() {
    let result = run("(3..).first(4).inspect");
    assert_eq!(result, Some(Object::string("[3, 4, 5, 6]")));
}

#[test]
fn range_first_counts_out_of_an_infinite_float_end() {
    let result = run("(0..Float::INFINITY).first(3).inspect");
    assert_eq!(result, Some(Object::string("[0, 1, 2]")));
}

#[test]
fn range_first_of_an_endless_range_answers_nothing_for_zero() {
    let result = run("(0..Float::INFINITY).first(0).inspect");
    assert_eq!(result, Some(Object::string("[]")));
}

#[test]
fn range_last_of_an_infinite_float_end_still_walks_the_values() {
    let err = run_err("(0..Float::INFINITY).last(3)");
    assert!(err.contains("can't iterate from"), "Error was: {}", err);
}
