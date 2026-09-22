// Ranges and the walks they hand back.

use super::super::run_example;
use super::*;
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
fn test_builtins_range_bounds_execution() {
    let output = run_example("builtins/range_bounds.rb");
    assert_eq!(output, RANGE_BOUNDS_OUTPUT);
}

#[test]
fn test_builtins_range_bounds_no_parens_execution() {
    let output = run_example("builtins/range_bounds_no_parens.rb");
    assert_eq!(output, RANGE_BOUNDS_OUTPUT);
}

#[test]
fn test_builtins_enumerator_walk_shapes_execution() {
    let output = run_example("builtins/enumerator/walk_shapes.rb");
    assert_eq!(output, WALK_SHAPES_OUTPUT);
}

#[test]
fn test_builtins_enumerator_walk_shapes_parens_execution() {
    let output = run_example("builtins/enumerator/walk_shapes_parens.rb");
    assert_eq!(output, WALK_SHAPES_OUTPUT);
}
