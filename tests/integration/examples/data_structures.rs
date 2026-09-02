use super::run_example;

#[test]
fn test_data_structures_simple_dict_execution() {
    let output = run_example("data_structures/simple_dict.rb");
    let valid_output1 = "{\"bob\" => 25, \"alice\" => 30}\n30\n";
    let valid_output2 = "{\"alice\" => 30, \"bob\" => 25}\n30\n";
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

#[test]
fn test_hash_primitive_keys_execution() {
    let output = run_example("data_structures/hash_primitive_keys.rb");
    assert_eq!(output, "7\n7\nn\nt\nf\ns\ni\nfl\nst\n");
}

#[test]
fn test_hash_primitive_keys_no_parens_execution() {
    let output = run_example("data_structures/hash_primitive_keys_no_parens.rb");
    assert_eq!(output, "7\n7\nn\nt\nf\ns\ni\nfl\nst\n");
}

#[test]
fn test_hash_non_primitive_keys_execution() {
    let output = run_example("data_structures/hash_non_primitive_keys.rb");
    assert_eq!(output, "first\nsecond\n");
}

#[test]
fn test_data_structures_hash_ordering() {
    let expected = concat!(
        "[\"name\", \"servings\", \"vegetarian\"]\n",
        "[\"stew\", 4, false]\n",
        "[\"name\", \"servings\", \"vegetarian\", \"rating\"]\n",
        "[\"name\", \"servings\", \"vegetarian\", \"rating\"]\n",
        "[\"name\", \"vegetarian\", \"rating\"]\n",
        "[\"name\", \"vegetarian\", \"rating\"]\n",
        "3\n[:first, :second]\n1\n[\"x\", \"y\"]\n[:alpha, :beta]\n"
    );
    let output = run_example("data_structures/hash_ordering.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_data_structures_hash_ordering_no_parens() {
    let expected = concat!(
        "[\"name\", \"servings\", \"vegetarian\"]\n",
        "[\"stew\", 4, false]\n",
        "[\"name\", \"servings\", \"vegetarian\", \"rating\"]\n",
        "[\"name\", \"servings\", \"vegetarian\", \"rating\"]\n",
        "[\"name\", \"vegetarian\", \"rating\"]\n",
        "[\"name\", \"vegetarian\", \"rating\"]\n",
        "3\n[:first, :second]\n1\n[\"x\", \"y\"]\n[:alpha, :beta]\n"
    );
    let output = run_example("data_structures/hash_ordering_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_data_structures_array_splat_literals_execution() {
    let expected = concat!(
        "[:one]\n",
        "[:one, :two, :three]\n",
        "[1, 2, 3, 4]\n",
        "[2, 3]\n",
        "[]\n",
        "[0, \"solo\"]\n",
        "[[1, 2], [3]]\n"
    );
    let output = run_example("data_structures/array_splat/literals.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_data_structures_array_splat_literals_no_parens_execution() {
    let expected = concat!(
        "[:one]\n",
        "[:one, :two, :three]\n",
        "[1, 2, 3, 4]\n",
        "[2, 3]\n",
        "[]\n",
        "[0, \"solo\"]\n",
        "[[1, 2], [3]]\n"
    );
    let output = run_example("data_structures/array_splat/literals_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_data_structures_array_take_drop_execution() {
    let expected = concat!(
        "[1, 2]\n",
        "[3, 4]\n",
        "[]\n",
        "[]\n",
        "[1, 2, 3, 4]\n",
        "attempt to take negative size\n",
        "attempt to drop negative size\n",
    );
    let output = run_example("data_structures/array_take_drop.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_data_structures_array_take_drop_parens_execution() {
    let expected = concat!(
        "[1, 2]\n",
        "[3, 4]\n",
        "[]\n",
        "[]\n",
        "[1, 2, 3, 4]\n",
        "attempt to take negative size\n",
        "attempt to drop negative size\n",
    );
    let output = run_example("data_structures/array_take_drop_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_data_structures_hash_map_execution() {
    let expected = concat!(
        "[\"apple:2\", \"pear:3\"]\n",
        "[4, 6]\n",
        "[]\n",
        "map requires a block\n",
    );
    let output = run_example("data_structures/hash_map.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_data_structures_hash_map_parens_execution() {
    let expected = concat!(
        "[\"apple:2\", \"pear:3\"]\n",
        "[4, 6]\n",
        "[]\n",
        "map requires a block\n",
    );
    let output = run_example("data_structures/hash_map_parens.rb");
    assert_eq!(output, expected);
}
