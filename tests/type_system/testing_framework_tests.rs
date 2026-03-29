// Tests for the built-in testing framework assertion functions

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
// assert
// ============================================================================

#[test]
fn assert_true_returns_true() {
    let result = run("assert(true)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn assert_truthy_value_returns_true() {
    let result = run("assert(42)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn assert_false_raises_error() {
    let err = run_err("assert(false)");
    assert!(err.contains("Assertion failed"));
}

#[test]
fn assert_nil_raises_error() {
    let err = run_err("assert(nil)");
    assert!(err.contains("Assertion failed"));
}

#[test]
fn assert_with_custom_message() {
    let err = run_err(r#"assert(false, "custom message")"#);
    assert!(err.contains("custom message"));
}

#[test]
fn assert_error_no_args() {
    let err = run_err("assert()");
    assert!(err.contains("argument"));
}

#[test]
fn assert_error_too_many_args() {
    let err = run_err("assert(true, \"msg\", \"extra\")");
    assert!(err.contains("argument"));
}

// ============================================================================
// assert_equal
// ============================================================================

#[test]
fn assert_equal_matching_returns_true() {
    let result = run("assert_equal(42, 42)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn assert_equal_strings() {
    let result = run(r#"assert_equal("hello", "hello")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn assert_equal_mismatch_raises() {
    let err = run_err("assert_equal(1, 2)");
    assert!(err.contains("Expected 1") || err.contains("got"));
}

#[test]
fn assert_equal_with_custom_message() {
    let err = run_err(r#"assert_equal(1, 2, "values differ")"#);
    assert!(err.contains("values differ"));
}

#[test]
fn assert_equal_error_one_arg() {
    let err = run_err("assert_equal(1)");
    assert!(err.contains("argument"));
}

#[test]
fn assert_equal_error_too_many_args() {
    let err = run_err(r#"assert_equal(1, 1, "ok", "extra")"#);
    assert!(err.contains("argument"));
}

// ============================================================================
// assert_raises
// ============================================================================

#[test]
fn assert_raises_with_raising_block() {
    let result = run(r#"assert_raises { raise "error" }"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn assert_raises_with_non_raising_block_fails() {
    let err = run_err("assert_raises { 42 }");
    assert!(err.contains("Expected block to raise"));
}

#[test]
fn assert_raises_error_with_args() {
    let err = run_err("assert_raises(1) { 42 }");
    assert!(err.contains("argument"));
}

#[test]
fn assert_raises_error_no_block() {
    let err = run_err("assert_raises()");
    assert!(err.contains("block") || err.contains("Block"));
}

// ============================================================================
// Integration: describe/it/expect framework
// ============================================================================

#[test]
fn testing_framework_all_pass() {
    let result = run(r#"
class Expectation
  def initialize(value)
    @value = value
  end
  def to_equal(expected)
    assert_equal(expected, @value)
  end
end

def expect(value)
  Expectation.new(value)
end

class TestSuite
  def initialize(name)
    @name = name
    @tests = []
    @passed = 0
  end
  def it(desc, &block)
    @tests.push([desc, block])
  end
  def run
    @tests.each do |pair|
      pair[1].call
      @passed += 1
    end
    @passed
  end
end

suite = TestSuite.new("test")
suite.it("one") { expect(1 + 1).to_equal(2) }
suite.it("two") { expect(3 * 4).to_equal(12) }
suite.run
"#);
    assert_eq!(result, Some(Object::Int(2)));
}
