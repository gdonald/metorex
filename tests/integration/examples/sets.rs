use super::run_example;

/// The expected output of both `sets/set_algebra` variants, which differ only
/// in whether the calls are written with parentheses.
const SET_ALGEBRA_OUTPUT: &str = "[3, 1, 2]\n3\ntrue\n\"Set[3, 1, 2]\"\n[:a, \"b\", 3, [4, 5], nil]\ntrue\n[3, 1, 2, 4]\nnil\n[3, 1, 2, 4, 5, 6]\n[3, 1, 2, 4]\n[1, 2, 3, 4]\n[1, 2, 3, 4]\n[1, 2]\n[3]\n[4, 1, 2]\n[1, 2, 3, 4]\n[1, 2]\n[2, 3]\ntrue\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\n[2, 4, 6]\n[1, 3]\n[1, 2, 3]\n[1, 2, 3]\n\"1-2-3\"\n[2, 4]\n[1, 3]\n[11, 12, 13]\n[7, 8]\n[]\ntrue\n";

#[test]
fn test_sets_set_algebra_execution() {
    let output = run_example("sets/set_algebra.rb");
    assert_eq!(output, SET_ALGEBRA_OUTPUT);
}

#[test]
fn test_sets_set_algebra_no_parens_execution() {
    let output = run_example("sets/set_algebra_no_parens.rb");
    assert_eq!(output, SET_ALGEBRA_OUTPUT);
}

/// The expected output of both `sets/membership_and_allocation` variants,
/// which differ only in whether the calls are written with parentheses.
const MEMBERSHIP_OUTPUT: &str = "true\ntrue\ntrue\n2\ntrue\nfalse\ntrue\ntrue\n[:a, :b]\n\"abc\"\n\"a-b\"\n\"xy\"\n[]\n{}\n\"allocator undefined for Proc\"\n\"undefined method 'allocate' for class 'MatchData'\"\ntrue\ntrue\ntrue\n{c: -1, d: -2}\n{a: 101, b: 102, c: 3}\n16\nnil\n\"can't iterate from Float\"\n";

#[test]
fn test_sets_membership_and_allocation_execution() {
    let output = run_example("sets/membership_and_allocation.rb");
    assert_eq!(output, MEMBERSHIP_OUTPUT);
}

#[test]
fn test_sets_membership_and_allocation_no_parens_execution() {
    let output = run_example("sets/membership_and_allocation_no_parens.rb");
    assert_eq!(output, MEMBERSHIP_OUTPUT);
}

/// The expected output of both `sets/grouping_and_flattening` variants, which
/// differ only in whether the calls are written with parentheses.
const GROUPING_AND_FLATTENING_OUTPUT: &str = "{3 => Set[\"one\", \"two\"], 5 => Set[\"three\"], 4 => Set[\"four\", \"five\"]}\n[[\"five\", \"four\"], [\"one\", \"two\"], [\"three\"]]\n[[1], [3, 4], [6]]\nEnumerator\n[1, 2, 3, 4, 5, 6]\nfalse\n[1, 2, 3]\nnil\ntrue\n1\n1\n[1, 2]\n";

#[test]
fn test_sets_grouping_and_flattening_execution() {
    let output = run_example("sets/grouping_and_flattening.rb");
    assert_eq!(output, GROUPING_AND_FLATTENING_OUTPUT);
}

#[test]
fn test_sets_grouping_and_flattening_no_parens_execution() {
    let output = run_example("sets/grouping_and_flattening_no_parens.rb");
    assert_eq!(output, GROUPING_AND_FLATTENING_OUTPUT);
}
