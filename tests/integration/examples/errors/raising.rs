// Raising an exception, and the one behind it.

use super::super::run_example;
use super::*;
#[test]
fn test_raise_two_arg_execution() {
    let expected = "caught two-arg raise\n";
    let output = run_example("errors/raise_two_arg.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_raise_two_arg_parens_execution() {
    let expected = "caught two-arg raise\n";
    let output = run_example("errors/raise_two_arg_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_cause_chain_execution() {
    let expected = concat!(
        "nil\n",
        "the consequence\n",
        "Exception\n",
        "the cause\n",
        "ZeroDivisionError\n",
        "true\ntrue\ntrue\n",
        "nil\n"
    );
    let output = run_example("errors/cause/chain.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_cause_chain_parens_execution() {
    let expected = concat!(
        "nil\n",
        "the consequence\n",
        "Exception\n",
        "the cause\n",
        "ZeroDivisionError\n",
        "true\ntrue\ntrue\n",
        "nil\n"
    );
    let output = run_example("errors/cause/chain_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_no_method_error_args_execution() {
    let expected = "[\"args\"]\n\"name\"\nmsg\nnil\n:missing\n[]\n[1, :two, \"three\"]\n:missing\n[1, :two, \"three\"]\n\"Receiver\"\nababab\n-----\nnegative argument\n";
    let output = run_example("errors/no_method_error/args.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_no_method_error_args_parens_execution() {
    let expected = "[\"args\"]\n\"name\"\nmsg\nnil\n:missing\n[]\n[1, :two, \"three\"]\n:missing\n[1, :two, \"three\"]\n\"Receiver\"\nababab\n-----\nnegative argument\n";
    let output = run_example("errors/no_method_error/args_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_raised_position_execution() {
    let output = run_example("errors/raised_position.rb");
    assert_eq!(output, RAISED_POSITION_OUTPUT);
}

#[test]
fn test_errors_raised_position_no_parens_execution() {
    let output = run_example("errors/raised_position_no_parens.rb");
    assert_eq!(output, RAISED_POSITION_OUTPUT);
}

#[test]
fn test_errors_cause_named_execution() {
    let output = run_example("errors/cause/named.rb");
    assert_eq!(output, CAUSE_NAMED_OUTPUT);
}

#[test]
fn test_errors_cause_named_no_parens_execution() {
    let output = run_example("errors/cause/named_no_parens.rb");
    assert_eq!(output, CAUSE_NAMED_OUTPUT);
}
