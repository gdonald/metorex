// What a class reports about itself.

use super::*;
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
    assert!(err.contains("value must be enumerable"));
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
    assert_eq!(result, Some(Object::string("alice".to_string())));
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
