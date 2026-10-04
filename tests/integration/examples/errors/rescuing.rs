// Catching an exception and carrying on.

use super::super::run_example;
use super::*;
#[test]
fn test_errors_simple_rescue_execution() {
    let expected = "Before exception\nCaught an exception\nAfter rescue block\nCaught exception with message: An error message\nIn try block\nIn rescue block\nIn ensure block\n";
    let output = run_example("errors/simple_rescue.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_begin_else_ensure_execution() {
    let expected = "try block\nno error, x = 42\nensure ran\n";
    let output = run_example("errors/begin_else_ensure.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_begin_else_ensure_parens_execution() {
    let expected = "try block\nno error, x = 42\nensure ran\n";
    let output = run_example("errors/begin_else_ensure_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_stop_iteration_result_execution() {
    let expected = "3\n2\n1\niteration reached an end\n:liftoff\n3\nnil\nnil\n";
    let output = run_example("errors/stop_iteration/result.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_stop_iteration_result_parens_execution() {
    let expected = "3\n2\n1\niteration reached an end\n:liftoff\n3\nnil\nnil\n";
    let output = run_example("errors/stop_iteration/result_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_rescue_scope_bare_rescue_execution() {
    let expected =
        "caught\ncaught\ncaught\ncaught\ncaught\ncaught\nException\npassed the bare rescue\n";
    let output = run_example("errors/rescue_scope/bare_rescue.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_rescue_scope_bare_rescue_parens_execution() {
    let expected =
        "caught\ncaught\ncaught\ncaught\ncaught\ncaught\nException\npassed the bare rescue\n";
    let output = run_example("errors/rescue_scope/bare_rescue_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_uncaught_throw_execution() {
    let expected = concat!(
        ":abc\n",
        "nil\n",
        "uncaught throw :abc\n",
        ":b\n",
        "\"carried\"\n",
        "nil\n",
        "nil\n",
        "false\n"
    );
    let output = run_example("errors/uncaught_throw.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_uncaught_throw_parens_execution() {
    let expected = concat!(
        ":abc\n",
        "nil\n",
        "uncaught throw :abc\n",
        ":b\n",
        "\"carried\"\n",
        "nil\n",
        "nil\n",
        "false\n"
    );
    let output = run_example("errors/uncaught_throw_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_handled_then_outer_execution() {
    let output = run_example("errors/handled_then_outer.rb");
    assert_eq!(output, HANDLED_THEN_OUTER_OUTPUT);
}

#[test]
fn test_errors_handled_then_outer_parens_execution() {
    let output = run_example("errors/handled_then_outer_parens.rb");
    assert_eq!(output, HANDLED_THEN_OUTER_OUTPUT);
}

#[test]
fn test_errors_rescue_handler_shapes_execution() {
    let output = run_example("errors/rescue_handler_shapes.rb");
    assert_eq!(output, RESCUE_HANDLER_SHAPES_OUTPUT);
}

#[test]
fn test_errors_rescue_handler_shapes_no_parens_execution() {
    let output = run_example("errors/rescue_handler_shapes_no_parens.rb");
    assert_eq!(output, RESCUE_HANDLER_SHAPES_OUTPUT);
}

#[test]
fn test_errors_rescue_names_in_scope_execution() {
    let output = run_example("errors/rescue_names_in_scope.rb");
    assert_eq!(output, RESCUE_NAMES_IN_SCOPE_OUTPUT);
}

#[test]
fn test_errors_rescue_names_in_scope_no_parens_execution() {
    let output = run_example("errors/rescue_names_in_scope_no_parens.rb");
    assert_eq!(output, RESCUE_NAMES_IN_SCOPE_OUTPUT);
}

#[test]
fn test_errors_ensure_return_drops_error_execution() {
    let expected = ":returned\nnil\n:returned\n\"outer\"\n";
    let output = run_example("errors/ensure_return_drops_error.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_ensure_return_drops_error_no_parens_execution() {
    let expected = ":returned\nnil\n:returned\n\"outer\"\n";
    let output = run_example("errors/ensure_return_drops_error_no_parens.rb");
    assert_eq!(output, expected);
}
