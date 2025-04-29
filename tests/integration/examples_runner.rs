// Examples runner

use std::process::Command;

fn run_example(path: &str) -> String {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut cmd = Command::new(binary);
    cmd.current_dir(manifest_dir).arg(path);

    let output = cmd.output().expect("failed to execute example");
    assert!(
        output.status.success(),
        "example {} exited with status {:?}",
        path,
        output.status
    );

    String::from_utf8(output.stdout).expect("stdout was not utf8")
}

#[test]
fn test_basics_greeting_line_execution() {
    let output = run_example("examples/basics/greeting_line.mx");
    assert_eq!(output, "Hello, Ada!\n");
}

#[test]
fn test_basics_string_methods_execution() {
    let expected = r#"=== Basic String Methods ===
ALICE
alice
Hello, World!
xeroteM
11

=== String Inspection Methods ===
H
i
65
66
"#;

    let output = run_example("examples/basics/string_methods.mx");
    assert_eq!(output, expected.to_string());
}

#[test]
fn test_data_structures_simple_dict_execution() {
    let output = run_example("examples/data-structures/simple_dict.mx");
    // Hash map iteration order is non-deterministic, so check both possible orders
    let valid_output1 = "{bob: 25, alice: 30}\n30\n";
    let valid_output2 = "{alice: 30, bob: 25}\n30\n";
    assert!(
        output == valid_output1 || output == valid_output2,
        "Expected either '{}' or '{}', but got '{}'",
        valid_output1,
        valid_output2,
        output
    );
}

#[test]
fn test_data_structures_dict_access_execution() {
    let output = run_example("examples/data-structures/dict_access.mx");
    assert_eq!(output, "Ada lives in London\n");
}

#[test]
fn test_data_structures_hash_methods_execution() {
    let output = run_example("examples/data-structures/hash_methods.mx");
    // Hash map iteration order is non-deterministic, so check for valid orderings
    let fixed_part = "Has alice?\ntrue\nHas dave?\nfalse\nSize:\n3\n";
    assert!(
        output.contains(fixed_part)
            && output.contains("alice")
            && output.contains("bob")
            && output.contains("charlie")
            && output.contains("30")
            && output.contains("25")
            && output.contains("35"),
        "Expected output to contain all keys, values, and fixed text, but got: {}",
        output
    );
}

#[test]
fn test_type_annotations_collection_types_execution() {
    let output = run_example("examples/type-annotations/collection_types.mx");
    // Hash map iteration order is non-deterministic, so check both possible orders
    let valid_output1 = "numbers = [1, 2, 3, 4, 5]\nscores = {Bob: 85, Alice: 90}\nlength of numbers: 5\nAlice's score: 90\n";
    let valid_output2 = "numbers = [1, 2, 3, 4, 5]\nscores = {Alice: 90, Bob: 85}\nlength of numbers: 5\nAlice's score: 90\n";
    assert!(
        output == valid_output1 || output == valid_output2,
        "Expected either '{}' or '{}', but got '{}'",
        valid_output1,
        valid_output2,
        output
    );
}

#[test]
fn test_basics_simple_range_execution() {
    let expected = "1..5\n1...5\n[1, 2, 3, 4, 5]\n[1, 2, 3, 4]\n";
    let output = run_example("examples/basics/simple_range.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_each_block_execution() {
    let expected = "Range iteration:\n1\n2\n3\nArray iteration:\n10\n20\n30\n";
    let output = run_example("examples/basics/each_block.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_factorial_iterative_execution() {
    let expected = "720\n";
    let output = run_example("examples/algorithms/factorial_iterative.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_average_temperature_execution() {
    let expected = "69.9\n";
    let output = run_example("examples/algorithms/average_temperature.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_primes_under_fifty_execution() {
    let expected = "[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]\n";
    let output = run_example("examples/algorithms/primes_under_fifty.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_functions_closures_nested_execution() {
    let expected = "10\n12\n";
    let output = run_example("examples/functions/closures_nested.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_functions_nonlocal_counter_execution() {
    let expected = "1\n2\n3\n3\n0\n1\n";
    let output = run_example("examples/functions/nonlocal_counter.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_functions_locals_scope_execution() {
    let expected = "20\n[0, 2, 4, 6, 8]\n";
    let output = run_example("examples/functions/locals_scope.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_parser_lambdas_execution() {
    let expected = "10\n10\n42\n30\n23\n13\n18\n11\n14\n21\n24\n10\n";
    let output = run_example("examples/parser/lambdas.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_blocks_as_objects_execution() {
    let expected = r#"=== Blocks as First-Class Objects ===

1. Assigning blocks to variables:
double.call(5) = 10

2. Multiple parameter blocks:
add.call(3, 7) = 10

3. Zero parameter blocks:
get_pi.call = 3.14159

4. Passing blocks as arguments to functions:
apply_twice(increment, 5) = 7

5. Returning blocks from functions (closures):
times_three.call(4) = 12
times_ten.call(4) = 40

6. Blocks capturing variables from outer scope:
First call: 1
Second call: 2
Third call: 3

7. Storing blocks in data structures:
Starting with value: 20
After +1: 21
After *2: 40
After -3: 17
After /4: 5

8. Higher-order function example:
Squares of [1,2,3,4,5]: [1, 4, 9, 16, 25]

9. Blocks returned from class methods:
add_op.call(5, 3) = 8
mul_op.call(5, 3) = 15

10. Nested closures:
Nested closure result: 6

11. Function composition:
compose(add_ten, double).call(5) = 20

12. Partial application pattern:
Hello, Alice!
Goodbye, Bob!

13. Callback pattern:
Success! Data: 42
Error! Invalid data: -1

14. Closure with captured state:
counter.call = 5
counter.call = 10
counter.call = 15

15. Method chaining and lambda returns:
Query with 2 conditions

=== Blocks are truly first-class objects! ===
"#;

    let output = run_example("examples/metaprogramming/blocks_as_objects.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_filter_even_numbers_execution() {
    let expected = "[2, 4, 6]\n";
    let output = run_example("examples/algorithms/filter_even_numbers.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_character_counter_execution() {
    let output = run_example("examples/algorithms/character_counter.mx");
    // Hash map iteration order is non-deterministic, so check for all expected keys and values
    assert!(
        output.contains("b")
            && output.contains("a")
            && output.contains("n")
            && output.contains(": 1")
            && output.contains(": 3")
            && output.contains(": 2"),
        "Expected output to contain all characters (b:1, a:3, n:2), but got: {}",
        output
    );
}

#[test]
fn test_algorithms_zip_merger_execution() {
    let expected = "[[Ann, 88], [Ben, 93]]\n";
    let output = run_example("examples/algorithms/zip_merger.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_matrix_transpose_execution() {
    let expected = "[[1, 4], [2, 5], [3, 6]]\n";
    let output = run_example("examples/algorithms/matrix_transpose.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_matrix_transpose_comprehensive_execution() {
    let expected = r#"Basic 2x3 matrix:
[[1, 4], [2, 5], [3, 6]]
Double transpose (3x2 matrix):
[[1, 2], [3, 4], [5, 6]]
Single row matrix:
[[1], [2], [3], [4]]
Single column matrix:
[[1, 2, 3]]
Square 3x3 matrix:
[[1, 4, 7], [2, 5, 8], [3, 6, 9]]
"#;
    let output = run_example("examples/algorithms/matrix_transpose_comprehensive.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_matrix_nested_ops_execution() {
    let expected = r#"Original matrix:
[[1, 2, 3], [4, 5, 6]]
Element at [0][0]:
1
Element at [1][2]:
6
Doubled matrix:
[[2, 4, 6], [8, 10, 12]]
Sum of each column:
[5, 7, 9]
Rows where first element > 2:
[[3, 4], [5, 6]]
"#;
    let output = run_example("examples/algorithms/matrix_nested_ops.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_basic_execution() {
    let expected = "Buddy\nGolden Retriever\nSome sound -> Woof!\nI am an animal named Buddy\n";
    let output = run_example("examples/oop/super_basic.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_chain_basic_execution() {
    let expected = "GrandParent\nParent\nChild\n";
    let output = run_example("examples/oop/super_chain_basic.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_simple_execution() {
    let expected = "AB\n";
    let output = run_example("examples/oop/test_super_simple.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_reader_execution() {
    let expected = "Alice\n30\n";
    let output = run_example("examples/oop/attr_reader.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_writer_execution() {
    let expected = "Unknown\n0\nBob\n25\n";
    let output = run_example("examples/oop/attr_writer.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_accessor_execution() {
    let expected = "Charlie\n35\ncharlie@example.com\nCharles\n36\ncharles@example.com\n";
    let output = run_example("examples/oop/attr_accessor.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_str_execution() {
    let expected = "Person: Alice\n";
    let output = run_example("examples/oop/test_str.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_repr_execution() {
    let expected = "Point(0, 0)\n";
    let output = run_example("examples/oop/test_repr.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_special_methods_execution() {
    let expected = "Book: Ruby Guide\nMagazine: Tech Monthly\nnext_value\n";
    let output = run_example("examples/oop/special_methods.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_iter_execution() {
    let expected = "next\n";
    let output = run_example("examples/oop/test_iter.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_method_missing_execution() {
    let expected = "bar\n42\n1\n2\n3\n";
    let output = run_example("examples/oop/test_method_missing.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_array_execution() {
    let expected = "1\n2\n3\n";
    let output = run_example("examples/basics/for_loop_array.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_range_execution() {
    let expected = "1\n2\n3\n4\n5\n";
    let output = run_example("examples/basics/for_loop_range.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_break_execution() {
    let expected = "1\n2\n3\n4\n";
    let output = run_example("examples/basics/for_loop_break.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_continue_execution() {
    let expected = "1\n2\n4\n5\n";
    let output = run_example("examples/basics/for_loop_continue.mx");
    assert_eq!(output, expected);
}
