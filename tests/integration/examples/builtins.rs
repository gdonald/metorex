use super::run_example;

// 10.4.12 — Builtins

#[test]
fn test_builtins_type_introspection() {
    let expected = "true\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nObject\nnil\n2\ntrue\ntrue\nAnimal\n2\nRex\n3\n4\n";
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
