// Coverage tests for vm/statement.rs uncovered paths

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

// ── @var set when self is not an Instance (lines 199-201) ─────────────────────
// This triggers when trying to set @var in a context where self exists but
// is not an Instance (e.g., setting @var at top-level where self is absent
// → None path at 203-209). The Some(_) non-instance path is harder to trigger.

#[test]
fn instance_var_set_outside_method_error() {
    let err = run_err("@foo = 42");
    assert!(
        err.contains("instance variable")
            || err.contains("@foo")
            || err.contains("method")
            || err.contains("context")
    );
}

// ── @@var set when self is None (no context) ──────────────────────────────────

#[test]
fn class_var_set_outside_class_error() {
    let err = run_err("@@count = 1");
    assert!(
        err.contains("class variable")
            || err.contains("@@count")
            || err.contains("class")
            || err.contains("context")
    );
}

// ── @@var set inside class body (self is Class object) lines 221-223 ──────────

#[test]
fn class_var_set_in_class_body() {
    let result = run(r#"
class Counter
  @@total = 0
  def initialize
    @@total = @@total + 1
  end
  def total
    @@total
  end
end
c1 = Counter.new
c2 = Counter.new
c1.total
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ── @@var read in class body (self is Class) lines 483-484 ────────────────────

#[test]
fn class_var_read_in_class_body() {
    let result = run(r#"
class Config
  @@value = 42
  def get_value
    @@value
  end
end
Config.new.get_value
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── @var set when self is a Class (lines 199-201) ─────────────────────────────
// Metorex allows calling instance methods on the class itself (no def self. syntax).
// When called on the class, self = Object::Class, triggering the Some(_) branch.

#[test]
fn instance_var_in_method_called_on_class_error() {
    let err = run_err(
        r#"
class Foo
  def set_ivar
    @x = 10
  end
end
Foo.set_ivar
"#,
    );
    assert!(
        err.contains("instance variable")
            || err.contains("@x")
            || err.contains("non-instance")
            || err.contains("Cannot")
    );
}

// ── @@var set when self is a Class object (lines 221-223) ─────────────────────
// When an instance method is called on the class (Foo.method), self = Class.

#[test]
fn class_var_set_in_method_called_on_class() {
    let result = run(r#"
class Foo
  def set_class_var
    @@x = 42
  end
  def get_class_var
    @@x
  end
end
Foo.set_class_var
Foo.get_class_var
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── setter method called on non-instance type (lines 353-359) ────────────────

#[test]
fn setter_method_on_non_instance_error() {
    let err = run_err(
        r#"
arr = [1, 2, 3]
arr.foo = 42
"#,
    );
    assert!(
        err.contains("setter")
            || err.contains("foo")
            || err.contains("Cannot")
            || err.contains("Array")
            || err.contains("method")
    );
}

#[test]
fn setter_method_on_integer_error() {
    let err = run_err("1.foo = 2");
    assert!(
        err.contains("setter")
            || err.contains("foo")
            || err.contains("Cannot")
            || err.contains("Integer")
            || err.contains("method")
    );
}

// ── global variable assignment ────────────────────────────────────────────────

#[test]
fn global_var_assignment_and_read() {
    let result = run(r#"
$count = 10
$count = $count + 5
$count
"#);
    assert_eq!(result, Some(Object::Int(15)));
}

// ── @var read outside method error (None context, lines 464-470) ─────────────

#[test]
fn instance_var_read_outside_method_error() {
    let err = run_err("@foo");
    assert!(
        err.contains("instance variable")
            || err.contains("@foo")
            || err.contains("method")
            || err.contains("context")
    );
}

// ── @@var read outside class error ───────────────────────────────────────────

#[test]
fn class_var_read_outside_class_error() {
    let err = run_err("@@total");
    assert!(
        err.contains("class variable")
            || err.contains("@@total")
            || err.contains("class")
            || err.contains("context")
    );
}

// ── compound assignment operators (parser/statements/mod.rs) ─────────────────
// MinusEqual (lines 64-68) and SlashEqual (lines 76-81) desugaring

#[test]
fn minus_equal_compound_assignment() {
    let result = run("x = 10\nx -= 3\nx");
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn divide_equal_compound_assignment() {
    let result = run("x = 12\nx /= 4\nx");
    assert_eq!(result, Some(Object::Int(3)));
}
