use super::run_example;

#[test]
fn test_metaprogramming_implicit_blocks_execution() {
    let expected = r#"Howdy, Alice!
Hey, Bob!
Hello, Charlie!
Iteration: 0
Iteration: 1
Iteration: 2
no block
got a block
10
20
1
4
9
[1, 4, 9, 16]
[2, 4, 6]
1
2
3
4
5
"#;
    let output = run_example("metaprogramming/implicit_blocks.rb");
    assert_eq!(output, expected);
}

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
fn test_metaprogramming_get_source_execution() {
    let expected = "speak\nfetch\ntrue\nspeak\npurr\npurr\n";
    let output = run_example("metaprogramming/get_source.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_method_execution() {
    let expected = "Hello, World!\n10\n12\nHi there!\nzero\none\ntwo\ntrue\n";
    let output = run_example("metaprogramming/define_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_ast_inspection_execution() {
    let expected = "add\n2\n1\nBinaryOp\n+\n1\nBinaryOp\n*\n1\n1\n";
    let output = run_example("metaprogramming/ast_inspection.rb");
    assert_eq!(output, expected);
}
