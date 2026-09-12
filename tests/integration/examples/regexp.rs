use super::run_example;

/// The expected output of both `regexp/matching` variants, which differ only
/// in whether the calls are written with parentheses.
const MATCHING_OUTPUT: &str = "MatchData\n\"hello there\"\n\"hello\"\n\"there\"\n[\"hello\", \"there\"]\n[\"hello there\", \"hello\", \"there\"]\n\"\"\n\" world\"\n0\n11\n[6, 11]\n3\n\"hello there\"\n\"hello\"\n5\n[\"hello there\", \"there\", nil]\n\"hello there world\"\n\"hello\"\n\"hello\"\n[\"greeting\"]\n{\"greeting\" => \"hello\"}\n[\"hello\"]\n\"#<MatchData \\\"hello there\\\" greeting:\\\"hello\\\">\"\nnil\n\"index 9 out of matches\"\n\"the quick\"\n\"quick\"\n\"the quick\"\n\"\"\n\"quick\"\nnil\nnil\n\"ab+c\"\n1\ntrue\n\"ab\"\n\"cat|dog\"\ntrue\nRegexp\ntrue\ntrue\n[4]\n[\"apple\"]\n[\"grape\"]\n";

#[test]
fn test_regexp_matching_execution() {
    let output = run_example("regexp/matching.rb");
    assert_eq!(output, MATCHING_OUTPUT);
}

#[test]
fn test_regexp_matching_no_parens_execution() {
    let output = run_example("regexp/matching_no_parens.rb");
    assert_eq!(output, MATCHING_OUTPUT);
}

/// The expected output of both `regexp/named_groups` variants, which differ
/// only in whether the calls are written with parentheses.
const NAMED_GROUPS_OUTPUT: &str = "\"113\"\n3\n6\n[\"a\"]\n{\"a\" => \"113\"}\n\"H\"\n1\n[\"æ\", \"b\"]\n[\"a\", \"b\"]\n{\"a\" => [1, 3], \"b\" => [2]}\ntrue\ntrue\nfalse\n/\\[/\n\"[\"\n\"o w\"\n\"w\"\ntrue\ntrue\n\"hi\"\n[104, 105]\n[104, 233]\n(19022/1)\n(-190227/10)\nInfinity\n-Infinity\n50.0\n";

#[test]
fn test_regexp_named_groups_execution() {
    let output = run_example("regexp/named_groups.rb");
    assert_eq!(output, NAMED_GROUPS_OUTPUT);
}

#[test]
fn test_regexp_named_groups_no_parens_execution() {
    let output = run_example("regexp/named_groups_no_parens.rb");
    assert_eq!(output, NAMED_GROUPS_OUTPUT);
}

/// The expected output of both `regexp/pattern_readings` variants.
const PATTERN_READINGS_OUTPUT: &str = "\"\\\\*\\\\?\\\\{\\\\}\\\\.\\\\+\\\\^\\\\$\\\\[\\\\]\\\\(\\\\)\\\\-\\\\ \"\n\"\\\\n\\\\r\\\\f\\\\t\"\n#<Encoding:US-ASCII>\n\"a\\\\.b\"\n\"(?i-mx:nothing)\"\n\"(?i-mx:abc)\"\n\"(?-mix:abc)\"\n\"@\"\n\"\\\\+\"\n#<Encoding:US-ASCII>\n#<Encoding:UTF-8>\n/foo/\nnil\ntrue\nfalse\n[1, 2, 4]\nfalse\ntrue\n:\"foo bar\"\n";

#[test]
fn test_regexp_pattern_readings_execution() {
    let output = run_example("regexp/pattern_readings.rb");
    assert_eq!(output, PATTERN_READINGS_OUTPUT);
}

#[test]
fn test_regexp_pattern_readings_no_parens_execution() {
    let output = run_example("regexp/pattern_readings_no_parens.rb");
    assert_eq!(output, PATTERN_READINGS_OUTPUT);
}

/// The expected output of both `regexp/pattern_options` variants, which differ only in whether
/// the calls are written with parentheses.
const PATTERN_OPTIONS_OUTPUT: &str = "0\n1\n2\n4\n7\n16\n16\n16\n0\n32\n\"uninitialized Regexp\"\n";

#[test]
fn test_regexp_pattern_options_execution() {
    let output = run_example("regexp/pattern_options.rb");
    assert_eq!(output, PATTERN_OPTIONS_OUTPUT);
}

#[test]
fn test_regexp_pattern_options_parens_execution() {
    let output = run_example("regexp/pattern_options_parens.rb");
    assert_eq!(output, PATTERN_OPTIONS_OUTPUT);
}

/// The expected output of both `regexp/end_of_subject` variants, which show
/// where `\z` and `\Z` stand in a subject and differ only in whether the calls are
/// written with parentheses.
const END_OF_SUBJECT_OUTPUT: &str = "2\nnil\n2\n2\n\"file\"\n0\n";

#[test]
fn test_regexp_end_of_subject_execution() {
    let output = run_example("regexp/end_of_subject.rb");
    assert_eq!(output, END_OF_SUBJECT_OUTPUT);
}

#[test]
fn test_regexp_end_of_subject_no_parens_execution() {
    let output = run_example("regexp/end_of_subject_no_parens.rb");
    assert_eq!(output, END_OF_SUBJECT_OUTPUT);
}
