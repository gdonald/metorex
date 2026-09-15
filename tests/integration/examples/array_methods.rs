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

/// The expected output of both `array_methods/to_h_pairs` variants, which
/// differ only in whether the calls are written with parentheses.
const TO_H_PAIRS_OUTPUT: &str = "{1 => 2, 3 => 4}\n{\"1\" => 4}\n\"wrong array length at 0 (expected 2, was 3)\"\n\"wrong element type String at 0 (expected array)\"\n{key: :value}\n[[1, 1], [2, 2]]\n";

#[test]
fn test_array_methods_to_h_pairs_execution() {
    let output = run_example("array_methods/to_h_pairs.rb");
    assert_eq!(output, TO_H_PAIRS_OUTPUT);
}

#[test]
fn test_array_methods_to_h_pairs_no_parens_execution() {
    let output = run_example("array_methods/to_h_pairs_no_parens.rb");
    assert_eq!(output, TO_H_PAIRS_OUTPUT);
}

/// The expected output of both `array_methods/fill_and_hash` variants, which
/// differ only in whether the calls are written with parentheses.
const FILL_AND_HASH_OUTPUT: &str = "[:x, :x, :x, :x]\n[1, 2, :x, :x]\n[1, :x, :x, 4]\n[1, :x, :x, 4]\n[0, 2, 4, 6]\n[1, 2, 20, 30]\n\"no implicit conversion of String into Integer\"\ntrue\ntrue\nfalse\n";

#[test]
fn test_array_methods_fill_and_hash_execution() {
    let output = run_example("array_methods/fill_and_hash.rb");
    assert_eq!(output, FILL_AND_HASH_OUTPUT);
}

#[test]
fn test_array_methods_fill_and_hash_no_parens_execution() {
    let output = run_example("array_methods/fill_and_hash_no_parens.rb");
    assert_eq!(output, FILL_AND_HASH_OUTPUT);
}

/// The expected output of both `array_methods/conversion_and_draws` variants,
/// which differ only in whether the calls are written with parentheses.
const CONVERSION_AND_DRAWS_OUTPUT: &str = concat!(
    "[1, 2]\n",
    "nil\n",
    "{a: 1}\n",
    "nil\n",
    "[3, 4]\n",
    "\"can't convert Object into Hash (Object#to_hash gives Symbol)\"\n",
    "4\n",
    "[4, 1]\n",
    "[1, 2, 3]\n",
    "nil\n",
    "\"negative sample number\"\n",
    "RangeError\n"
);

#[test]
fn test_array_methods_conversion_and_draws_execution() {
    let output = run_example("array_methods/conversion_and_draws.rb");
    assert_eq!(output, CONVERSION_AND_DRAWS_OUTPUT);
}

#[test]
fn test_array_methods_conversion_and_draws_no_parens_execution() {
    let output = run_example("array_methods/conversion_and_draws_no_parens.rb");
    assert_eq!(output, CONVERSION_AND_DRAWS_OUTPUT);
}

/// The expected output of both `array_methods/splat_coercion` variants.
const SPLAT_COERCION_OUTPUT: &str = concat!(
    "[1, 2, 3, 4]\n",
    "Array\n",
    "[1, 2, 3, 4]\n",
    "[\"a\", \"x\", \"e\"]\n",
    "[]\n",
    "[1, 2]\n",
    "[\"a \", \"b\\tc\"]\n",
    "[\"a\", \"b c\"]\n"
);

#[test]
fn test_array_methods_splat_coercion_execution() {
    let output = run_example("array_methods/splat_coercion.rb");
    assert_eq!(output, SPLAT_COERCION_OUTPUT);
}

#[test]
fn test_array_methods_splat_coercion_no_parens_execution() {
    let output = run_example("array_methods/splat_coercion_no_parens.rb");
    assert_eq!(output, SPLAT_COERCION_OUTPUT);
}

/// The expected output of both `array_methods/joining_and_inspecting`
/// variants, which differ only in whether the calls are written with
/// parentheses.
const JOINING_AND_INSPECTING_OUTPUT: &str = concat!(
    "\"123\"\n",
    "\"1_2_3\"\n",
    "\"1_2_3\"\n",
    "\"spelled\"\n",
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:EUC-JP>\n",
    "true\n",
    "\"[answered]\"\n",
    "{a: 1, a!: 1, a?: 1}\n",
    "{\"needs-quotes\": 1}\n",
    "{\"\": 1}\n",
);

#[test]
fn test_array_methods_joining_and_inspecting_execution() {
    let output = run_example("array_methods/joining_and_inspecting.rb");
    assert_eq!(output, JOINING_AND_INSPECTING_OUTPUT);
}

#[test]
fn test_array_methods_joining_and_inspecting_no_parens_execution() {
    let output = run_example("array_methods/joining_and_inspecting_no_parens.rb");
    assert_eq!(output, JOINING_AND_INSPECTING_OUTPUT);
}

/// The expected output of both `array_methods/strided_slices` variants, which
/// differ only in whether the calls are written with parentheses.
const STRIDED_SLICES_OUTPUT: &str = concat!(
    "[0, 2, 4]\n",
    "[1, 3, 5]\n",
    "[0, 3]\n",
    "[0, 2, 4]\n",
    "[5, 3, 1]\n",
    "[5, 3, 1]\n",
    "[]\n",
    "((0..6).step(2)) out of range\n"
);

#[test]
fn test_array_methods_strided_slices_execution() {
    let output = run_example("array_methods/strided_slices.rb");
    assert_eq!(output, STRIDED_SLICES_OUTPUT);
}

#[test]
fn test_array_methods_strided_slices_parens_execution() {
    let output = run_example("array_methods/strided_slices_parens.rb");
    assert_eq!(output, STRIDED_SLICES_OUTPUT);
}
