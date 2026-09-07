use super::run_example;

/// The expected output of both `enumerable/walking_a_collection` variants,
/// which differ only in whether the calls are written with parentheses.
const WALKING_A_COLLECTION_OUTPUT: &str = "[4, 1, 3, 2]\n[8, 2, 6, 4]\n[1, 3]\n[4, 2]\n[[4, 3], [1, 2]]\n{false => [4, 2], true => [1, 3]}\n[1, 2, 3, 4]\n[4, 3, 2, 1]\n1\n4\n[1, 2]\n[4, 3]\n4\n1\n[1, 4]\ntrue\n[4, 1]\n4\n[[4, 0], [1, 1], [3, 2], [2, 3]]\n[2, 3, 1, 4]\n10\n4\n4\n\"none\"\n4\n[[1, 2], [3, 4]]\n[[1, 2], [3, 4]]\n[[3, 4]]\n[5, 6, 7]\n[:hello, \"world\"]\n2\n";

#[test]
fn test_enumerable_walking_a_collection_execution() {
    let output = run_example("enumerable/walking_a_collection.rb");
    assert_eq!(output, WALKING_A_COLLECTION_OUTPUT);
}

#[test]
fn test_enumerable_walking_a_collection_no_parens_execution() {
    let output = run_example("enumerable/walking_a_collection_no_parens.rb");
    assert_eq!(output, WALKING_A_COLLECTION_OUTPUT);
}

/// The expected output of both `enumerable/predicates_and_slices` variants,
/// which differ only in whether the calls are written with parentheses.
const PREDICATES_AND_SLICES_OUTPUT: &str = "true\ntrue\ntrue\ntrue\ntrue\nfalse\ntrue\n1\n3\n3\ntrue\ntrue\n[[1, 2], [3, 4], [5]]\n[[1, 2], [2, 3], [3, 4], [4, 5]]\n3\n4\n1\n[1, 2]\n[1, 2]\n[3, 4, 5]\n[3, 4, 5]\n[1, 2]\n\"attempt to take negative size\"\n\"no implicit conversion into Integer\"\n{\"a\" => 2, \"b\" => 1}\n{\"a\" => 3, \"b\" => 1}\n[[1, 10], [2, 20], [3, 30], [4, 40], [5, 50]]\n[1, -1, 2, -2, 3, -3, 4, -4, 5, -5]\n15\n50.0\n:stopped\n[:unnamed, []]\n[:given, [1, 2]]\n";

#[test]
fn test_enumerable_predicates_and_slices_execution() {
    let output = run_example("enumerable/predicates_and_slices.rb");
    assert_eq!(output, PREDICATES_AND_SLICES_OUTPUT);
}

#[test]
fn test_enumerable_predicates_and_slices_no_parens_execution() {
    let output = run_example("enumerable/predicates_and_slices_no_parens.rb");
    assert_eq!(output, PREDICATES_AND_SLICES_OUTPUT);
}

/// The expected output of both `enumerable/folding_and_matching` variants,
/// which differ only in whether the calls are written with parentheses.
const FOLDING_AND_MATCHING_OUTPUT: &str = "10\n20\n10\n10\n0\n24\n10\n0\n0\ntrue\n\"wrong number of arguments (given 0, expected 1)\"\n[1, 2, 3, 11, 12, 13]\n[\"banana\"]\n[\"apple\", \"cherry\"]\n[\"CHERRY\"]\n[1, 4]\n[1, 3]\n[2, 4]\n";

#[test]
fn test_enumerable_folding_and_matching_execution() {
    let output = run_example("enumerable/folding_and_matching.rb");
    assert_eq!(output, FOLDING_AND_MATCHING_OUTPUT);
}

#[test]
fn test_enumerable_folding_and_matching_no_parens_execution() {
    let output = run_example("enumerable/folding_and_matching_no_parens.rb");
    assert_eq!(output, FOLDING_AND_MATCHING_OUTPUT);
}

/// The expected output of both `enumerable/lazy_walks` variants, which differ
/// only in whether the calls are written with parentheses.
const LAZY_WALKS_OUTPUT: &str = "[2, 4, 6, 8]\n[2, 4, 6]\n[1, 3, 5]\n[3, 9, 15]\n[1, 2, 3, 4]\n[1, 2, 3, 4]\n[4, 5, 6]\n[4, 5, 6]\n[[1, 10], [2, 11], [3, 12]]\n[[1, :a], [2, :b], [3, :c]]\n[1, 2]\n[1, 2, 3]\n[2, 3]\n[1, 4]\n[1, 2, 3]\n[[1, 2], [4, 5]]\n[[1, 2], [4, 5]]\n[[1, 2], [3, 4]]\n[[1, 2], [3, 4]]\n10\n10\n4\n6\nnil\nEnumerator\ntrue\n[1, 2, 3, 4, 5]\n5\n\"#<Enumerator::Chain: [[1, 2], [3], 4..5]>\"\n[1, 2]\n";

#[test]
fn test_enumerable_lazy_walks_execution() {
    let output = run_example("enumerable/lazy_walks.rb");
    assert_eq!(output, LAZY_WALKS_OUTPUT);
}

#[test]
fn test_enumerable_lazy_walks_no_parens_execution() {
    let output = run_example("enumerable/lazy_walks_no_parens.rb");
    assert_eq!(output, LAZY_WALKS_OUTPUT);
}
