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
    let expected = "42\n3\n1\n0\n42\nnil\n[]\n[1, 2]\n[hi]\n";
    let output = run_example("builtins/kernel_conversion.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_kernel_conversion_parens_execution() {
    let expected = "42\n3\n1\n0\n42\nnil\n[]\n[1, 2]\n[hi]\n";
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
