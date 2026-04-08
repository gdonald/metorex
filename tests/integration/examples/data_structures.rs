use super::run_example;

#[test]
fn test_data_structures_simple_dict_execution() {
    let output = run_example("data_structures/simple_dict.rb");
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
    let output = run_example("data_structures/dict_access.rb");
    assert_eq!(output, "Ada lives in London\n");
}

#[test]
fn test_data_structures_hash_methods_execution() {
    let output = run_example("data_structures/hash_methods.rb");
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
fn test_multiple_assignment_execution() {
    let expected = "1\n2\n3\n10\n20\nnil\n42\nnil\nnil\n100\n200\n7\n8\nnil\n";
    let output = run_example("data_structures/multiple_assignment.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_multiple_assignment_parens_execution() {
    let expected = "1\n2\n3\n10\n20\nnil\n42\nnil\nnil\n100\n200\n7\n8\nnil\n";
    let output = run_example("data_structures/multiple_assignment_parens.rb");
    assert_eq!(output, expected);
}
