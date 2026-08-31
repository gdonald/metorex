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
    assert_eq!(result, Some(Object::String(Rc::new("Numeric".to_string()))));
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
    // Integer, Numeric, Object.
    let result = run("Integer.ancestors.length");
    assert_eq!(result, Some(Object::Int(3)));
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
fn methods_without_ancestors_leaves_out_a_module_attached_by_extend() {
    // `methods(false)` is `singleton_methods(false)`, which reports only the
    // methods defined directly on the object.
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
[widget.methods(false).length, widget.methods.include?(:greet)].inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[0, true]".to_string())))
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

// ── Object#private_methods ───────────────────────────────────────────────────

const PRIVATE_FIXTURE: &str = r#"
module Helpers
  def mixed_in
  end
  private :mixed_in
end

class Parent
  def parent_secret
  end
  private :parent_secret

  class << self
    def parent_class_secret
    end
    private :parent_class_secret
  end
end

class Child < Parent
  include Helpers

  def child_secret
  end
  private :child_secret

  class << self
    def child_class_secret
    end
    private :child_class_secret
  end
end
"#;

fn secrets(code: &str) -> Option<Object> {
    run(&format!(
        "{PRIVATE_FIXTURE}\n({code}).select {{ |n| n.to_s.end_with?(\"secret\") }}.sort.inspect"
    ))
}

#[test]
fn private_methods_without_ancestors_lists_only_the_objects_own() {
    let result = secrets("Child.new.private_methods(false)");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:child_secret]".to_string())))
    );
}

#[test]
fn private_methods_with_ancestors_reaches_the_superclass() {
    let result = secrets("Child.new.private_methods");
    assert_eq!(
        result,
        Some(Object::String(Rc::new(
            "[:child_secret, :parent_secret]".to_string()
        )))
    );
}

#[test]
fn private_methods_treats_nil_like_false() {
    let result = secrets("Child.new.private_methods(nil)");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:child_secret]".to_string())))
    );
}

#[test]
fn a_classs_private_methods_come_from_its_singleton_chain() {
    let result = secrets("Child.private_methods(false)");
    assert_eq!(
        result,
        Some(Object::String(Rc::new(
            "[:child_class_secret, :parent_class_secret]".to_string()
        )))
    );
}

#[test]
fn private_methods_includes_a_class_private_singleton_method() {
    let result = run(&format!(
        "{PRIVATE_FIXTURE}\nChild.private_methods.include?(:child_class_secret)"
    ));
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn private_methods_includes_a_module_attached_by_extend() {
    let result = run(&format!(
        "{PRIVATE_FIXTURE}\nobj = Object.new\nobj.extend(Helpers)\nobj.private_methods.include?(:mixed_in)"
    ));
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn private_methods_excludes_a_mixin_when_ancestors_are_excluded() {
    let result = run(&format!(
        "{PRIVATE_FIXTURE}\nChild.new.private_methods(false).include?(:mixed_in)"
    ));
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn private_methods_rejects_extra_arguments() {
    let error = run_err("Object.new.private_methods(true, false)");
    assert!(error.contains("expected 1 argument(s) but received 2"));
}

// ── =~ against a Symbol ──────────────────────────────────────────────────────

#[test]
fn a_regexp_matches_a_symbol_by_its_name() {
    let result = run(r"(/_secret\z/ =~ :child_secret)");
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn a_symbol_matches_a_regexp_on_the_left() {
    let result = run(r"(:child_secret =~ /_secret\z/)");
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn a_regexp_that_misses_a_symbol_is_nil() {
    let result = run(r"(/nope\z/ =~ :child_secret).inspect");
    assert_eq!(result, Some(Object::String(Rc::new("nil".to_string()))));
}

#[test]
fn not_match_negates_a_symbol_match() {
    let result = run(r"(:child_secret !~ /_secret\z/)");
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── Object#protected_methods ─────────────────────────────────────────────────

const PROTECTED_FIXTURE: &str = r#"
module Helpers
  def mixed_in
  end
  protected :mixed_in
end

class Parent
  def parent_guard
  end
  protected :parent_guard

  class << self
    def parent_class_guard
    end
    protected :parent_class_guard
  end
end

class Child < Parent
  include Helpers

  def child_guard
  end
  protected :child_guard

  class << self
    def child_class_guard
    end
    protected :child_class_guard
  end
end
"#;

fn guards(code: &str) -> Option<Object> {
    run(&format!(
        "{PROTECTED_FIXTURE}\n({code}).select {{ |n| n.to_s.end_with?(\"guard\") }}.sort.inspect"
    ))
}

#[test]
fn protected_methods_without_ancestors_lists_only_the_objects_own() {
    let result = guards("Child.new.protected_methods(false)");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:child_guard]".to_string())))
    );
}

#[test]
fn protected_methods_with_ancestors_reaches_the_superclass() {
    let result = guards("Child.new.protected_methods");
    assert_eq!(
        result,
        Some(Object::String(Rc::new(
            "[:child_guard, :parent_guard]".to_string()
        )))
    );
}

#[test]
fn protected_methods_treats_nil_like_false() {
    let result = guards("Child.new.protected_methods(nil)");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:child_guard]".to_string())))
    );
}

#[test]
fn a_classs_protected_methods_come_from_its_singleton_chain() {
    let result = guards("Child.protected_methods(false)");
    assert_eq!(
        result,
        Some(Object::String(Rc::new(
            "[:child_class_guard, :parent_class_guard]".to_string()
        )))
    );
}

#[test]
fn protected_methods_includes_a_module_attached_by_extend() {
    let result = run(&format!(
        "{PROTECTED_FIXTURE}\nobj = Object.new\nobj.extend(Helpers)\nobj.protected_methods.include?(:mixed_in)"
    ));
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn protected_methods_excludes_a_mixin_when_ancestors_are_excluded() {
    let result = run(&format!(
        "{PROTECTED_FIXTURE}\nChild.new.protected_methods(false).include?(:mixed_in)"
    ));
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn protected_methods_includes_a_singleton_method() {
    let result = run(r#"
widget = Object.new
class << widget
  def singleton_guard
  end
  protected :singleton_guard
end
widget.protected_methods(false).inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:singleton_guard]".to_string())))
    );
}

#[test]
fn protected_methods_rejects_extra_arguments() {
    let error = run_err("Object.new.protected_methods(true, false)");
    assert!(error.contains("expected 1 argument(s) but received 2"));
}

// ── Object#public_method ─────────────────────────────────────────────────────

const VAULT: &str = r#"
class Vault
  def open_door
    :opened
  end

  def hidden
    :hidden
  end
  private :hidden

  def guarded
    :guarded
  end
  protected :guarded

  def self.build
    :built
  end
end
"#;

#[test]
fn public_method_returns_a_bound_method_for_a_public_name() {
    let result = run(&format!(
        "{VAULT}\nVault.new.public_method(:open_door).call.inspect"
    ));
    assert_eq!(result, Some(Object::String(Rc::new(":opened".to_string()))));
}

#[test]
fn public_method_reaches_a_class_method() {
    let result = run(&format!(
        "{VAULT}\nVault.public_method(:build).call.inspect"
    ));
    assert_eq!(result, Some(Object::String(Rc::new(":built".to_string()))));
}

#[test]
fn public_method_refuses_a_private_name() {
    let error = run_err(&format!("{VAULT}\nVault.new.public_method(:hidden)"));
    assert!(error.contains("undefined method 'hidden' for class 'Vault'"));
}

#[test]
fn public_method_refuses_a_protected_name() {
    let error = run_err(&format!("{VAULT}\nVault.new.public_method(:guarded)"));
    assert!(error.contains("undefined method 'guarded' for class 'Vault'"));
}

#[test]
fn method_still_answers_a_private_name() {
    let result = run(&format!("{VAULT}\nVault.new.method(:hidden).call.inspect"));
    assert_eq!(result, Some(Object::String(Rc::new(":hidden".to_string()))));
}

const GHOST: &str = r#"
class Ghost
  def respond_to_missing?(name, include_private = false)
    return true if name == :publicly_handled
    include_private && name == :privately_handled
  end

  def method_missing(name, *args)
    "called #{name}"
  end
end
"#;

#[test]
fn public_method_asks_respond_to_missing_without_private() {
    let result = run(&format!(
        "{GHOST}\nGhost.new.public_method(:publicly_handled).call"
    ));
    assert_eq!(
        result,
        Some(Object::String(Rc::new(
            "called publicly_handled".to_string()
        )))
    );
}

#[test]
fn public_method_refuses_a_name_only_claimed_privately() {
    let error = run_err(&format!(
        "{GHOST}\nGhost.new.public_method(:privately_handled)"
    ));
    assert!(error.contains("undefined method 'privately_handled'"));
}

#[test]
fn method_accepts_a_name_claimed_privately() {
    let result = run(&format!(
        "{GHOST}\nGhost.new.method(:privately_handled).call"
    ));
    assert_eq!(
        result,
        Some(Object::String(Rc::new(
            "called privately_handled".to_string()
        )))
    );
}

#[test]
fn a_class_method_named_public_method_wins_over_the_native() {
    let result = run(r#"
class Parent
  def self.public_method
    :its_own
  end
end
Parent.public_method.inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new(":its_own".to_string())))
    );
}

// ── Object#public_methods ────────────────────────────────────────────────────

const PUBLIC_FIXTURE: &str = r#"
module Helpers
  def mixed_in_open
  end
end

class Parent
  def parent_open
  end

  def parent_shut
  end
  private :parent_shut

  def self.parent_class_open
  end
end

class Child < Parent
  include Helpers

  def child_open
  end

  def child_guarded
  end
  protected :child_guarded

  def self.child_class_open
  end
end
"#;

fn opens(code: &str) -> Option<Object> {
    run(&format!(
        "{PUBLIC_FIXTURE}\n({code}).select {{ |n| n.to_s.include?(\"open\") }}.sort.inspect"
    ))
}

#[test]
fn public_methods_without_ancestors_lists_only_the_objects_own() {
    let result = opens("Child.new.public_methods(false)");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:child_open]".to_string())))
    );
}

#[test]
fn public_methods_with_ancestors_reaches_the_superclass_and_mixins() {
    let result = opens("Child.new.public_methods");
    assert_eq!(
        result,
        Some(Object::String(Rc::new(
            "[:child_open, :mixed_in_open, :parent_open]".to_string()
        )))
    );
}

#[test]
fn public_methods_treats_nil_like_false() {
    let result = opens("Child.new.public_methods(nil)");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:child_open]".to_string())))
    );
}

#[test]
fn a_classs_public_methods_are_its_class_methods() {
    let result = opens("Child.public_methods(false)");
    assert_eq!(
        result,
        Some(Object::String(Rc::new(
            "[:child_class_open, :parent_class_open]".to_string()
        )))
    );
}

#[test]
fn public_methods_leaves_out_a_protected_name() {
    let result = run(&format!(
        "{PUBLIC_FIXTURE}\nChild.new.public_methods.include?(:child_guarded)"
    ));
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn public_methods_leaves_out_a_private_name() {
    let result = run(&format!(
        "{PUBLIC_FIXTURE}\nChild.new.public_methods.include?(:parent_shut)"
    ));
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn public_methods_lists_an_immediates_native_methods() {
    let result = run("1.public_methods.include?(:divmod)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Integer#divmod ───────────────────────────────────────────────────────────

#[test]
fn divmod_floors_toward_negative_infinity() {
    let result = run("[13.divmod(4), 13.divmod(-4), (-13).divmod(4), (-13).divmod(-4)].inspect");
    assert_eq!(
        result,
        Some(Object::String(Rc::new(
            "[[3, 1], [-4, -3], [-4, 3], [3, -1]]".to_string()
        )))
    );
}

#[test]
fn divmod_by_a_float_gives_an_integer_quotient() {
    let result = run("13.divmod(4.0).first");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn divmod_by_zero_raises() {
    let error = run_err("1.divmod(0)");
    assert!(error.contains("divided by 0"));
}

#[test]
fn divmod_rejects_a_non_numeric_divisor() {
    let error = run_err(r#"1.divmod("two")"#);
    assert!(error.contains("Integer or Float"));
}

#[test]
fn divmod_requires_one_argument() {
    let error = run_err("1.divmod");
    assert!(error.contains("expected 1 argument(s) but received 0"));
}

// ── Kernel#remove_instance_variable ──────────────────────────────────────────

const GREETER: &str = r#"
class Greeter
  def initialize
    @greeting = "hello"
    @name = "world"
  end
end
"#;

#[test]
fn remove_instance_variable_answers_the_value_it_took() {
    let result = run(&format!(
        "{GREETER}\nGreeter.new.remove_instance_variable(:@greeting)"
    ));
    assert_eq!(result, Some(Object::String(Rc::new("hello".to_string()))));
}

#[test]
fn remove_instance_variable_takes_the_variable_off() {
    let result = run(&format!(
        r#"
{GREETER}
greeter = Greeter.new
greeter.remove_instance_variable(:@greeting)
greeter.instance_variables.inspect
"#
    ));
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:@name]".to_string())))
    );
}

#[test]
fn remove_instance_variable_accepts_a_string_name() {
    let result = run(&format!(
        r#"{GREETER}{}"#,
        "\nGreeter.new.remove_instance_variable(\"@name\")"
    ));
    assert_eq!(result, Some(Object::String(Rc::new("world".to_string()))));
}

#[test]
fn remove_instance_variable_converts_its_argument_with_to_str() {
    let result = run(&format!(
        r#"
{GREETER}
class Name
  def to_str
    "@greeting"
  end
end
Greeter.new.remove_instance_variable(Name.new)
"#
    ));
    assert_eq!(result, Some(Object::String(Rc::new("hello".to_string()))));
}

#[test]
fn remove_instance_variable_raises_for_an_undefined_variable() {
    let error = run_err(&format!(
        "{GREETER}\nGreeter.new.remove_instance_variable(:@unknown)"
    ));
    assert!(error.contains("instance variable @unknown not defined"));
}

#[test]
fn remove_instance_variable_raises_for_an_invalid_name() {
    let error = run_err(&format!(
        "{GREETER}\nGreeter.new.remove_instance_variable(:\"@0\")"
    ));
    assert!(error.contains("`@0' is not allowed as an instance variable name"));
}

#[test]
fn remove_instance_variable_raises_type_error_without_to_str() {
    let error = run_err(&format!(
        "{GREETER}\nGreeter.new.remove_instance_variable(Object.new)"
    ));
    assert!(error.contains("no implicit conversion of Object into String"));
}

#[test]
fn remove_instance_variable_raises_on_a_frozen_object() {
    let error = run_err(&format!(
        r#"
{GREETER}
greeter = Greeter.new
greeter.freeze
greeter.remove_instance_variable(:@greeting)
"#
    ));
    assert!(error.contains("can't modify frozen Greeter"));
}

#[test]
fn remove_instance_variable_validates_the_name_before_the_frozen_check() {
    let error = run_err("nil.remove_instance_variable(:not_a_variable)");
    assert!(error.contains("`not_a_variable' is not allowed as an instance variable name"));
}

#[test]
fn remove_instance_variable_is_public_on_kernel() {
    let result = run("Kernel.public_instance_methods(false).include?(:remove_instance_variable)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn kernel_lists_its_native_instance_methods() {
    let result = run("Kernel.instance_methods(false).include?(:instance_variable_get)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── respond_to? consults respond_to_missing? ─────────────────────────────────

const RESPOND_GHOST: &str = r#"
class Ghost
  def respond_to_missing?(name, include_private = false)
    return true if name == :publicly_handled
    include_private && name == :privately_handled
  end
end
"#;

#[test]
fn respond_to_answers_a_name_claimed_by_respond_to_missing() {
    let result = run(&format!(
        "{RESPOND_GHOST}\nGhost.new.respond_to?(:publicly_handled)"
    ));
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn respond_to_passes_false_for_the_private_flag_by_default() {
    let result = run(&format!(
        "{RESPOND_GHOST}\nGhost.new.respond_to?(:privately_handled)"
    ));
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn respond_to_passes_the_private_flag_it_was_given() {
    let result = run(&format!(
        "{RESPOND_GHOST}\nGhost.new.respond_to?(:privately_handled, true)"
    ));
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn respond_to_is_false_for_a_name_nothing_claims() {
    let result = run(&format!(
        "{RESPOND_GHOST}\nGhost.new.respond_to?(:not_handled)"
    ));
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn a_class_can_override_respond_to_missing_for_itself() {
    let result = run(r#"
class Registry
  def self.respond_to_missing?(name, include_private = false)
    name == :lookup
  end
end
[Registry.respond_to?(:lookup), Registry.respond_to?(:missing_entirely)].inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[true, false]".to_string())))
    );
}

// ── The default respond_to_missing? ──────────────────────────────────────────

#[test]
fn every_object_answers_respond_to_missing_with_false() {
    let result = run("Object.new.respond_to_missing?(:anything, true)");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn every_object_responds_to_respond_to_missing() {
    let result = run("Object.new.respond_to?(:respond_to_missing?, true)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn a_class_responds_to_respond_to_missing() {
    let result = run("Object.respond_to?(:respond_to_missing?, true)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn respond_to_missing_is_private_on_kernel() {
    let result = run("Kernel.private_instance_methods(false).include?(:respond_to_missing?)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Method#owner answers the module ──────────────────────────────────────────

#[test]
fn a_native_kernel_methods_owner_is_the_kernel_module() {
    let result = run("Kernel.method(:respond_to_missing?).owner == Kernel");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── respond_to? and native class-method visibility ───────────────────────────

const SEALED: &str = r#"
class Sealed
  class << self
    private :new
  end
end
"#;

#[test]
fn respond_to_is_false_for_a_private_native_class_method() {
    let result = run(&format!("{SEALED}\nSealed.respond_to?(:new)"));
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn respond_to_with_private_allowed_finds_it() {
    let result = run(&format!("{SEALED}\nSealed.respond_to?(:new, true)"));
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn calling_a_private_native_class_method_raises() {
    let error = run_err(&format!("{SEALED}\nSealed.new"));
    assert!(error.contains("private method 'new' called for Sealed"));
}

#[test]
fn a_public_class_still_answers_new() {
    let result = run(r#"
class Open
end
Open.respond_to?(:new)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn a_class_responds_to_modules_native_methods() {
    let result = run("Object.respond_to?(:instance_methods)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn respond_to_converts_its_argument_with_to_str() {
    let result = run(r#"
class Named
  def to_str
    "upcase"
  end
end
"text".respond_to?(Named.new)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn respond_to_reports_an_uncoercible_argument() {
    let error = run_err("Object.new.respond_to?(42)");
    assert!(error.contains("42 is not a symbol nor a string"));
}

// ── Kernel#singleton_method ──────────────────────────────────────────────────

#[test]
fn singleton_method_finds_a_def_on_the_object() {
    let result = run(r#"
widget = Object.new
def widget.polish
  :shiny
end
widget.singleton_method(:polish).call.inspect
"#);
    assert_eq!(result, Some(Object::String(Rc::new(":shiny".to_string()))));
}

#[test]
fn singleton_method_answers_a_method_object() {
    let result = run(r#"
widget = Object.new
def widget.polish
end
widget.singleton_method(:polish).class.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Method".to_string()))));
}

#[test]
fn singleton_method_finds_a_module_included_in_the_singleton_class() {
    let result = run(r#"
included = Module.new do
  def from_include
    :included
  end
end
widget = Object.new
widget.singleton_class.include(included)
widget.singleton_method(:from_include).call.inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new(":included".to_string())))
    );
}

#[test]
fn singleton_method_finds_a_module_attached_by_extend() {
    let result = run(r#"
extension = Module.new do
  def from_extend
    :extended
  end
end
widget = Object.new
widget.extend(extension)
widget.singleton_method(:from_extend).call.inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new(":extended".to_string())))
    );
}

#[test]
fn singleton_method_finds_a_class_method() {
    let result = run(r#"
class Registry
  def self.lookup
    :found
  end
end
Registry.singleton_method(:lookup).call.inspect
"#);
    assert_eq!(result, Some(Object::String(Rc::new(":found".to_string()))));
}

#[test]
fn singleton_method_does_not_look_at_the_objects_class() {
    let error = run_err(
        r#"
class Widget
  def instance_level
  end
end
Widget.new.singleton_method(:instance_level)
"#,
    );
    assert!(error.contains("undefined singleton method 'instance_level'"));
}

#[test]
fn singleton_method_raises_for_a_name_nothing_defines() {
    let error = run_err("Object.new.singleton_method(:never_defined)");
    assert!(error.contains("undefined singleton method 'never_defined'"));
}

// ── Kernel#singleton_methods ─────────────────────────────────────────────────

#[test]
fn singleton_methods_is_empty_for_a_plain_object() {
    let result = run("Object.new.singleton_methods.inspect");
    assert_eq!(result, Some(Object::String(Rc::new("[]".to_string()))));
}

#[test]
fn singleton_methods_lists_a_def_on_the_object() {
    let result = run(r#"
widget = Object.new
def widget.polish
end
widget.singleton_methods.inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:polish]".to_string())))
    );
}

#[test]
fn singleton_methods_includes_an_extended_module_by_default() {
    let result = run(r#"
module Greeting
  def greet
  end
end
widget = Object.new
widget.extend(Greeting)
widget.singleton_methods.inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[:greet]".to_string())))
    );
}

#[test]
fn singleton_methods_without_ancestors_leaves_out_an_extended_module() {
    let result = run(r#"
module Greeting
  def greet
  end
end
widget = Object.new
widget.extend(Greeting)
widget.singleton_methods(false).inspect
"#);
    assert_eq!(result, Some(Object::String(Rc::new("[]".to_string()))));
}

const SINGLETON_CLASSES: &str = r#"
class Parent
  def self.parent_class_method
  end
end

class Child < Parent
  def self.child_class_method
  end

  class << self
    def opened_on_child
    end

    private

    def hidden_class_method
    end
  end
end
"#;

#[test]
fn singleton_methods_reaches_an_inherited_class_method() {
    let result = run(&format!(
        "{SINGLETON_CLASSES}\nChild.singleton_methods.sort.inspect"
    ));
    assert_eq!(
        result,
        Some(Object::String(Rc::new(
            "[:child_class_method, :opened_on_child, :parent_class_method]".to_string()
        )))
    );
}

#[test]
fn singleton_methods_without_ancestors_stops_at_the_class() {
    let result = run(&format!(
        "{SINGLETON_CLASSES}\nChild.singleton_methods(false).sort.inspect"
    ));
    assert_eq!(
        result,
        Some(Object::String(Rc::new(
            "[:child_class_method, :opened_on_child]".to_string()
        )))
    );
}

#[test]
fn singleton_methods_leaves_out_a_private_class_method() {
    let result = run(&format!(
        "{SINGLETON_CLASSES}\nChild.singleton_methods.include?(:hidden_class_method)"
    ));
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── extend self ──────────────────────────────────────────────────────────────

#[test]
fn extend_self_makes_a_modules_methods_callable_on_it() {
    let result = run(r#"
module Helper
  extend self

  def assist
    :assisted
  end
end
Helper.assist.inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new(":assisted".to_string())))
    );
}

#[test]
fn top_level_extend_makes_the_methods_callable() {
    let result = run(r#"
module Helper
  def assist
    :assisted
  end
end
extend Helper
assist.inspect
"#);
    assert_eq!(
        result,
        Some(Object::String(Rc::new(":assisted".to_string())))
    );
}

// ── Array slicing with a Range ───────────────────────────────────────────────

#[test]
fn an_array_slices_with_an_inclusive_range() {
    let result = run("[1, 2, 3, 4, 5][1..3].inspect");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[2, 3, 4]".to_string())))
    );
}

#[test]
fn an_array_slices_with_an_exclusive_range() {
    let result = run("[1, 2, 3, 4, 5][1...3].inspect");
    assert_eq!(result, Some(Object::String(Rc::new("[2, 3]".to_string()))));
}

#[test]
fn an_array_slices_with_an_endless_range() {
    let result = run("[1, 2, 3, 4, 5][2..].inspect");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[3, 4, 5]".to_string())))
    );
}

#[test]
fn an_array_slices_with_a_beginless_range() {
    let result = run("[1, 2, 3, 4, 5][..2].inspect");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("[1, 2, 3]".to_string())))
    );
}

#[test]
fn an_array_slice_counts_a_negative_bound_from_the_end() {
    let result = run("[1, 2, 3, 4, 5][-2..].inspect");
    assert_eq!(result, Some(Object::String(Rc::new("[4, 5]".to_string()))));
}

#[test]
fn an_array_slice_past_the_end_is_nil() {
    let result = run("[1, 2, 3][9..].inspect");
    assert_eq!(result, Some(Object::String(Rc::new("nil".to_string()))));
}

#[test]
fn an_array_slice_at_the_end_is_empty() {
    let result = run("[1, 2, 3][3..].inspect");
    assert_eq!(result, Some(Object::String(Rc::new("[]".to_string()))));
}
