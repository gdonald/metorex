// Targeted coverage tests for uncovered lines in src/vm/eval/super_expr.rs.

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

// ── Class-method super chain (lines 50-77) ───────────────────────────────────

#[test]
fn class_method_super_walks_chain() {
    let result = run(r#"
class ParentCls
  def self.greet
    "parent"
  end
end
class ChildCls < ParentCls
  def self.greet
    super + " + child"
  end
end
ChildCls.greet
"#);
    assert_eq!(result, Some(Object::string("parent + child")));
}

#[test]
fn class_method_super_with_args() {
    let result = run(r#"
class P1
  def self.calc(x)
    x * 2
  end
end
class C1 < P1
  def self.calc(x)
    super(x) + 1
  end
end
C1.calc(5)
"#);
    assert_eq!(result, Some(Object::Int(11)));
}

#[test]
fn class_method_super_bare_forwards_args() {
    let result = run(r#"
class P2
  def self.echo(a, b)
    a + b
  end
end
class C2 < P2
  def self.echo(a, b)
    super
  end
end
C2.echo(3, 4)
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

// ── `inherited` hook super as silent no-op (lines 82-84) ─────────────────────

#[test]
fn inherited_hook_super_is_silent_noop() {
    // Class#inherited has a default no-op behavior, so calling super in it
    // at the top of the hierarchy should be harmless.
    let result = run(r#"
class Watcher
  def self.inherited(sub)
    super
  end
end
class Derived < Watcher
end
:ok
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("ok".to_string())))
    );
}

// ── Class-method super when no match found → error (line 85-92) ──────────────

#[test]
fn class_method_super_with_no_parent_definition_errors() {
    let err = run_err(
        r#"
class LonelyParent
end
class LonelyChild < LonelyParent
  def self.missing_method
    super
  end
end
LonelyChild.missing_method
"#,
    );
    assert!(
        err.contains("super")
            || err.contains("no superclass method")
            || err.contains("missing_method")
    );
}

// ── Object#<=> super default (lines 201-216) ─────────────────────────────────

#[test]
fn object_spaceship_super_returns_some_value() {
    // Exercise the Object#<=> super fallback path.
    let result = run(r#"
class SpaceShip
  def <=>(other)
    super
  end
end
s = SpaceShip.new
s <=> s
"#);
    assert!(matches!(result, Some(Object::Int(0)) | Some(Object::Nil)));
}

#[test]
fn object_spaceship_super_returns_nil_for_different_instances() {
    let result = run(r#"
class SpaceShip2
  def <=>(other)
    super
  end
end
a = SpaceShip2.new
b = SpaceShip2.new
a <=> b
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── super with no parent class raises (line 217-222) ─────────────────────────

#[test]
fn super_unresolved_raises_when_parent_missing() {
    let err = run_err(
        r#"
class Alone
  def lonely_call
    super
  end
end
Alone.new.lonely_call
"#,
    );
    assert!(
        err.contains("super")
            || err.contains("superclass")
            || err.contains("lonely_call")
            || err.contains("no super")
    );
}

// ── Normal instance super (lines 197-260) ────────────────────────────────────

#[test]
fn instance_super_to_parent_method() {
    let result = run(r#"
class Animal
  def speak
    "generic"
  end
end
class Dog < Animal
  def speak
    super + "!"
  end
end
Dog.new.speak
"#);
    assert_eq!(result, Some(Object::string("generic!")));
}

#[test]
fn instance_super_with_args_explicit() {
    let result = run(r#"
class Pr
  def add(a, b)
    a + b
  end
end
class Ch < Pr
  def add(a, b)
    super(a, b) * 2
  end
end
Ch.new.add(3, 4)
"#);
    assert_eq!(result, Some(Object::Int(14)));
}

#[test]
fn instance_super_bare_forwards_args() {
    let result = run(r#"
class Pr2
  def greet(name)
    "hello, " + name
  end
end
class Ch2 < Pr2
  def greet(name)
    super
  end
end
Ch2.new.greet("world")
"#);
    assert_eq!(result, Some(Object::string("hello, world")));
}

// ── super via mixin (walks the chain using mixin_chain) ──────────────────────

#[test]
fn instance_super_through_mixin() {
    let result = run(r#"
module Greet
  def hello
    "module_hello"
  end
end
class Host
  include Greet
  def hello
    super + "!"
  end
end
Host.new.hello
"#);
    assert_eq!(result, Some(Object::string("module_hello!")));
}

// ── super called outside any method → error ──────────────────────────────────

#[test]
fn super_outside_any_method_errors() {
    let err = run_err(r#"super"#);
    assert!(err.contains("super") || err.contains("method context") || err.contains("outside"));
}
