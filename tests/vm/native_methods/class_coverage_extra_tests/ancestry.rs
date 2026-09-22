// Mixins, ancestry, and the singleton class.

use super::*;
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
    assert_eq!(result, Some(Object::symbol("ok".to_string())));
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
    assert_eq!(result, Some(Object::symbol("defined".to_string())));
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
    assert_eq!(result, Some(Object::symbol("defined".to_string())));
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
