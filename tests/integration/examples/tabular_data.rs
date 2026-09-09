// Examples covering CSV

use super::run_example;

/// The expected output of both `tabular_data/comma_separated` variants, which differ only
/// in whether the calls are written with parentheses.
const COMMA_SEPARATED_OUTPUT: &str = "[[\"name\", \"age\"], [\"ruby\", \"30\"]]\n[[\"foo\", nil, \"baz\"]]\n[]\n[[], [], [\"bar\"]]\n[[\"foo\", \"bar\"]]\n[[\"Johnson, Dwayne\", \"actor\"]]\n[\"a\", \"b\", \"c\"]\n\"foo,bar\\n\"\n\"\\n\"\n\"foo,,bar\\n\"\n\"\\\"a,b\\\",\\\"say \\\"\\\"hi\\\"\\\"\\\"\\n\"\n\"foo;bar\\n\"\n\"Any value after quoted field isn't allowed in line 1.\"\n[[\"Johnson, Dwayne\", \"Dwayne \\\"The Rock\\\" Johnson\"]]\nfalse\n\"a,1\\nb,2\\n\"\n[[\"a\", \"1\"], [\"b\", \"2\"]]\n\",\"\n[[\"x\", \"1\"], [\"y\", \"2\"]]\n";

#[test]
fn test_tabular_data_comma_separated_execution() {
    let output = run_example("tabular_data/comma_separated.rb");
    assert_eq!(output, COMMA_SEPARATED_OUTPUT);
}

#[test]
fn test_tabular_data_comma_separated_no_parens_execution() {
    let output = run_example("tabular_data/comma_separated_no_parens.rb");
    assert_eq!(output, COMMA_SEPARATED_OUTPUT);
}
