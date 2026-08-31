// Tests for type introspection built-ins:
// is_a?, kind_of?, superclass, ancestors, instance_variables,
// instance_variable_get, dup, clone

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

// ============================================================================
// is_a? / kind_of?
// ============================================================================

#[test]
fn is_a_integer() {
    let result = run("42.is_a?(Integer)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_float() {
    let result = run("3.14.is_a?(Float)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_string() {
    let result = run(r#""hello".is_a?(String)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_array() {
    let result = run("[1, 2].is_a?(Array)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_hash() {
    let result = run(r#"{"a" => 1}.is_a?(Hash)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_returns_false_for_wrong_type() {
    let result = run("42.is_a?(String)");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn is_a_checks_inheritance() {
    let result = run("42.is_a?(Object)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_custom_class() {
    let result = run(r#"
class Dog
end
d = Dog.new
d.is_a?(Dog)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_custom_class_with_inheritance() {
    let result = run(r#"
class Animal
end
class Dog < Animal
end
d = Dog.new
d.is_a?(Animal)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn kind_of_is_alias_for_is_a() {
    let result = run("42.kind_of?(Integer)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn is_a_error_no_args() {
    let err = run_err("42.is_a?");
    assert!(err.contains("argument"));
}

#[test]
fn is_a_error_non_class_arg() {
    let err = run_err("42.is_a?(42)");
    assert!(err.contains("Class") || err.contains("type"));
}

// ============================================================================
// superclass
// ============================================================================

#[test]
fn superclass_of_integer() {
    let result = run("Integer.superclass.name");
    assert_eq!(result, Some(Object::String(Rc::new("Object".to_string()))));
}

#[test]
fn superclass_of_custom_class() {
    let result = run(r#"
class Animal
end
class Dog < Animal
end
Dog.superclass.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Animal".to_string()))));
}

#[test]
fn superclass_of_object_is_basicobject() {
    let result = run("Object.superclass.name");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("BasicObject".to_string())))
    );
}

#[test]
fn superclass_of_basicobject_is_nil() {
    let result = run("BasicObject.superclass");
    assert_eq!(result, Some(Object::Nil));
}

// ============================================================================
// ancestors
// ============================================================================

#[test]
fn ancestors_of_integer() {
    let result = run("Integer.ancestors.length");
    assert_eq!(result, Some(Object::Int(2))); // Integer, Object (builtin Object has no superclass)
}

#[test]
fn ancestors_of_custom_chain() {
    let result = run(r#"
class A
end
class B < A
end
class C < B
end
C.ancestors.length
"#);
    // C, B, A, Object, Kernel, BasicObject — Object chain joins at A because
    // user classes without an explicit `<` parent inherit from Object.
    assert_eq!(result, Some(Object::Int(6)));
}

// ============================================================================
// instance_variables
// ============================================================================

#[test]
fn itself_returns_the_same_instance() {
    let result = run(r#"
class Widget
end
widget = Widget.new
widget.itself.equal?(widget)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn itself_returns_an_immediate_receiver() {
    let result = run("42.itself");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn itself_returns_a_class_receiver() {
    let result = run(r#"
class Widget
end
Widget.itself.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Widget".to_string()))));
}

#[test]
fn itself_with_an_argument_raises_argument_error() {
    let error = run_err("Object.new.itself(1)");
    assert!(error.contains("Method 'itself' expected 0 argument(s) but received 1"));
}

#[test]
fn itself_is_reported_by_respond_to() {
    let result = run("Object.new.respond_to?(:itself)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn instance_variables_returns_array() {
    let result = run(r#"
class Person
  def initialize(name, age)
    @name = name
    @age = age
  end
end
p = Person.new("Alice", 30)
p.instance_variables.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn instance_variables_returns_symbols_in_declaration_order() {
    let result = run(r#"
class Recipe
  def initialize
    @c = 1
    @a = 2
    @b = 3
  end
end
Recipe.new.instance_variables.inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:@c, :@a, :@b]".to_string())))
    );
}

#[test]
fn instance_variables_appends_a_later_assignment_last() {
    let result = run(r#"
class Recipe
  def initialize
    @name = "stew"
  end
end
recipe = Recipe.new
recipe.instance_variable_set(:@rating, 5)
recipe.instance_variables.inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:@name, :@rating]".to_string())))
    );
}

#[test]
fn instance_variables_keeps_position_when_reassigned() {
    let result = run(r#"
class Recipe
  def initialize
    @name = "stew"
    @servings = 4
  end
end
recipe = Recipe.new
recipe.instance_variable_set(:@name, "soup")
recipe.instance_variables.inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:@name, :@servings]".to_string())))
    );
}

#[test]
fn instance_variables_empty_for_non_instance() {
    let result = run("42.instance_variables.length");
    assert_eq!(result, Some(Object::Int(0)));
}

// ============================================================================
// instance_variable_get
// ============================================================================

#[test]
fn instance_variable_get_existing() {
    let result = run(r#"
class Person
  def initialize(name)
    @name = name
  end
end
p = Person.new("Alice")
p.instance_variable_get("@name")
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Alice".to_string()))));
}

#[test]
fn instance_variable_get_string_without_at_prefix_raises_name_error() {
    let error = run_err(
        r#"
class Person
  def initialize(name)
    @name = name
  end
end
p = Person.new("Bob")
p.instance_variable_get("name")
"#,
    );
    assert!(error.contains("`name' is not allowed as an instance variable name"));
}

#[test]
fn instance_variable_set_without_at_prefix_raises_name_error() {
    let error = run_err(r#"Object.new.instance_variable_set("name", 1)"#);
    assert!(error.contains("`name' is not allowed as an instance variable name"));
}

#[test]
fn instance_variable_set_validates_name_before_frozen_receiver() {
    let error = run_err(r#""".instance_variable_set(:name, 1)"#);
    assert!(error.contains("`name' is not allowed as an instance variable name"));
}

#[test]
fn instance_variable_set_converts_argument_with_to_str() {
    let result = run(r#"
class Name
  def to_str
    "@test"
  end
end
obj = Object.new
obj.instance_variable_set(Name.new, 7)
obj.instance_variable_get(:@test)
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn instance_variable_set_integer_raises_type_error() {
    let error = run_err("Object.new.instance_variable_set(10, 1)");
    assert!(error.contains("no implicit conversion of Integer into String"));
}

#[test]
fn instance_variable_set_accepts_a_non_ascii_name() {
    let result = run(r#"
obj = Object.new
obj.instance_variable_set(:@été, 5)
obj.instance_variable_get(:@été)
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn instance_variable_get_symbol_without_at_prefix_raises_name_error() {
    let error = run_err("Object.new.instance_variable_get(:name)");
    assert!(error.contains("`name' is not allowed as an instance variable name"));
}

#[test]
fn instance_variable_get_bare_at_raises_name_error() {
    let error = run_err(r#"Object.new.instance_variable_get("@")"#);
    assert!(error.contains("`@' is not allowed as an instance variable name"));
}

#[test]
fn instance_variable_get_digit_start_raises_name_error() {
    let error = run_err(r#"Object.new.instance_variable_get("@0")"#);
    assert!(error.contains("`@0' is not allowed as an instance variable name"));
}

#[test]
fn instance_variable_get_class_variable_name_raises_name_error() {
    let error = run_err(r#"Object.new.instance_variable_get("@@name")"#);
    assert!(error.contains("`@@name' is not allowed as an instance variable name"));
}

#[test]
fn instance_variable_get_integer_raises_type_error() {
    let error = run_err("Object.new.instance_variable_get(10)");
    assert!(error.contains("no implicit conversion of Integer into String"));
}

#[test]
fn instance_variable_get_converts_argument_with_to_str() {
    let result = run(r#"
class Name
  def to_str
    "@test"
  end
end
obj = Object.new
obj.instance_variable_set(:@test, 7)
obj.instance_variable_get(Name.new)
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn instance_variable_get_to_str_returning_non_string_raises_type_error() {
    let error = run_err(
        r#"
class Name
  def to_str
    123
  end
end
Object.new.instance_variable_get(Name.new)
"#,
    );
    assert!(error.contains("can't convert Integer to String"));
}

#[test]
fn instance_variable_get_on_nil_returns_nil() {
    let result = run("nil.instance_variable_get(:@foo)");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn instance_variable_get_missing_returns_nil() {
    let result = run(r#"
class Foo
end
f = Foo.new
f.instance_variable_get("@missing")
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn instance_variable_get_on_non_instance_returns_nil() {
    let result = run(r#"42.instance_variable_get("@x")"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn instance_variable_get_error_no_args() {
    let err = run_err(
        r#"
class Foo
end
Foo.new.instance_variable_get
"#,
    );
    assert!(err.contains("argument"));
}

// ============================================================================
// dup / clone
// ============================================================================

#[test]
fn dup_instance_creates_copy() {
    let result = run(r#"
class Point
  def initialize(x, y)
    @x = x
    @y = y
  end
  def x
    @x
  end
end
p1 = Point.new(1, 2)
p2 = p1.dup
p2.x
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn dup_instance_is_independent() {
    let result = run(r#"
class Box
  attr_accessor :value
  def initialize(v)
    @value = v
  end
end
b1 = Box.new(10)
b2 = b1.dup
b2.value = 99
b1.value
"#);
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn clone_array_creates_independent_copy() {
    let result = run(r#"
a = [1, 2, 3]
b = a.clone
b.push(4)
a.length
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn clone_hash_creates_independent_copy() {
    let result = run(r#"
h = {"a" => 1}
h2 = h.clone
h.size
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn dup_integer_returns_same_value() {
    let result = run("42.dup");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn dup_string_returns_same_value() {
    let result = run(r#""hello".dup"#);
    assert_eq!(result, Some(Object::String(Rc::new("hello".to_string()))));
}

// ============================================================================
// Error paths for coverage
// ============================================================================

#[test]
fn instance_variables_error_with_args() {
    let err = run_err("42.instance_variables(1)");
    assert!(err.contains("argument"));
}

#[test]
fn instance_variable_get_error_non_string_arg() {
    let err = run_err(
        r#"
class Foo
end
Foo.new.instance_variable_get(42)
"#,
    );
    assert!(err.contains("String") || err.contains("type"));
}

#[test]
fn dup_error_with_args() {
    let err = run_err("42.dup(1)");
    assert!(err.contains("argument"));
}

#[test]
fn instance_variable_get_with_symbol() {
    let result = run(r#"
class Person
  def initialize(name)
    @name = name
  end
end
p = Person.new("Alice")
p.instance_variable_get(:@name)
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Alice".to_string()))));
}

#[test]
fn clone_dict_creates_independent_copy() {
    let result = run(r#"
h1 = {"a" => 1, "b" => 2}
h2 = h1.dup
h2["c"] = 3
h1.size
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

// ── Kernel#local_variables ───────────────────────────────────────────────────

#[test]
fn local_variables_reports_top_level_locals() {
    let result = run("a = 1\nb = 2\nlocal_variables.inspect");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:a, :b]".to_string())))
    );
}

#[test]
fn local_variables_excludes_the_callers_locals_inside_a_method() {
    let result = run(r#"
outer = 1
def only_mine
  mine = 2
  local_variables
end
only_mine().inspect
"#);
    assert_eq!(result, Some(Object::String(Rc::new("[:mine]".to_string()))));
}

#[test]
fn local_variables_reports_a_name_once_when_a_block_shadows_it() {
    let result = run(r#"
def shadowing
  name = 1
  1.times do |;name|
    return local_variables
  end
end
shadowing().inspect
"#);
    assert_eq!(result, Some(Object::String(Rc::new("[:name]".to_string()))));
}

#[test]
fn local_variables_reports_a_bindings_locals() {
    let result = run(r#"
def bound
  first = 1
  second = 2
  binding
end
eval("local_variables", bound()).inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:first, :second]".to_string())))
    );
}

#[test]
fn local_variables_rejects_arguments() {
    let error = run_err("local_variables(1)");
    assert!(error.contains("local_variables() expects 0 arguments, got 1"));
}

#[test]
fn local_variables_is_a_private_instance_method_on_kernel() {
    let result = run("Kernel.private_instance_methods(false).include?(:local_variables)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── case/when with no matching branch ────────────────────────────────────────

#[test]
fn case_when_without_a_matching_branch_is_nil() {
    let result = run("case 5\nwhen 1 then :one\nwhen 2 then :two\nend");
    assert_eq!(result, Some(Object::Nil));
}

// ── A bare zero-argument def name is a call ──────────────────────────────────

#[test]
fn a_bare_zero_argument_method_name_calls_it() {
    let result = run("def answer\n  42\nend\nanswer");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn a_local_holding_a_method_object_stays_a_value() {
    let result = run(r#"
def answer
  42
end
held = method(:answer)
held.class.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Method".to_string()))));
}

// ── File.executable? ─────────────────────────────────────────────────────────

#[test]
fn file_executable_is_false_for_a_missing_path() {
    let result = run(r#"File.executable?("/no/such/path/at/all")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn file_executable_is_true_for_a_shell_binary() {
    let result = run(r#"File.executable?("/bin/sh")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn file_executable_is_false_for_a_plain_file() {
    let result = run(r#"File.executable?("/etc/hosts")"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── Object#method ────────────────────────────────────────────────────────────

#[test]
fn method_converts_its_name_argument_with_to_str() {
    let result = run(r#"
class Named
  def to_str
    "upcase"
  end
end
"shout".method(Named.new).call
"#);
    assert_eq!(result, Some(Object::String(Rc::new("SHOUT".to_string()))));
}

#[test]
fn method_raises_type_error_for_a_name_it_cannot_convert() {
    let error = run_err("Object.new.method([])");
    assert!(error.contains("no implicit conversion of Array into String"));
}

#[test]
fn method_propagates_an_error_raised_inside_to_str() {
    let error = run_err(
        r#"
class Exploding
  def to_str
    raise NoMethodError, "from to_str"
  end
end
Object.new.method(Exploding.new)
"#,
    );
    assert!(error.contains("from to_str"));
}

#[test]
fn method_answers_a_name_claimed_by_respond_to_missing() {
    let result = run(r#"
class Ghost
  def respond_to_missing?(name, include_private = false)
    name == :haunt
  end

  def method_missing(name, *args)
    "called #{name} with #{args.inspect}"
  end
end
Ghost.new.method(:haunt).call(1, 2)
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new(
            "called haunt with [1, 2]".to_string()
        )))
    );
}

#[test]
fn method_asks_respond_to_missing_with_private_allowed() {
    let result = run(r#"
class Ghost
  def respond_to_missing?(name, include_private = false)
    name == :whisper && include_private
  end

  def method_missing(name, *args)
    name
  end
end
Ghost.new.method(:whisper).call.inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new(":whisper".to_string())))
    );
}

#[test]
fn method_missing_dispatcher_keeps_its_own_arity() {
    let error = run_err(
        r#"
class OneArgument
  def respond_to_missing?(name, include_private = false)
    name == :only_name
  end

  def method_missing(name)
    name
  end
end
OneArgument.new.method(:only_name).call(1)
"#,
    );
    assert!(error.contains("expected 1 argument(s) but received 2"));
}

#[test]
fn method_still_raises_name_error_when_respond_to_missing_says_no() {
    let error = run_err(
        r#"
class Ghost
  def respond_to_missing?(name, include_private = false)
    false
  end
end
Ghost.new.method(:unknown)
"#,
    );
    assert!(error.contains("undefined method 'unknown' for class 'Ghost'"));
}

// ── Object#methods ───────────────────────────────────────────────────────────

#[test]
fn methods_lists_a_def_on_the_object_itself() {
    let result = run(r#"
class Widget
end
widget = Widget.new
def widget.polish
  :shiny
end
widget.methods(false).inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:polish]".to_string())))
    );
}

#[test]
fn methods_lists_a_module_attached_by_extend() {
    let result = run(r#"
module Greeting
  def greet
    "hello"
  end
end
class Widget
end
widget = Widget.new
widget.extend(Greeting)
widget.methods(false).inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:greet]".to_string())))
    );
}

#[test]
fn methods_omits_a_private_singleton_method() {
    let result = run(r#"
class Widget
end
widget = Widget.new
class << widget
  def buff
    :buffed
  end

  private

  def secret
    :hidden
  end
end
widget.methods(false).inspect
"#);
    assert_eq!(result, Some(Object::String(Rc::new("[:buff]".to_string()))));
}

#[test]
fn methods_omits_a_singleton_method_that_was_undefined() {
    let result = run(r#"
class Widget
end
widget = Widget.new
def widget.polish
  :shiny
end
singleton = class << widget
  self
end
singleton.send(:undef_method, :polish)
widget.methods(false).inspect
"#);
    assert_eq!(result, Some(Object::String(Rc::new("[]".to_string()))));
}

#[test]
fn methods_omits_an_inherited_method_the_class_undefined() {
    let result = run(r#"
class Parent
  def inherited_method
    :from_parent
  end
end
class Child < Parent
  undef_method :inherited_method
end
Child.new.methods.include?(:inherited_method)
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn methods_omits_class_methods_from_an_instance() {
    let result = run(r#"
class Widget
  def self.build
    :built
  end
end
Widget.new.methods.any? { |name| name.to_s.start_with?("__class__") }
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── Symbol is its own class ──────────────────────────────────────────────────

#[test]
fn a_symbol_reports_the_symbol_class() {
    let result = run(":name.class.name");
    assert_eq!(result, Some(Object::String(Rc::new("Symbol".to_string()))));
}

#[test]
fn a_symbol_is_not_a_string() {
    let result = run("String === :name");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn a_symbol_keeps_the_character_level_methods() {
    let result = run(":alpha.length");
    assert_eq!(result, Some(Object::Int(5)));
}

// ── Array intersection and union ─────────────────────────────────────────────

#[test]
fn array_intersection_keeps_left_order_without_duplicates() {
    let result = run("([1, 2, 3, 2] & [2, 3, 4]).inspect");
    assert_eq!(result, Some(Object::String(Rc::new("[2, 3]".to_string()))));
}

#[test]
fn array_union_keeps_first_seen_order_without_duplicates() {
    let result = run("([1, 2, 3, 2] | [2, 3, 4]).inspect");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[1, 2, 3, 4]".to_string())))
    );
}

// ── A body ending in `if` under a method-level rescue ────────────────────────

#[test]
fn a_method_with_a_rescue_clause_returns_its_trailing_if() {
    let result = run(r#"
def choose(flag)
  if flag
    "yes"
  else
    "no"
  end
rescue => error
  "rescued"
end
choose(true)
"#);
    assert_eq!(result, Some(Object::String(Rc::new("yes".to_string()))));
}

#[test]
fn a_begin_block_returns_its_trailing_unless() {
    let result = run(r#"
begin
  unless false
    "taken"
  end
rescue => error
  "rescued"
end
"#);
    assert_eq!(result, Some(Object::String(Rc::new("taken".to_string()))));
}
