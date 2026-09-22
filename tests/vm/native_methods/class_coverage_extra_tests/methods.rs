// Defining, removing and renaming the methods of a class.

use super::*;
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
fn extend_with_class_raises_type_error() {
    // `extend` takes a module; a class is rejected.
    let error = run_err(
        r#"
class SrcClass
  def shared
    "class-mixin"
  end
end
class ExtTarget
end
ExtTarget.extend(SrcClass)
"#,
    );
    assert!(
        error.contains("TypeError") || error.contains("Module"),
        "{}",
        error
    );
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
