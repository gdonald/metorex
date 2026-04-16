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
Object.new.methods.include?(:custom_xyz)
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
fn instance_eval_with_block_arg() {
    let result = run(r#"
o = Object.new
b = lambda { self.class }
o.instance_eval(&b)
"#);
    assert!(result.is_some());
}

// ── nil.to_r / nil.rationalize / nil.to_c ───────────────────────────────────

#[test]
fn nil_to_r() {
    let result = run("nil.to_r");
    assert!(result.is_some());
}

#[test]
fn nil_rationalize() {
    let result = run("nil.rationalize");
    assert!(result.is_some());
}

#[test]
fn nil_rationalize_too_many_args() {
    let err = run_err("nil.rationalize(1, 2)");
    assert!(err.contains("argument"));
}

#[test]
fn nil_to_c() {
    let result = run("nil.to_c");
    assert!(result.is_some());
}

// ── frozen? / freeze / to_sym ───────────────────────────────────────────────

#[test]
fn nil_frozen() {
    assert_eq!(run("nil.frozen?"), Some(Object::Bool(true)));
}

#[test]
fn int_frozen() {
    assert_eq!(run("42.frozen?"), Some(Object::Bool(true)));
}

#[test]
fn string_frozen() {
    assert_eq!(run("'hi'.frozen?"), Some(Object::Bool(true)));
}

#[test]
fn symbol_frozen() {
    assert_eq!(run(":foo.frozen?"), Some(Object::Bool(true)));
}

#[test]
fn float_frozen() {
    assert_eq!(run("3.14.frozen?"), Some(Object::Bool(true)));
}

#[test]
fn bool_frozen() {
    assert_eq!(run("true.frozen?"), Some(Object::Bool(true)));
}

#[test]
fn freeze_returns_self() {
    assert_eq!(run("42.freeze"), Some(Object::Int(42)));
}

#[test]
fn symbol_to_sym() {
    let result = run(":foo.to_sym");
    assert!(matches!(result, Some(Object::Symbol(_))));
}

#[test]
fn string_to_sym() {
    let result = run("'bar'.to_sym");
    assert!(matches!(result, Some(Object::Symbol(_))));
}

// ── object_id / __id__ ──────────────────────────────────────────────────────

#[test]
fn int_object_id() {
    let result = run("42.object_id");
    assert_eq!(result, Some(Object::Int(85))); // 2*42+1
}

#[test]
fn true_object_id() {
    assert_eq!(run("true.object_id"), Some(Object::Int(2)));
}

#[test]
fn false_object_id() {
    assert_eq!(run("false.object_id"), Some(Object::Int(0)));
}

#[test]
fn nil_object_id() {
    assert_eq!(run("nil.object_id"), Some(Object::Int(4)));
}

#[test]
fn instance_object_id() {
    let result = run("Object.new.object_id");
    assert!(matches!(result, Some(Object::Int(_))));
}

#[test]
fn array_object_id() {
    let result = run("[1,2].object_id");
    assert!(matches!(result, Some(Object::Int(_))));
}

#[test]
fn dict_object_id() {
    let result = run("{}.object_id");
    assert!(matches!(result, Some(Object::Int(_))));
}

// ── clamp ───────────────────────────────────────────────────────────────────

#[test]
fn clamp_two_args_in_range() {
    assert_eq!(run("5.clamp(1, 10)"), Some(Object::Int(5)));
}

#[test]
fn clamp_two_args_below_min() {
    assert_eq!(run("0.clamp(1, 10)"), Some(Object::Int(1)));
}

#[test]
fn clamp_two_args_above_max() {
    assert_eq!(run("20.clamp(1, 10)"), Some(Object::Int(10)));
}

#[test]
fn clamp_range_arg() {
    assert_eq!(run("5.clamp(1..10)"), Some(Object::Int(5)));
}

#[test]
fn clamp_exclusive_range_errors() {
    let err = run_err("5.clamp(1...10)");
    assert!(err.contains("exclusive"));
}

#[test]
fn clamp_min_greater_than_max_errors() {
    let err = run_err("5.clamp(10, 1)");
    assert!(err.contains("min") || err.contains("smaller"));
}

#[test]
fn clamp_wrong_arg_count_errors() {
    let err = run_err("5.clamp(1, 2, 3)");
    assert!(err.contains("argument"));
}

// ── between? ────────────────────────────────────────────────────────────────

#[test]
fn between_true() {
    assert_eq!(run("5.between?(1, 10)"), Some(Object::Bool(true)));
}

#[test]
fn between_false_below() {
    assert_eq!(run("0.between?(1, 10)"), Some(Object::Bool(false)));
}

#[test]
fn between_false_above() {
    assert_eq!(run("20.between?(1, 10)"), Some(Object::Bool(false)));
}

#[test]
fn between_wrong_arg_count_errors() {
    let err = run_err("5.between?(1)");
    assert!(err.contains("argument"));
}

// ── singleton_class / singleton_method ──────────────────────────────────────

#[test]
fn singleton_class_returns_class() {
    let result = run("42.singleton_class");
    assert!(matches!(result, Some(Object::Class(_))));
}

#[test]
fn singleton_method_errors() {
    let err = run_err("42.singleton_method(:foo)");
    assert!(err.contains("singleton") || err.contains("undefined"));
}

#[test]
fn singleton_method_wrong_arg_count() {
    let err = run_err("42.singleton_method");
    assert!(err.contains("argument"));
}

#[test]
fn singleton_method_non_string_arg() {
    let err = run_err("42.singleton_method(123)");
    assert!(err.contains("String") || err.contains("conversion"));
}

// ── eql? / equal? ──────────────────────────────────────────────────────────

#[test]
fn eql_same_value() {
    assert_eq!(run("42.eql?(42)"), Some(Object::Bool(true)));
}

#[test]
fn eql_different_value() {
    assert_eq!(run("42.eql?(43)"), Some(Object::Bool(false)));
}

#[test]
fn eql_wrong_arg_count() {
    let err = run_err("42.eql?");
    assert!(err.contains("argument"));
}

#[test]
fn equal_same_object() {
    assert_eq!(run("a = 42; a.equal?(a)"), Some(Object::Bool(true)));
}

#[test]
fn equal_different_objects() {
    let result = run(r#"
a = Object.new
b = Object.new
a.equal?(b)
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn equal_wrong_arg_count() {
    let err = run_err("42.equal?");
    assert!(err.contains("argument"));
}

#[test]
fn equal_same_instance() {
    let result = run(r#"
a = Object.new
a.equal?(a)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn equal_arrays_same_ref() {
    let result = run(r#"
a = [1, 2]
a.equal?(a)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn equal_dicts_same_ref() {
    let result = run(r#"
a = {"x" => 1}
a.equal?(a)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── dup on Instance / Hash / other ──────────────────────────────────────────

#[test]
fn instance_dup() {
    let result = run(r#"
class Pt
  def initialize(x)
    @x = x
  end
  def x
    @x
  end
end
a = Pt.new(5)
b = a.dup
b.x
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn hash_dup() {
    let result = run(r#"
h = {"a" => 1}
h2 = h.dup
h2["b"] = 2
h.keys.length
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ── dispatch_spaceship on built-in types ────────────────────────────────────

#[test]
fn dispatch_spaceship_int_via_clamp() {
    assert_eq!(run("5.clamp(1, 10)"), Some(Object::Int(5)));
}

#[test]
fn dispatch_spaceship_float_via_between() {
    assert_eq!(run("3.14.between?(1.0, 5.0)"), Some(Object::Bool(true)));
}

#[test]
fn dispatch_spaceship_string_via_between() {
    assert_eq!(run("'c'.between?('a', 'z')"), Some(Object::Bool(true)));
}
