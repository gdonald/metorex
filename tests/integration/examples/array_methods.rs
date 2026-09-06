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
