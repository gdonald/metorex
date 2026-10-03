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

/// The expected output of both `struct/shaped_by_initialize` variants.
const SHAPED_BY_INITIALIZE_OUTPUT: &str =
    "true\n[3, 4]\n[5, nil]\n\"struct size differs\"\n[\"held\", :tagged]\ntrue\n";

#[test]
fn test_struct_shaped_by_initialize_execution() {
    let output = run_example("struct/shaped_by_initialize.rb");
    assert_eq!(output, SHAPED_BY_INITIALIZE_OUTPUT);
}

#[test]
fn test_struct_shaped_by_initialize_parens_execution() {
    let output = run_example("struct/shaped_by_initialize_parens.rb");
    assert_eq!(output, SHAPED_BY_INITIALIZE_OUTPUT);
}

/// The expected output of both `struct/named_shapes` variants, which differ
/// only in whether the calls are written with parentheses.
const NAMED_SHAPES_OUTPUT: &str = concat!(
    "\"Struct::Waypoint\"\n",
    "\"summit\"\n",
    "[1, 2]\n",
    "[1, nil]\n",
    "[\"elefant\", 4]\n",
    "[\"mouse\", 4]\n",
    "duplicate member: foo\n",
    "NameError\n",
    "unknown keywords: missing\n"
);

#[test]
fn test_struct_named_shapes_execution() {
    let output = run_example("struct/named_shapes.rb");
    assert_eq!(output, NAMED_SHAPES_OUTPUT);
}

#[test]
fn test_struct_named_shapes_parens_execution() {
    let output = run_example("struct/named_shapes_parens.rb");
    assert_eq!(output, NAMED_SHAPES_OUTPUT);
}
