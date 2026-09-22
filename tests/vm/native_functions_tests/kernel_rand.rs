// Every argument shape `Kernel#rand` accepts.

use super::*;

#[test]
fn rand_without_arguments_gives_a_float_below_one() {
    let result = run("value = rand\nvalue.is_a?(Float) && value >= 0.0 && value < 1.0");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_with_an_integer_bound_stays_below_it() {
    let result = run("1000.times.all? { |i| (0...100).include?(rand(100)) }");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_ignores_the_sign_of_its_bound() {
    let result = run("1000.times.all? { |i| (0...4).include?(rand(-4)) }");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_with_a_float_below_one_gives_a_float() {
    let result = run("rand(0.999).is_a?(Float)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_over_an_integer_range_gives_an_integer_inside_it() {
    let result =
        run("1000.times.all? { |i| x = rand(4...6); x.is_a?(Integer) && (4...6).include?(x) }");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_over_a_mixed_range_gives_a_float_inside_it() {
    let result =
        run("1000.times.all? { |i| x = rand(4...6.5); x.is_a?(Float) && (4...6.5).include?(x) }");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_over_a_backwards_range_is_nil() {
    let result = run("rand(1..0)");
    assert_eq!(result, Some(Object::Nil));
}

#[test]
fn rand_over_a_zero_width_integer_range_is_that_integer() {
    let result = run("rand(42..42)");
    assert_eq!(result, Some(Object::Int(42)));
}

#[test]
fn rand_calls_to_int_on_its_argument() {
    let result = run(r#"
class Limit
  def to_int
    7
  end
end
1000.times.all? { |i| (0...7).include?(rand(Limit.new)) }
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn rand_is_a_private_instance_method_on_kernel() {
    let result = run("Kernel.private_instance_methods(false).include?(:rand)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn srand_answers_the_previous_seed() {
    let result = run("srand(1)\nsrand(2).is_a?(Integer)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn the_same_seed_draws_the_same_sequence() {
    let result = run("srand(99)\nfirst = rand\nsrand(99)\nfirst == rand");
    assert_eq!(result, Some(Object::Bool(true)));
}
