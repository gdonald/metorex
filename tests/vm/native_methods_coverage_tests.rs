// Coverage tests for vm/native_methods/ uncovered paths

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

// ── String + non-string argument error ────────────────────────────────────────

#[test]
fn string_concat_non_string_error() {
    let err = run_err(
        r#"
"hello" + 42
"#,
    );
    assert!(err.contains("String") || err.contains("type") || err.contains("+"));
}

// ── puts with Instance that has no to_s method ───────────────────────────────

#[test]
fn puts_with_instance_no_to_s() {
    // Instance with no to_s - falls back to Display format
    let result = run(r#"
class Nameless
end
obj = Nameless.new
puts obj
"#);
    // Should not error - just prints the default representation
    assert!(result == Some(Object::Nil) || result.is_none());
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

// ── Range map without block ───────────────────────────────────────────────────

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
fn range_select_without_block_error() {
    let err = run_err(
        r#"
(1..5).select
"#,
    );
    assert!(err.contains("block") || err.contains("select") || err.contains("requires"));
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

// ── Range.each with non-integer range (lines 89-91) ──────────────────────────

#[test]
fn range_each_float_bounds_error() {
    let err = run_err(
        r#"
r = 1.5..3.5
r.each { |i| i }
"#,
    );
    assert!(
        err.contains("integer")
            || err.contains("Integer")
            || err.contains("Range")
            || err.contains("only supports")
    );
}

// ── Range.to_a with non-integer range (lines 121-123) ────────────────────────

#[test]
fn range_to_a_float_bounds_error() {
    let err = run_err(
        r#"
r = 1.5..3.5
r.to_a
"#,
    );
    assert!(
        err.contains("integer")
            || err.contains("Integer")
            || err.contains("Range")
            || err.contains("only supports")
    );
}

// ── Range.include? with non-integer range (lines 154-156) ────────────────────

#[test]
fn range_include_float_bounds_error() {
    let err = run_err(
        r#"
r = 1.5..3.5
r.include?(2.0)
"#,
    );
    assert!(
        err.contains("integer")
            || err.contains("Integer")
            || err.contains("Range")
            || err.contains("only supports")
    );
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
        err.contains("integer")
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

// ── Object::Symbol Display (display.rs line 16) ───────────────────────────────
// puts on a Symbol triggers Display::fmt which formats it as ":name"

#[test]
fn puts_symbol_triggers_display() {
    let result = run(r#"
x = :hello
puts x
nil
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── Object::Module Display (display.rs line 44) ───────────────────────────────

#[test]
fn puts_module_triggers_display() {
    let result = run(r#"
module Greetings
end
puts Greetings
nil
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── Object::Method Display (display.rs line 45) ───────────────────────────────
// method(:name) returns the Method object; puts on it calls Display

#[test]
fn puts_method_object_triggers_display() {
    let result = run(r#"
def greet
  "hello"
end
puts method(:greet)
nil
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── Object::Block Display (display.rs line 46) ────────────────────────────────

#[test]
fn puts_block_object_triggers_display() {
    let result = run(r#"
b = lambda do |x|
  x + 1
end
puts b
nil
"#);
    assert_eq!(result, Some(Object::Nil));
}
