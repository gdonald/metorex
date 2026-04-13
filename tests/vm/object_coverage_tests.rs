// Object coverage tests — BlockArg, begin/rescue as expression,
// Comparable derivation, super error paths, nil conversions, dup/clone, send.

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

// ── BlockArg in argument position ────────────────────────────────────────────

#[test]
fn blockarg_passes_block_to_method() {
    let result = run(r#"
def run_block(&block)
  block.call(5)
end
b = lambda { |x| x * 2 }
run_block(&b)
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

// ── begin/rescue as expression ───────────────────────────────────────────────

#[test]
fn begin_rescue_expression() {
    let result = run(r#"
x = begin
  raise "oops"
rescue
  99
end
x
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn begin_rescue_with_ensure_expression() {
    let result = run(r#"
x = begin
  42
ensure
  nil
end
x
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

// ── Comparable derivation from <=> ───────────────────────────────────────────

#[test]
fn comparable_less_from_spaceship() {
    let result = run(r#"
class Weight
  def initialize(n)
    @n = n
  end
  def <=>(other)
    @n <=> other.value
  end
  def value
    @n
  end
end
a = Weight.new(1)
b = Weight.new(2)
a < b
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn comparable_greater_equal_from_spaceship() {
    let result = run(r#"
class Weight
  def initialize(n)
    @n = n
  end
  def <=>(other)
    @n <=> other.value
  end
  def value
    @n
  end
end
a = Weight.new(3)
b = Weight.new(2)
a >= b
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn comparable_less_equal_from_spaceship() {
    let result = run(r#"
class Weight
  def initialize(n)
    @n = n
  end
  def <=>(other)
    @n <=> other.value
  end
  def value
    @n
  end
end
a = Weight.new(2)
b = Weight.new(2)
a <= b
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── super error paths ────────────────────────────────────────────────────────

#[test]
fn super_outside_method_error() {
    let err = run_err("super");
    assert!(err.contains("super") || err.contains("method"));
}

#[test]
fn super_with_no_parent_class_error() {
    let err = run_err(
        r#"
class Base
  def greet
    super
  end
end
Base.new.greet
"#,
    );
    assert!(err.contains("super") || err.contains("parent") || err.contains("superclass"));
}

// ── nil conversion methods ───────────────────────────────────────────────────

#[test]
fn nil_to_a_returns_empty_array() {
    let result = run("nil.to_a.length");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn nil_to_h_returns_empty_hash() {
    let result = run("nil.to_h.length");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn nil_to_i_returns_zero() {
    let result = run("nil.to_i");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn nil_to_f_returns_zero() {
    let result = run("nil.to_f");
    assert_eq!(result, Some(Object::Float(0.0)));
}

#[test]
fn nil_inspect_returns_nil_string() {
    let result = run("nil.inspect");
    assert_eq!(result, Some(Object::string("nil")));
}

// ── dup / clone ──────────────────────────────────────────────────────────────

#[test]
fn dup_instance() {
    let result = run(r#"
class Foo
  attr_accessor :x
  def initialize(x)
    @x = x
  end
end
a = Foo.new(1)
b = a.dup
b.x = 2
a.x
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn dup_array() {
    let result = run(r#"
a = [1, 2, 3]
b = a.dup
b.push(4)
a.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn dup_hash() {
    let result = run(r#"
a = {"x" => 1}
b = a.dup
b["y"] = 2
a.length
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn dup_immutable_returns_self() {
    let result = run("42.dup");
    assert_eq!(result, Some(Object::Int(42)));
}

// ── send ─────────────────────────────────────────────────────────────────────

#[test]
fn send_with_symbol() {
    let result = run("'hello'.send(:length)");
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn send_with_args() {
    let result = run("[1, 2, 3].send(:push, 4).length");
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn send_no_args_error() {
    let err = run_err("'hello'.send");
    assert!(err.contains("argument"));
}

// ── object_methods.rs lines 45-49: to_s with args on Object ─────────────────

#[test]
fn object_to_s_with_args_errors() {
    let err = run_err("Object.new.to_s(1)");
    assert!(err.contains("argument"));
}

// ── object_methods.rs line 308: methods on Object with user-defined methods ──

#[test]
fn object_methods_includes_user_defined() {
    let result = run(r#"
class Object
  def custom_xyz
    42
  end
end
Object.new.methods.include?("custom_xyz")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── object_methods.rs lines 445, 448, 452: !~ operator ──────────────────────

#[test]
fn regex_not_match_operator_false() {
    let result = run(r#"r = /ell/; r !~ "hello""#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn regex_not_match_operator_true() {
    let result = run(r#"r = /xyz/; r !~ "hello""#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn regex_not_match_case_insensitive() {
    let result = run(r#"r = /ELL/i; r !~ "hello""#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn regex_not_match_wrong_arg_count_error() {
    let err = run_err(r#"r = /ell/; r.send(:"!~")"#);
    assert!(err.contains("argument"));
}

// ── object_methods.rs lines 467-469: instance_eval with block as positional arg

#[test]
fn instance_eval_with_block_as_positional_arg() {
    let result = run(r#"
o = Object.new
b = lambda { |x| x.class }
o.instance_eval(b)
"#);
    assert!(result.is_some());
}
