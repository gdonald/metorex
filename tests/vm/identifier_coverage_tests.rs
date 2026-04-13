// Identifier resolution coverage tests — splat, additional_tests, String indexing,
// Class === type check, bare identifier dispatch, assignment as expression,
// identifier resolution class constants.

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

fn run_err(code: &str) -> String {
    let tokens = Lexer::new(code).tokenize();
    let stmts = Parser::new(tokens).parse().expect("parse failed");
    let mut vm = VirtualMachine::new();
    vm.execute_program(&stmts).unwrap_err().to_string()
}

// ── Splat in argument list ────────────────────────────────────────────────

#[test]
fn splat_with_non_array_value_treated_as_single_arg() {
    let result = run(r#"
def f(a, b)
  a + b
end
x = 5
f(*x, 10)
"#);
    assert_eq!(result, Some(Object::Int(15)));
}

#[test]
fn splat_with_nil_value_treated_as_single_arg() {
    let result = run(r#"
def f(x)
  x.nil?
end
f(*nil)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── From vm/additional_tests ────────────────────────────────────────────────

#[test]
fn global_variable_read_undefined_returns_nil() {
    assert_eq!(run("$undefined_global_var_xyz"), Some(Object::Nil));
}

#[test]
fn scope_resolution_on_class_with_constant() {
    let result = run("class Foo\n  VERSION = 42\nend\nFoo::VERSION");
    assert_eq!(result, Some(Object::Int(42)));
}

// ── String indexing ─────────────────────────────────────────────────────────

#[test]
fn string_index_int() {
    assert_eq!(
        run(r#""hello"[0]"#),
        Some(Object::String(std::rc::Rc::new("h".to_string())))
    );
}

#[test]
fn string_index_negative() {
    assert_eq!(
        run(r#""hello"[-1]"#),
        Some(Object::String(std::rc::Rc::new("o".to_string())))
    );
}

#[test]
fn string_index_out_of_bounds() {
    assert_eq!(run(r#""hello"[99]"#), Some(Object::Nil));
}

#[test]
fn string_index_range() {
    assert_eq!(
        run(r#""hello"[1..3]"#),
        Some(Object::String(std::rc::Rc::new("ell".to_string())))
    );
}

// ── Class === type check ────────────────────────────────────────────────────

#[test]
fn class_triple_eq_match() {
    assert_eq!(run("Integer === 42"), Some(Object::Bool(true)));
}

#[test]
fn class_triple_eq_no_match() {
    assert_eq!(run("Integer === \"hello\""), Some(Object::Bool(false)));
}

#[test]
fn range_class_triple_eq() {
    assert_eq!(run("Range === (1..5)"), Some(Object::Bool(true)));
}

// ── Bare identifier to method dispatch ──────────────────────────────────────

#[test]
fn bare_ident_dispatches_to_self_method() {
    let result = run(r#"
module Foo
  def self.greet
    "hello"
  end
  def self.test
    greet
  end
end
Foo.test
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("hello".to_string())))
    );
}

#[test]
fn bare_ident_method_with_args_returns_bound() {
    let result = run(r#"
module Foo
  def self.add(a, b)
    a + b
  end
  def self.test
    add(3, 4)
  end
end
Foo.test
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

// ── Assignment as last expression returns value ─────────────────────────────

#[test]
fn assignment_last_expr_returns_value() {
    let result = run(r#"
def test
  x = 42
end
test()
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn or_assign_last_expr_returns_value() {
    let result = run(r#"
class Foo
  def self.config
    @config ||= { "x" => 1 }
  end
end
Foo.config
"#);
    assert!(result.is_some());
    assert!(!matches!(result, Some(Object::Nil)));
}

// ── Identifier resolution: class constants from instance methods ──────────

#[test]
fn identifier_class_constant_from_instance_method() {
    let result = run(r#"
class Foo
  BAR = 42
  def get_bar
    BAR
  end
end
Foo.new.get_bar
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn identifier_class_constant_from_class_method() {
    let result = run(r#"
class Foo
  BAR = 99
  def self.get_bar
    BAR
  end
end
Foo.get_bar
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn identifier_class_constant_from_module_context() {
    let result = run(r#"
module M
  X = 7
  def self.get_x
    X
  end
end
M.get_x
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn identifier_bare_new_in_class_method() {
    let result = run(r#"
class Foo
  def self.create
    new
  end
  def to_s
    "a Foo"
  end
end
Foo.create.to_s
"#);
    assert_eq!(result, Some(Object::String(Rc::new("a Foo".to_string()))));
}

#[test]
fn identifier_object_class_method_zero_arg() {
    let result = run(r#"
class Object
  def helper
    42
  end
end
helper
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn identifier_object_class_method_with_args_returns_bound() {
    let result = run(r#"
class Object
  def helper(x)
    x + 1
  end
end
helper(10)
"#);
    assert_eq!(result, Some(Object::Int(11)));
}

#[test]
fn identifier_object_method_fallback_with_receiver() {
    let result = run(r#"
class Object
  def greet(name)
    "hi #{name}"
  end
end
class Foo
  def test
    greet("world")
  end
end
Foo.new.test
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("hi world".to_string())))
    );
}

#[test]
fn identifier_undefined_variable_error() {
    let err = run_err("nonexistent_var");
    assert!(err.contains("Undefined"));
}

#[test]
fn identifier_global_constant_resolution() {
    let result = run(r#"
class MyClass
end
def check
  MyClass
end
check
"#);
    assert!(matches!(result, Some(Object::Class(_))));
}

#[test]
fn identifier_no_self_no_global_error() {
    let err = run_err("unknown_thing");
    assert!(err.contains("Undefined"));
}

#[test]
fn identifier_object_method_zero_arg_fallback_with_self() {
    let result = run(r#"
class Object
  def injected_helper
    "from object"
  end
end
class Foo
  def test
    injected_helper
  end
end
Foo.new.test
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("from object".to_string())))
    );
}

#[test]
fn identifier_object_method_with_args_fallback_with_self() {
    let result = run(r#"
class Object
  def injected_add(a, b)
    a + b
  end
end
class Foo
  def test
    injected_add(3, 4)
  end
end
Foo.new.test
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn identifier_variadic_object_method() {
    let result = run(r#"
class Object
  def variadic_helper(*args)
    args.length
  end
end
variadic_helper
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn identifier_undefined_with_self_in_scope() {
    let err = run_err(
        r#"
class Foo
  def test
    totally_unknown
  end
end
Foo.new.test
"#,
    );
    assert!(err.contains("Undefined"));
}

#[test]
fn identifier_class_constant_not_on_non_class_receiver() {
    let result = run(r#"
class Foo
  BAR = 42
  def get_bar
    BAR
  end
end
Foo.new.get_bar
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn identifier_method_with_args_returns_bound_method_on_self() {
    let result = run(r#"
class Foo
  def add(a, b)
    a + b
  end
  def test
    add(3, 4)
  end
end
Foo.new.test
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn identifier_variadic_method_on_self_auto_calls() {
    let result = run(r#"
class Foo
  def items(*args)
    args
  end
  def test
    items
  end
end
Foo.new.test.length
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn identifier_object_variadic_method_with_self() {
    let result = run(r#"
class Object
  def obj_variadic(*args)
    args.length
  end
end
class Bar
  def test
    obj_variadic
  end
end
Bar.new.test
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn identifier_object_method_with_args_and_self() {
    let result = run(r#"
class Object
  def obj_calc(x, y)
    x * y
  end
end
class Baz
  def test
    obj_calc(5, 6)
  end
end
Baz.new.test
"#);
    assert_eq!(result, Some(Object::Int(30)));
}

#[test]
fn identifier_bare_method_name_returns_bound() {
    let result = run(r#"
class Foo
  def compute(x)
    x * 2
  end
  def get_method
    compute
  end
end
f = Foo.new
m = f.get_method
m.class.to_s
"#);
    assert!(result.is_some());
}

#[test]
fn identifier_constant_from_globals_inside_method() {
    let result = run(r#"
class Widget
end
class Foo
  def check
    Widget
  end
end
Foo.new.check.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Widget".to_string()))));
}

// ── Splat in expression context ──────────────────────────────────────────────

#[test]
fn splat_non_array_wraps() {
    let result = run("a = *42\na.length");
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn splat_array_passes_through() {
    let result = run("a = *[1, 2]\na.length");
    assert_eq!(result, Some(Object::Int(2)));
}
