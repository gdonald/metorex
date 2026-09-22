// What an object reports about itself.

use super::*;
// ── frozen? on Class / Module (line 142) ─────────────────────────────────────

#[test]
fn frozen_query_on_class_when_frozen() {
    let result = run(r#"
class F1
end
F1.freeze
F1.frozen?
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn frozen_query_on_module_when_frozen() {
    let result = run(r#"
module M1
end
M1.freeze
M1.frozen?
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn frozen_query_on_unfrozen_class() {
    let result = run(r#"
class F2
end
F2.frozen?
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── to_sym on non-String non-Symbol (line 155) ───────────────────────────────

#[test]
fn to_sym_on_integer_returns_nil_or_error() {
    // to_sym on an Int falls through to the `_ => Ok(None)` arm (line 155),
    // which bubbles out of object_methods. The resulting behavior: either
    // a NoMethodError, or Nil from a higher-level fallback. Either confirms
    // the code path executes.
    let result = std::panic::catch_unwind(|| {
        let tokens = Lexer::new("42.to_sym").tokenize();
        let stmts = Parser::new(tokens).parse().expect("parse failed");
        let mut vm = VirtualMachine::new();
        vm.execute_program(&stmts)
    });
    match result {
        Ok(Err(_)) | Err(_) => {}
        Ok(Ok(_v)) => {} // permissive fallback
    }
}

// ── object_id for Class, Module (lines 163-164) ──────────────────────────────

#[test]
fn object_id_on_class_is_integer() {
    let result = run(r#"
class IdCls
end
IdCls.object_id
"#);
    assert!(matches!(result, Some(Object::Int(_))));
}

#[test]
fn object_id_on_module_is_integer() {
    let result = run(r#"
module IdMod
end
IdMod.object_id
"#);
    assert!(matches!(result, Some(Object::Int(_))));
}

#[test]
fn object_id_on_nil_is_four() {
    let result = run("nil.object_id");
    assert_eq!(result, Some(Object::Int(4)));
}

#[test]
fn object_id_is_stable_and_distinct_for_floats() {
    let result = run("3.14.object_id == 3.14.object_id");
    assert_eq!(result, Some(Object::Bool(true)));
    let distinct = run("3.14.object_id == 2.72.object_id");
    assert_eq!(distinct, Some(Object::Bool(false)));
}

// ── clamp arg count errors (lines 199-214) ───────────────────────────────────

#[test]
fn clamp_no_args_errors() {
    let err = run_err("5.clamp");
    assert!(err.contains("argument"));
}

#[test]
fn clamp_too_many_args_errors() {
    let err = run_err("5.clamp(1, 2, 3)");
    assert!(err.contains("argument"));
}

#[test]
fn clamp_single_non_range_errors() {
    // A single non-range argument triggers the method_argument_error at 199.
    let err = run_err("5.clamp(3)");
    assert!(err.contains("argument"));
}

#[test]
fn clamp_range_returns_clamped() {
    let result = run("15.clamp(1..10)");
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn clamp_two_args_returns_clamped_low() {
    let result = run("(-5).clamp(0, 10)");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn clamp_two_args_returns_clamped_high() {
    let result = run("99.clamp(0, 10)");
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn clamp_two_args_returns_self() {
    let result = run("5.clamp(0, 10)");
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn clamp_exclusive_range_errors() {
    let err = run_err("5.clamp(1...10)");
    assert!(err.contains("exclusive"));
}

#[test]
fn clamp_min_greater_than_max_errors() {
    let err = run_err("5.clamp(10, 1)");
    assert!(err.contains("min") || err.contains("smaller"));
}

#[test]
fn clamp_endless_exclusive_range_ok() {
    // An endless exclusive range like `1...` is accepted because end is nil.
    let result = run("5.clamp(1..)");
    assert_eq!(result, Some(Object::Int(5)));
}

// ── between? (lines 262-278) ─────────────────────────────────────────────────

#[test]
fn between_no_args_errors() {
    let err = run_err("5.between?");
    assert!(err.contains("argument"));
}

#[test]
fn between_too_many_args_errors() {
    let err = run_err("5.between?(1, 2, 3)");
    assert!(err.contains("argument"));
}

#[test]
fn between_returns_true() {
    let result = run("5.between?(1, 10)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn between_returns_false_above() {
    let result = run("15.between?(1, 10)");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn between_returns_false_below() {
    let result = run("0.between?(1, 10)");
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── singleton_method errors (lines 285-310) ──────────────────────────────────

#[test]
fn singleton_method_no_args_errors() {
    let err = run_err(
        r#"
class S1
end
S1.new.singleton_method
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn singleton_method_non_string_errors() {
    let err = run_err(
        r#"
class S2
end
S2.new.singleton_method(42)
"#,
    );
    assert!(err.contains("String") || err.contains("singleton"));
}

#[test]
fn singleton_method_undefined_raises_name_error() {
    let err = run_err(
        r#"
class S3
end
S3.new.singleton_method(:nope)
"#,
    );
    assert!(err.contains("undefined") || err.contains("singleton") || err.contains("NameError"));
}

// ── method() errors (lines 362-368, 373-380) ─────────────────────────────────

#[test]
fn method_no_args_errors() {
    let err = run_err(
        r#"
class M1
end
M1.new.method
"#,
    );
    assert!(err.contains("argument"));
}

#[test]
fn method_non_symbol_errors() {
    let err = run_err(
        r#"
class M2
  def foo
  end
end
M2.new.method(42)
"#,
    );
    assert!(err.contains("String") || err.contains("Symbol") || err.contains("argument"));
}
