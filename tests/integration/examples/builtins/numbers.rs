// The numeric classes and the arithmetic they answer.

use super::super::run_example;
use super::*;
#[test]
fn test_operators_coverage_execution() {
    let output = run_example("builtins/operators_coverage.rb");
    assert_eq!(
        output,
        "9223372036854775806\n13835058055282163709\n-9223372036854775808\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n1\n7\n6\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\n"
    );
}

#[test]
fn test_operators_coverage_no_parens_execution() {
    let output = run_example("builtins/operators_coverage_no_parens.rb");
    assert_eq!(
        output,
        "9223372036854775806\n13835058055282163709\n-9223372036854775808\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n1\n7\n6\n"
    );
}

#[test]
fn test_integer_iteration_execution() {
    // `upto` and `downto` answer an Enumerator without a block, so the example
    // collects it before inspecting.
    let expected = "1\n2\n3\n3\n2\n1\n[1, 2, 3]\n[3, 2, 1]\n3/2\n1\n1.5\n3\n2\n2\n2/1\n";
    let output = run_example("builtins/integer_iteration.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_rational_execution() {
    let expected = "1/2\n1\n2\n1/2\n3/5\n7/1\n1/3\n1/2\n5/1\n3/2\n5/6\n1/3\n1/4\n2/1\ntrue\n0.5\n2\n(1/2)\ntrue\n13/25\n13/15\n3/4\n3/5\n3/1\n1/2\ntrue\ntrue\nfalse\ndivided by 0\ncan't convert nil into Rational\nnil\n";
    let output = run_example("builtins/rational.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_integer_bits_operations_execution() {
    let expected = concat!(
        "[8, 34359738368, 2, -2]\n",
        "[2, 10]\n",
        "[1606938044258990275541962092341162602522202993782792835301376, -1, 0]\n",
        "[-6, -1, 0]\n",
        "[8, 9, 0, 0]\n",
        "[1, 3, 2]\n",
        "42\n",
        "true\n"
    );
    let output = run_example("builtins/integer_bits/operations.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_integer_bits_operations_parens_execution() {
    let expected = concat!(
        "[8, 34359738368, 2, -2]\n",
        "[2, 10]\n",
        "[1606938044258990275541962092341162602522202993782792835301376, -1, 0]\n",
        "[-6, -1, 0]\n",
        "[8, 9, 0, 0]\n",
        "[1, 3, 2]\n",
        "42\n",
        "true\n"
    );
    let output = run_example("builtins/integer_bits/operations_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_big_integers_execution() {
    let expected = concat!(
        "[18446744073709551616, 1267650600228229401496703205376]\n",
        "Integer\n",
        "true\n",
        "[9223372036854775808, 18446744073709551614, -9223372036854775809]\n",
        "42\n",
        "0\n",
        "Integer\n",
        "true\n",
        "1\n",
        "[-18446744073709551616, 1, 3, 18446744073709551616]\n",
        "18446744073709551616\n",
        "18446744073709551616\n",
        "true\n",
        "18446744073709551617\n",
        "65\n",
        "[18446744073709551, 616]\n",
        "4294967296\n",
        "-18446744073709551617\n",
        "Integer\n",
        "340282366920938463463374607431768211456\n",
        "18446744073709551616\n",
        "3\n",
        "true\n",
        "false\n",
        "true\n"
    );
    let output = run_example("builtins/big_integers/arithmetic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_big_integers_parens_execution() {
    let expected = concat!(
        "[18446744073709551616, 1267650600228229401496703205376]\n",
        "Integer\n",
        "true\n",
        "[9223372036854775808, 18446744073709551614, -9223372036854775809]\n",
        "42\n",
        "0\n",
        "Integer\n",
        "true\n",
        "1\n",
        "[-18446744073709551616, 1, 3, 18446744073709551616]\n",
        "18446744073709551616\n",
        "18446744073709551616\n",
        "true\n",
        "18446744073709551617\n",
        "65\n",
        "[18446744073709551, 616]\n",
        "4294967296\n",
        "-18446744073709551617\n",
        "Integer\n",
        "340282366920938463463374607431768211456\n",
        "18446744073709551616\n",
        "3\n",
        "true\n",
        "false\n",
        "true\n"
    );
    let output = run_example("builtins/big_integers/arithmetic_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_numeric_parts_execution() {
    let output = run_example("builtins/numeric_parts.rb");
    assert_eq!(output, NUMERIC_PARTS_OUTPUT);
}

#[test]
fn test_builtins_numeric_parts_no_parens_execution() {
    let output = run_example("builtins/numeric_parts_no_parens.rb");
    assert_eq!(output, NUMERIC_PARTS_OUTPUT);
}

#[test]
fn test_builtins_comparing_values_execution() {
    let output = run_example("builtins/comparing_values.rb");
    assert_eq!(output, COMPARING_VALUES_OUTPUT);
}

#[test]
fn test_builtins_comparing_values_no_parens_execution() {
    let output = run_example("builtins/comparing_values_no_parens.rb");
    assert_eq!(output, COMPARING_VALUES_OUTPUT);
}

#[test]
fn test_builtins_optimized_redefinition_execution() {
    let output = run_example("builtins/optimized_redefinition.rb");
    assert_eq!(output, OPTIMIZED_REDEFINITION_OUTPUT);
}

#[test]
fn test_builtins_optimized_redefinition_parens_execution() {
    let output = run_example("builtins/optimized_redefinition_parens.rb");
    assert_eq!(output, OPTIMIZED_REDEFINITION_OUTPUT);
}

#[test]
fn test_builtins_complex_powers_execution() {
    let output = run_example("builtins/complex_powers.rb");
    assert_eq!(output, COMPLEX_POWERS_OUTPUT);
}

#[test]
fn test_builtins_complex_powers_no_parens_execution() {
    let output = run_example("builtins/complex_powers_no_parens.rb");
    assert_eq!(output, COMPLEX_POWERS_OUTPUT);
}

#[test]
fn test_builtins_numbers_written_out_execution() {
    let output = run_example("builtins/numbers_written_out.rb");
    assert_eq!(output, NUMBERS_WRITTEN_OUT_OUTPUT);
}

#[test]
fn test_builtins_numbers_written_out_no_parens_execution() {
    let output = run_example("builtins/numbers_written_out_no_parens.rb");
    assert_eq!(output, NUMBERS_WRITTEN_OUT_OUTPUT);
}

/// The expected output of both `builtins/operators_by_name` variants.
const OPERATORS_BY_NAME: &str = concat!(
    "Integer +: TypeError: nil can't be coerced into Integer\n",
    "Integer <<: TypeError: no implicit conversion of nil into Integer\n",
    "Integer &: TypeError: true can't be coerced into Integer\n",
    "Integer []: TypeError: no implicit conversion of nil into Integer\n",
    "Float &: NoMethodError: undefined method '&' for an instance of Float\n",
    "Float <: ArgumentError: comparison of Float with :sym failed\n",
    "Rational +: TypeError: :sym can't be coerced into Rational\n",
    "Rational <: ArgumentError: comparison of Rational with String failed\n",
    "Rational |: NoMethodError: undefined method '|' for an instance of Rational\n",
    "Complex +: TypeError: String can't be coerced into Complex\n",
    "Complex **: TypeError: Object can't be coerced into Complex\n",
    "Complex <: NoMethodError: undefined method '<' for an instance of Complex\n",
    "Complex %: NoMethodError: undefined method '%' for an instance of Complex\n",
    "String +: TypeError: no implicit conversion of nil into String\n",
    "String *: TypeError: no implicit conversion from nil to integer\n",
    "String *: TypeError: no implicit conversion of String into Integer\n",
    "String []: TypeError: no implicit conversion of Symbol into Integer\n",
    "String <<: TypeError: no implicit conversion of true into String\n",
    "String =~: TypeError: type mismatch: String given\n",
    "String =~: NoMethodError: undefined method '=~' for an instance of Array\n",
    "String &: NoMethodError: undefined method '&' for an instance of String\n",
    "Array +: TypeError: no implicit conversion of nil into Array\n",
    "Array -: TypeError: no implicit conversion of String into Array\n",
    "Array &: TypeError: no implicit conversion of true into Array\n",
    "Array *: TypeError: no implicit conversion from nil to integer\n",
    "Array ^: NoMethodError: undefined method '^' for an instance of Array\n",
    "Hash <: TypeError: no implicit conversion of nil into Hash\n",
    "Hash +: NoMethodError: undefined method '+' for an instance of Hash\n",
    "Symbol <: false\n",
    "Symbol <: ArgumentError: comparison of Symbol with 1 failed\n",
    "Symbol -@: NoMethodError: undefined method '-@' for an instance of Symbol\n",
    "Symbol []: TypeError: no implicit conversion from nil to integer\n",
    "NilClass +: NoMethodError: undefined method '+' for nil\n",
    "TrueClass <: NoMethodError: undefined method '<' for true\n",
    "Object -: NoMethodError: undefined method '-' for an instance of Object\n",
    "Range +: NoMethodError: undefined method '+' for an instance of Range\n",
    "Time -: TypeError: can't convert String into an exact number\n",
    "Integer =~: NoMethodError: undefined method '=~' for an instance of Integer\n",
    "[false, false, false, false, true, false, false]\n",
);

#[test]
fn test_builtins_operators_by_name_execution() {
    let output = run_example("builtins/operators_by_name.rb");
    assert_eq!(output, OPERATORS_BY_NAME);
}

#[test]
fn test_builtins_operators_by_name_no_parens_execution() {
    let output = run_example("builtins/operators_by_name_no_parens.rb");
    assert_eq!(output, OPERATORS_BY_NAME);
}

/// The expected output of both `builtins/numeric_coerce` variants.
const NUMERIC_COERCE: &str = concat!(
    "[3.0, 2.0]\n",
    "[4.0, 2.0]\n",
    "can't define singleton method \"unit\" for Measure\n",
    "[3.0, 2.0]\n",
    "true\n",
);

#[test]
fn test_builtins_numeric_coerce_execution() {
    let output = run_example("builtins/numeric_coerce.rb");
    assert_eq!(output, NUMERIC_COERCE);
}

#[test]
fn test_builtins_numeric_coerce_no_parens_execution() {
    let output = run_example("builtins/numeric_coerce_no_parens.rb");
    assert_eq!(output, NUMERIC_COERCE);
}
