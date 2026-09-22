// The constants a class holds, and the autoloads on them.

use super::*;
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
fn file_join_with_symbols_raises() {
    let message = run_err(r#"File.join(:foo, :bar)"#);
    assert!(
        message.contains("no implicit conversion of Symbol into String"),
        "unexpected message: {}",
        message
    );
}

#[test]
fn file_join_with_array_elements() {
    let result = run(r#"File.join(["a", "b"], "c")"#);
    assert_eq!(result, Some(Object::string("a/b/c")));
}

#[test]
fn file_join_with_non_string_raises() {
    let message = run_err(r#"File.join("a", 42, "b")"#);
    assert!(
        message.contains("no implicit conversion of Integer into String"),
        "unexpected message: {}",
        message
    );
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
fn thread_current_is_a_thread() {
    let result = run(r#"Thread.current.class.name"#);
    assert_eq!(result, Some(Object::string("Thread")));
}

#[test]
fn thread_main_is_a_thread() {
    let result = run(r#"Thread.main.class.name"#);
    assert_eq!(result, Some(Object::string("Thread")));
}
