// Building classes and modules while the program runs.

use super::super::run_example;
use super::*;
#[test]
fn test_metaprogramming_top_level_define_method_execution() {
    let expected = "true\nboing\nfalse\nfalse\n";
    let output = run_example("metaprogramming/top_level_define_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_anonymous_class_execution() {
    let expected = "foo\nfoo\nbaz\nModule\n";
    let output = run_example("metaprogramming/anonymous_class.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_eval_execution() {
    let expected = "true\na widget\n42\n2\ntrue\nA WIDGET\n[\"custom.rb\", 102]\n:ok\n";
    let output = run_example("metaprogramming/class_eval.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_eval_parens_execution() {
    let expected = "true\na widget\n42\n2\ntrue\nA WIDGET\n[\"custom.rb\", 102]\n:ok\n";
    let output = run_example("metaprogramming/class_eval_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_exec_execution() {
    let expected = "gadget\n42\n2\ntag\n42\n";
    let output = run_example("metaprogramming/class_exec.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_exec_parens_execution() {
    let expected = "gadget\n42\n2\ntag\n42\n";
    let output = run_example("metaprogramming/class_exec_parens.rb");
    assert_eq!(output, expected);
}

// 14.3 — Runtime Class Modification

#[test]
fn test_metaprogramming_class_modification_execution() {
    let expected = r#"=== alias_method ===
Hello, Alice!
Hello, Bob!

=== remove_method ===
moving
no method: speak

=== undef_method ===
Base farewell
undefined: greet

=== multiple aliases ===
HELLO
HELLO
HELLO

=== module_function ===
14
12

=== remove_method with inheritance ===
Parent greet
"#;
    let output = run_example("metaprogramming/class_modification/class_modification.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_modification_no_parens_execution() {
    let expected = r#"=== alias_method ===
Hello, Alice!
Hello, Bob!

=== remove_method ===
moving
no method: speak

=== undef_method ===
Base farewell
undefined: greet

=== multiple aliases ===
HELLO
HELLO
HELLO

=== module_function ===
14
12

=== remove_method with inheritance ===
Parent greet
"#;
    let output = run_example("metaprogramming/class_modification/class_modification_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_variable_defined_execution() {
    let expected = "true\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\n";
    let output = run_example("metaprogramming/class_variable_defined.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_variable_defined_parens_execution() {
    let expected = "true\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\n";
    let output = run_example("metaprogramming/class_variable_defined_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_variable_get_execution() {
    let expected = "7\n7\nyes\n7\nhere\nmissing raises NameError\n";
    let output = run_example("metaprogramming/class_variable_get.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_variable_get_parens_execution() {
    let expected = "7\n7\nyes\n7\nhere\nmissing raises NameError\n";
    let output = run_example("metaprogramming/class_variable_get_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_variable_set_execution() {
    let expected =
        "on\non\n3\n3\nfrozen Class raises FrozenError\nfrozen Module raises FrozenError\n";
    let output = run_example("metaprogramming/class_variable_set.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_variable_set_parens_execution() {
    let expected =
        "on\non\n3\n3\nfrozen Class raises FrozenError\nfrozen Module raises FrozenError\n";
    let output = run_example("metaprogramming/class_variable_set_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_variables_execution() {
    let expected =
        "[:@@base, :@@shared]\n[:@@derived, :@@base, :@@shared]\n[:@@derived]\n[:@@flag]\n";
    let output = run_example("metaprogramming/class_variables.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_variables_parens_execution() {
    let expected =
        "[:@@base, :@@shared]\n[:@@derived, :@@base, :@@shared]\n[:@@derived]\n[:@@flag]\n";
    let output = run_example("metaprogramming/class_variables_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_module_constants_execution() {
    let expected = "[:DEEP_CONST, :OWN_CONST, :SHALLOW_CONST]\n[:OWN_CONST]\n[:OWN_CONST]\n[:DEEP_CONST, :SHALLOW_CONST]\nW\ntrue\ntrue\ntrue\n";
    let output = run_example("metaprogramming/module_constants.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_module_constants_parens_execution() {
    let expected = "[:DEEP_CONST, :OWN_CONST, :SHALLOW_CONST]\n[:OWN_CONST]\n[:OWN_CONST]\n[:DEEP_CONST, :SHALLOW_CONST]\nW\ntrue\ntrue\ntrue\n";
    let output = run_example("metaprogramming/module_constants_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_inside_a_singleton_class_execution() {
    let output = run_example("metaprogramming/inside_a_singleton_class.rb");
    assert_eq!(output, INSIDE_A_SINGLETON_CLASS_OUTPUT);
}

#[test]
fn test_metaprogramming_inside_a_singleton_class_no_parens_execution() {
    let output = run_example("metaprogramming/inside_a_singleton_class_no_parens.rb");
    assert_eq!(output, INSIDE_A_SINGLETON_CLASS_OUTPUT);
}
