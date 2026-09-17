// Examples covering the forms the parser reads

use super::run_example;

/// The expected output of both `syntax/reading_forms` variants, which differ
/// only in whether the calls are written with parentheses.
const READING_FORMS_OUTPUT: &str = "\"matched\"\n\"kept\"\n\"nil\"\n[0, nil, 2]\n{nil => nil}\n3\n[\"a\", \"3\", \"c\"]\n[:a, :b3]\n[\"x\", \"y\"]\n[:x, :y]\n\"a\\#{1}b\"\n\"a(b\"\n[4, 6]\nnil\n5\nnil\n[2, 3]\n";

#[test]
fn test_syntax_reading_forms_execution() {
    let output = run_example("syntax/reading_forms.rb");
    assert_eq!(output, READING_FORMS_OUTPUT);
}

#[test]
fn test_syntax_reading_forms_no_parens_execution() {
    let output = run_example("syntax/reading_forms_no_parens.rb");
    assert_eq!(output, READING_FORMS_OUTPUT);
}

/// The expected output of both `syntax/splat_assignment` variants.
const SPLAT_ASSIGNMENT_OUTPUT: &str = "1\n[2, 3]\n1\n[]\n[1, 2]\n3\n1\n[2, 3, 4]\n5\n1\n[]\n2\n[1, 2]\n9\n[]\n1\n[2, 3]\n\"first\"\n[\"second\", \"third\"]\n";

#[test]
fn test_syntax_splat_assignment_execution() {
    let output = run_example("syntax/splat_assignment.rb");
    assert_eq!(output, SPLAT_ASSIGNMENT_OUTPUT);
}

#[test]
fn test_syntax_splat_assignment_no_parens_execution() {
    let output = run_example("syntax/splat_assignment_no_parens.rb");
    assert_eq!(output, SPLAT_ASSIGNMENT_OUTPUT);
}

/// The expected output of both `syntax/escape_runs/bytes` variants.
const ESCAPE_RUNS_OUTPUT: &str =
    "\"ロ\"\n\"ロ\"\n[227, 131, 173]\n[0]\n\"aAb\"\n5\n\"tab\\there\"\n\"ロ\"\n[27, 91, 49, 109]\n";

#[test]
fn test_syntax_escape_runs_bytes_execution() {
    let output = run_example("syntax/escape_runs/bytes.rb");
    assert_eq!(output, ESCAPE_RUNS_OUTPUT);
}

#[test]
fn test_syntax_escape_runs_bytes_parens_execution() {
    let output = run_example("syntax/escape_runs/bytes_parens.rb");
    assert_eq!(output, ESCAPE_RUNS_OUTPUT);
}

/// The expected output of both `syntax/binary_source/bytes` variants.
const BINARY_SOURCE_OUTPUT: &str = "3\n3\n3\n";

#[test]
fn test_syntax_binary_source_bytes_execution() {
    let output = run_example("syntax/binary_source/bytes.rb");
    assert_eq!(output, BINARY_SOURCE_OUTPUT);
}

#[test]
fn test_syntax_binary_source_bytes_parens_execution() {
    let output = run_example("syntax/binary_source/bytes_parens.rb");
    assert_eq!(output, BINARY_SOURCE_OUTPUT);
}

/// The expected output of both `syntax/grouped_assignment` variants, which
/// differ only in whether the calls are written with parentheses.
const GROUPED_ASSIGNMENT_OUTPUT: &str = "[1, 2]\n[1, 2]\n1\n[1, nil, nil]\n\"assignment\"\n\"assignment\"\n\"assignment\"\n\"assignment\"\n\"method\"\n[]\n";

#[test]
fn test_syntax_grouped_assignment_execution() {
    let output = run_example("syntax/grouped_assignment.rb");
    assert_eq!(output, GROUPED_ASSIGNMENT_OUTPUT);
}

#[test]
fn test_syntax_grouped_assignment_no_parens_execution() {
    let output = run_example("syntax/grouped_assignment_no_parens.rb");
    assert_eq!(output, GROUPED_ASSIGNMENT_OUTPUT);
}

/// The expected output of both `syntax/retry_and_rescue` variants.
const RETRY_AND_RESCUE_OUTPUT: &str = "[4, :done]\n[2, 4]\nrefused outside a rescue\n";

#[test]
fn test_syntax_retry_and_rescue_execution() {
    let output = run_example("syntax/retry_and_rescue.rb");
    assert_eq!(output, RETRY_AND_RESCUE_OUTPUT);
}

#[test]
fn test_syntax_retry_and_rescue_parens_execution() {
    let output = run_example("syntax/retry_and_rescue_parens.rb");
    assert_eq!(output, RETRY_AND_RESCUE_OUTPUT);
}

/// The expected output of both `syntax/grouped_targets` variants, which differ only in whether
/// the calls are written with parentheses.
const GROUPED_TARGETS_OUTPUT: &str = "[1, 2, 3]\n[1, nil, nil, nil, nil, nil, nil, nil]\n[4, [5, 6]]\n[nil]\n[7, 8, 9]\n[3, 4, 1, 2]\n[:a, :b]\n[:one, :two]\nnil\n2\n";

#[test]
fn test_syntax_grouped_targets_execution() {
    let output = run_example("syntax/grouped_targets.rb");
    assert_eq!(output, GROUPED_TARGETS_OUTPUT);
}

#[test]
fn test_syntax_grouped_targets_parens_execution() {
    let output = run_example("syntax/grouped_targets_parens.rb");
    assert_eq!(output, GROUPED_TARGETS_OUTPUT);
}

/// The expected output of both `syntax/numbered_and_shorthand` variants, which show
/// a block that names its arguments `_1`, and a call that passes a keyword by name alone and differ only in whether the calls are
/// written with parentheses.
const NUMBERED_AND_SHORTHAND_OUTPUT: &str =
    "[2, 4, 6]\n[[1, 10], [2, 20]]\n[4, 6, 10]\n[1, 6, 7]\n{first: 4, second: 6}\n";

#[test]
fn test_syntax_numbered_and_shorthand_execution() {
    let output = run_example("syntax/numbered_and_shorthand.rb");
    assert_eq!(output, NUMBERED_AND_SHORTHAND_OUTPUT);
}

#[test]
fn test_syntax_numbered_and_shorthand_no_parens_execution() {
    let output = run_example("syntax/numbered_and_shorthand_no_parens.rb");
    assert_eq!(output, NUMBERED_AND_SHORTHAND_OUTPUT);
}

/// The expected output of both `syntax/do_block_binding` variants, which
/// differ only in whether the calls are written with parentheses.
const DO_BLOCK_BINDING_OUTPUT: &str =
    concat!("\"outer got the block\"\n", "\"outer got the block\"\n",);

#[test]
fn test_syntax_do_block_binding_execution() {
    let output = run_example("syntax/do_block_binding.rb");
    assert_eq!(output, DO_BLOCK_BINDING_OUTPUT);
}

#[test]
fn test_syntax_do_block_binding_no_parens_execution() {
    let output = run_example("syntax/do_block_binding_no_parens.rb");
    assert_eq!(output, DO_BLOCK_BINDING_OUTPUT);
}

/// The expected output of both `syntax/operator_binding` variants, which
/// differ only in whether the calls are written with parentheses.
const OPERATOR_BINDING_OUTPUT: &str = concat!(
    "1..10\n1...10\n",
    "a range takes no range\n",
    "a relation takes no relation\n",
    "a comparison takes no comparison\n",
    "10\n[4]\n"
);

#[test]
fn test_syntax_operator_binding_execution() {
    let output = run_example("syntax/operator_binding.rb");
    assert_eq!(output, OPERATOR_BINDING_OUTPUT);
}

#[test]
fn test_syntax_operator_binding_parens_execution() {
    let output = run_example("syntax/operator_binding_parens.rb");
    assert_eq!(output, OPERATOR_BINDING_OUTPUT);
}
