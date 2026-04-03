// Tests for method_missing hook (14.2)

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

// ── Basic method_missing ────────────────────────────────────────────

#[test]
fn method_missing_basic_intercept() {
    let result = run(r#"
class Catcher
  def method_missing(name)
    "caught: #{name}"
  end
end
c = Catcher.new
c.anything
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new(
            "caught: anything".to_string()
        )))
    );
}

#[test]
fn method_missing_returns_value() {
    let result = run(r#"
class Echo
  def method_missing(name)
    name
  end
end
e = Echo.new
e.hello
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hello".to_string())))
    );
}

#[test]
fn method_missing_does_not_intercept_defined_methods() {
    let result = run(r#"
class Selective
  def real
    42
  end
  def method_missing(name)
    0
  end
end
s = Selective.new
s.real
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── method_missing with arguments ───────────────────────────────────

#[test]
fn method_missing_receives_arguments_as_array() {
    let result = run(r#"
class ArgChecker
  def method_missing(name, args)
    args.length
  end
end
a = ArgChecker.new
a.test(1, 2, 3)
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn method_missing_no_args_gives_empty_array() {
    let result = run(r#"
class ArgChecker
  def method_missing(name, args)
    args.length
  end
end
a = ArgChecker.new
a.test
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn method_missing_args_can_be_iterated() {
    let result = run(r#"
class Summer
  def method_missing(name, args)
    total = 0
    args.each do |n|
      total = total + n
    end
    total
  end
end
s = Summer.new
s.sum(10, 20, 30)
"#);
    assert_eq!(result, Some(Object::Int(60)));
}

// ── Inherited method_missing ────────────────────────────────────────

#[test]
fn method_missing_inherited_from_parent() {
    let result = run(r#"
class Base
  def method_missing(name)
    "Base: #{name}"
  end
end
class Child < Base
end
c = Child.new
c.foo
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("Base: foo".to_string())))
    );
}

#[test]
fn method_missing_overridden_in_child() {
    let result = run(r#"
class Base
  def method_missing(name)
    "Base: #{name}"
  end
end
class Child < Base
  def method_missing(name)
    "Child: #{name}"
  end
end
c = Child.new
c.bar
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("Child: bar".to_string())))
    );
}

// ── method_missing with dynamic attribute access ────────────────────

#[test]
fn method_missing_dynamic_hash_lookup() {
    let result = run(r#"
class Record
  def initialize
    @data = {"name" => "Alice"}
  end
  def method_missing(name)
    @data[name]
  end
end
r = Record.new
r.name
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("Alice".to_string())))
    );
}

// ── Error when no method_missing defined ────────────────────────────

#[test]
fn no_method_missing_raises_error() {
    let err = run_err(
        r#"
class Plain
end
p = Plain.new
p.nonexistent
"#,
    );
    assert!(err.contains("Undefined method 'nonexistent'"));
}

// ── method_missing with one param receives only method name ─────────

#[test]
fn method_missing_single_param_ignores_call_args() {
    let result = run(r#"
class OneName
  def method_missing(name)
    name
  end
end
o = OneName.new
o.test(1, 2, 3)
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("test".to_string())))
    );
}
