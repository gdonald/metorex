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
