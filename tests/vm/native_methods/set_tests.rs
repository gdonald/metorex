// Coverage tests for set native methods

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

// ── Set each with break ────────────────────────────────────────────────

#[test]
fn set_each_with_break() {
    let result = run(
        "s = Set.new\ns.add(\"a\")\ns.add(\"b\")\ns.add(\"c\")\ncount = 0\ns.each do |x|\n  count = count + 1\n  if count == 2\n    break\n  end\nend\ncount",
    );
    assert_eq!(result, Some(Object::Int(2)));
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

// ── From remaining_tests ────────────────────────────────────────────────────

#[test]
fn set_union_method_remaining() {
    let result = run("a = Set.new([1,2]); b = Set.new([2,3]); a.union(b).size");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn set_intersection_method_remaining() {
    let result = run("a = Set.new([1,2,3]); b = Set.new([2,3,4]); a.intersection(b).size");
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn set_difference_method_remaining() {
    let result = run("a = Set.new([1,2,3]); b = Set.new([2,3]); a.difference(b).size");
    assert_eq!(result, Some(Object::Int(1)));
}
