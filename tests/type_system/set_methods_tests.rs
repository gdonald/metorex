// Tests for Set native methods

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

// ============================================================================
// Set.new
// ============================================================================

#[test]
fn set_new_empty() {
    let result = run("Set.new.size");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn set_new_from_array() {
    let result = run("Set.new([1, 2, 3]).size");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn set_new_deduplicates() {
    let result = run("Set.new([1, 1, 2, 2, 3]).size");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn set_new_error_non_array_arg() {
    let err = run_err("Set.new(42)");
    assert!(err.contains("Array") || err.contains("type"));
}

#[test]
fn set_new_error_too_many_args() {
    let err = run_err("Set.new([1], [2])");
    assert!(err.contains("argument"));
}

// ============================================================================
// add / insert
// ============================================================================

#[test]
fn set_add_element() {
    let result = run(r#"
s = Set.new
s.add(1)
s.size
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn set_add_duplicate_no_change() {
    let result = run(r#"
s = Set.new([1, 2])
s.add(1)
s.size
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn set_insert_alias() {
    let result = run(r#"
s = Set.new
s.insert(42)
s.size
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn set_add_returns_set() {
    let result = run(r#"
s = Set.new
s.add(1).add(2).size
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn set_add_error_no_args() {
    let err = run_err("Set.new.add");
    assert!(err.contains("argument"));
}

// ============================================================================
// remove / delete
// ============================================================================

#[test]
fn set_remove_existing() {
    let result = run(r#"
s = Set.new([1, 2, 3])
s.remove(2)
s.size
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn set_remove_returns_bool() {
    let result = run("Set.new([1, 2]).remove(1)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn set_remove_missing_returns_false() {
    let result = run("Set.new([1, 2]).remove(99)");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn set_delete_alias() {
    let result = run("Set.new([1, 2]).delete(1)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ============================================================================
// contains? / include? / has?
// ============================================================================

#[test]
fn set_contains_true() {
    let result = run(r#"Set.new([1, 2, 3]).contains?(2)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn set_contains_false() {
    let result = run(r#"Set.new([1, 2, 3]).contains?(99)"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn set_include_alias() {
    let result = run(r#"Set.new([1, 2]).include?(1)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn set_has_alias() {
    let result = run(r#"Set.new([1, 2]).has?(2)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn set_contains_string() {
    let result = run(r#"Set.new(["a", "b"]).contains?("a")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ============================================================================
// size / length
// ============================================================================

#[test]
fn set_size() {
    let result = run("Set.new([1, 2, 3]).size");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn set_length_alias() {
    let result = run("Set.new([1, 2]).length");
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn set_size_error_with_args() {
    let err = run_err("Set.new.size(1)");
    assert!(err.contains("argument"));
}

// ============================================================================
// empty?
// ============================================================================

#[test]
fn set_empty_true() {
    let result = run("Set.new.empty?");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn set_empty_false() {
    let result = run("Set.new([1]).empty?");
    assert_eq!(result, Some(Object::Bool(false)));
}

// ============================================================================
// to_a
// ============================================================================

#[test]
fn set_to_a_returns_array() {
    let result = run("Set.new([1]).to_a.length");
    assert_eq!(result, Some(Object::Int(1)));
}

// ============================================================================
// union
// ============================================================================

#[test]
fn set_union() {
    let result = run("Set.new([1, 2]).union(Set.new([2, 3])).size");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn set_union_error_non_set() {
    let err = run_err("Set.new([1]).union(42)");
    assert!(err.contains("Set") || err.contains("type"));
}

// ============================================================================
// intersection
// ============================================================================

#[test]
fn set_intersection() {
    let result = run("Set.new([1, 2, 3]).intersection(Set.new([2, 3, 4])).size");
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn set_intersection_empty() {
    let result = run("Set.new([1, 2]).intersection(Set.new([3, 4])).size");
    assert_eq!(result, Some(Object::Int(0)));
}

// ============================================================================
// difference
// ============================================================================

#[test]
fn set_difference() {
    let result = run("Set.new([1, 2, 3]).difference(Set.new([2, 3])).size");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn set_difference_contains_correct_element() {
    let result = run("Set.new([1, 2, 3]).difference(Set.new([2, 3])).contains?(1)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ============================================================================
// each
// ============================================================================

#[test]
fn set_each_iterates() {
    let result = run(r#"
count = 0
Set.new([1, 2, 3]).each { |x| count += 1 }
count
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn set_each_returns_set() {
    let result = run("Set.new([1]).each { |x| x }.size");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn set_each_error_no_block() {
    let err = run_err("Set.new([1]).each");
    assert!(err.contains("block") || err.contains("Block"));
}

#[test]
fn set_each_with_break() {
    let result = run(r#"
count = 0
Set.new([1, 2, 3, 4, 5]).each do |x|
  count += 1
  if count == 3
    break
  end
end
count
"#);
    assert_eq!(result, Some(Object::Int(3)));
}
