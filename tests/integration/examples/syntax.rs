// Examples covering the forms the parser reads

use super::run_example;

/// The expected output of both `syntax/reading_forms` variants, which differ
/// only in whether the calls are written with parentheses.
const READING_FORMS_OUTPUT: &str = "\"matched\"\n\"kept\"\n\"nil\"\n[0, nil, 2]\n{nil => nil}\n3\n[\"a\", \"3\", \"c\"]\n[:a, :b3]\n[\"x\", \"y\"]\n[:x, :y]\n\"a#{1}b\"\n\"a(b\"\n[4, 6]\nnil\n5\nnil\n[2, 3]\n";

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
