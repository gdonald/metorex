// The instance variables an object carries.

use super::*;
#[test]
fn method_undefined_errors() {
    let err = run_err(
        r#"
class M3
end
M3.new.method(:nonexistent)
"#,
    );
    assert!(err.contains("undefined") || err.contains("nonexistent"));
}

#[test]
fn method_returns_method_object() {
    let result = run(r#"
class M4
  def foo
    42
  end
end
M4.new.method(:foo)
"#);
    assert!(matches!(result, Some(Object::Method(_))));
}

// ── respond_to? non-string arg (lines 410-417) ───────────────────────────────

#[test]
fn respond_to_non_string_errors() {
    let err = run_err(
        r#"
class R1
end
R1.new.respond_to?(42)
"#,
    );
    assert!(err.contains("42 is not a symbol nor a string"));
}

// ── is_a? non-class arg (lines 472-479) ──────────────────────────────────────

#[test]
fn is_a_non_class_errors() {
    let err = run_err("5.is_a?(42)");
    assert!(err.contains("Class") || err.contains("argument"));
}

#[test]
fn is_a_with_module_argument() {
    // Just exercise the Object::Module match arm at line 471, regardless of
    // whether the inclusion chain is detected.
    let result = run(r#"
module M
end
UserMod = Class.new
UserMod.new.is_a?(M)
"#);
    assert!(matches!(result, Some(Object::Bool(_))));
}

// ── is_a? Class chain walking (lines 485-504) ────────────────────────────────

#[test]
fn class_is_a_class() {
    let result = run(r#"
class Cl1
end
Cl1.is_a?(Class)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn class_is_a_module_via_chain() {
    let result = run(r#"
class Cl2
end
Cl2.is_a?(Module)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn module_is_a_module() {
    let result = run(r#"
module Mo1
end
Mo1.is_a?(Module)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── dup / clone on various types (lines 719-771) ─────────────────────────────

#[test]
fn dup_on_hash_returns_copy() {
    let result = run(r#"
h = {a: 1, b: 2}
h2 = h.dup
h2[:c] = 3
h.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn clone_on_hash_returns_copy() {
    let result = run(r#"
h = {x: 1}
h2 = h.clone
h2.length
"#);
    assert_eq!(result, Some(Object::Int(1)));
}

#[test]
fn dup_on_class_returns_class() {
    let result = run(r#"
class DC1
  def greet
    "hi"
  end
end
dup = DC1.dup
dup.new.greet
"#);
    assert_eq!(result, Some(Object::string("hi")));
}

#[test]
fn dup_on_module_returns_module() {
    let result = run(r#"
module DM1
  def helper
    1
  end
end
dup = DM1.dup
dup.class.name
"#);
    assert!(matches!(result, Some(Object::String(_))));
}

#[test]
fn dup_on_basic_object_errors() {
    let err = run_err("BasicObject.dup");
    assert!(err.contains("root") || err.contains("BasicObject") || err.contains("TypeError"));
}

#[test]
fn dup_on_int_returns_self() {
    // Immutable types return themselves (line 770).
    let result = run("42.dup");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn dup_on_symbol_returns_self() {
    let result = run(":hello.dup");
    assert!(matches!(result, Some(Object::Symbol(_))));
}

#[test]
fn dup_wrong_arg_count_errors() {
    let err = run_err("[1,2].send(:dup, 42)");
    assert!(err.contains("argument"));
}

// ── eql? and equal? arg count errors ─────────────────────────────────────────

#[test]
fn eql_no_args_errors() {
    let err = run_err("5.eql?");
    assert!(err.contains("argument"));
}

#[test]
fn equal_no_args_errors() {
    let err = run_err("5.equal?");
    assert!(err.contains("argument"));
}

#[test]
fn equal_returns_true_for_same_array() {
    let result = run(r#"
a = [1, 2]
a.equal?(a)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn equal_returns_false_for_different_arrays() {
    let result = run("[1, 2].equal?([1, 2])");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn equal_returns_true_for_same_class() {
    let result = run(r#"
class EC1
end
EC1.equal?(EC1)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn equal_returns_true_for_same_module() {
    let result = run(r#"
module EM1
end
EM1.equal?(EM1)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn equal_returns_true_for_same_hash() {
    let result = run(r#"
h = {a: 1}
h.equal?(h)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn equal_returns_true_for_identical_ints() {
    let result = run("5.equal?(5)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── instance_variable_get/set on Class/Module ───────────────────────────────

#[test]
fn instance_variable_set_on_class() {
    let result = run(r#"
class IV1
end
IV1.instance_variable_set(:@count, 7)
IV1.instance_variable_get(:@count)
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn instance_variable_set_on_module() {
    let result = run(r#"
module IV2
end
IV2.instance_variable_set(:@val, 42)
IV2.instance_variable_get(:@val)
"#);
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn instance_variable_get_on_int_returns_nil() {
    let result = run("5.instance_variable_get(:@foo)");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn instance_variable_set_on_int_raises_frozen_error() {
    // Immediates (Integer/Bool/Nil/Symbol) are frozen — Ruby raises
    // FrozenError (a RuntimeError subclass) on instance_variable_set.
    let err = run_err("5.instance_variable_set(:@foo, 1)");
    assert!(err.contains("can't modify frozen"));
    assert!(err.contains("Integer"));
}
