use super::run_example;

// 10.4.12 — Builtins

#[test]
fn test_builtins_type_introspection() {
    let expected = "true\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nNumeric\nBasicObject\n3\ntrue\ntrue\nAnimal\n2\nRex\n3\n4\n";
    let output = run_example("builtins/type_introspection.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_defined_keyword_execution() {
    let expected = "local-variable\nnil\nmethod\nconstant\nnil\nglobal-variable\nnil\nexpression\nexpression\nexpression\nexpression\nlocal-variable\n";
    let output = run_example("builtins/defined_keyword.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_defined_keyword_parens_execution() {
    let expected = "local-variable\nnil\nmethod\nconstant\nexpression\n";
    let output = run_example("builtins/defined_keyword_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_kernel_conversion_execution() {
    let expected = "42\n3\n42\n\n\n1\n2\nhi\ncan't convert TrueClass into Integer\ncan't convert nil into Integer\n";
    let output = run_example("builtins/kernel_conversion.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_kernel_conversion_parens_execution() {
    let expected = "42\n3\n42\n\n\n1\n2\nhi\ncan't convert TrueClass into Integer\ncan't convert nil into Integer\n";
    let output = run_example("builtins/kernel_conversion_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_keyword_symbols_execution() {
    let expected = "def\nclass\nif\nelse\nend\ndo\nnil\ntrue\nfalse\nreturn\nbegin\nrescue\nensure\nwhile\nfor\ncase\nwhen\nmodule\ninclude\nyield\nsuper\nlambda\nbreak\nnext\nraise\n@ivar\n@@cvar\n";
    let output = run_example("builtins/keyword_symbols.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_keyword_symbols_parens_execution() {
    let expected = "def\nclass\n@ivar\n@@cvar\nyield\n";
    let output = run_example("builtins/keyword_symbols_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_or_assign_execution() {
    let expected = "42\n42\nfalse\nfalse\nworld\n";
    let output = run_example("builtins/or_assign.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_or_assign_parens_execution() {
    let expected = "42\n42\nfalse\nfalse\nworld\n";
    let output = run_example("builtins/or_assign_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_defined_extended_execution() {
    let expected = "local-variable\nmethod\nconstant\nnil\nglobal-variable\nnil\ninstance-variable\nnil\nexpression\nexpression\nexpression\nexpression\nyield\nnil\nnil\n";
    let output = run_example("builtins/defined_extended.rb");
    assert_eq!(output, expected);
}

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
fn test_builtins_range_include_comparable_execution() {
    let expected = "true\nfalse\nfalse\ntrue\nfalse\ntrue\ntrue\ntrue\n";
    let output = run_example("builtins/range_include_comparable.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_range_include_comparable_parens_execution() {
    let expected = "true\nfalse\nfalse\ntrue\nfalse\ntrue\ntrue\ntrue\n";
    let output = run_example("builtins/range_include_comparable_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_kernel_hash_execution() {
    let expected = "0\n0\n0\n1\nfast\nfast\ncan't convert Broken to Hash (Broken#to_hash gives String)\ncan't convert Object into Hash\ntrue\ntrue\ntrue\ntrue\nfalse\n";
    let output = run_example("builtins/kernel_hash.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_kernel_integer_execution() {
    let expected = "42\n3\n-3\n42\n42\n1000\n7\n-7\n31\n10\n15\n15\n99\n255\n5\n14929\n4\nnil\nnil\nnil\n12\n5\ninvalid value for Integer(): \"1__2\"\ncan't convert nil into Integer\nbase specified for non string value (Integer)\nNaN\n10\ntrue\n";
    let output = run_example("builtins/kernel_integer.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_integer_iteration_execution() {
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
fn test_kernel_string_execution() {
    let expected = "\"already\"\n\"\"\n\"1.12\"\n\"true\"\n\"false\"\n\"42\"\n\"Object\"\nsymbol\ntag\ncan't convert Silent into String\ncan't convert Wrong to String (Wrong#to_s gives Integer)\ntrue\nmetorex\n7\nMETOREX\ntrue\ntrue\n7\ntrue\n";
    let output = run_example("builtins/kernel_string.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_rand_and_numeric() {
    let expected = concat!(
        "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n",
        "nil\n42\n1.5\ntrue\n",
        "TypeError: no implicit conversion of String into Integer\n",
        "true\ntrue\nNumeric\nNumeric\n",
        "true\ntrue\ntrue\n1\n-1\ntrue\n"
    );
    let output = run_example("builtins/rand_and_numeric.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_rand_and_numeric_parens() {
    let expected = concat!(
        "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n",
        "nil\n42\n1.5\ntrue\n",
        "TypeError: no implicit conversion of String into Integer\n",
        "true\ntrue\nNumeric\nNumeric\n",
        "true\ntrue\ntrue\n1\n-1\ntrue\n"
    );
    let output = run_example("builtins/rand_and_numeric_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_sprintf_and_float_constants() {
    let expected = concat!(
        "one and two\n42\nsymbol\nconverted format\n",
        "TypeError: no implicit conversion of Integer into String\n",
        "TypeError\nInfinity\ntrue\ntrue\nfalse\n15\n53\ntrue\ntrue\n"
    );
    let output = run_example("builtins/sprintf_and_float_constants.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_sprintf_and_float_constants_no_parens() {
    let expected = concat!(
        "one and two\n42\nsymbol\nconverted format\n",
        "TypeError: no implicit conversion of Integer into String\n",
        "TypeError\nInfinity\ntrue\ntrue\nfalse\n15\n53\ntrue\ntrue\n"
    );
    let output = run_example("builtins/sprintf_and_float_constants_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_srand_seeding() {
    let expected = concat!(
        "10\n20\n0\ntrue\ntrue\n3\n7\ntrue\ntrue\n",
        "TypeError\nTypeError\ntrue\n"
    );
    let output = run_example("builtins/srand_seeding.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_srand_seeding_parens() {
    let expected = concat!(
        "10\n20\n0\ntrue\ntrue\n3\n7\ntrue\ntrue\n",
        "TypeError\nTypeError\ntrue\n"
    );
    let output = run_example("builtins/srand_seeding_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_enumerator_stepping_execution() {
    let expected = concat!(
        "[\"a\", \"b\"]\n",
        "a\n",
        "b\n",
        "iteration reached an end\n",
        "a\n",
        "[1, 2, 3]\n",
        "[[1, 2], [3, 4]]\n",
        "[\"a\", \"b\"]\n",
        "Enumerator\n",
        "1\n",
        "true\n",
        "true\n",
        "false\n",
        "true\n"
    );
    let output = run_example("builtins/enumerator/stepping.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_enumerator_stepping_parens_execution() {
    let expected = concat!(
        "[\"a\", \"b\"]\n",
        "a\n",
        "b\n",
        "iteration reached an end\n",
        "a\n",
        "[1, 2, 3]\n",
        "[[1, 2], [3, 4]]\n",
        "[\"a\", \"b\"]\n",
        "Enumerator\n",
        "1\n",
        "true\n",
        "true\n",
        "false\n",
        "true\n"
    );
    let output = run_example("builtins/enumerator/stepping_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_integer_bits_operations_execution() {
    let expected = concat!(
        "[8, 34359738368, 2, -2]\n",
        "[2, 10]\n",
        "[0, -1, 0]\n",
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
        "[0, -1, 0]\n",
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
fn test_builtins_kernel_array_execution() {
    let expected = concat!(
        "[]\n",
        "[1, 2]\n",
        "[3]\n",
        "[[:a, 1]]\n",
        "[1, 2]\n",
        "[3, 4]\n",
        "[5, 6]\n",
        "[7, 8]\n",
        "can't convert BadAry to Array (BadAry#to_ary gives String)\n",
        "can't convert BadToA to Array (BadToA#to_a gives String)\n",
        "true\n",
    );
    let output = run_example("builtins/kernel_array.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_kernel_array_parens_execution() {
    let expected = concat!(
        "[]\n",
        "[1, 2]\n",
        "[3]\n",
        "[[:a, 1]]\n",
        "[1, 2]\n",
        "[3, 4]\n",
        "[5, 6]\n",
        "[7, 8]\n",
        "can't convert BadAry to Array (BadAry#to_ary gives String)\n",
        "can't convert BadToA to Array (BadToA#to_a gives String)\n",
        "true\n",
    );
    let output = run_example("builtins/kernel_array_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_kernel_complex_execution() {
    let expected = concat!(
        "3\n",
        "4\n",
        "3+4i\n",
        "(3+4i)\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "invalid value for convert(): \"ruby\"\n",
        "can't convert nil into Complex\n",
        "nil\n",
        "true\n"
    );
    let output = run_example("builtins/kernel_complex.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_kernel_complex_parens_execution() {
    let expected = concat!(
        "3\n",
        "4\n",
        "3+4i\n",
        "(3+4i)\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "invalid value for convert(): \"ruby\"\n",
        "can't convert nil into Complex\n",
        "nil\n",
        "true\n"
    );
    let output = run_example("builtins/kernel_complex_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_kernel_float_execution() {
    let expected = concat!(
        "1.0\n",
        "1.5\n",
        "10.0\n",
        "10.0\n",
        "10.0\n",
        "-10.0\n",
        "1000.0\n",
        "1.0\n",
        "2000.0\n",
        "0.002\n",
        "16.0\n",
        "-123.0\n",
        "0.5\n",
        "1024.0\n",
        "1.0\n",
        "Infinity\n",
        "0.0\n",
        "true\n",
        "true\n",
        "1\n",
        "true\n",
        "1.25\n",
        "invalid value for Float(): \"float\"\n",
        "invalid value for Float(): \"10.0.0\"\n",
        "invalid value for Float(): \"10D\"\n",
        "invalid value for Float(): \"1+1\"\n",
        "invalid value for Float(): \"_1\"\n",
        "invalid value for Float(): \"10_\"\n",
        "invalid value for Float(): \" \"\n",
        "invalid value for Float(): \"1 2\"\n",
        "invalid value for Float(): \"2e\"\n",
        "invalid value for Float(): \"e2\"\n",
        "invalid value for Float(): \"0x_10\"\n",
        "can't convert nil into Float\n",
        "can't convert 2+3i into Float\n",
        "nil\n",
        "nil\n",
        "true\n"
    );
    let output = run_example("builtins/kernel_float.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_kernel_float_parens_execution() {
    let expected = concat!(
        "1.0\n",
        "1.5\n",
        "10.0\n",
        "10.0\n",
        "10.0\n",
        "-10.0\n",
        "1000.0\n",
        "1.0\n",
        "2000.0\n",
        "0.002\n",
        "16.0\n",
        "-123.0\n",
        "0.5\n",
        "1024.0\n",
        "1.0\n",
        "Infinity\n",
        "0.0\n",
        "true\n",
        "true\n",
        "1\n",
        "true\n",
        "1.25\n",
        "invalid value for Float(): \"float\"\n",
        "invalid value for Float(): \"10.0.0\"\n",
        "invalid value for Float(): \"10D\"\n",
        "invalid value for Float(): \"1+1\"\n",
        "invalid value for Float(): \"_1\"\n",
        "invalid value for Float(): \"10_\"\n",
        "invalid value for Float(): \" \"\n",
        "invalid value for Float(): \"1 2\"\n",
        "invalid value for Float(): \"2e\"\n",
        "invalid value for Float(): \"e2\"\n",
        "invalid value for Float(): \"0x_10\"\n",
        "can't convert nil into Float\n",
        "can't convert 2+3i into Float\n",
        "nil\n",
        "nil\n",
        "true\n"
    );
    let output = run_example("builtins/kernel_float_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_loop_enumerator_execution() {
    let expected = concat!(
        "3\n",
        "1\n",
        "2\n",
        "finished\n",
        "nil\n",
        "true\n",
        "Infinity\n",
        "4\n",
        "[[1, 2], [3, 4]]\n",
    );
    let output = run_example("builtins/loop_enumerator.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_loop_enumerator_parens_execution() {
    let expected = concat!(
        "3\n",
        "1\n",
        "2\n",
        "finished\n",
        "nil\n",
        "true\n",
        "Infinity\n",
        "4\n",
        "[[1, 2], [3, 4]]\n",
    );
    let output = run_example("builtins/loop_enumerator_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_kernel_open_execution() {
    let expected = concat!(
        "File\n",
        "first line\n",
        "second line\n",
        "nil\n",
        "first line\n",
        "first line\n",
        "[1, 2, 3]\n",
        "[]\n",
        "wrong number of arguments (given 0, expected 1..3)\n",
        "wrong number of arguments (given 4, expected 1..3)\n",
        "64\n",
        "true\n"
    );
    let output = run_example("builtins/kernel_open.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_kernel_open_parens_execution() {
    let expected = concat!(
        "File\n",
        "first line\n",
        "second line\n",
        "nil\n",
        "first line\n",
        "first line\n",
        "[1, 2, 3]\n",
        "[]\n",
        "wrong number of arguments (given 0, expected 1..3)\n",
        "wrong number of arguments (given 4, expected 1..3)\n",
        "64\n",
        "true\n"
    );
    let output = run_example("builtins/kernel_open_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_kernel_pp_execution() {
    let expected = concat!(
        "[1, 2, 3]\n",
        "{a: 1, \"b\" => 2}\n",
        "\"text\"\n",
        ":symbol\n",
        ":symbol\n",
        "1\n",
        "2\n",
        "[1, 2]\n",
        "nil\n",
        "true\n",
    );
    let output = run_example("builtins/kernel_pp.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_kernel_pp_parens_execution() {
    let expected = concat!(
        "[1, 2, 3]\n",
        "{a: 1, \"b\" => 2}\n",
        "\"text\"\n",
        ":symbol\n",
        ":symbol\n",
        "1\n",
        "2\n",
        "[1, 2]\n",
        "nil\n",
        "true\n",
    );
    let output = run_example("builtins/kernel_pp_parens.rb");
    assert_eq!(output, expected);
}
