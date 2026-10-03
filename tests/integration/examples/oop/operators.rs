// The operators a class defines, and the comparisons they make.

use super::super::run_example;
use super::*;
#[test]
fn test_oop_operator_methods_execution() {
    let expected = "(4, 6)\ntrue\nfalse\n";
    let output = run_example("oop/operator_methods/operator_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_parens_execution() {
    let expected = "(4, 6)\ntrue\nfalse\n";
    let output = run_example("oop/operator_methods/operator_methods_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_index_execution() {
    let expected = "42\n99\n0\n";
    let output = run_example("oop/operator_methods/index.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_index_parens_execution() {
    let expected = "42\n99\n0\n";
    let output = run_example("oop/operator_methods/index_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_extended_execution() {
    let expected = "-1\n1\n0\n11\n";
    let output = run_example("oop/operator_methods/extended.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_extended_parens_execution() {
    let expected = "-1\n1\n0\n11\n";
    let output = run_example("oop/operator_methods/extended_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_multi_arg_bracket_execution() {
    let expected = "1\n2\n3\n4\n";
    let output = run_example("oop/multi_arg_bracket/multi_arg_bracket.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_multi_arg_bracket_parens_execution() {
    let expected = "1\n2\n3\n4\n";
    let output = run_example("oop/multi_arg_bracket/multi_arg_bracket_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_empty_bracket_call_execution() {
    let expected = "3\n";
    let output = run_example("oop/empty_bracket_call/empty_bracket_call.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_empty_bracket_call_parens_execution() {
    let expected = "3\n";
    let output = run_example("oop/empty_bracket_call/empty_bracket_call_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_comparable_execution() {
    let expected = "true\nfalse\ntrue\nfalse\ntrue\ntrue\nfalse\ntrue\n";
    let output = run_example("oop/comparable/comparable.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_comparable_parens_execution() {
    let expected = "true\nfalse\ntrue\nfalse\ntrue\ntrue\nfalse\ntrue\n";
    let output = run_example("oop/comparable/comparable_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_case_compare_extend_execution() {
    let expected = "Basic === obj: true\nSuper === obj: true\nobj.is_a?(Basic): true\nobj.is_a?(Super): true\nBasic === Child.new: true\nSuper === Child.new: true\n";
    let output = run_example("oop/case_compare_extend.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_case_compare_extend_parens_execution() {
    let expected = "Basic === obj: true\nSuper === obj: true\nobj.is_a?(Basic): true\nobj.is_a?(Super): true\nBasic === Child.new: true\nSuper === Child.new: true\n";
    let output = run_example("oop/case_compare_extend_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_comparable_singleton_spaceship_execution() {
    let expected = "true\n1\n";
    let output = run_example("oop/comparable/singleton_spaceship.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_comparable_singleton_class_expr_execution() {
    let expected = "true\n";
    let output = run_example("oop/comparable/singleton_class_expr.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_case_equality_execution() {
    let expected = "true\ntrue\ntrue\nfalse\nfalse\ntrue\nfalse\n";
    let output = run_example("oop/case_equality.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_method_names_execution() {
    let expected = "-1\n1\n30\n30\n9\n-1\n0\nnil\nnil\n";
    let output = run_example("oop/operator_method_names.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_equality_not_equal_execution() {
    let expected = "true\nfalse\ntrue\nfalse\ntrue\nfalse\nfalse\nfalse\ntrue\n";
    let output = run_example("oop/equality/not_equal.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_equality_not_equal_parens_execution() {
    let expected = "true\nfalse\ntrue\nfalse\ntrue\nfalse\nfalse\nfalse\ntrue\n";
    let output = run_example("oop/equality/not_equal_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_case_equality_of_classes_execution() {
    let output = run_example("oop/case_equality_of_classes.rb");
    assert_eq!(output, CASE_EQUALITY_OF_CLASSES_OUTPUT);
}

#[test]
fn test_oop_case_equality_of_classes_parens_execution() {
    let output = run_example("oop/case_equality_of_classes_parens.rb");
    assert_eq!(output, CASE_EQUALITY_OF_CLASSES_OUTPUT);
}

/// The expected output of both `oop/comparable/operators_need_comparable`
/// variants.
const OPERATORS_NEED_COMPARABLE_OUTPUT: &str = concat!(
    "undefined method '<' for an instance of Plain\n",
    "true\n",
    "comparison of Ranked with nil failed\n",
    "comparison of Ranked with 2.5 failed\n",
    "comparison of Ranked with :sym failed\n",
    "comparison of Ranked with String failed\n",
    "comparison of Ranked with Array failed\n",
    "true\n",
);

#[test]
fn test_oop_comparable_operators_need_comparable_execution() {
    let output = run_example("oop/comparable/operators_need_comparable.rb");
    assert_eq!(output, OPERATORS_NEED_COMPARABLE_OUTPUT);
}

#[test]
fn test_oop_comparable_operators_need_comparable_no_parens_execution() {
    let output = run_example("oop/comparable/operators_need_comparable_no_parens.rb");
    assert_eq!(output, OPERATORS_NEED_COMPARABLE_OUTPUT);
}
