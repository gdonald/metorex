// Conditions, loops and the operators that read them.

use super::super::run_example;
use super::*;
#[test]
fn test_basics_simple_range_execution() {
    let expected = "1..5\n1...5\n1\n2\n3\n4\n5\n1\n2\n3\n4\n";
    let output = run_example("basics/simple_range.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_array_execution() {
    let expected = "1\n2\n3\n";
    let output = run_example("basics/for_loop_array.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_range_execution() {
    let expected = "1\n2\n3\n4\n5\n";
    let output = run_example("basics/for_loop_range.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_break_execution() {
    let expected = "1\n2\n3\n4\n";
    let output = run_example("basics/for_loop_break.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_continue_execution() {
    let expected = "1\n2\n4\n5\n";
    let output = run_example("basics/for_loop_continue.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_elsif_basic_execution() {
    let expected = "small positive\n";
    let output = run_example("basics/elsif_basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_elsif_without_else_execution() {
    let expected = "C\n";
    let output = run_example("basics/elsif_without_else.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_elsif_no_parens_execution() {
    let expected = "warm\n";
    let output = run_example("basics/elsif_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_bitwise_ops_execution() {
    let expected = "8\n14\n6\n170\n255\n85\n0\n255\n255\n";
    let output = run_example("basics/bitwise_ops.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_bitwise_ops_parens_execution() {
    let expected = "8\n14\n6\n170\n255\n85\n0\n255\n255\n";
    let output = run_example("basics/bitwise_ops_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_type_annotations_collection_types_execution() {
    let output = run_example("type_annotations/collection_types.rb");
    let valid_output1 = "numbers = [1, 2, 3, 4, 5]\nscores = {\"Bob\" => 85, \"Alice\" => 90}\nlength of numbers: 5\nAlice's score: 90\n";
    let valid_output2 = "numbers = [1, 2, 3, 4, 5]\nscores = {\"Alice\" => 90, \"Bob\" => 85}\nlength of numbers: 5\nAlice's score: 90\n";
    assert!(
        output == valid_output1 || output == valid_output2,
        "Expected either '{}' or '{}', but got '{}'",
        valid_output1,
        valid_output2,
        output
    );
}

#[test]
fn test_basics_then_yield_self_tap_execution() {
    let expected = "6\n10\n5\n5\nVALUE\n3\n42\n";
    let output = run_example("basics/then_yield_self_tap.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_not_match_operator() {
    let expected = concat!(
        "false\ntrue\nfalse\nfalse\ntrue\n:custom\n",
        "NoMethodError: undefined method '=~' for an instance of Object\n",
        "undefined method '=~' for an instance of Integer\n"
    );
    let output = run_example("basics/not_match_operator.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_not_match_operator_no_parens() {
    let expected = concat!(
        "false\ntrue\nfalse\nfalse\ntrue\n:custom\n",
        "NoMethodError: undefined method '=~' for an instance of Object\n",
        "undefined method '=~' for an instance of Integer\n"
    );
    let output = run_example("basics/not_match_operator_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_logical_keyword_precedence_execution() {
    let output = run_example("basics/logical_keyword_precedence.rb");
    assert_eq!(output, LOGICAL_KEYWORD_PRECEDENCE_OUTPUT);
}

#[test]
fn test_basics_logical_keyword_precedence_no_parens_execution() {
    let output = run_example("basics/logical_keyword_precedence_no_parens.rb");
    assert_eq!(output, LOGICAL_KEYWORD_PRECEDENCE_OUTPUT);
}
