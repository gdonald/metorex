// VM tests for class/module, instance/class variables, comparable, undef_method.

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

// ── class method instance variable ───────────────────────────────────────────

#[test]
fn class_instance_var_in_class_method() {
    let result = run(r#"
class C
  def self.set_val
    @val = 42
  end
end
C.set_val
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── class reopen ─────────────────────────────────────────────────────────────

#[test]
fn class_reopen_from_local_environment() {
    let result = run(r#"
def make_class
  class LocalFoo
    def greet
      "hello"
    end
  end
  class LocalFoo
    def farewell
      "bye"
    end
  end
  LocalFoo.new.farewell
end
make_class
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("bye".to_string())))
    );
}

// ── module reopen ─────────────────────────────────────────────────────────────

#[test]
fn module_reopen_from_globals() {
    let result = run(r#"
module Greetings
  def hello
    "hello"
  end
end
module Greetings
  def farewell
    "bye"
  end
end
class MyClass
  include Greetings
end
MyClass.new.farewell
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("bye".to_string())))
    );
}

// ── module setter dispatch ───────────────────────────────────────────────────

#[test]
fn module_setter_with_defined_method() {
    let result = run(r#"
module Storage
  def self.data=(val)
    @data = val
  end
  def self.data
    @data
  end
end
Storage.data = 99
Storage.data
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn module_setter_method_dispatch() {
    let result = run(r#"
module Cfg
  def config_value=(val)
    @config_value = val
  end
  def config_value
    @config_value
  end
end
Cfg.config_value = "metorex"
Cfg.config_value
"#);
    assert_eq!(result, Some(Object::string("metorex")));
}

// ── instance variable errors ─────────────────────────────────────────────────

#[test]
fn instance_var_assign_on_immediate_raises_frozen() {
    let err = run_err(
        r#"
class Foo
  def test
    1.instance_eval { @x = 1 }
  end
end
Foo.new.test
"#,
    );
    assert!(
        err.contains("FrozenError") || err.contains("frozen"),
        "expected FrozenError, got: {err}"
    );
}

#[test]
fn instance_var_read_on_immediate_returns_nil() {
    // Reading an unset ivar on an immediate self (e.g. `1.instance_eval { @x }`)
    // returns nil — matching Ruby. No error should be raised.
    let result = run(r#"
class Foo
  def test
    1.instance_eval { @x }
  end
end
Foo.new.test
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn class_var_read_on_non_class_self_errors() {
    let err = run_err(
        r#"
class Foo
  def test
    1.instance_eval { @@x }
  end
end
Foo.new.test
"#,
    );
    assert!(err.contains("Cannot read") || err.contains("context") || err.contains("@@x"));
}

#[test]
fn class_var_assignment_on_non_class_self_errors() {
    let err = run_err(
        r#"
class Foo
  def test
    1.instance_eval { @@x = 5 }
  end
end
Foo.new.test
"#,
    );
    assert!(err.contains("Cannot set") || err.contains("class variable") || err.contains("@@x"));
}

// ── Comparable <=> fallback ───────────────────────────────────────────────────

#[test]
fn comparable_spaceship_fallback_for_greater_than() {
    let result = run(r#"
class Weight
  def initialize(v)
    @val = v
  end
  def <=>(other)
    @val <=> other.get_val
  end
  def get_val
    @val
  end
end
a = Weight.new 10
b = Weight.new 5
a > b
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── undef_method ─────────────────────────────────────────────────────────────

#[test]
fn undef_method_via_method_missing_path() {
    let err = run_err(
        r#"
class GhostClass
  def method_missing(name)
    "caught #{name}"
  end
end
GhostClass.undef_method("method_missing")
GhostClass.new.no_such_method
"#,
    );
    assert!(
        err.contains("undefined")
            || err.contains("Undefined")
            || err.contains("no_such_method")
            || err.contains("method_missing"),
        "Error was: {}",
        err
    );
}

// ── exclusive float range include? ───────────────────────────────────────────

#[test]
fn exclusive_float_range_include() {
    let result = run("(1.0...5.0).include?(3.0)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn exclusive_float_range_include_at_end_is_false() {
    let result = run("(1.0...5.0).include?(5.0)");
    assert_eq!(result, Some(Object::Bool(false)));
}
