// Set error/edge coverage tests (split from native_methods_error_coverage_tests.rs)

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
