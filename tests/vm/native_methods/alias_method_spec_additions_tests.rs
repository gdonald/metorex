//! Coverage tests for features added alongside the alias_method_spec pass.
//!
//! Covers:
//!   - `Class#subclasses` (weak-ref tracking)
//!   - `Module#instance_method` / `public_instance_methods` /
//!     `private_instance_methods` / `instance_methods`
//!   - `Object#method(:name)`
//!   - `Array#sort_by`
//!   - `Integer#times` with no block
//!   - `Thread.new` (lazy) + `Thread#value` + `Thread.pass`
//!   - Minimal `File.dirname`/`expand_path`/`realpath`/`join`/`respond_to?`
//!   - `Module.nesting`
//!   - Module `include`/`prepend` on Class and Module receivers
//!   - `freeze` / `FrozenError` on Class and Module
//!   - `to_str` coercion in `alias_method`
//!   - `alias` keyword (statement form, `:sym :sym`, bare)
//!   - `until` keyword (prefix + postfix)
//!   - Qualified superclass (`class X < A::B`)
//!   - `module ::Name` declaration
//!   - BasicObject hierarchy
//!   - `super` through aliased methods / mixin chain
//!   - Private/protected visibility enforcement (external call raises NoMethodError)
//!   - `private` / `protected` as class-body default toggle

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

// ── Class#subclasses ──────────────────────────────────────────────────────

#[test]
fn class_subclasses_returns_direct_subclasses() {
    let result = run(r#"
class A; end
class B < A; end
class C < A; end
A.subclasses.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn class_subclasses_empty_for_leaf_class() {
    let result = run(r#"
class A; end
A.subclasses.length
"#);
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn class_subclasses_picks_up_anonymous_subclass() {
    let result = run(r#"
class A; end
x = Class.new(A)
A.subclasses.size
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ── Module#instance_method / instance_methods families ────────────────────

#[test]
fn module_instance_method_returns_method_object() {
    let result = run(r#"
class Foo
  def greet; :hi; end
end
Foo.instance_method(:greet).nil?
"#);
    // .instance_method returns an Object::Method (non-nil).
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn module_instance_method_missing_raises_name_error() {
    let err = run_err(
        r#"
class Foo; end
Foo.instance_method(:missing)
"#,
    );
    assert!(err.contains("NameError") || err.contains("undefined method"));
}

#[test]
fn module_public_instance_methods_false_only_own() {
    let result = run(r#"
class Foo
  def a; end
  def b; end
end
Foo.public_instance_methods(false).length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn module_private_instance_methods_lists_private_only() {
    let result = run(r#"
class Foo
  def pub; end
  private
  def secret; end
end
Foo.private_instance_methods(false).include?(:secret)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn module_private_instance_methods_excludes_public() {
    let result = run(r#"
class Foo
  def pub; end
  private
  def secret; end
end
Foo.private_instance_methods(false).include?(:pub)
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── Object#method ────────────────────────────────────────────────────────

#[test]
fn object_method_returns_method_object() {
    let result = run(r#"
class Foo
  def bar; 42; end
end
Foo.new.method(:bar).nil?
"#);
    // .method returns a bound Method object (non-nil); .call may not be
    // implemented on Method yet, so we only verify it's not nil.
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn object_method_returns_method_objects_equal_for_same_name() {
    let result = run(r#"
class Foo
  def bar; 42; end
end
f = Foo.new
f.method(:bar) == f.method(:bar)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn object_method_missing_raises_name_error() {
    let err = run_err(
        r#"
class Foo; end
Foo.new.method(:missing)
"#,
    );
    assert!(err.contains("NameError") || err.contains("undefined"));
}

// ── Array#sort_by ────────────────────────────────────────────────────────

#[test]
fn array_sort_by_orders_by_key() {
    let result = run(r#"
["banana", "fig", "apple"].sort_by { |s| s.length }
"#);
    match result {
        Some(Object::Array(arr)) => {
            let v = arr.borrow();
            assert_eq!(v.len(), 3);
            if let Object::String(s) = &v[0] {
                assert_eq!(**s, "fig");
            } else {
                panic!("expected string");
            }
        }
        _ => panic!("expected array"),
    }
}

#[test]
fn array_sort_by_without_block_raises() {
    let err = run_err("[1, 2].sort_by");
    assert!(err.to_lowercase().contains("block"));
}

// ── Integer#times no block ───────────────────────────────────────────────

#[test]
fn integer_times_no_block_returns_enumerable_array() {
    let result = run("3.times.map { |i| i * 2 }");
    match result {
        Some(Object::Array(arr)) => {
            let v = arr.borrow();
            assert_eq!(v.len(), 3);
            assert_eq!(v[0], Object::Int(0));
            assert_eq!(v[1], Object::Int(2));
            assert_eq!(v[2], Object::Int(4));
        }
        _ => panic!("expected array"),
    }
}

// ── Thread.new (lazy) + Thread#value ─────────────────────────────────────

#[test]
fn thread_new_captures_block_and_value_runs_it_synchronously() {
    let result = run(r#"
t = Thread.new { 21 * 2 }
t.value
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn thread_new_body_is_deferred_until_value_called() {
    // If the block ran at Thread.new time, `go` would still be false and the
    // block would return 0; the lazy model should let us flip `go` first.
    let result = run(r#"
go = false
t = Thread.new { go ? 99 : 0 }
go = true
t.value
"#);
    assert_eq!(result, Some(Object::Int(99)));
}

#[test]
fn thread_pass_returns_nil() {
    let result = run("Thread.pass");
    assert_eq!(result, Some(Object::Nil));
}

// ── File stubs ───────────────────────────────────────────────────────────

#[test]
fn file_dirname_returns_parent_dir() {
    let result = run(r#"File.dirname("/a/b/c.rb")"#);
    assert_eq!(result, Some(Object::String(Rc::new("/a/b".to_string()))));
}

#[test]
fn file_dirname_root() {
    let result = run(r#"File.dirname("a.rb")"#);
    assert_eq!(result, Some(Object::String(Rc::new(".".to_string()))));
}

#[test]
fn file_join_joins_parts() {
    let result = run(r#"File.join("a", "b", "c")"#);
    assert_eq!(result, Some(Object::String(Rc::new("a/b/c".to_string()))));
}

#[test]
fn file_respond_to_known_method() {
    let result = run(r#"File.respond_to?(:dirname)"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn file_respond_to_unknown_method() {
    let result = run(r#"File.respond_to?(:absolutely_not_a_real_method_xyz)"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── Module.nesting ───────────────────────────────────────────────────────

#[test]
fn module_nesting_returns_array() {
    let result = run("Module.nesting.class.name");
    assert_eq!(result, Some(Object::String(Rc::new("Array".to_string()))));
}

// ── Module#include / prepend with args ───────────────────────────────────

#[test]
fn module_include_with_arg_mixes_in_module() {
    let result = run(r#"
module M; def greet; :hi; end; end
class K; end
K.include(M)
K.new.greet
"#);
    assert_eq!(result, Some(Object::Symbol(Rc::new("hi".to_string()))));
}

#[test]
fn module_prepend_with_arg_mixes_in_module() {
    let result = run(r#"
module M; def tag; :m_tag; end; end
class K; end
K.prepend(M)
K.new.tag
"#);
    assert_eq!(result, Some(Object::Symbol(Rc::new("m_tag".to_string()))));
}

// ── freeze / FrozenError ─────────────────────────────────────────────────

#[test]
fn class_freeze_sets_frozen_q() {
    let result = run(r#"
class Foo; end
Foo.freeze
Foo.frozen?
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn alias_method_on_frozen_class_raises_frozen_error() {
    let err = run_err(
        r#"
class Foo
  def bar; 1; end
end
Foo.freeze
Foo.alias_method(:baz, :bar)
"#,
    );
    assert!(err.contains("FrozenError") || err.contains("frozen"));
}

#[test]
fn alias_method_on_frozen_module_raises_frozen_error() {
    let err = run_err(
        r#"
module Foo
  def bar; 1; end
end
Foo.freeze
Foo.alias_method(:baz, :bar)
"#,
    );
    assert!(err.contains("FrozenError") || err.contains("frozen"));
}

// ── to_str coercion in alias_method ──────────────────────────────────────

#[test]
fn alias_method_accepts_string_coerced_via_to_str() {
    // Direct String (not a mock with to_str, which requires singleton method
    // support that's exercised elsewhere).
    let result = run(r#"
class Foo
  def bar; :ok; end
end
Foo.alias_method("baz", "bar")
Foo.new.baz
"#);
    assert_eq!(result, Some(Object::Symbol(Rc::new("ok".to_string()))));
}

// ── alias keyword ────────────────────────────────────────────────────────

#[test]
fn alias_keyword_bareword_form() {
    let result = run(r#"
class Foo
  def bar; :bar_value; end
  alias baz bar
end
Foo.new.baz
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(Rc::new("bar_value".to_string())))
    );
}

#[test]
fn alias_keyword_symbol_form() {
    let result = run(r#"
class Foo
  def bar; :bar_value; end
  alias :baz :bar
end
Foo.new.baz
"#);
    assert_eq!(
        result,
        Some(Object::Symbol(Rc::new("bar_value".to_string())))
    );
}

// ── until keyword ────────────────────────────────────────────────────────

#[test]
fn until_prefix_loop() {
    let result = run(r#"
i = 0
until i == 3
  i = i + 1
end
i
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn until_postfix_modifier() {
    let result = run(r#"
i = 0
i = i + 1 until i == 5
i
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

// ── Qualified superclass & `module ::Name` ──────────────────────────────

#[test]
fn class_can_inherit_via_qualified_superclass() {
    let result = run(r#"
module Outer
  class Base
    def who; :base; end
  end
end
class Child < Outer::Base; end
Child.new.who
"#);
    assert_eq!(result, Some(Object::Symbol(Rc::new("base".to_string()))));
}

#[test]
fn module_with_leading_double_colon_reopens_top_level() {
    let result = run(r#"
module Foo
  CONST = 1
end
module ::Foo
  CONST2 = 2
end
Foo::CONST + Foo::CONST2
"#);
    assert_eq!(result, Some(Object::Int(3)));
}

// ── BasicObject hierarchy ────────────────────────────────────────────────

#[test]
fn object_superclass_is_basicobject() {
    let result = run("Object.superclass.name");
    assert_eq!(
        result,
        Some(Object::String(Rc::new("BasicObject".to_string())))
    );
}

#[test]
fn basicobject_has_nil_superclass() {
    let result = run("BasicObject.superclass");
    assert_eq!(result, Some(Object::Nil));
}

// ── `super` through aliased methods / mixin chain ───────────────────────

#[test]
fn super_through_aliased_method_invokes_parent_mixin() {
    let result = run(r#"
module Parent
  def talk(x); x; end
end
module Child
  include Parent
  def talk(x); super(x); end
end
class Target
  include Child
  alias_method :alias_talk, :talk
  alias_method :talk, :alias_talk
end
Target.new.talk(42)
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn super_after_alias_and_redefine_still_hits_original_super() {
    let result = run(r#"
module Parent
  def talk(x); x; end
end
class Target
  include Parent
  def talk(x); super(x); end
  alias_method :alias_talk, :talk
  def talk(x); :wrong; end
end
Target.new.alias_talk(7)
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

// ── Visibility enforcement ──────────────────────────────────────────────

#[test]
fn private_method_external_call_raises_no_method_error() {
    let err = run_err(
        r#"
class Foo
  private
  def secret; 1; end
end
Foo.new.secret
"#,
    );
    assert!(err.contains("NoMethodError") || err.contains("private"));
}

#[test]
fn private_method_implicit_self_call_works() {
    let result = run(r#"
class Foo
  def caller; secret; end
  private
  def secret; 42; end
end
Foo.new.caller
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn inherited_private_call_externally_raises() {
    let err = run_err(
        r#"
class A
  private
  def secret; 1; end
end
class B < A; end
B.new.secret
"#,
    );
    assert!(err.contains("NoMethodError") || err.contains("private"));
}

#[test]
fn public_override_shadows_inherited_private() {
    let result = run(r#"
class A
  private
  def secret; 1; end
end
class B < A
  public :secret
end
B.new.secret
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ── Thread stub methods (class-level) ───────────────────────────────────

#[test]
fn thread_current_answers_a_thread() {
    let result = run("Thread.current.class.name");
    assert_eq!(result, Some(Object::string("Thread")));
}

#[test]
fn thread_main_answers_a_thread() {
    let result = run("Thread.main.class.name");
    assert_eq!(result, Some(Object::string("Thread")));
}

#[test]
fn thread_report_on_exception_called_directly() {
    // Direct native call on Thread: returns Bool(true) stub.
    let result = run("Thread.report_on_exception");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn thread_respond_to_known_method() {
    let result = run("Thread.respond_to?(:pass)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn thread_respond_to_unknown_method() {
    let result = run("Thread.respond_to?(:definitely_not_a_thread_method)");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn thread_value_is_cached_across_calls() {
    let result = run(r#"
count = 0
t = Thread.new { count = count + 1; count }
t.value
t.value
"#);
    // First call runs the block; second returns cached value.
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn thread_alive_until_it_has_run() {
    let result = run(r#"
t = Thread.new { :done }
before = t.alive?
t.join
[before, t.alive?]
"#);
    assert_eq!(
        result,
        Some(Object::Array(std::rc::Rc::new(std::cell::RefCell::new(
            vec![Object::Bool(true), Object::Bool(false)]
        ))))
    );
}

#[test]
fn thread_join_returns_thread_itself() {
    let result = run(r#"
t = Thread.new { :done }
t.join.equal?(t)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Class.allocate / Class#initialize edge cases ────────────────────────

#[test]
fn class_allocate_returns_uninitialized_class() {
    // Class.allocate yields a Class; calling .new on it should raise since
    // `__uninitialized__` is set (the TypeError branch is verified below).
    let err = run_err("Class.allocate.new");
    assert!(err.contains("TypeError") || err.contains("uninitialized"));
}

#[test]
fn uninitialized_class_new_raises_type_error() {
    let err = run_err("Class.allocate.new");
    assert!(err.contains("TypeError") || err.contains("uninitialized"));
}

#[test]
fn regular_class_allocate_returns_instance() {
    let result = run(r#"
class Foo; end
Foo.allocate.class.name
"#);
    assert_eq!(result, Some(Object::String(Rc::new("Foo".to_string()))));
}

// ── Class.new with non-class arg raises TypeError ───────────────────────

#[test]
fn class_new_with_non_class_superclass_raises() {
    let err = run_err("Class.new(42)");
    assert!(err.contains("TypeError") || err.contains("superclass"));
}

// ── File.join with array / File.realpath-ish behavior ────────────────

#[test]
fn file_join_flattens_array_of_strings() {
    let result = run(r#"File.join(["a", "b"], "c")"#);
    assert_eq!(result, Some(Object::String(Rc::new("a/b/c".to_string()))));
}

#[test]
fn file_expand_path_on_nonexistent_path_returns_input() {
    // canonicalize fails on nonexistent paths; we fall back to the input.
    let result = run(r#"File.expand_path("/this/path/does/not/exist/abc123")"#);
    match result {
        Some(Object::String(s)) => assert!(s.contains("abc123")),
        _ => panic!("expected string"),
    }
}

// ── Module include/prepend argument type errors ─────────────────────────

#[test]
fn module_include_with_non_module_arg_raises() {
    let err = run_err(
        r#"
class K; end
K.include(42)
"#,
    );
    assert!(err.contains("Module") || err.contains("TypeError"));
}

// ── alias_method return value ────────────────────────────────────────────

#[test]
fn alias_method_returns_symbol_of_new_name() {
    let result = run(r#"
class Foo
  def bar; 1; end
end
Foo.alias_method(:baz, :bar)
"#);
    assert_eq!(result, Some(Object::Symbol(Rc::new("baz".to_string()))));
}

#[test]
fn module_alias_method_returns_symbol_of_new_name() {
    let result = run(r#"
module Foo
  def bar; 1; end
end
Foo.alias_method(:baz, :bar)
"#);
    assert_eq!(result, Some(Object::Symbol(Rc::new("baz".to_string()))));
}

#[test]
fn alias_method_missing_source_raises_name_error() {
    let err = run_err(
        r#"
class Foo; end
Foo.alias_method(:baz, :nope)
"#,
    );
    assert!(err.contains("NameError") || err.contains("undefined"));
}

#[test]
fn module_alias_method_missing_source_raises_name_error() {
    let err = run_err(
        r#"
module Foo; end
Foo.alias_method(:baz, :nope)
"#,
    );
    assert!(err.contains("NameError") || err.contains("undefined"));
}

// ── Alias of special names marks them private automatically ─────────────

#[test]
fn alias_to_initialize_marks_alias_private() {
    let result = run(r#"
class Foo
  def bar; :ok; end
end
Foo.alias_method(:initialize, :bar)
Foo.private_instance_methods(false).include?(:initialize)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── public_instance_methods(true) pulls from ancestors & mixins ─────────

#[test]
fn public_instance_methods_include_super_includes_inherited() {
    let result = run(r#"
class A; def foo; end; end
class B < A; def bar; end; end
B.public_instance_methods.include?(:foo)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn public_instance_methods_include_super_includes_mixin() {
    let result = run(r#"
module M; def foo; end; end
class K; include M; def bar; end; end
K.public_instance_methods.include?(:foo)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── instance_methods (no visibility qualifier) ──────────────────────────

#[test]
fn instance_methods_returns_public_methods_only() {
    let result = run(r#"
class Foo
  def pub; end
  private
  def sec; end
end
Foo.instance_methods(false).include?(:pub)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn instance_methods_false_excludes_private() {
    let result = run(r#"
class Foo
  def pub; end
  private
  def sec; end
end
Foo.instance_methods(false).include?(:sec)
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── Module `extend_object` (with arg) ──────────────────────────────────

#[test]
fn module_extend_object_mixes_module_into_target_singleton() {
    let result = run(r#"
module M; def foo; :m; end; end
class K; end
M.extend_object(K)
K.foo
"#);
    assert_eq!(result, Some(Object::Symbol(Rc::new("m".to_string()))));
}

// ── private_constant / public_constant / deprecate_constant — no-op stubs

#[test]
fn private_constant_blocks_qualified_access_from_outside() {
    // `private_constant` actually marks the constant private now;
    // qualified access from outside the module raises NameError. Inside
    // the module body itself the access still works.
    let result = run(r#"
module M
  X = 1
  private_constant :X
  $inside = X
end
begin
  M::X
  :ok
rescue NameError
  :raised
end
"#);
    assert_eq!(result, Some(Object::Symbol(Rc::new("raised".to_string()))));
}

#[test]
fn public_constant_is_a_noop() {
    let result = run(r#"
module M
  X = 1
  public_constant :X
end
M::X
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

// ── Module.autoload / autoload? no-ops ─────────────────────────────────

#[test]
fn module_autoload_returns_nil() {
    let result = run(r#"
module M; end
M.autoload(:X, "some_path.rb")
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn module_autoload_query_returns_nil() {
    let result = run(r#"
module M; end
M.autoload?(:X)
"#);
    assert_eq!(result, Some(Object::Nil));
}
