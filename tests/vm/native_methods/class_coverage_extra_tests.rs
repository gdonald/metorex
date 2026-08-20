// Targeted coverage tests for uncovered lines in class_methods.rs.

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

// ── `constants` on a class (lines 82-90) ─────────────────────────────────────

#[test]
fn class_constants_returns_uppercase_names() {
    let result = run(r#"
class Colors
  RED = 1
  BLUE = 2
  lower = 3
end
Colors.constants.length
"#);
    if let Some(Object::Int(n)) = result {
        assert!(n >= 2, "Expected at least 2 constants, got {}", n);
    } else {
        panic!("expected Int, got {:?}", result);
    }
}

#[test]
fn class_constants_filters_internal() {
    let result = run(r#"
class Empty
end
Empty.constants
"#);
    assert!(matches!(result, Some(Object::Array(_))));
}

// ── `attached_object` on non-singleton class (lines 92-101) ─────────────────

#[test]
fn attached_object_on_non_singleton_errors() {
    let err = run_err(
        r#"
class Plain
end
Plain.attached_object
"#,
    );
    assert!(err.contains("singleton"));
}

#[test]
fn attached_object_on_instance_singleton_returns_instance() {
    // Singleton class's attached_object returns the original receiver.
    let result = run(r#"
class Subject
end
obj = Subject.new
sc = obj.singleton_class
sc.attached_object.class.name
"#);
    assert_eq!(result, Some(Object::string("Subject")));
}

#[test]
fn attached_object_on_nil_singleton_errors() {
    let err = run_err("nil.singleton_class.attached_object");
    assert!(err.contains("singleton"));
}

#[test]
fn attached_object_on_true_singleton_errors() {
    let err = run_err("true.singleton_class.attached_object");
    assert!(err.contains("singleton"));
}

#[test]
fn attached_object_on_false_singleton_errors() {
    let err = run_err("false.singleton_class.attached_object");
    assert!(err.contains("singleton"));
}

// ── `Class.new(singleton_class)` error (lines 142-146) ───────────────────────

#[test]
fn class_new_with_singleton_superclass_errors() {
    let err = run_err(
        r#"
class Host
end
sc = Host.new.singleton_class
Class.new(sc)
"#,
    );
    assert!(err.contains("singleton"));
}

// ── `include` with Class argument ────────────────────────────────────────────

#[test]
fn include_with_class_argument_via_send_raises_type_error() {
    let err = run_err(
        r#"
class Mixin
  def shared
    "shared"
  end
end
class Host2
end
Host2.send(:include, Mixin)
"#,
    );
    assert!(err.contains("Module"), "unexpected error: {err}");
}

#[test]
fn include_via_send_with_non_module_errors() {
    let err = run_err(
        r#"
class Host3
end
Host3.send(:include, 42)
"#,
    );
    assert!(err.contains("Module") || err.contains("argument"));
}

#[test]
fn prepend_via_send_with_module() {
    let result = run(r#"
module Pre
  def greet
    "pre"
  end
end
class PreHost
end
PreHost.send(:prepend, Pre)
PreHost.new.greet
"#);
    assert_eq!(result, Some(Object::string("pre")));
}

// ── `File.join` with Symbol / Array elements (lines 273, 281) ────────────────

#[test]
fn file_join_with_symbols() {
    let result = run(r#"File.join(:foo, :bar)"#);
    assert_eq!(result, Some(Object::string("foo/bar")));
}

#[test]
fn file_join_with_array_elements() {
    let result = run(r#"File.join(["a", "b"], "c")"#);
    assert_eq!(result, Some(Object::string("a/b/c")));
}

#[test]
fn file_join_with_non_string_ignored() {
    let result = run(r#"File.join("a", 42, "b")"#);
    assert_eq!(result, Some(Object::string("a/b")));
}

// ── `File.respond_to?` (lines 289-305) ───────────────────────────────────────

#[test]
fn file_respond_to_known_method() {
    let result = run(r#"File.respond_to?("dirname")"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn file_respond_to_symbol_method() {
    let result = run(r#"File.respond_to?(:join)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn file_respond_to_unknown_method() {
    let result = run(r#"File.respond_to?(:nonexistent_zzz)"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn file_respond_to_non_string_arg_returns_false() {
    let result = run(r#"File.respond_to?(42)"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── `Thread.respond_to?` (lines 340-352) ─────────────────────────────────────

#[test]
fn thread_respond_to_known_method() {
    let result = run(r#"Thread.respond_to?(:pass)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn thread_respond_to_unknown_method() {
    let result = run(r#"Thread.respond_to?(:nonexistent_yyy)"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn thread_respond_to_non_string_arg_returns_false() {
    let result = run(r#"Thread.respond_to?(42)"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn thread_pass_is_nil() {
    let result = run(r#"Thread.pass"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn thread_current_is_nil() {
    let result = run(r#"Thread.current"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn thread_main_is_nil() {
    let result = run(r#"Thread.main"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn thread_report_on_exception_returns_true() {
    let result = run(r#"Thread.report_on_exception"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── `instance_method` / `public_instance_method` errors (lines 406-416) ─────

#[test]
fn instance_method_non_string_non_symbol_errors() {
    let err = run_err(
        r#"
class A
  def m
  end
end
A.instance_method(42)
"#,
    );
    assert!(
        err.contains("42 is not a symbol nor a string"),
        "unexpected error: {err}"
    );
}

#[test]
fn public_instance_method_non_string_errors() {
    let err = run_err(
        r#"
class B
  def m
  end
end
B.public_instance_method(42)
"#,
    );
    assert!(
        err.contains("42 is not a symbol nor a string"),
        "unexpected error: {err}"
    );
}

#[test]
fn public_instance_method_returns_method_object() {
    let result = run(r#"
class C
  def greet
    "hi"
  end
end
C.public_instance_method(:greet)
"#);
    assert!(matches!(result, Some(Object::Method(_))));
}

#[test]
fn instance_method_undefined_errors() {
    let err = run_err(
        r#"
class D
end
D.instance_method(:missing)
"#,
    );
    assert!(err.contains("undefined") || err.contains("missing"));
}

// ── `instance_methods` with `false` (only own methods) ───────────────────────

#[test]
fn instance_methods_false_skips_inherited() {
    let result = run(r#"
class Base1
  def base_only
    1
  end
end
class Child1 < Base1
  def child_only
    2
  end
end
methods = Child1.instance_methods(false)
methods.include?(:child_only) && !methods.include?(:base_only)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn private_instance_methods_filter_excludes_public() {
    let result = run(r#"
class Mix
  def visible
    1
  end
  private
  def hidden
    2
  end
end
Mix.private_instance_methods(false).include?(:hidden)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn protected_instance_methods_returns_empty() {
    let result = run(r#"
class P
  def pub
    1
  end
end
P.protected_instance_methods(false).length
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn module_instance_methods_advertises_natives() {
    let result = run(r#"Module.instance_methods.include?(:alias_method)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn class_instance_methods_advertises_natives() {
    let result = run(r#"Class.instance_methods.include?(:define_method)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── `extend` method on a Class (lines 523-545) ───────────────────────────────

#[test]
fn extend_class_with_module() {
    let result = run(r#"
module Greeter
  def hi
    "hi"
  end
end
class Target
end
Target.extend(Greeter)
Target.hi
"#);
    assert_eq!(result, Some(Object::string("hi")));
}

#[test]
fn extend_with_no_args_errors() {
    let err = run_err(
        r#"
class T1
end
T1.extend
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn extend_with_non_module_errors() {
    let err = run_err(
        r#"
class T2
end
T2.extend(42)
"#,
    );
    assert!(err.contains("Module") || err.contains("argument"));
}

#[test]
fn extend_with_class_works() {
    // Class is accepted wherever Module is, since Class.is_a?(Module).
    let result = run(r#"
class SrcClass
  def shared
    "class-mixin"
  end
end
class ExtTarget
end
ExtTarget.extend(SrcClass)
ExtTarget.shared
"#);
    assert_eq!(result, Some(Object::string("class-mixin")));
}

// ── `private_class_method` / `public_class_method` (lines 552-588) ──────────

#[test]
fn private_class_method_returns_receiver() {
    // private_class_method should return the receiver class.
    let result = run(r#"
class Locked
  def self.secret
    42
  end
end
Locked.send(:private_class_method, :secret)
"#);
    assert!(matches!(result, Some(Object::Class(_))));
}

#[test]
fn private_class_method_no_args_errors() {
    let err = run_err(
        r#"
class Bare
end
Bare.send(:private_class_method)
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn private_class_method_non_string_errors() {
    let err = run_err(
        r#"
class Bare2
  def self.m
  end
end
Bare2.send(:private_class_method, 42)
"#,
    );
    assert!(
        err.contains("42 is not a symbol nor a string"),
        "unexpected error: {err}"
    );
}

#[test]
fn public_class_method_returns_class() {
    let result = run(r#"
class Rev
  def self.method_back
    :here
  end
  private_class_method :method_back
  public_class_method :method_back
end
Rev.method_back
"#);
    assert!(matches!(result, Some(Object::Symbol(_))));
}

// ── `remove_const` (lines 591-613) ───────────────────────────────────────────

#[test]
fn remove_const_removes_and_returns_value() {
    let result = run(r#"
class Holder
  VAL = 99
end
Holder.send(:remove_const, :VAL)
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn remove_const_missing_errors() {
    let err = run_err(
        r#"
class Holder2
end
Holder2.send(:remove_const, :NOTDEFINED)
"#,
    );
    assert!(
        err.contains("constant Holder2::NOTDEFINED not defined"),
        "unexpected error: {err}"
    );
}

#[test]
fn remove_const_no_args_errors() {
    let err = run_err(
        r#"
class Holder3
end
Holder3.send(:remove_const)
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn remove_const_non_symbol_errors() {
    let err = run_err(
        r#"
class Holder4
end
Holder4.send(:remove_const, 42)
"#,
    );
    assert!(err.contains("Symbol") || err.contains("String") || err.contains("argument"));
}

#[test]
fn remove_const_with_string_name() {
    let result = run(r#"
class Holder5
  X = 7
end
Holder5.send(:remove_const, "X")
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

// ── `const_get` (lines 679-709) ──────────────────────────────────────────────

#[test]
fn const_get_returns_value() {
    let result = run(r#"
class K
  PI = 3
end
K.const_get(:PI)
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn const_get_with_string_name() {
    let result = run(r#"
class K2
  V = 11
end
K2.const_get("V")
"#);
    assert_eq!(result, Some(Object::Int(11)));
}

#[test]
fn const_get_undefined_errors() {
    let err = run_err(
        r#"
class K3
end
K3.const_get(:MISSING)
"#,
    );
    assert!(err.contains("uninitialized") || err.contains("MISSING") || err.contains("constant"));
}

#[test]
fn const_get_no_args_errors() {
    let err = run_err(
        r#"
class K4
end
K4.const_get
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn const_get_non_symbol_errors() {
    let err = run_err(
        r#"
class K5
end
K5.const_get(42)
"#,
    );
    assert!(
        err.contains("is not a symbol nor a string"),
        "unexpected error: {}",
        err
    );
}

// ── `alias_method` error path (lines 867-879) ────────────────────────────────

#[test]
fn alias_method_undefined_errors_via_send() {
    // send(:alias_method, ...) on a class with no superclass triggers the
    // NameError path when the target method doesn't exist anywhere.
    let err = run_err(
        r#"
class NoSuch
end
NoSuch.send(:alias_method, :new_one, :not_real_method_xyz)
"#,
    );
    assert!(
        err.contains("undefined") || err.contains("not_real_method") || err.contains("NameError")
    );
}

// ── `deprecate_constant` / `ruby2_keywords` no-op (line 931) ─────────────────

#[test]
fn deprecate_constant_is_noop() {
    let result = run(r#"
class DepHost
  OLD = 1
  deprecate_constant :OLD
end
DepHost::OLD
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn ruby2_keywords_is_noop() {
    let result = run(r#"
class K2Host
  def m
    1
  end
  ruby2_keywords :m
end
K2Host.new.m
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ── `autoload` / `autoload?` no-op (lines 194-196) ───────────────────────────

#[test]
fn autoload_is_noop() {
    let result = run(r#"
class AutoHost
  autoload :Foo, "foo.rb"
end
:ok
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("ok".to_string())))
    );
}

#[test]
fn autoload_query_is_noop() {
    let result = run(r#"
class AutoHost2
end
AutoHost2.autoload?(:Foo)
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── `Class.new` (no superclass arg) defaults to Object (line 156-162) ───────

#[test]
fn class_new_no_args_defaults_to_object_superclass() {
    let result = run(r#"
klass = Class.new
klass.superclass.name
"#);
    assert_eq!(result, Some(Object::string("Object")));
}

// ── `define_method` with variadic and block parameters (lines 764-770) ──────

#[test]
fn define_method_with_variadic_param_defines_successfully() {
    // Exercises the `*name` branch at lines 764-766.
    // We only verify the definition completes — invocation semantics for
    // block-created variadic methods are a separate concern.
    let result = run(r#"
class Vari
  define_method(:sum) do |*nums|
    nums
  end
end
:defined
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("defined".to_string())))
    );
}

#[test]
fn define_method_with_block_param_defines_successfully() {
    // Exercises the `&name` branch at lines 767-769.
    let result = run(r#"
class BlockParam
  define_method(:wrap) do |&blk|
    blk
  end
end
:defined
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(std::rc::Rc::new("defined".to_string())))
    );
}

#[test]
fn define_method_with_no_args_errors() {
    let err = run_err(
        r#"
class DefErr
end
DefErr.send(:define_method)
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn define_method_non_symbol_name_errors() {
    let err = run_err(
        r#"
class DefErr2
end
DefErr2.send(:define_method, 42)
"#,
    );
    assert!(err.contains("is not a symbol nor a string"));
}

#[test]
fn define_method_without_block_errors() {
    let err = run_err(
        r#"
class DefErr3
end
DefErr3.send(:define_method, :foo)
"#,
    );
    assert!(err.contains("block") || err.contains("define_method"));
}

// ── `name` on anonymous class returns nil (line 400) ─────────────────────────

#[test]
fn anonymous_class_name_is_nil() {
    let result = run(r#"
k = Class.new
k.name
"#);
    assert_eq!(result, Some(Object::Nil));
}

// ── `subclasses` returns direct children (line 223-229) ──────────────────────

#[test]
fn subclasses_returns_direct_children() {
    let result = run(r#"
class Parent
end
class ChildA < Parent; end
class ChildB < Parent; end
Parent.subclasses.length
"#);
    if let Some(Object::Int(n)) = result {
        assert!(n >= 2, "Expected 2+ subclasses, got {}", n);
    }
}

// ── `Module.nesting` returns current scope stack (line 233-242) ──────────────

#[test]
fn module_nesting_returns_array() {
    let result = run(r#"Module.nesting"#);
    assert!(matches!(result, Some(Object::Array(_))));
}

// ── `remove_method` / `undef_method` ─────────────────────────────────────────

#[test]
fn remove_method_deletes_definition() {
    let err = run_err(
        r#"
class Remover
  def doomed
    1
  end
  remove_method :doomed
end
Remover.new.doomed
"#,
    );
    assert!(err.contains("undefined") || err.contains("doomed"));
}

#[test]
fn remove_method_undefined_errors() {
    let err = run_err(
        r#"
class R2
end
R2.send(:remove_method, :nope)
"#,
    );
    assert!(err.contains("not defined") || err.contains("nope"));
}

#[test]
fn remove_method_without_arguments_returns_self() {
    let result = run(r#"
class R3
end
R3.send(:remove_method)
"#);
    assert!(matches!(result, Some(Object::Class(_))));
}

#[test]
fn remove_method_non_symbol_errors() {
    let err = run_err(
        r#"
class R4
end
R4.send(:remove_method, 42)
"#,
    );
    assert!(
        err.contains("42 is not a symbol nor a string"),
        "unexpected error: {err}"
    );
}

#[test]
fn undef_method_prevents_call() {
    let err = run_err(
        r#"
class U1
  def gone
    1
  end
  undef_method :gone
end
U1.new.gone
"#,
    );
    assert!(err.contains("undefined") || err.contains("gone"));
}

#[test]
fn undef_method_without_arguments_returns_self() {
    let result = run(r#"
class U2
end
U2.send(:undef_method)
"#);
    assert!(matches!(result, Some(Object::Class(_))));
}

#[test]
fn undef_method_non_symbol_errors() {
    let err = run_err(
        r#"
class U3
end
U3.send(:undef_method, 42)
"#,
    );
    assert!(
        err.contains("42 is not a symbol nor a string"),
        "unexpected error: {err}"
    );
}

// ── `alias_method` error paths ───────────────────────────────────────────────

#[test]
fn alias_method_wrong_arg_count_errors() {
    let err = run_err(
        r#"
class AE
end
AE.send(:alias_method, :x)
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn alias_method_on_frozen_class_errors() {
    let err = run_err(
        r#"
class Frz
  def a
    1
  end
end
Frz.freeze
Frz.send(:alias_method, :b, :a)
"#,
    );
    assert!(err.contains("frozen") || err.contains("FrozenError"));
}

// ── `Set.new` from array ─────────────────────────────────────────────────────

#[test]
fn set_new_from_array() {
    let result = run(r#"Set.new([1, 2, 3]).size"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn set_new_from_non_array_errors() {
    let err = run_err(r#"Set.new(42)"#);
    assert!(err.contains("Array") || err.contains("argument"));
}

#[test]
fn set_new_too_many_args_errors() {
    let err = run_err(r#"Set.new([1], [2])"#);
    assert!(err.contains("argument") || err.contains("expects"));
}

// ── attr_reader/writer/accessor invoked as method on a Class (lines 600-676) ─

#[test]
fn class_attr_reader_via_send_defines_getter() {
    let result = run(r#"
class Foo; end
Foo.send(:attr_reader, :name)
f = Foo.new
f.instance_variable_set(:@name, "alice")
f.name
"#);
    assert_eq!(
        result,
        Some(Object::String(std::rc::Rc::new("alice".to_string())))
    );
}

#[test]
fn class_attr_reader_via_send_returns_array_of_symbols() {
    let result = run(r#"
class Bar; end
Bar.send(:attr_reader, :a, :b).length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn class_attr_writer_via_send_defines_setter() {
    let result = run(r#"
class Baz; end
Baz.send(:attr_writer, :x)
b = Baz.new
b.x = 42
b.instance_variable_get(:@x)
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn class_attr_accessor_via_send_defines_both() {
    let result = run(r#"
class Qux; end
Qux.send(:attr_accessor, :v).length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn class_attr_via_send_with_string_arg() {
    let result = run(r#"
class Sx; end
Sx.send(:attr_reader, "name")
s = Sx.new
s.instance_variable_set(:@name, 5)
s.name
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn class_attr_reader_via_send_no_args_errors() {
    let err = run_err(
        r#"
class Eattr; end
Eattr.send(:attr_reader)
"#,
    );
    assert!(err.contains("argument"));
}

// ── method_defined?/public_method_defined?/private_method_defined? on Class ─

#[test]
fn class_method_defined_returns_true_for_defined() {
    let result = run(r#"
class M1
  def foo; end
end
M1.method_defined?(:foo)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn class_method_defined_returns_false_for_missing() {
    let result = run(r#"
class M2; end
M2.method_defined?(:bogus)
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn class_method_defined_walks_superclass() {
    let result = run(r#"
class Pmd
  def hello; end
end
class Cmd < Pmd; end
Cmd.method_defined?(:hello)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn class_method_defined_string_argument_works() {
    let result = run(r#"
class M3
  def bar; end
end
M3.method_defined?("bar")
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn class_method_defined_walks_mixins() {
    let result = run(r#"
module Helpful
  def helped; end
end
class WithMix
  include Helpful
end
WithMix.method_defined?(:helped)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn class_private_method_defined_true_for_private() {
    let result = run(r#"
class PrivCls
  def pubm; end
  private
  def privm; end
end
PrivCls.private_method_defined?(:privm)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn class_method_defined_no_args_errors() {
    let err = run_err(
        r#"
class M4; end
M4.method_defined?
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn class_method_defined_too_many_args_errors() {
    let err = run_err(
        r#"
class M5; end
M5.method_defined?(:a, true, :extra)
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn class_method_defined_non_string_arg_errors() {
    let err = run_err(
        r#"
class M6; end
M6.method_defined?(42)
"#,
    );
    assert!(
        err.contains("42 is not a symbol nor a string"),
        "unexpected error: {err}"
    );
}

#[test]
fn class_protected_method_defined_returns_false() {
    let result = run(r#"
class M7
  def foo; end
end
M7.protected_method_defined?(:foo)
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}
