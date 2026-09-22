// Arrays and hashes written out as literals.

use super::super::run_example;
use super::*;
#[test]
fn test_basics_array_delete_execution() {
    let expected = "2\n3\ntrue\n1\n2\nfalse\n";
    let output = run_example("basics/array_delete.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_each_block_execution() {
    let expected = "Range iteration:\n1\n2\n3\nArray iteration:\n10\n20\n30\n";
    let output = run_example("basics/each_block.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_hash_and_array_constructors_execution() {
    let output = run_example("basics/hash_and_array_constructors.rb");
    assert_eq!(output, HASH_AND_ARRAY_CONSTRUCTORS_OUTPUT);
}

#[test]
fn test_basics_hash_and_array_constructors_no_parens_execution() {
    let output = run_example("basics/hash_and_array_constructors_no_parens.rb");
    assert_eq!(output, HASH_AND_ARRAY_CONSTRUCTORS_OUTPUT);
}
