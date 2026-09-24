// The numeric literals and the arithmetic they answer.

use super::super::run_example;
use super::*;
#[test]
fn test_spaceship_operator_execution() {
    let expected = "-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n";
    let output = run_example("basics/spaceship_operator.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_spaceship_operator_parens_execution() {
    let expected = "-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n";
    let output = run_example("basics/spaceship_operator_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_power_operator_execution() {
    let expected = "1024\n27\n1\n0.1\n2.0\n25\n256\n";
    let output = run_example("basics/power_operator.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_power_operator_parens_execution() {
    let expected = "1024\n27\n1\n0.1\n2.0\n25\n256\n";
    let output = run_example("basics/power_operator_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_scientific_notation_execution() {
    let expected = "2000.0\n2000.0\n2000.0\n0.0015\nFloat\n2001.0\n-2000.0\n10\n";
    let output = run_example("basics/scientific_notation.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_radix_literals_execution() {
    let expected = "31\n31\n10\n10\n15\n15\n15\n99\n99\n0\n0.5\n1000000\n65535\n";
    let output = run_example("basics/radix_literals.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_float_division_execution() {
    let expected = "Infinity\n-Infinity\nNaN\nInfinity\nInfinity\n2.0\ninteger division raises\n";
    let output = run_example("basics/float_division.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_imaginary_literals_execution() {
    let expected = "Complex\nComplex\ntrue\nComplex\n2\nComplex\nRational\n";
    let output = run_example("basics/imaginary_literals.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_radix_suffixes() {
    let expected = concat!(
        "(255/1)\n(34/1)\n(15/1)\n(-255/1)\n",
        "(0+255i)\n(0+34i)\n(0+14i)\n",
        "(3/10)\n(174532925199432957/10000000000000000000)\n",
        "(1111111111111111111111111111111111111111111111/1)\n"
    );
    assert_eq!(run_example("basics/radix_suffixes.rb"), expected);
    assert_eq!(run_example("basics/radix_suffixes_parens.rb"), expected);
}

#[test]
fn test_basics_numeric_equality_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "false\n",
        "false\n",
        "true\n",
        "true\n",
        "\"method\"\n",
        "nil\n",
        "\"method\"\n",
        "nil\n",
    );
    let output = run_example("basics/numeric_equality.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_numeric_equality_parens_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "false\n",
        "false\n",
        "true\n",
        "true\n",
        "\"method\"\n",
        "nil\n",
        "\"method\"\n",
        "nil\n",
    );
    let output = run_example("basics/numeric_equality_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_bit_predicates_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "no implicit conversion of String into Integer\n"
    );
    let output = run_example("basics/integers/bit_predicates.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_bit_predicates_parens_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "no implicit conversion of String into Integer\n"
    );
    let output = run_example("basics/integers/bit_predicates_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_rounding_execution() {
    let expected = concat!(
        "15\n",
        "15\n",
        "15\n",
        "15\n",
        "20\n",
        "10\n",
        "10\n",
        "-10\n",
        "-20\n",
        "200\n",
        "300\n",
        "30\n",
        "20\n",
        "20\n",
        "40\n",
        "-30\n",
        "-20\n",
        "invalid rounding mode: foo\n",
        "1\n",
        "-4\n",
        "200\n"
    );
    let output = run_example("basics/integers/rounding.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_rounding_parens_execution() {
    let expected = concat!(
        "15\n",
        "15\n",
        "15\n",
        "15\n",
        "20\n",
        "10\n",
        "10\n",
        "-10\n",
        "-20\n",
        "200\n",
        "300\n",
        "30\n",
        "20\n",
        "20\n",
        "40\n",
        "-30\n",
        "-20\n",
        "invalid rounding mode: foo\n",
        "1\n",
        "-4\n",
        "200\n"
    );
    let output = run_example("basics/integers/rounding_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_coercion_execution() {
    let expected = concat!(
        "2\n",
        "7\n",
        "5\n",
        "18\n",
        "3\n",
        "true\n",
        "1\n",
        "will not coerce\n",
        "false\n",
        "true\n",
        "6\n",
        "true\n"
    );
    let output = run_example("basics/integers/coercion.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_coercion_parens_execution() {
    let expected = concat!(
        "2\n",
        "7\n",
        "5\n",
        "18\n",
        "3\n",
        "true\n",
        "1\n",
        "will not coerce\n",
        "false\n",
        "true\n",
        "6\n",
        "true\n"
    );
    let output = run_example("basics/integers/coercion_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_division_execution() {
    let expected = concat!(
        "2\n",
        "-3\n",
        "2\n",
        "1\n",
        "[1, 2]\n",
        "[-2, 1]\n",
        "2\n",
        "-2\n",
        "2\n",
        "2.5\n",
        "2\n",
        "-1\n",
        "10.0\n",
        "divided by 0\n",
        "Infinity\n",
        "4\n",
        "true\n",
        "[5, 4, 3, 2, 1]\n",
        "[2, 13, 4]\n",
        "[2, 1]\n",
        "[2.5, 1.0]\n"
    );
    let output = run_example("basics/integers/division.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_division_parens_execution() {
    let expected = concat!(
        "2\n",
        "-3\n",
        "2\n",
        "1\n",
        "[1, 2]\n",
        "[-2, 1]\n",
        "2\n",
        "-2\n",
        "2\n",
        "2.5\n",
        "2\n",
        "-1\n",
        "10.0\n",
        "divided by 0\n",
        "Infinity\n",
        "4\n",
        "true\n",
        "[5, 4, 3, 2, 1]\n",
        "[2, 13, 4]\n",
        "[2, 1]\n",
        "[2.5, 1.0]\n"
    );
    let output = run_example("basics/integers/division_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_comparison_and_scope_execution() {
    let expected = concat!(
        "true\n",
        "[Integer, Numeric, Comparable, Object, Kernel, BasicObject]\n",
        "true\n",
        "true\n",
        "false\n",
        "nil\n",
        "3 metric\n",
        "metric\n",
        "34\n",
        "1\n",
        "(34/1)\n",
        "8\n"
    );
    let output = run_example("basics/integers/comparison_and_scope.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_comparison_and_scope_parens_execution() {
    let expected = concat!(
        "true\n",
        "[Integer, Numeric, Comparable, Object, Kernel, BasicObject]\n",
        "true\n",
        "true\n",
        "false\n",
        "nil\n",
        "3 metric\n",
        "metric\n",
        "34\n",
        "1\n",
        "(34/1)\n",
        "8\n"
    );
    let output = run_example("basics/integers/comparison_and_scope_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_float_methods_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "true\n",
        "2.5\n",
        "0\n",
        "true\n",
        "true\n",
        "true\n",
        "0.0\n",
        "Infinity\n",
        "[1.0, 1.2]\n",
        "[2.5, 1.0]\n",
        "1\n",
        "2\n",
        "(12/1)\n",
        "3.1\n",
        "3.2\n",
        "34.5\n",
        "1200\n",
        "false\n",
        "false\n",
        "true\n",
        "divided by 0\n",
        "[2, 1.0]\n",
        "3.5\n"
    );
    let output = run_example("basics/floats/methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_float_methods_parens_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "true\n",
        "2.5\n",
        "0\n",
        "true\n",
        "true\n",
        "true\n",
        "0.0\n",
        "Infinity\n",
        "[1.0, 1.2]\n",
        "[2.5, 1.0]\n",
        "1\n",
        "2\n",
        "(12/1)\n",
        "3.1\n",
        "3.2\n",
        "34.5\n",
        "1200\n",
        "false\n",
        "false\n",
        "true\n",
        "divided by 0\n",
        "[2, 1.0]\n",
        "3.5\n"
    );
    let output = run_example("basics/floats/methods_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_numeric_protocol_execution() {
    let expected = concat!(
        "true\n",
        "false\n",
        "false\n",
        "250\n",
        "-250\n",
        "false\n",
        "3\n",
        "2\n",
        "3\n",
        "-2\n",
        "2\n",
        "false\n",
        "true\n",
        "-250\n",
        "[200, 100]\n",
        "[3.0, 1.0]\n",
        "true\n",
        "false\n"
    );
    let output = run_example("basics/floats/numeric_protocol.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_numeric_protocol_parens_execution() {
    let expected = concat!(
        "true\n",
        "false\n",
        "false\n",
        "250\n",
        "-250\n",
        "false\n",
        "3\n",
        "2\n",
        "3\n",
        "-2\n",
        "2\n",
        "false\n",
        "true\n",
        "-250\n",
        "[200, 100]\n",
        "[3.0, 1.0]\n",
        "true\n",
        "false\n"
    );
    let output = run_example("basics/floats/numeric_protocol_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_rational_rounding_execution() {
    let expected = concat!(
        "315\n",
        "314\n",
        "314\n",
        "314\n",
        "(3143/10)\n",
        "(7857/25)\n",
        "(157143/500)\n",
        "400\n",
        "310\n",
        "3\n",
        "2\n",
        "2\n",
        "-2\n",
        "10.0\n",
        "(1/1)\n",
        "1\n"
    );
    let output = run_example("basics/rationals/rounding.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_rational_rounding_parens_execution() {
    let expected = concat!(
        "315\n",
        "314\n",
        "314\n",
        "314\n",
        "(3143/10)\n",
        "(7857/25)\n",
        "(157143/500)\n",
        "400\n",
        "310\n",
        "3\n",
        "2\n",
        "2\n",
        "-2\n",
        "10.0\n",
        "(1/1)\n",
        "1\n"
    );
    let output = run_example("basics/rationals/rounding_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_complex_arithmetic_execution() {
    let expected = concat!(
        "(4+6i)\n",
        "(-2-2i)\n",
        "(-5+10i)\n",
        "((11/25)+(2/25)*i)\n",
        "(-3+4i)\n",
        "(2+2i)\n",
        "(2+2i)\n",
        "(2+4i)\n",
        "5.0\n",
        "25\n",
        "true\n",
        "5.0\n",
        "[3, 4]\n",
        "(1-2i)\n",
        "(-1-2i)\n",
        "3\n",
        "can't convert 3+1i into Float\n",
        "can't convert 3+0.0i into Integer\n",
        "true\n",
        "false\n",
        "\"1+2i\"\n"
    );
    let output = run_example("basics/complexes/arithmetic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_complex_arithmetic_parens_execution() {
    let expected = concat!(
        "(4+6i)\n",
        "(-2-2i)\n",
        "(-5+10i)\n",
        "((11/25)+(2/25)*i)\n",
        "(-3+4i)\n",
        "(2+2i)\n",
        "(2+2i)\n",
        "(2+4i)\n",
        "5.0\n",
        "25\n",
        "true\n",
        "5.0\n",
        "[3, 4]\n",
        "(1-2i)\n",
        "(-1-2i)\n",
        "3\n",
        "can't convert 3+1i into Float\n",
        "can't convert 3+0.0i into Integer\n",
        "true\n",
        "false\n",
        "\"1+2i\"\n"
    );
    let output = run_example("basics/complexes/arithmetic_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_complex_protocol_execution() {
    let expected = concat!(
        "(0+1i)\n",
        "(-1+0i)\n",
        "(7+16i)\n",
        "8\n",
        "1\n",
        "1\n",
        "nil\n",
        "true\n",
        "false\n",
        "false\n",
        "undefined method 'new' for class 'Float'\n",
        "can't unfreeze Integer\n",
        "3\n",
        "2\n"
    );
    let output = run_example("basics/complexes/protocol.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_complex_protocol_parens_execution() {
    let expected = concat!(
        "(0+1i)\n",
        "(-1+0i)\n",
        "(7+16i)\n",
        "8\n",
        "1\n",
        "1\n",
        "nil\n",
        "true\n",
        "false\n",
        "false\n",
        "undefined method 'new' for class 'Float'\n",
        "can't unfreeze Integer\n",
        "3\n",
        "2\n"
    );
    let output = run_example("basics/complexes/protocol_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_bits_execution() {
    let output = run_example("basics/integer_bits.rb");
    assert_eq!(output, INTEGER_BITS_OUTPUT);
}

#[test]
fn test_basics_integer_bits_parens_execution() {
    let output = run_example("basics/integer_bits_parens.rb");
    assert_eq!(output, INTEGER_BITS_OUTPUT);
}

#[test]
fn test_basics_whole_number_limits_execution() {
    let output = run_example("basics/whole_number_limits.rb");
    assert_eq!(output, WHOLE_NUMBER_LIMITS_OUTPUT);
}

#[test]
fn test_basics_whole_number_limits_parens_execution() {
    let output = run_example("basics/whole_number_limits_parens.rb");
    assert_eq!(output, WHOLE_NUMBER_LIMITS_OUTPUT);
}

#[test]
fn test_basics_exact_powers_execution() {
    let output = run_example("basics/exact_powers.rb");
    assert_eq!(output, EXACT_POWERS_OUTPUT);
}

#[test]
fn test_basics_exact_powers_parens_execution() {
    let output = run_example("basics/exact_powers_parens.rb");
    assert_eq!(output, EXACT_POWERS_OUTPUT);
}

#[test]
fn test_basics_integer_stepping_execution() {
    let expected = concat!(
        "[1, 5, 9]\n",
        "[1, 5, 9]\n",
        "[1, 5, 9]\n",
        "[1, 5, 9]\n",
        "Infinity\n",
        "Infinity\n",
        "1\n",
        "1\n",
        "to is given twice\n",
        "step is given twice\n",
        "unknown keyword: :step\n",
        "comparison of String with 0 failed\n",
        "Enumerator\n",
        "comparison of String with 0 failed\n",
        "[]\n",
        "NaN\n"
    );
    let output = run_example("basics/integers/stepping.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_stepping_parens_execution() {
    let expected = concat!(
        "[1, 5, 9]\n",
        "[1, 5, 9]\n",
        "[1, 5, 9]\n",
        "[1, 5, 9]\n",
        "Infinity\n",
        "Infinity\n",
        "1\n",
        "1\n",
        "to is given twice\n",
        "step is given twice\n",
        "unknown keyword: :step\n",
        "comparison of String with 0 failed\n",
        "Enumerator\n",
        "comparison of String with 0 failed\n",
        "[]\n",
        "NaN\n"
    );
    let output = run_example("basics/integers/stepping_parens.rb");
    assert_eq!(output, expected);
}
