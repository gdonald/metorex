use super::run_example;

/// The expected output of both `array_methods/in_place_and_walks` variants,
/// which differ only in whether the calls are written with parentheses.
const IN_PLACE_AND_WALKS_OUTPUT: &str = "[3, 1, 2]\n[3, 1, 2]\nnil\n[1, 2, 3]\n[3, 2, 1]\n[6, 4, 2]\n[4, 2]\nnil\nnil\n[1, 2]\n[2, 3, 1]\n[3, 1, 2]\n[1, 2, 3, 4]\n[1, 2, [3, [4]]]\n20\n30\nnil\n3\n2\n2\n[1, 2]\n[3, 1]\n[\"2\", \"60\"]\n4\n4\n\"Enumerator\"\n\"Enumerator\"\nfalse\ntrue\n[1, 2]\nStack\n2\ntrue\n[1, 2, 3]\n[7, 8, 9]\n[[...]]\ntrue\n[1, 2, 3, 4]\n";

#[test]
fn test_array_in_place_and_walks_execution() {
    let output = run_example("array_methods/in_place_and_walks.rb");
    assert_eq!(output, IN_PLACE_AND_WALKS_OUTPUT);
}

#[test]
fn test_array_in_place_and_walks_no_parens_execution() {
    let output = run_example("array_methods/in_place_and_walks_no_parens.rb");
    assert_eq!(output, IN_PLACE_AND_WALKS_OUTPUT);
}

/// The expected output of both `array_methods/set_operations_and_lookup`
/// variants, which differ only in whether the calls are written with
/// parentheses.
const SET_OPERATIONS_OUTPUT: &str = "[:a, :b, :c, :d]\n[:b, :c]\n[:a, :c]\n[:a, :b, :c]\n[1, 2, 3]\n[1, 3]\n[1, 2, 3]\n[2, 3]\n[1, 2, 1, 2, 1, 2]\n\"1, 2\"\n-1\n-1\nnil\n[:two, 2]\n[:one, 1]\nnil\n20\n40\n:none\n18\n[10, 30]\n[10, 30, 40]\n[20, 30, 40]\n[10, 20]\n[nil]\nindex 9 outside of array bounds: -4...4\n[10, 20]\n[30, 40]\n10\n40\n[10, 40]\n10\n[3, 4]\n[1]\n[2]\n[:front, 2]\n[:front, 2, :back]\n[[1, 3], [2, 4]]\nelement size differs (1 should be 2)\n[[1, 3, 5], [2, 4, 6]]\n[:x]\nRangeError\n";

#[test]
fn test_array_set_operations_and_lookup_execution() {
    let output = run_example("array_methods/set_operations_and_lookup.rb");
    assert_eq!(output, SET_OPERATIONS_OUTPUT);
}

#[test]
fn test_array_set_operations_and_lookup_no_parens_execution() {
    let output = run_example("array_methods/set_operations_and_lookup_no_parens.rb");
    assert_eq!(output, SET_OPERATIONS_OUTPUT);
}

/// The expected output of both `array_methods/pickings_and_search` variants,
/// which differ only in whether the calls are written with parentheses.
const PICKINGS_AND_SEARCH_OUTPUT: &str = "[[1, 2], [1, 3], [1, 4], [2, 3], [2, 4], [3, 4]]\n[[]]\n4\n[[1, 2], [1, 3], [1, 4], [2, 1], [2, 3], [2, 4], [3, 1], [3, 2], [3, 4], [4, 1], [4, 2], [4, 3]]\n24\n10\n[[10, 10], [10, 11], [11, 11]]\n[[10, 10], [10, 11], [11, 10], [11, 11]]\n8\n[[1, 3, 5], [1, 4, 5], [2, 3, 5], [2, 4, 5]]\n[[1], [2]]\n[]\n6\ntrue\n3\nnil\n2\n3\nnil\nEnumerator\n";

#[test]
fn test_array_methods_pickings_and_search_execution() {
    let output = run_example("array_methods/pickings_and_search.rb");
    assert_eq!(output, PICKINGS_AND_SEARCH_OUTPUT);
}

#[test]
fn test_array_methods_pickings_and_search_no_parens_execution() {
    let output = run_example("array_methods/pickings_and_search_no_parens.rb");
    assert_eq!(output, PICKINGS_AND_SEARCH_OUTPUT);
}
