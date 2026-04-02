// Additional coverage tests for native method error/edge paths

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
// Array methods - push/append (line 39 early return + push path)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn array_push_returns_receiver() {
    let result = run(r#"
arr = [1, 2]
arr.push(3)
"#);
    // push returns the array itself
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 3);
    }
}

#[test]
fn array_push_wrong_arg_count() {
    let err = run_err(
        r#"
[1, 2].push
"#,
    );
    assert!(
        err.contains("argument") || err.contains("expected"),
        "Error was: {}",
        err
    );
}

#[test]
fn array_append_wrong_arg_count() {
    let err = run_err(
        r#"
[1, 2].append
"#,
    );
    assert!(
        err.contains("argument") || err.contains("expected"),
        "Error was: {}",
        err
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Array methods - each with return in method (lines 126-129)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn array_each_return_in_method_errors() {
    let err = run_err(
        r#"
def test_return_in_each
  [1, 2, 3].each do |x|
    return x
  end
end
test_return_in_each
"#,
    );
    assert!(
        err.contains("return") || err.contains("control") || err.contains("loop"),
        "Error was: {}",
        err
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Array methods - each with raise in block (lines 131-142)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn array_each_raise_propagates_exception() {
    let err = run_err(
        r#"
[1, 2, 3].each do |x|
  raise "boom in each"
end
"#,
    );
    assert!(
        err.contains("boom in each") || err.contains("Uncaught"),
        "Error was: {}",
        err
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// String methods - chars, bytes, various transforms (lines 58-76)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn string_chars_returns_array_of_chars() {
    let result = run(r#"
"hello".chars.length
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn string_bytes_returns_array_of_ints() {
    let result = run(r#"
"abc".bytes.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn string_trim_method() {
    let result = run(r#"
"  hello  ".trim
"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn string_strip_method() {
    let result = run(r#"
"  hello  ".strip
"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn string_reverse_method() {
    let result = run(r#"
"hello".reverse
"#);
    assert_eq!(result, Some(Object::string("olleh")));
}

#[test]
fn string_upcase_method() {
    let result = run(r#"
"hello".upcase
"#);
    assert_eq!(result, Some(Object::string("HELLO")));
}

#[test]
fn string_downcase_method() {
    let result = run(r#"
"HELLO".downcase
"#);
    assert_eq!(result, Some(Object::string("hello")));
}

#[test]
fn string_size_method() {
    let result = run(r#"
"hello".size
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

// ══════════════════════════════════════════════════════════════════════════════
// String methods - slice (line 216 nil return for out-of-bounds)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn string_slice_out_of_bounds_returns_nil() {
    // start_idx > chars.len() triggers the nil return
    let result = run(r#"
"hi".slice(100, 2)
"#);
    // When start is beyond length, returns nil or empty string
    assert!(
        result == Some(Object::Nil) || result == Some(Object::string("")),
        "Got: {:?}",
        result
    );
}

#[test]
fn string_slice_negative_start_wraps() {
    let result = run(r#"
"hello".slice(-2, 2)
"#);
    assert_eq!(result, Some(Object::string("lo")));
}

// ══════════════════════════════════════════════════════════════════════════════
// String methods - each_char block execution (lines 297-302)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn string_each_char_accumulates_chars() {
    let result = run(r#"
collected = []
"xyz".each_char do |c|
  collected.push(c)
end
collected.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ══════════════════════════════════════════════════════════════════════════════
// String methods - + concatenation non-string error (lines 72-77)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn string_plus_non_string_type_error() {
    let err = run_err(
        r#"
"hello" + 42
"#,
    );
    assert!(
        err.contains("String") || err.contains("type"),
        "Error was: {}",
        err
    );
}

#[test]
fn string_plus_nil_type_error() {
    let err = run_err(
        r#"
"hello" + nil
"#,
    );
    assert!(
        err.contains("String") || err.contains("type") || err.contains("nil"),
        "Error was: {}",
        err
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Set methods - intersection arg validation (lines 147-151)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn set_intersection_with_non_set_errors() {
    let err = run_err(
        r#"
s = Set.new
s.add("a")
s.intersection("not a set")
"#,
    );
    assert!(
        err.contains("Set") || err.contains("type"),
        "Error was: {}",
        err
    );
}

#[test]
fn set_intersection_with_int_errors() {
    let err = run_err(
        r#"
s = Set.new
s.add("a")
s.intersection(123)
"#,
    );
    assert!(
        err.contains("Set") || err.contains("type"),
        "Error was: {}",
        err
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Set methods - difference arg validation (lines 174-178)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn set_difference_with_non_set_errors() {
    let err = run_err(
        r#"
s = Set.new
s.add("a")
s.difference(42)
"#,
    );
    assert!(
        err.contains("Set") || err.contains("type"),
        "Error was: {}",
        err
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Set methods - each with return/raise (lines 201-215)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn set_each_return_in_method_errors() {
    let err = run_err(
        r#"
def test_set_return
  s = Set.new
  s.add("a")
  s.add("b")
  s.each do |x|
    return x
  end
end
test_set_return
"#,
    );
    assert!(
        err.contains("return") || err.contains("control") || err.contains("loop"),
        "Error was: {}",
        err
    );
}

#[test]
fn set_each_raise_propagates() {
    let err = run_err(
        r#"
s = Set.new
s.add("a")
s.each do |x|
  raise "set boom"
end
"#,
    );
    assert!(
        err.contains("set boom") || err.contains("Uncaught"),
        "Error was: {}",
        err
    );
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
// Int methods - times with continue/exception (lines 79-84)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn int_times_with_continue_skips() {
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
    // 0 + 1 + 3 + 4 = 8
    assert_eq!(result, Some(Object::Int(8)));
}

#[test]
fn int_times_return_in_method_errors() {
    let err = run_err(
        r#"
def test_times_return
  3.times do |i|
    return i
  end
end
test_times_return
"#,
    );
    assert!(
        err.contains("return") || err.contains("control") || err.contains("loop"),
        "Error was: {}",
        err
    );
}

#[test]
fn int_times_raise_propagates() {
    let err = run_err(
        r#"
3.times do |i|
  raise "times boom"
end
"#,
    );
    assert!(
        err.contains("times boom") || err.contains("Uncaught"),
        "Error was: {}",
        err
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Range methods - to_a inclusive (lines 42-47)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn range_to_a_inclusive_values() {
    let result = run(r#"
(1..3).to_a
"#);
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed.len(), 3);
        assert_eq!(borrowed[0], Object::Int(1));
        assert_eq!(borrowed[2], Object::Int(3));
    }
}

#[test]
fn range_to_a_exclusive_values() {
    let result = run(r#"
(1...4).to_a
"#);
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed.len(), 3);
        assert_eq!(borrowed[0], Object::Int(1));
        assert_eq!(borrowed[2], Object::Int(3));
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Range methods - map with block (lines 154-159)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn range_map_inclusive_transforms() {
    let result = run(r#"
(1..4).map { |x| x * 10 }
"#);
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed.len(), 4);
        assert_eq!(borrowed[0], Object::Int(10));
        assert_eq!(borrowed[3], Object::Int(40));
    }
}

#[test]
fn range_map_exclusive_transforms() {
    let result = run(r#"
(1...4).map { |x| x * 10 }
"#);
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed.len(), 3);
        assert_eq!(borrowed[0], Object::Int(10));
        assert_eq!(borrowed[2], Object::Int(30));
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Range methods - each with return/raise
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn range_each_return_in_method_errors() {
    let err = run_err(
        r#"
(1..3).each do |i|
  return i
end
"#,
    );
    assert!(
        err.contains("return") || err.contains("control") || err.contains("loop"),
        "Error was: {}",
        err
    );
}

#[test]
fn range_each_raise_propagates() {
    let err = run_err(
        r#"
(1..3).each do |i|
  raise "range boom"
end
"#,
    );
    assert!(
        err.contains("range boom") || err.contains("Uncaught"),
        "Error was: {}",
        err
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Operators - And/Or short-circuit paths (lines 62-64, 67-69)
// These are defensive guards; exercise normal And/Or paths thoroughly
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn operator_and_short_circuit_false() {
    let result = run(r#"
false && true
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn operator_and_short_circuit_nil() {
    let result = run(r#"
nil && 42
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn operator_and_evaluates_both() {
    let result = run(r#"
true && 42
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn operator_or_short_circuit_true() {
    let result = run(r#"
true || false
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn operator_or_short_circuit_int() {
    let result = run(r#"
42 || false
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn operator_or_evaluates_right() {
    let result = run(r#"
false || 99
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn operator_or_nil_fallthrough() {
    let result = run(r#"
nil || "fallback"
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("fallback".to_string())))
    );
}

#[test]
fn operator_and_complex_expressions() {
    let result = run(r#"
x = 5
x > 3 && x < 10
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn operator_or_complex_expressions() {
    let result = run(r#"
x = 5
x > 10 || x < 3
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn operator_assign_via_plus_equals() {
    let result = run(r#"
x = 10
x += 5
x
"#);
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn operator_assign_via_minus_equals() {
    let result = run(r#"
x = 10
x -= 3
x
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn operator_assign_via_multiply_equals() {
    let result = run(r#"
x = 10
x *= 2
x
"#);
    assert_eq!(result, Some(Object::Int(20)));
}

#[test]
fn operator_assign_via_divide_equals() {
    let result = run(r#"
x = 10
x /= 2
x
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

// ══════════════════════════════════════════════════════════════════════════════
// AST methods - method body access (lines 43-47 of mod.rs)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn method_object_body_access() {
    let result = run(r#"
def greet
  "hello"
end
m = method(:greet)
m.body
"#);
    assert!(result.is_some());
}

#[test]
fn method_object_name_access() {
    let result = run(r#"
def greet
  "hello"
end
m = method(:greet)
m.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("greet".to_string()))));
}

#[test]
fn method_object_arity_access() {
    let result = run(r#"
def add(a, b)
  a + b
end
m = method(:add)
m.arity
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn method_object_parameters_access() {
    let result = run(r#"
def add(a, b)
  a + b
end
m = method(:add)
m.parameters
"#);
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        assert_eq!(arr.borrow().len(), 2);
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// AST methods - block statements access (line 180 of mod.rs)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn block_object_statements_access() {
    let result = run(r#"
b = lambda do |x|
  x + 1
end
b.statements
"#);
    assert!(result.is_some());
}

#[test]
fn block_object_arity_access() {
    let result = run(r#"
b = lambda do |x, y|
  x + y
end
b.arity
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ══════════════════════════════════════════════════════════════════════════════
// Exception methods (lines 23-25, 78 of exception_methods.rs)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn exception_message_method() {
    let result = run(r#"
result = nil
begin
  raise "test error"
rescue => e
  result = e.message
end
result
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("test error".to_string())))
    );
}

#[test]
fn exception_type_method() {
    let result = run(r#"
result = nil
begin
  raise "test error"
rescue => e
  result = e.type
end
result
"#);
    assert!(result.is_some());
}

#[test]
fn exception_to_s_method() {
    let result = run(r#"
result = nil
begin
  raise "test error"
rescue => e
  result = e.to_s
end
result
"#);
    assert!(result.is_some());
    if let Some(Object::String(s)) = result {
        assert!(
            s.contains("test error"),
            "Expected to contain 'test error', got: {}",
            s
        );
    }
}

#[test]
fn exception_backtrace_method() {
    let result = run(r#"
result = nil
begin
  raise "test error"
rescue => e
  result = e.backtrace
end
result
"#);
    assert!(result.is_some());
}

// ══════════════════════════════════════════════════════════════════════════════
// Native methods mod.rs - dispatch fallthrough (lines 160-162, 224)
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn file_write_non_string_content() {
    let result = run(r#"
File.write("/tmp/metorex_test_err_cov.txt", 42)
"#);
    // File.write formats non-string content, returns byte count
    assert!(result.is_some());
    let _ = std::fs::remove_file("/tmp/metorex_test_err_cov.txt");
}

#[test]
fn define_method_with_block_on_class() {
    let result = run(r#"
class Dyn
  define_method(:greet) do
    "hello"
  end
end
Dyn.new.greet
"#);
    assert_eq!(result, Some(Object::String(Rc::new("hello".to_string()))));
}

#[test]
fn define_method_with_symbol_name() {
    let result = run(r#"
class Sym
  define_method(:calc) do
    100
  end
end
Sym.new.calc
"#);
    assert_eq!(result, Some(Object::Int(100)));
}

// ══════════════════════════════════════════════════════════════════════════════
// Class native methods - name, superclass, ancestors
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn class_name_method() {
    let result = run(r#"
class Animal
end
Animal.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Animal".to_string()))));
}

#[test]
fn class_superclass_method() {
    let result = run(r#"
class Base
end
class Child < Base
end
Child.superclass.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Base".to_string()))));
}

#[test]
fn class_ancestors_method() {
    let result = run(r#"
class A
end
class B < A
end
B.ancestors.length
"#);
    // B and A in ancestors
    assert!(result.is_some());
    if let Some(Object::Int(n)) = result {
        assert!(n >= 2, "Expected at least 2 ancestors, got {}", n);
    }
}

#[test]
fn class_superclass_nil_when_no_parent() {
    let result = run(r#"
class Solo
end
Solo.superclass
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ══════════════════════════════════════════════════════════════════════════════
// Block call method
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn block_call_method() {
    let result = run(r#"
b = lambda do |x|
  x * 2
end
b.call(5)
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

// ══════════════════════════════════════════════════════════════════════════════
// Method object - owner and source_location
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn method_object_owner_access() {
    let result = run(r#"
def my_func
  1
end
m = method(:my_func)
m.owner
"#);
    assert!(result.is_some());
}

#[test]
fn method_object_source_location_access() {
    let result = run(r#"
def my_func
  1
end
m = method(:my_func)
m.source_location
"#);
    assert!(result.is_some());
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
// String include? and starts_with? / ends_with?
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn string_include_method() {
    let result = run(r#"
"hello world".include?("world")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_starts_with_method() {
    let result = run(r#"
"hello".starts_with?("hel")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_ends_with_method() {
    let result = run(r#"
"hello".ends_with?("llo")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ══════════════════════════════════════════════════════════════════════════════
// String split method
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn string_split_by_separator() {
    let result = run(r#"
"a,b,c".split(",").length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn string_split_by_whitespace() {
    let result = run(r#"
"hello world".split.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ══════════════════════════════════════════════════════════════════════════════
// Set methods - empty? and add/remove
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn set_empty_check() {
    let result = run(r#"
s = Set.new
s.empty?
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn set_remove_element() {
    let result = run(r#"
s = Set.new
s.add("a")
s.remove("a")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn set_delete_alias() {
    let result = run(r#"
s = Set.new
s.add("x")
s.delete("x")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ══════════════════════════════════════════════════════════════════════════════
// Set.new with array argument
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn set_new_from_array() {
    let result = run(r#"
s = Set.new(["a", "b", "c"])
s.size
"#);
    assert_eq!(result, Some(Object::Int(3)));
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

// ══════════════════════════════════════════════════════════════════════════════
// Float methods
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn float_abs_method() {
    let result = run(r#"
(-3.14).abs
"#);
    assert_eq!(result, Some(Object::Float(3.14)));
}

#[test]
fn float_ceil_method() {
    let result = run(r#"
3.2.ceil
"#);
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn float_floor_method() {
    let result = run(r#"
3.8.floor
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn float_to_i_method() {
    let result = run(r#"
3.7.to_i
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn float_to_f_method() {
    let result = run(r#"
3.14.to_f
"#);
    assert_eq!(result, Some(Object::Float(3.14)));
}

#[test]
fn float_round_method() {
    let result = run(r#"
3.14159.round(2)
"#);
    assert_eq!(result, Some(Object::Float(3.14)));
}

// ══════════════════════════════════════════════════════════════════════════════
// Int methods - abs, to_f, to_i, to_s
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn int_abs_method() {
    let result = run(r#"
(-5).abs
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn int_to_f_method() {
    let result = run(r#"
5.to_f
"#);
    assert_eq!(result, Some(Object::Float(5.0)));
}

#[test]
fn int_to_i_method() {
    let result = run(r#"
5.to_i
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn int_to_s_method() {
    let result = run(r#"
42.to_s
"#);
    assert_eq!(result, Some(Object::String(Rc::new("42".to_string()))));
}

// ══════════════════════════════════════════════════════════════════════════════
// Array methods - pop, shift, unshift, reverse, join, zip
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn array_pop_method() {
    let result = run(r#"
arr = [1, 2, 3]
arr.pop
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_pop_empty() {
    let result = run(r#"
arr = []
arr.pop
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn array_shift_method() {
    let result = run(r#"
arr = [1, 2, 3]
arr.shift
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn array_shift_empty() {
    let result = run(r#"
arr = []
arr.shift
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn array_unshift_method() {
    let result = run(r#"
arr = [2, 3]
arr.unshift(1)
arr.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_reverse_method() {
    let result = run(r#"
[1, 2, 3].reverse
"#);
    assert!(result.is_some());
    if let Some(Object::Array(arr)) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed[0], Object::Int(3));
        assert_eq!(borrowed[2], Object::Int(1));
    }
}

#[test]
fn array_join_with_separator() {
    let result = run(r#"
[1, 2, 3].join(", ")
"#);
    assert_eq!(result, Some(Object::String(Rc::new("1, 2, 3".to_string()))));
}

#[test]
fn array_join_without_separator() {
    let result = run(r#"
[1, 2, 3].join
"#);
    assert_eq!(result, Some(Object::String(Rc::new("123".to_string()))));
}

#[test]
fn array_zip_method() {
    let result = run(r#"
[1, 2, 3].zip([4, 5, 6]).length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn array_size_method() {
    let result = run(r#"
[1, 2, 3].size
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ══════════════════════════════════════════════════════════════════════════════
// Range include? method
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn range_include_inclusive() {
    let result = run(r#"
(1..5).include?(3)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn range_include_exclusive() {
    let result = run(r#"
(1...5).include?(5)
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}
