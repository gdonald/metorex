// Where the numeric classes sit, and how they compare and format.

use super::*;

#[test]
fn an_integer_is_a_numeric() {
    let result = run("5.is_a?(Numeric)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn a_float_is_a_numeric() {
    let result = run("0.5.is_a?(Numeric)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn float_reports_numeric_as_its_superclass() {
    let result = run("Float.superclass.name");
    assert_eq!(result.map(|o| o.to_string()), Some("Numeric".to_string()));
}

// ── Comparing an Integer against a Float ─────────────────────────────────────

#[test]
fn an_integer_range_includes_a_float_inside_it() {
    let result = run("(0...1).include?(0.38)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn a_range_with_one_float_side_includes_a_float() {
    let result = run("(3.5..6).include?(5.93)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn spaceship_compares_a_float_against_an_integer() {
    let result = run("[0.38 <=> 0, 5 <=> 5.5, 2 <=> 2.0].inspect");
    assert_eq!(
        result.map(|o| o.to_string()),
        Some("[1, -1, 0]".to_string())
    );
}

// ── sprintf format coercion ──────────────────────────────────────────────────

#[test]
fn sprintf_converts_its_format_with_to_str() {
    let result = run(r#"
class Template
  def to_str
    "converted %s"
  end
end
sprintf(Template.new, "format")
"#);
    assert_eq!(
        result.map(|o| o.to_string()),
        Some("converted format".to_string())
    );
}

#[test]
fn sprintf_raises_type_error_for_a_format_it_cannot_convert() {
    let error = run_err(r#"sprintf(42, "value")"#);
    assert!(error.contains("no implicit conversion of Integer into String"));
}

#[test]
fn a_numeric_modulo_by_a_string_raises() {
    let error = run_err(r#"42 % "not a format""#);
    assert!(error.contains("Cannot apply operator 'Modulo' to types 'Int' and 'String'"));
}

#[test]
fn percent_s_renders_a_symbol_with_to_s() {
    let result = run(r#"sprintf("%s", :symbol)"#);
    assert_eq!(result.map(|o| o.to_string()), Some("symbol".to_string()));
}

// ── Float constants ──────────────────────────────────────────────────────────

#[test]
fn float_infinity_is_larger_than_any_finite_value() {
    let result = run("Float::INFINITY > 1e308");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn float_nan_does_not_equal_itself() {
    let result = run("Float::NAN == Float::NAN");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn float_reports_its_precision_constants() {
    let result = run("[Float::DIG, Float::MANT_DIG].inspect");
    assert_eq!(result.map(|o| o.to_string()), Some("[15, 53]".to_string()));
}

#[test]
fn float_epsilon_and_bounds_are_present() {
    let result = run("[Float::EPSILON > 0, Float::MAX > Float::MIN].inspect");
    assert_eq!(
        result.map(|o| o.to_string()),
        Some("[true, true]".to_string())
    );
}
