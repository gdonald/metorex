// The copies an object hands back, and the blocks it runs.

use super::*;
// ── send / public_send arg errors ────────────────────────────────────────────

#[test]
fn send_no_args_errors() {
    let err = run_err("5.send");
    assert!(err.contains("no method name given"));
}

#[test]
fn send_non_string_method_errors() {
    let err = run_err("5.send(42)");
    assert!(err.contains("42 is not a symbol nor a string"));
}

#[test]
fn public_send_works() {
    let result = run("5.public_send(:to_s)");
    assert_eq!(result, Some(Object::string("5")));
}

// ── methods() arg count error ────────────────────────────────────────────────

#[test]
fn methods_with_args_errors() {
    // `methods` accepts a single optional include_super Boolean; passing two
    // positional args is the error now.
    let err = run_err("5.send(:methods, true, 1)");
    assert!(err.contains("argument"));
}

// ── instance_variables ──────────────────────────────────────────────────────

#[test]
fn instance_variables_on_instance() {
    let result = run(r#"
class IVars
  def initialize
    @a = 1
    @b = 2
  end
end
IVars.new.instance_variables.length
"#);
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn instance_variables_on_non_instance_returns_empty() {
    let result = run("5.instance_variables.length");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn instance_variables_with_args_errors() {
    let err = run_err("5.send(:instance_variables, 1)");
    assert!(err.contains("argument"));
}

// ── instance_variable_get / instance_variable_set arg errors ────────────────

#[test]
fn instance_variable_get_no_args_errors() {
    let err = run_err(
        r#"
class IVE
end
IVE.new.instance_variable_get
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn instance_variable_get_non_symbol_errors() {
    let err = run_err(
        r#"
class IVE2
end
IVE2.new.instance_variable_get(42)
"#,
    );
    assert!(err.contains("String") || err.contains("Symbol") || err.contains("argument"));
}

#[test]
fn instance_variable_set_no_args_errors() {
    let err = run_err(
        r#"
class IVS
end
IVS.new.instance_variable_set
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn instance_variable_set_non_symbol_errors() {
    let err = run_err(
        r#"
class IVS2
end
IVS2.new.instance_variable_set(42, 1)
"#,
    );
    assert!(err.contains("String") || err.contains("Symbol") || err.contains("argument"));
}

// ── instance_of? ─────────────────────────────────────────────────────────────

#[test]
fn instance_of_no_args_errors() {
    let err = run_err("5.instance_of?");
    assert!(err.contains("argument"));
}

#[test]
fn instance_of_non_class_errors() {
    let err = run_err("5.instance_of?(42)");
    assert!(err.contains("Class") || err.contains("argument"));
}

#[test]
fn instance_of_exact_class() {
    let result = run("5.instance_of?(Integer)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── get_source ──────────────────────────────────────────────────────────────

#[test]
fn get_source_returns_method_or_nil() {
    let result = run(r#"
class GS
  def foo
    1
  end
end
GS.new.get_source(:foo)
"#);
    assert!(matches!(
        result,
        Some(Object::Method(_)) | Some(Object::Nil)
    ));
}

#[test]
fn get_source_undefined_returns_nil() {
    let result = run(r#"
class GS2
end
GS2.new.get_source(:nothing_here)
"#);
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn get_source_no_args_errors() {
    let err = run_err(
        r#"
class GS3
end
GS3.new.get_source
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn get_source_non_symbol_errors() {
    let err = run_err(
        r#"
class GS4
end
GS4.new.get_source(42)
"#,
    );
    assert!(err.contains("String") || err.contains("Symbol") || err.contains("argument"));
}

// ── object_id for values metorex stores inline ───────────────────────────────

#[test]
fn object_id_matches_for_equal_symbols() {
    let result = run(":hello.object_id == :hello.object_id");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn object_id_differs_for_different_symbols() {
    let result = run(":hello.object_id == :goodbye.object_id");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn object_id_differs_for_two_equal_strings() {
    // Two literals holding the same text are two objects, so each answers an
    // id of its own.
    let result = run(r#""hello".object_id == "hello".object_id"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn object_id_is_the_same_each_time_one_string_is_asked() {
    let result = run(r#"held = "hello"
held.object_id == held.object_id"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn object_id_differs_for_different_strings() {
    let result = run(r#""hello".object_id == "goodbye".object_id"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn object_id_differs_for_an_object_and_its_dup() {
    let result = run(r#"
class Widget
end
widget = Widget.new
widget.object_id == widget.dup.object_id
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn object_id_does_not_overflow_at_the_top_of_the_integer_range() {
    let result = run("(2 ** 62 - 1).object_id.is_a?(Integer)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn object_id_differs_across_the_thirty_two_bit_boundary() {
    let result = run("(-1).object_id == (2 ** 30 - 1).object_id");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn object_id_of_a_symbol_is_not_negative() {
    let result = run(":anything.object_id >= 0");
    assert_eq!(result, Some(Object::Bool(true)));
}
