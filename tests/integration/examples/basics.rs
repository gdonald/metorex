use super::run_example;

#[test]
fn test_basics_greeting_line_execution() {
    let output = run_example("basics/greeting_line.rb");
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

    let output = run_example("basics/string_methods.rb");
    assert_eq!(output, expected.to_string());
}

#[test]
fn test_basics_simple_range_execution() {
    let expected = "1..5\n1...5\n[1, 2, 3, 4, 5]\n[1, 2, 3, 4]\n";
    let output = run_example("basics/simple_range.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_each_block_execution() {
    let expected = "Range iteration:\n1\n2\n3\nArray iteration:\n10\n20\n30\n";
    let output = run_example("basics/each_block.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_array_execution() {
    let expected = "1\n2\n3\n";
    let output = run_example("basics/for_loop_array.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_range_execution() {
    let expected = "1\n2\n3\n4\n5\n";
    let output = run_example("basics/for_loop_range.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_break_execution() {
    let expected = "1\n2\n3\n4\n";
    let output = run_example("basics/for_loop_break.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_continue_execution() {
    let expected = "1\n2\n4\n5\n";
    let output = run_example("basics/for_loop_continue.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_elsif_basic_execution() {
    let expected = "small positive\n";
    let output = run_example("basics/elsif_basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_elsif_without_else_execution() {
    let expected = "C\n";
    let output = run_example("basics/elsif_without_else.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_elsif_no_parens_execution() {
    let expected = "warm\n";
    let output = run_example("basics/elsif_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_spaceship_operator_execution() {
    let expected = "-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n";
    let output = run_example("basics/spaceship_operator.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_spaceship_operator_parens_execution() {
    let expected = "-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n";
    let output = run_example("basics/spaceship_operator_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_power_operator_execution() {
    let expected = "1024\n27\n1\n0.1\n2\n25\n256\n";
    let output = run_example("basics/power_operator.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_power_operator_parens_execution() {
    let expected = "1024\n27\n1\n0.1\n2\n25\n256\n";
    let output = run_example("basics/power_operator_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_bitwise_ops_execution() {
    let expected = "8\n14\n6\n170\n255\n85\n0\n255\n255\n";
    let output = run_example("basics/bitwise_ops.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_bitwise_ops_parens_execution() {
    let expected = "8\n14\n6\n170\n255\n85\n0\n255\n255\n";
    let output = run_example("basics/bitwise_ops_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_heredoc_execution() {
    let expected = "Hello, World!\nThis is a heredoc.\n\nGood morning!\n";
    let output = run_example("basics/heredoc.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_heredoc_parens_execution() {
    let expected = "Hello, World!\nThis is a heredoc.\n\nGood morning!\n";
    let output = run_example("basics/heredoc_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_type_annotations_collection_types_execution() {
    let output = run_example("type_annotations/collection_types.rb");
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
