use super::run_example;

// 10.4.12 — Builtins

#[test]
fn test_builtins_type_introspection() {
    let expected = "true\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nObject\nBasicObject\n2\ntrue\ntrue\nAnimal\n2\nRex\n3\n4\n";
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
    let expected = "42\n3\n42\n\n[]\n[1, 2]\n[hi]\ncan't convert TrueClass into Integer\ncan't convert nil into Integer\n";
    let output = run_example("builtins/kernel_conversion.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_kernel_conversion_parens_execution() {
    let expected = "42\n3\n42\n\n[]\n[1, 2]\n[hi]\ncan't convert TrueClass into Integer\ncan't convert nil into Integer\n";
    let output = run_example("builtins/kernel_conversion_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_keyword_symbols_execution() {
    let expected = ":def\n:class\n:if\n:else\n:end\n:do\n:nil\n:true\n:false\n:return\n:begin\n:rescue\n:ensure\n:while\n:for\n:case\n:when\n:module\n:include\n:yield\n:super\n:lambda\n:break\n:next\n:raise\n:@ivar\n:@@cvar\n";
    let output = run_example("builtins/keyword_symbols.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_keyword_symbols_parens_execution() {
    let expected = ":def\n:class\n:@ivar\n:@@cvar\n:yield\n";
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
        "9223372036854775806\n13835058055282164000\n-9223372036854775808\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n1\n7\n6\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\n"
    );
}

#[test]
fn test_operators_coverage_no_parens_execution() {
    let output = run_example("builtins/operators_coverage_no_parens.rb");
    assert_eq!(
        output,
        "9223372036854775806\n13835058055282164000\n-9223372036854775808\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n1\n7\n6\n"
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
