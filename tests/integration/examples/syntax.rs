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
