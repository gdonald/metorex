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
