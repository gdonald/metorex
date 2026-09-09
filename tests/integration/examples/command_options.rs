// Examples covering GetoptLong

use super::run_example;

/// The expected output of both `command_options/reading_switches` variants,
/// which differ only in whether the calls are written with parentheses.
const READING_SWITCHES_OUTPUT: &str = "true\nfalse\n[[\"--size\", \"10k\"], [\"--verbose\", \"\"], [\"--check\", \"\"]]\ntrue\n[\"a.txt\", \"b.txt\"]\n[\"--size\", \"4k\"]\n[\"--verbose\", \"\"]\n[\"--check\", \"\"]\nnil\n[\"--verbose\", \"\"]\nnil\n[\"-c\"]\nGetoptLong::MissingArgument\n\"option `--size' requires an argument\"\n";

#[test]
fn test_command_options_reading_switches_execution() {
    let output = run_example("command_options/reading_switches.rb");
    assert_eq!(output, READING_SWITCHES_OUTPUT);
}

#[test]
fn test_command_options_reading_switches_no_parens_execution() {
    let output = run_example("command_options/reading_switches_no_parens.rb");
    assert_eq!(output, READING_SWITCHES_OUTPUT);
}
