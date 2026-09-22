// Blocks held as objects, and where they were written.

use super::super::run_example;
#[test]
fn test_metaprogramming_blocks_as_objects_execution() {
    let expected = r#"=== Blocks as First-Class Objects ===

1. Assigning blocks to variables:
double.call(5) = 10

2. Multiple parameter blocks:
add.call(3, 7) = 10

3. Passing blocks as arguments to functions:
apply_twice(increment, 5) = 7

4. Returning blocks from functions (closures):
times_three.call(4) = 12
times_ten.call(4) = 40

5. Blocks capturing variables from outer scope:
First call: 1
Second call: 2
Third call: 3

6. Partial application pattern:
Hello, Alice!
Goodbye, Bob!

=== Blocks are truly first-class objects! ===
"#;

    let output = run_example("metaprogramming/blocks_as_objects.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_block_param_defaults_execution() {
    let expected = "[5, 1]\n[5, 6]\n[1, 1]\n[1, 2]\nArgumentError\nArgumentError\n9\n";
    let output = run_example("metaprogramming/block_param_defaults.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_block_param_defaults_parens_execution() {
    let expected = "[5, 1]\n[5, 6]\n[1, 1]\n[1, 2]\nArgumentError\nArgumentError\n9\n";
    let output = run_example("metaprogramming/block_param_defaults_parens.rb");
    assert_eq!(output, expected);
}
