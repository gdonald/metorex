// The Kernel functions every object reaches.

use super::super::run_example;
use super::*;
// 10.4.12 — Builtins

#[test]
fn test_builtins_type_introspection() {
    let expected = "true\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nNumeric\nBasicObject\n6\ntrue\ntrue\nAnimal\n2\nRex\n3\n4\n";
    let output = run_example("builtins/type_introspection.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_defined_keyword_execution() {
    let expected = "local-variable\n\nmethod\nconstant\n\nglobal-variable\n\nexpression\nexpression\nexpression\nexpression\nlocal-variable\n";
    let output = run_example("builtins/defined_keyword.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_defined_keyword_parens_execution() {
    let expected = "local-variable\n\nmethod\nconstant\nexpression\n";
    let output = run_example("builtins/defined_keyword_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_kernel_conversion_execution() {
    let expected = "42\n3\n42\n\n1\n2\nhi\ncan't convert TrueClass into Integer\ncan't convert nil into Integer\n";
    let output = run_example("builtins/kernel_conversion.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_kernel_conversion_parens_execution() {
    let expected = "42\n3\n42\n\n1\n2\nhi\ncan't convert TrueClass into Integer\ncan't convert nil into Integer\n";
    let output = run_example("builtins/kernel_conversion_parens.rb");
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
    let expected = "local-variable\nmethod\nconstant\n\nglobal-variable\n\ninstance-variable\n\nexpression\nexpression\nexpression\nexpression\nyield\n\n\n";
    let output = run_example("builtins/defined_extended.rb");
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
fn test_kernel_string_execution() {
    let expected = "\"already\"\n\"\"\n\"1.12\"\n\"true\"\n\"false\"\n\"42\"\n\"Object\"\nsymbol\ntag\ncan't convert Silent into String\ncan't convert Wrong to String (Wrong#to_s gives Integer)\ntrue\nmetorex\n7\nMETOREX\ntrue\ntrue\n7\ntrue\n";
    let output = run_example("builtins/kernel_string.rb");
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
        "true\n",
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
        "true\n",
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

#[test]
fn test_builtins_argf_stream_execution() {
    let expected = concat!(
        "\"ARGF\"\n",
        "true\n",
        "[\"/tmp/metorex_argf_plain_two.txt\"]\n",
        "\"alpha\\n\"\n",
        "1\n",
        "false\n",
        "\"beta\\n\"\n",
        "true\n",
        "\"gamma\\n\"\n",
        "true\n",
        "[\"delta\\n\"]\n",
        "\"closed stream\"\n",
        "\"al\"\n",
        "2\n",
        "0\n",
        "\"a\"\n",
        "\"lpha\\nbeta\\n\"\n",
    );
    let output = run_example("builtins/argf_stream.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_argf_stream_parens_execution() {
    let expected = concat!(
        "\"ARGF\"\n",
        "true\n",
        "[\"/tmp/metorex_argf_parens_two.txt\"]\n",
        "\"alpha\\n\"\n",
        "1\n",
        "false\n",
        "\"beta\\n\"\n",
        "true\n",
        "\"gamma\\n\"\n",
        "true\n",
        "[\"delta\\n\"]\n",
        "\"closed stream\"\n",
        "\"al\"\n",
        "2\n",
        "0\n",
        "\"a\"\n",
        "\"lpha\\nbeta\\n\"\n",
    );
    let output = run_example("builtins/argf_stream_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_environment_lookup_coercion_execution() {
    let output = run_example("builtins/environment_lookup/coercion.rb");
    assert_eq!(output, ENVIRONMENT_LOOKUP_OUTPUT);
}

#[test]
fn test_builtins_environment_lookup_coercion_parens_execution() {
    let output = run_example("builtins/environment_lookup/coercion_parens.rb");
    assert_eq!(output, ENVIRONMENT_LOOKUP_OUTPUT);
}

#[test]
fn test_builtins_frozen_answers_execution() {
    let output = run_example("builtins/frozen_answers.rb");
    assert_eq!(output, FROZEN_ANSWERS_OUTPUT);
}

#[test]
fn test_builtins_frozen_answers_no_parens_execution() {
    let output = run_example("builtins/frozen_answers_no_parens.rb");
    assert_eq!(output, FROZEN_ANSWERS_OUTPUT);
}
