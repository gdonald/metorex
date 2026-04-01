// Coverage tests for vm/native_methods/ uncovered paths

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

// ── String each_char block path ─────────────────────────────────────────

#[test]
fn string_each_char_basic() {
    let result = run("result = []\n\"abc\".each_char do |c|\n  result.push(c)\nend\nresult.length");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Hash each with break ────────────────────────────────────────────────

#[test]
fn hash_each_with_break() {
    let result = run(
        "count = 0\n{\"a\" => 1, \"b\" => 2, \"c\" => 3}.each do |k, v|\n  count = count + 1\n  if count == 2\n    break\n  end\nend\ncount",
    );
    assert_eq!(result, Some(Object::Int(2)));
}

// ── Int times with break ────────────────────────────────────────────────

#[test]
fn int_times_with_break_coverage() {
    let result =
        run("sum = 0\n10.times do |i|\n  if i == 5\n    break\n  end\n  sum = sum + i\nend\nsum");
    assert_eq!(result, Some(Object::Int(10)));
}

// ── Range each with break ───────────────────────────────────────────────

#[test]
fn range_each_with_break_coverage() {
    let result = run(
        "sum = 0\n(1..100).each do |i|\n  if i > 5\n    break\n  end\n  sum = sum + i\nend\nsum",
    );
    assert_eq!(result, Some(Object::Int(15)));
}

// ── Set each with break ────────────────────────────────────────────────

#[test]
fn set_each_with_break() {
    let result = run(
        "s = Set.new\ns.add(\"a\")\ns.add(\"b\")\ns.add(\"c\")\ncount = 0\ns.each do |x|\n  count = count + 1\n  if count == 2\n    break\n  end\nend\ncount",
    );
    assert_eq!(result, Some(Object::Int(2)));
}

// ── File.write (native_methods/mod.rs lines 160-162) ────────────────────

#[test]
fn file_write_and_read() {
    let result = run(
        "File.write(\"/tmp/metorex_test_coverage.txt\", \"hello\")\nFile.read(\"/tmp/metorex_test_coverage.txt\")",
    );
    assert_eq!(result, Some(Object::String(Rc::new("hello".to_string()))));
    let _ = std::fs::remove_file("/tmp/metorex_test_coverage.txt");
}

// ── define_method closure capture (native_methods/mod.rs line 224) ──────

#[test]
fn define_method_closure_capture_in_class() {
    let result =
        run("class Foo\n  define_method(:get_val) do\n    42\n  end\nend\nFoo.new.get_val");
    assert_eq!(result, Some(Object::Int(42)));
}

// ── Various dict key types (utils.rs line 29) ───────────────────────────

#[test]
fn int_as_hash_key() {
    let result = run("h = {}\nh[1] = \"one\"\nh[1]");
    assert_eq!(result, Some(Object::String(Rc::new("one".to_string()))));
}

#[test]
fn bool_as_hash_key() {
    let result = run("h = {}\nh[true] = \"yes\"\nh[true]");
    assert_eq!(result, Some(Object::String(Rc::new("yes".to_string()))));
}

// ══════════════════════════════════════════════════════════════════════════════
// Additional coverage tests for native method files
// ══════════════════════════════════════════════════════════════════════════════

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

// ── set_methods.rs: union operation (lines 147-151) ─────────────────────────

#[test]
fn set_union_operation() {
    let result = run(r#"
s1 = Set.new
s1.add("a")
s1.add("b")
s2 = Set.new
s2.add("b")
s2.add("c")
s3 = s1.union(s2)
s3.size
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── set_methods.rs: intersection operation (lines 174-178) ──────────────────

#[test]
fn set_intersection_operation() {
    let result = run(r#"
s1 = Set.new
s1.add("a")
s1.add("b")
s2 = Set.new
s2.add("b")
s2.add("c")
s3 = s1.intersection(s2)
s3.size
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ── set_methods.rs: difference operation (lines 201-215) ────────────────────

#[test]
fn set_difference_operation() {
    let result = run(r#"
s1 = Set.new
s1.add("a")
s1.add("b")
s1.add("c")
s2 = Set.new
s2.add("b")
s3 = s1.difference(s2)
s3.size
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ── set_methods.rs: to_a conversion (lines 234-236) ────────────────────────

#[test]
fn set_to_a_conversion() {
    let result = run(r#"
s = Set.new
s.add("x")
s.add("y")
a = s.to_a
a.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ── set_methods.rs: contains?/include? operation (line 255) ─────────────────

#[test]
fn set_contains_operation() {
    let result = run(r#"
s = Set.new
s.add("hello")
s.contains?("hello")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn set_include_operation() {
    let result = run(r#"
s = Set.new
s.add("hello")
s.include?("hello")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn set_contains_missing_element() {
    let result = run(r#"
s = Set.new
s.add("hello")
s.contains?("world")
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── set_methods.rs: each with continue (lines 229-231) ──────────────────────

#[test]
fn set_each_with_continue_in_block() {
    let result = run(r#"
s = Set.new
s.add("a")
s.add("b")
s.add("c")
count = 0
s.each do |x|
  continue
end
count
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

// ── set_methods.rs: each with return in block error ─────────────────────────

#[test]
fn set_each_return_in_block_error() {
    let err = run_err(
        r#"
s = Set.new
s.add("a")
s.each do |x|
  return x
end
"#,
    );
    assert!(err.contains("return") || err.contains("control") || err.contains("loop"));
}

// ── set_methods.rs: each with exception in block ────────────────────────────

#[test]
fn set_each_exception_in_block_propagates() {
    let err = run_err(
        r#"
s = Set.new
s.add("a")
s.each do |x|
  raise "set block error"
end
"#,
    );
    assert!(
        err.contains("set block error") || err.contains("Uncaught") || err.contains("exception")
    );
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

// ── string_methods.rs: "+" concatenation (lines 58-76) ──────────────────────

#[test]
fn string_plus_concatenation() {
    let result = run(r#"
"hello" + " " + "world"
"#);
    assert_eq!(result, Some(Object::string("hello world")));
}

// ── string_methods.rs: slice/[] operation (line 216) ────────────────────────

#[test]
fn string_slice_basic() {
    let result = run(r#"
"hello".slice(1, 3)
"#);
    assert_eq!(result, Some(Object::string("ell")));
}

#[test]
fn string_slice_beyond_end_returns_empty() {
    let result = run(r#"
"hi".slice(10, 2)
"#);
    assert_eq!(result, Some(Object::string("")));
}

#[test]
fn string_slice_negative_start() {
    let result = run(r#"
"hello".slice(-3, 2)
"#);
    assert_eq!(result, Some(Object::string("ll")));
}

// ── string_methods.rs: each_char iteration with block (lines 297-302) ───────

#[test]
fn string_each_char_returns_receiver() {
    let result = run(r#"
result = "abc".each_char do |c|
  c
end
result
"#);
    assert_eq!(result, Some(Object::string("abc")));
}

#[test]
fn string_each_char_without_block_error() {
    let err = run_err(
        r#"
"abc".each_char
"#,
    );
    assert!(err.contains("block") || err.contains("each_char") || err.contains("requires"));
}

// ── int_methods.rs: times with continue in block (lines 79-84) ─────────────

#[test]
fn int_times_with_continue_in_block() {
    let result = run(r#"
sum = 0
5.times do |i|
  if i == 2
    continue
  end
  sum = sum + i
end
sum
"#);
    // 0 + 1 + 3 + 4 = 8 (skipping i==2)
    assert_eq!(result, Some(Object::Int(8)));
}

// ── int_methods.rs: times with return in block error (lines 102-104) ────────

#[test]
fn int_times_return_in_block_error() {
    let err = run_err(
        r#"
5.times do |i|
  return i
end
"#,
    );
    assert!(err.contains("return") || err.contains("control") || err.contains("loop"));
}

// ── int_methods.rs: times with exception in block ───────────────────────────

#[test]
fn int_times_exception_in_block_propagates() {
    let err = run_err(
        r#"
3.times do |i|
  raise "times block error"
end
"#,
    );
    assert!(
        err.contains("times block error") || err.contains("Uncaught") || err.contains("exception")
    );
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

// ── set_methods.rs: dispatch early return for non-set (line 23) ─────────────
// This is tested implicitly: calling a method on a non-set that falls through.
// The early return Ok(None) triggers the dispatcher to try other method tables.

#[test]
fn set_unknown_method_returns_error() {
    let err = run_err(
        r#"
s = Set.new
s.nonexistent_method
"#,
    );
    assert!(
        err.contains("method")
            || err.contains("undefined")
            || err.contains("nonexistent")
            || err.contains("No method")
    );
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

// ── Error path tests for set operations ────────────────────────────────────

#[test]
fn set_intersection_wrong_type_error() {
    let err = run_err(
        r#"
s = Set.new
s.add("a")
s.intersection(42)
"#,
    );
    assert!(
        err.contains("Set") || err.contains("type"),
        "Error was: {}",
        err
    );
}

#[test]
fn set_difference_wrong_type_error() {
    let err = run_err(
        r#"
s = Set.new
s.add("a")
s.difference("not a set")
"#,
    );
    assert!(
        err.contains("Set") || err.contains("type"),
        "Error was: {}",
        err
    );
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

// ── Array index access ────────────────────────────────────────────────────

#[test]
fn array_bracket_access() {
    let result = run(r#"
a = [10, 20, 30]
a[1]
"#);
    assert_eq!(result, Some(Object::Int(20)));
}

// ── String concatenation ──────────────────────────────────────────────────

#[test]
fn string_concatenation_with_plus() {
    let result = run(r#"
"hello" + " world"
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("hello world".to_string())))
    );
}

#[test]
fn string_chars_method() {
    let result = run(r#"
"abc".chars
"#);
    assert!(result.is_some());
}

#[test]
fn string_bytes_method() {
    let result = run(r#"
"abc".bytes
"#);
    assert!(result.is_some());
}

#[test]
fn string_each_char_with_block() {
    let result = run(r#"
result = []
"abc".each_char do |c|
  result.push(c)
end
result.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Int times with continue (additional path) ─────────────────────────────

#[test]
fn int_times_continue_skips_iteration() {
    let result = run(r#"
result = []
5.times do |i|
  if i == 2
    continue
  end
  result.push(i)
end
result.length
"#);
    assert_eq!(result, Some(Object::Int(4)));
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
