use super::run_example;

/// The expected output of both `callables/blocks_and_methods` variants, which
/// differ only in whether the calls are written with parentheses.
const BLOCKS_AND_METHODS_OUTPUT: &str = "[1, 2]\n[3, nil]\n[1, 2, 3]\n5\n9\n:fallback\n[1, 2]\n[3, 4]\n7\n3\n{lambda: true, if: 1, class: 2}\n:greet\n:hello\n\"hi ada\"\n\"Greeter\"\n\"Greeter\"\n3\n:+\n\"Enumerator\"\n1\n2\n[1, 2, 3]\n[:a]\n[:b1, :b2]\n[1, 2, 3]\n[8, 9, 10]\n1\n9\n10\nInfinity\ntrue\ntrue\ntrue\nfalse\ntrue\n4\n[4, 3, 2, 1]\n\"padded  \"\n\"  padded\"\n[\"a\\n\", \"b\\n\"]\n2\n";

#[test]
fn test_callables_blocks_and_methods_execution() {
    let output = run_example("callables/blocks_and_methods.rb");
    assert_eq!(output, BLOCKS_AND_METHODS_OUTPUT);
}

#[test]
fn test_callables_blocks_and_methods_no_parens_execution() {
    let output = run_example("callables/blocks_and_methods_no_parens.rb");
    assert_eq!(output, BLOCKS_AND_METHODS_OUTPUT);
}

/// The expected output of both `callables/lambda_parameters` variants, which
/// differ only in whether the calls are written with parentheses.
const LAMBDA_PARAMETERS_OUTPUT: &str = "\"hello world\"\n\"hello there\"\n[1, 2, 3, nil, nil]\n[[1, 2], :none]\n[[1], :marked]\n3\n\"wrong number of arguments (given 1, expected 2)\"\n7\n[[\"a\", 1], [\"b\", 2]]\n\"l\"\n\"llo\"\ntrue\nfalse\n\"undefined method 'new' for Symbol:Class\"\ntrue\nInteger\n";

#[test]
fn test_callables_lambda_parameters_execution() {
    let output = run_example("callables/lambda_parameters.rb");
    assert_eq!(output, LAMBDA_PARAMETERS_OUTPUT);
}

#[test]
fn test_callables_lambda_parameters_no_parens_execution() {
    let output = run_example("callables/lambda_parameters_no_parens.rb");
    assert_eq!(output, LAMBDA_PARAMETERS_OUTPUT);
}

/// The expected output of both `callables/currying_and_forwarding` variants,
/// which differ only in whether the calls are written with parentheses.
const CURRYING_AND_FORWARDING_OUTPUT: &str =
    "6\ntrue\nfalse\n[1, 2]\ntrue\n9\n[true, 2]\n[1, 2, 3]\n[1, 2]\n";

#[test]
fn test_callables_currying_and_forwarding_execution() {
    let output = run_example("callables/currying_and_forwarding.rb");
    assert_eq!(output, CURRYING_AND_FORWARDING_OUTPUT);
}

#[test]
fn test_callables_currying_and_forwarding_no_parens_execution() {
    let output = run_example("callables/currying_and_forwarding_no_parens.rb");
    assert_eq!(output, CURRYING_AND_FORWARDING_OUTPUT);
}

/// The expected output of both `callables/copies_and_visibility` variants.
const COPIES_AND_VISIBILITY_OUTPUT: &str = "true\nfalse\n:open\n[:@note]\ntrue\nfalse\n:open\nprotected method 'shut_door' called for an instance of Holder\nprivate method 'hidden_door' called for an instance of Holder\ntrue\n[[:req], [:rest]]\n\"SHOUT\"\n";

#[test]
fn test_callables_copies_and_visibility_execution() {
    let output = run_example("callables/copies_and_visibility.rb");
    assert_eq!(output, COPIES_AND_VISIBILITY_OUTPUT);
}

#[test]
fn test_callables_copies_and_visibility_parens_execution() {
    let output = run_example("callables/copies_and_visibility_parens.rb");
    assert_eq!(output, COPIES_AND_VISIBILITY_OUTPUT);
}

/// The expected output of both `callables/curried_shapes` variants, which differ only in whether
/// the calls are written with parentheses.
const CURRIED_SHAPES_OUTPUT: &str = "[[:rest]]\n-1\n3\nArgumentError\n6\n[[:rest]]\n42\nnil\n\"city is missing\"\nnil\nnil\nTypeError\nTypeError\n";

#[test]
fn test_callables_curried_shapes_execution() {
    let output = run_example("callables/curried_shapes.rb");
    assert_eq!(output, CURRIED_SHAPES_OUTPUT);
}

#[test]
fn test_callables_curried_shapes_parens_execution() {
    let output = run_example("callables/curried_shapes_parens.rb");
    assert_eq!(output, CURRIED_SHAPES_OUTPUT);
}
