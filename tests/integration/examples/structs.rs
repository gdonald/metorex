use super::run_example;

/// The expected output of both `struct/member_methods` variants, which differ
/// only in whether the calls are written with parentheses.
const MEMBER_METHODS_OUTPUT: &str = "42\n7\n[]\n[1, 2]\n2\n[\"[:x, 1]\", \"[:y, 2]\"]\n[2]\n[1]\n[1, 2]\n[1, 2, nil, nil]\n{x: 1}\n{x: 1, y: 2}\n{\"x\" => 10, \"y\" => 20}\nnil\nfalse\nempty::\nFrozenError\ntrue\nfalse\n";

#[test]
fn test_struct_member_methods_execution() {
    let output = run_example("struct/member_methods.rb");
    assert_eq!(output, MEMBER_METHODS_OUTPUT);
}

#[test]
fn test_struct_member_methods_no_parens_execution() {
    let output = run_example("struct/member_methods_no_parens.rb");
    assert_eq!(output, MEMBER_METHODS_OUTPUT);
}
