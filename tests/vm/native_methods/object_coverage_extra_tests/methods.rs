// The methods an object answers, and how it is sent one.

use super::*;
// ── =~ / !~ ──────────────────────────────────────────────────────────────────

#[test]
fn regex_match_on_non_regex_non_string_returns_nil() {
    // e.g. symbol =~ int — not a regex pair, falls into `_ => Ok(Some(Nil))`.
    let result = run(":foo =~ 1");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn not_match_no_args_errors() {
    let err = run_err(r#""hello".send(:!~)"#);
    assert!(err.contains("argument"));
}

#[test]
fn not_match_without_a_match_method_raises() {
    let error = run_err("1 !~ 2");
    assert!(error.contains("undefined method '=~' for an instance of Integer"));
}

#[test]
fn not_match_string_regex_no_match() {
    let result = run(r#""abc" !~ /xyz/"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn not_match_string_regex_matches() {
    let result = run(r#""abc" !~ /b/"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── instance_exec / instance_eval ────────────────────────────────────────────

#[test]
fn instance_exec_with_block_receiver() {
    let result = run(r#"
"hello".instance_eval { length }
"#);
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn instance_eval_without_block_or_source_errors() {
    let err = run_err(r#""x".send(:instance_eval)"#);
    assert!(
        err.contains("wrong number of arguments (given 0, expected 1..3)"),
        "unexpected error: {err}"
    );
}

// ── frozen?/freeze on immutable values ───────────────────────────────────────

#[test]
fn int_is_always_frozen() {
    let result = run("42.frozen?");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn string_is_not_frozen_until_it_is_frozen() {
    let result = run(r#""hi".frozen?"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn string_is_frozen_once_frozen() {
    let result = run(r#""hi".freeze.frozen?"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn array_is_not_frozen() {
    let result = run("[1,2,3].frozen?");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn freeze_on_class_returns_class() {
    let result = run(r#"
class FF1
end
FF1.freeze.name
"#);
    assert_eq!(result, Some(Object::string("FF1")));
}

#[test]
fn instance_frozen_query_returns_false_by_default() {
    let result = run(r#"
class IFC1; end
IFC1.new.frozen?
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn instance_frozen_query_returns_true_after_freeze() {
    let result = run(r#"
class IFC2; end
i = IFC2.new
i.freeze
i.frozen?
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn obj_method_with_string_arg_returns_method() {
    let result = run(r#"
class MWS
  def hi
    "hello"
  end
end
MWS.new.method("hi")
"#);
    assert!(matches!(result, Some(Object::Method(_))));
}

// ── class on bool/nil ────────────────────────────────────────────────────────

#[test]
fn class_on_true_returns_true_class() {
    let result = run("true.class.name");
    assert_eq!(result, Some(Object::string("TrueClass")));
}

#[test]
fn class_on_false_returns_false_class() {
    let result = run("false.class.name");
    assert_eq!(result, Some(Object::string("FalseClass")));
}

#[test]
fn class_on_nil_returns_nil_class() {
    let result = run("nil.class.name");
    assert_eq!(result, Some(Object::string("NilClass")));
}

#[test]
fn class_with_args_errors() {
    let err = run_err("5.send(:class, 42)");
    assert!(err.contains("argument"));
}

// ── to_s on object with args errors ──────────────────────────────────────────

#[test]
fn inspect_with_a_base_writes_the_integer_in_it() {
    assert_eq!(run("5.send(:inspect, 2)"), Some(Object::string("101")));
    assert!(run_err("5.send(:inspect, 2, 3)").contains("argument"));
}

// ── object_id for same int ───────────────────────────────────────────────────

#[test]
fn object_id_on_integer_follows_fixnum_formula() {
    let result = run("5.object_id");
    // Ruby's fixnum object_id: 2*n + 1
    assert_eq!(result, Some(Object::Int(11)));
}

#[test]
fn object_id_on_true_is_two() {
    let result = run("true.object_id");
    assert_eq!(result, Some(Object::Int(2)));
}

#[test]
fn object_id_on_false_is_zero() {
    let result = run("false.object_id");
    assert_eq!(result, Some(Object::Int(0)));
}

// ── Nil-specific conversions ─────────────────────────────────────────────────

#[test]
fn nil_to_r_returns_rational() {
    // nil.to_r via Rational class (when available).
    let result = run("nil.to_r");
    assert!(matches!(
        result,
        Some(Object::Int(_)) | Some(Object::Instance(_))
    ));
}

#[test]
fn nil_to_c_returns_complex() {
    let result = run("nil.to_c");
    assert!(matches!(
        result,
        Some(Object::Int(_)) | Some(Object::Instance(_))
    ));
}

#[test]
fn nil_rationalize_too_many_args_errors() {
    let err = run_err("nil.rationalize(1, 2)");
    assert!(err.contains("argument"));
}

#[test]
fn nil_to_h_returns_empty_hash() {
    let result = run("nil.to_h.length");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn nil_inspect_returns_nil_string() {
    let result = run("nil.inspect");
    assert_eq!(result, Some(Object::string("nil")));
}
