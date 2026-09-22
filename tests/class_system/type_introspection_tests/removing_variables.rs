// Dividing with a remainder, and removing an instance variable.

use super::*;

#[test]
fn divmod_floors_toward_negative_infinity() {
    let result = run("[13.divmod(4), 13.divmod(-4), (-13).divmod(4), (-13).divmod(-4)].inspect");
    assert_eq!(
        result,
        Some(Object::string(
            "[[3, 1], [-4, -3], [-4, 3], [3, -1]]".to_string()
        ))
    );
}

#[test]
fn divmod_by_a_float_gives_an_integer_quotient() {
    let result = run("13.divmod(4.0).first");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn divmod_by_zero_raises() {
    let error = run_err("1.divmod(0)");
    assert!(error.contains("divided by 0"));
}

#[test]
fn divmod_rejects_a_non_numeric_divisor() {
    let error = run_err(r#"1.divmod("two")"#);
    assert!(error.contains("String can't be coerced into Integer"));
}

#[test]
fn divmod_requires_one_argument() {
    let error = run_err("1.divmod");
    assert!(error.contains("wrong number of arguments (given 0, expected 1)"));
}

// ── Kernel#remove_instance_variable ──────────────────────────────────────────

const GREETER: &str = r#"
class Greeter
  def initialize
    @greeting = "hello"
    @name = "world"
  end
end
"#;

#[test]
fn remove_instance_variable_answers_the_value_it_took() {
    let result = run(&format!(
        "{GREETER}\nGreeter.new.remove_instance_variable(:@greeting)"
    ));
    assert_eq!(result, Some(Object::string("hello".to_string())));
}

#[test]
fn remove_instance_variable_takes_the_variable_off() {
    let result = run(&format!(
        r#"
{GREETER}
greeter = Greeter.new
greeter.remove_instance_variable(:@greeting)
greeter.instance_variables.inspect
"#
    ));
    assert_eq!(result, Some(Object::string("[:@name]".to_string())));
}

#[test]
fn remove_instance_variable_accepts_a_string_name() {
    let result = run(&format!(
        r#"{GREETER}{}"#,
        "\nGreeter.new.remove_instance_variable(\"@name\")"
    ));
    assert_eq!(result, Some(Object::string("world".to_string())));
}

#[test]
fn remove_instance_variable_converts_its_argument_with_to_str() {
    let result = run(&format!(
        r#"
{GREETER}
class Name
  def to_str
    "@greeting"
  end
end
Greeter.new.remove_instance_variable(Name.new)
"#
    ));
    assert_eq!(result, Some(Object::string("hello".to_string())));
}

#[test]
fn remove_instance_variable_raises_for_an_undefined_variable() {
    let error = run_err(&format!(
        "{GREETER}\nGreeter.new.remove_instance_variable(:@unknown)"
    ));
    assert!(error.contains("instance variable @unknown not defined"));
}

#[test]
fn remove_instance_variable_raises_for_an_invalid_name() {
    let error = run_err(&format!(
        "{GREETER}\nGreeter.new.remove_instance_variable(:\"@0\")"
    ));
    assert!(error.contains("`@0' is not allowed as an instance variable name"));
}

#[test]
fn remove_instance_variable_raises_type_error_without_to_str() {
    let error = run_err(&format!(
        "{GREETER}\nGreeter.new.remove_instance_variable(Object.new)"
    ));
    assert!(error.contains("no implicit conversion of Object into String"));
}

#[test]
fn remove_instance_variable_raises_on_a_frozen_object() {
    let error = run_err(&format!(
        r#"
{GREETER}
greeter = Greeter.new
greeter.freeze
greeter.remove_instance_variable(:@greeting)
"#
    ));
    assert!(error.contains("can't modify frozen Greeter"));
}

#[test]
fn remove_instance_variable_validates_the_name_before_the_frozen_check() {
    let error = run_err("nil.remove_instance_variable(:not_a_variable)");
    assert!(error.contains("`not_a_variable' is not allowed as an instance variable name"));
}

#[test]
fn remove_instance_variable_is_public_on_kernel() {
    let result = run("Kernel.public_instance_methods(false).include?(:remove_instance_variable)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn kernel_lists_its_native_instance_methods() {
    let result = run("Kernel.instance_methods(false).include?(:instance_variable_get)");
    assert_eq!(result, Some(Object::Bool(true)));
}
