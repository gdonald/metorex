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

/// The expected output of both `regexp/built_patterns` variants, which differ
/// only in whether the calls are written with parentheses.
const BUILT_PATTERNS_OUTPUT: &str = concat!(
    "1\n",
    "7\n",
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:UTF-8>\n",
    "unknown regexp option: e\n",
    "premature end of char-class: /^[$/\n",
    "no implicit conversion of Symbol into String\n",
    "true\n",
    "false\n"
);

#[test]
fn test_regexp_built_patterns_execution() {
    let output = run_example("regexp/built_patterns.rb");
    assert_eq!(output, BUILT_PATTERNS_OUTPUT);
}

#[test]
fn test_regexp_built_patterns_parens_execution() {
    let output = run_example("regexp/built_patterns_parens.rb");
    assert_eq!(output, BUILT_PATTERNS_OUTPUT);
}

/// The expected output of both `regexp/pattern_soundness` variants, which
/// differ only in whether the calls are written with parentheses.
const PATTERN_SOUNDNESS_OUTPUT: &str = concat!(
    "end pattern with unmatched parenthesis: /(hay(st)ack/\n",
    "unmatched close parenthesis: /hay)stack/\n",
    "invalid group name <1a>: /(?<1a>a)/\n",
    "invalid group name <-a>: /(?<-a>a)/\n",
    "unknown regexp option: n\n",
    "\"Hi\"\n"
);

#[test]
fn test_regexp_pattern_soundness_execution() {
    let output = run_example("regexp/pattern_soundness.rb");
    assert_eq!(output, PATTERN_SOUNDNESS_OUTPUT);
}

#[test]
fn test_regexp_pattern_soundness_parens_execution() {
    let output = run_example("regexp/pattern_soundness_parens.rb");
    assert_eq!(output, PATTERN_SOUNDNESS_OUTPUT);
}

/// The expected output of both `regexp/backtracking_engine` variants, which
/// differ only in whether the calls are written with parentheses.
const BACKTRACKING_ENGINE_OUTPUT: &str = concat!(
    "\"abab\"\n\"abab\"\n",
    "\"bar\"\n\"bar\"\n",
    "nil\nnil\n\"aaa\"\n",
    "\"right\"\n\"aa\"\n",
    "nil\n0\n",
    "\"🤘🏽\"\n"
);

#[test]
fn test_regexp_backtracking_engine_execution() {
    let output = run_example("regexp/backtracking_engine.rb");
    assert_eq!(output, BACKTRACKING_ENGINE_OUTPUT);
}

#[test]
fn test_regexp_backtracking_engine_no_parens_execution() {
    let output = run_example("regexp/backtracking_engine_no_parens.rb");
    assert_eq!(output, BACKTRACKING_ENGINE_OUTPUT);
}

/// The expected output of both `regexp/pattern_time_limits` variants, which differ only in whether the
/// calls are written with parentheses.
const PATTERN_TIME_LIMITS_OUTPUT: &str = concat!(
    "nil\n",
    "0.5\n",
    "\"(a*)*b\"\n",
    "nil\n",
    "\"invalid timeout: 0\"\n",
    "\"invalid timeout: -1\"\n",
    "5.0\n",
    "nil\n",
    "\"aaa\"\n",
    "nil\n",
    "RegexpError\n",
);

#[test]
fn test_regexp_pattern_time_limits_execution() {
    let output = run_example("regexp/pattern_time_limits.rb");
    assert_eq!(output, PATTERN_TIME_LIMITS_OUTPUT);
}

#[test]
fn test_regexp_pattern_time_limits_parens_execution() {
    let output = run_example("regexp/pattern_time_limits_parens.rb");
    assert_eq!(output, PATTERN_TIME_LIMITS_OUTPUT);
}

/// The expected output of both `regexp/match_dispatch` variants, which differ
/// only in whether the calls are written with parentheses.
const MATCH_DISPATCH_OUTPUT: &str = concat!(
    "type mismatch: String given\n",
    "matched w00t\n",
    "ll\n",
    "asked hello\n",
    "asked hello\n",
);

#[test]
fn test_regexp_match_dispatch_execution() {
    let output = run_example("regexp/match_dispatch.rb");
    assert_eq!(output, MATCH_DISPATCH_OUTPUT);
}

#[test]
fn test_regexp_match_dispatch_no_parens_execution() {
    let output = run_example("regexp/match_dispatch_no_parens.rb");
    assert_eq!(output, MATCH_DISPATCH_OUTPUT);
}

/// The expected output of both `regexp/percent_r_and_conditions` variants,
/// which differ only in whether the calls are written with parentheses.
const PERCENT_R_AND_CONDITIONS_OUTPUT: &str = concat!(
    " () [c]{1} \n",
    "(?-mix:\\/)\n",
    "SyntaxError\n",
    "SyntaxError\n",
    "SyntaxError\n",
    "[\"foo1barfoo2\", \"foo2\"]\n",
    "true\n",
    "nil\n",
);

#[test]
fn test_regexp_percent_r_and_conditions_execution() {
    let output = run_example("regexp/percent_r_and_conditions.rb");
    assert_eq!(output, PERCENT_R_AND_CONDITIONS_OUTPUT);
}

#[test]
fn test_regexp_percent_r_and_conditions_parens_execution() {
    let output = run_example("regexp/percent_r_and_conditions_parens.rb");
    assert_eq!(output, PERCENT_R_AND_CONDITIONS_OUTPUT);
}

/// The expected output of both `regexp/pattern_unions` variants.
const PATTERN_UNIONS_OUTPUT: &str = concat!(
    "/(?!)/\n",
    "/n|\\./\n",
    "/(?-mix:dogs)|(?i-mx:cats)/\n",
    "/(?-mix:dogs)|(?i-mx:cats)/\n",
    "/foo/\n",
    "true\n",
    "/from_regexp/\n",
    "/(?-mix:from_regexp)|bar/\n",
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:ISO-8859-1>\n",
    "#<Encoding:UTF-16LE>\n",
    "#<Encoding:BINARY (ASCII-8BIT)>\n",
    "#<Encoding:UTF-8>\n",
    "[ArgumentError, \"incompatible encodings: UTF-16LE and UTF-16BE\"]\n",
    "[ArgumentError, \"incompatible encodings: ISO-8859-1 and UTF-8\"]\n",
    "[ArgumentError, \"ASCII incompatible encoding: UTF-16LE\"]\n",
    "[TypeError, \"no implicit conversion of Array into String\"]\n"
);

#[test]
fn test_regexp_pattern_unions_execution() {
    let output = run_example("regexp/pattern_unions.rb");
    assert_eq!(output, PATTERN_UNIONS_OUTPUT);
}

#[test]
fn test_regexp_pattern_unions_parens_execution() {
    let output = run_example("regexp/pattern_unions_parens.rb");
    assert_eq!(output, PATTERN_UNIONS_OUTPUT);
}
