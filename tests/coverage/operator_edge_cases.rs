// Coverage tests for operator edge cases and other small gaps

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

// ── Builtin class edge cases ────────────────────────────────────────────

#[test]
fn set_new_with_array_arg() {
    let result = run("Set.new([1, 2, 3]).size");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn class_name_method() {
    let result = run("class Foo\nend\nFoo.name");
    assert_eq!(result, Some(Object::String(Rc::new("Foo".to_string()))));
}

// ── Float method edge cases ─────────────────────────────────────────────

#[test]
fn float_to_i() {
    let result = run("3.7.to_i");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Pattern matching edge cases ─────────────────────────────────────────

#[test]
fn case_with_range_pattern() {
    let result = run("case 5\nwhen 1..10\n  \"in range\"\nelse\n  \"out\"\nend");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("in range".to_string())))
    );
}

// ── String edge cases ───────────────────────────────────────────────────

#[test]
fn string_chars() {
    let result = run("\"abc\".chars");
    assert!(result.is_some());
}

// ── Array edge cases ────────────────────────────────────────────────────

#[test]
fn array_reverse() {
    let result = run("[1, 2, 3].reverse");
    assert!(result.is_some());
}

// ── Hash edge cases ─────────────────────────────────────────────────────

#[test]
fn hash_merge() {
    let result = run("{\"a\" => 1}.merge({\"b\" => 2}).size");
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn hash_entries() {
    let result = run("{\"a\" => 1}.entries");
    assert!(result.is_some());
}

// ── Range edge cases ────────────────────────────────────────────────────

#[test]
fn range_include() {
    let result = run("(1..10).include?(5)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Multiple assignment edge cases ──────────────────────────────────────

#[test]
fn compound_add_assign() {
    let result = run("x = 5\nx += 3\nx");
    assert_eq!(result, Some(Object::Int(8)));
}

#[test]
fn compound_subtract_assign() {
    let result = run("x = 5\nx -= 3\nx");
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn compound_multiply_assign() {
    let result = run("x = 5\nx *= 3\nx");
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn compound_divide_assign() {
    let result = run("x = 6\nx /= 2\nx");
    assert_eq!(result, Some(Object::Int(3)));
}

// ── Unless statement ────────────────────────────────────────────────────

#[test]
fn unless_false_runs_body() {
    let result = run("x = nil\nunless false\n  x = 42\nend\nx");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn unless_true_skips_body() {
    let result = run("x = 1\nunless true\n  x = 42\nend\nx");
    assert_eq!(result, Some(Object::Int(1)));
}

// ── Respond_to? ─────────────────────────────────────────────────────────

#[test]
fn respond_to_known_method() {
    let result = run("class Foo\n  def bar\n    1\n  end\nend\nFoo.new.respond_to?(\"bar\")");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn respond_to_unknown_method() {
    let result = run("class Foo\nend\nFoo.new.respond_to?(\"baz\")");
    assert_eq!(result, Some(Object::Bool(false)));
}
