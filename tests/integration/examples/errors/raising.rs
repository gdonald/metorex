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
        "nil\n",
        "#<RuntimeError: handled first>\n",
        "#<RuntimeError: handled second>\n",
        "handled first\n"
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
        "nil\n",
        "#<RuntimeError: handled first>\n",
        "#<RuntimeError: handled second>\n",
        "handled first\n"
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

/// The expected output of both `errors/no_method_error/anonymous_receivers`
/// variants.
const ANONYMOUS_RECEIVERS_OUTPUT: &str = concat!(
    "private method 'hidden' called for an instance of #<Class:0xADDRESS>\n",
    "undefined method 'missing' for an instance of #<Class:0xADDRESS>\n",
);

#[test]
fn test_errors_no_method_error_anonymous_receivers_execution() {
    let output = run_example("errors/no_method_error/anonymous_receivers.rb");
    assert_eq!(output, ANONYMOUS_RECEIVERS_OUTPUT);
}

#[test]
fn test_errors_no_method_error_anonymous_receivers_no_parens_execution() {
    let output = run_example("errors/no_method_error/anonymous_receivers_no_parens.rb");
    assert_eq!(output, ANONYMOUS_RECEIVERS_OUTPUT);
}

/// The expected output of both `errors/no_method_error/missing_setter`
/// variants.
const MISSING_SETTER_OUTPUT: &str = concat!(
    "undefined method 'value=' for an instance of Reading\n",
    ":value=\n",
    "true\n",
    "1\n",
);

#[test]
fn test_errors_no_method_error_missing_setter_execution() {
    let output = run_example("errors/no_method_error/missing_setter.rb");
    assert_eq!(output, MISSING_SETTER_OUTPUT);
}

#[test]
fn test_errors_no_method_error_missing_setter_parens_execution() {
    let output = run_example("errors/no_method_error/missing_setter_parens.rb");
    assert_eq!(output, MISSING_SETTER_OUTPUT);
}

/// The expected output of both `errors/implicit_integer_conversion` variants.
const IMPLICIT_INTEGER_CONVERSION_OUTPUT: &str = concat!(
    "no implicit conversion from nil to integer\n",
    "no implicit conversion of true into Integer\n",
    "no implicit conversion of false into Integer\n",
    "no implicit conversion of Object into Integer\n",
    "no implicit conversion of String into Integer\n",
);

#[test]
fn test_errors_implicit_integer_conversion_execution() {
    let output = run_example("errors/implicit_integer_conversion.rb");
    assert_eq!(output, IMPLICIT_INTEGER_CONVERSION_OUTPUT);
}

#[test]
fn test_errors_implicit_integer_conversion_parens_execution() {
    let output = run_example("errors/implicit_integer_conversion_parens.rb");
    assert_eq!(output, IMPLICIT_INTEGER_CONVERSION_OUTPUT);
}
