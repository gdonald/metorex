// Every argument shape `Kernel#srand` accepts.

use super::*;

#[test]
fn srand_answers_the_seed_it_replaced() {
    let result = run("srand(10)\nsrand(20)");
    assert_eq!(result, Some(Object::Int(10)));
}

#[test]
fn srand_accepts_a_seed_of_zero() {
    let result = run("srand(0)\nsrand");
    assert_eq!(result, Some(Object::Int(0)));
}

#[test]
fn srand_accepts_a_negative_seed() {
    let result = run("srand(-17)\nsrand");
    assert_eq!(result, Some(Object::Int(-17)));
}

#[test]
fn srand_truncates_a_float_seed() {
    let result = run("srand(3.8)\nsrand");
    assert_eq!(result, Some(Object::Int(3)));
}

#[test]
fn srand_calls_to_int_on_its_seed() {
    let result = run(r#"
class Seed
  def to_int
    7
  end
end
srand(Seed.new)
srand
"#);
    assert_eq!(result, Some(Object::Int(7)));
}

#[test]
fn srand_with_no_argument_picks_a_seed() {
    let result = run("srand.is_a?(Integer)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn the_same_seed_repeats_a_whole_sequence() {
    let result = run(r#"
srand(99)
first = 3.times.map { rand }
srand(99)
first == 3.times.map { rand }
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn srand_raises_type_error_for_nil() {
    let error = run_err("srand(nil)");
    assert!(error.contains("into Integer"));
}

#[test]
fn srand_raises_type_error_for_a_string() {
    let error = run_err(r#"srand("7")"#);
    assert!(error.contains("no implicit conversion of String into Integer"));
}

#[test]
fn srand_is_a_private_instance_method_on_kernel() {
    let result = run("Kernel.private_instance_methods(false).include?(:srand)");
    assert_eq!(result, Some(Object::Bool(true)));
}
