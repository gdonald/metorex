use super::run_example;

#[test]
fn test_metaprogramming_top_level_define_method_execution() {
    let expected = "true\n:boing\nfalse\nfalse\n";
    let output = run_example("metaprogramming/top_level_define_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_anonymous_class_execution() {
    let expected = "foo\nfoo\nbaz\nObject\n";
    let output = run_example("metaprogramming/anonymous_class.rb");
    assert_eq!(output, expected);
}

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
    let output = run_example("metaprogramming/ast/inspection.rb");
    assert_eq!(output, expected);
}

// 10.2.1 — Method Delegation

#[test]
fn test_metaprogramming_advanced_method_delegation() {
    let expected = r#"=== Simple Delegation ===
[LOG] Processing: input data
processed: input data

=== Dynamic Delegation ===
[LOG] delegated message

=== Forwarding with Hooks ===
Before: process
[LOG] Processing: test data
After: process -> processed: test data
"#;
    let output = run_example("metaprogramming/advanced/method/delegation.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_advanced_method_delegation_no_parens() {
    let expected = r#"=== Simple Delegation ===
[LOG] Processing: input data
processed: input data

=== Dynamic Delegation ===
[LOG] delegated message

=== Forwarding with Hooks ===
Before: process
[LOG] Processing: test data
After: process -> processed: test data
"#;
    let output = run_example("metaprogramming/advanced/method/delegation_no_parens.rb");
    assert_eq!(output, expected);
}

// 10.2.2 — Aspect-Oriented Programming

#[test]
fn test_metaprogramming_advanced_aspect_oriented() {
    let expected = r#"=== Before/After Aspects ===
>> entering calculation
<< exiting calculation => 5
>> entering string op
<< exiting string op => HELLO

=== Timing Aspect ===
Starting data processing...
Finished data processing

=== Retry Aspect ===
Attempt 1...
Attempt 2...
Success on attempt 2
"#;
    let output = run_example("metaprogramming/advanced/aspect_oriented.rb");
    assert_eq!(output, expected);
}

// 10.2.3 — Lazy Evaluation

#[test]
fn test_metaprogramming_advanced_lazy_evaluation() {
    let expected = r#"=== Lazy Evaluation ===
Created lazy value
Computed yet? false
  (computing expensive value...)
First access: 1764
Computed now? true
Second access: 1764

=== Lazy Chaining ===
Pipeline created (nothing computed yet)
  (step 1: loading data...)
  (step 2: transforming data...)
  (step 3: formatting result...)
Final result: Total: 60
Accessing again (cached): Total: 60
"#;
    let output = run_example("metaprogramming/advanced/lazy_evaluation.rb");
    assert_eq!(output, expected);
}

// 10.2.4 — Memoization

#[test]
fn test_metaprogramming_advanced_memoization() {
    let expected = r#"=== Memoized Fibonacci ===
fib(0) = 0
fib(1) = 1
fib(2) = 1
fib(3) = 2
fib(4) = 3
fib(5) = 5
fib(6) = 8
fib(7) = 13
fib(8) = 21
fib(9) = 34
fib(10) = 55
Cache entries: 11

=== Generic Memoizer ===
First calls:
  (computing 5 * 5)
25
  (computing 3 * 3)
9
25
Second calls (cached):
25
9
Cache size: 2
"#;
    let output = run_example("metaprogramming/advanced/memoization.rb");
    assert_eq!(output, expected);
}

// 10.2.5 — Custom Iterators

#[test]
fn test_metaprogramming_advanced_custom_iterators() {
    let expected = r#"=== each_with_index ===
0: apple
1: banana
2: cherry

=== times ===
Iteration 0
Iteration 1
Iteration 2
Iteration 3
Iteration 4

=== reduce ===
Sum: 15
Product: 120

=== flat_map ===
[1, 2, 3, 4, 5, 6]

=== take_while ===
[2, 4, 6]
"#;
    let output = run_example("metaprogramming/advanced/custom_iterators.rb");
    assert_eq!(output, expected);
}

// 10.2.6 — Method Chaining DSL

#[test]
fn test_metaprogramming_advanced_method_chaining_dsl() {
    let output = run_example("metaprogramming/advanced/method/chaining_dsl.rb");
    assert!(
        output.contains(
            "SELECT * FROM users WHERE age > 18 AND active = true ORDER BY name LIMIT 10"
        )
    );
    assert!(output.contains("SELECT * FROM products WHERE price < 100"));
    assert!(output.contains("<h1>Welcome</h1>"));
    assert!(output.contains("<p>This is a paragraph.</p>"));
    assert!(output.contains("host = localhost"));
    assert!(output.contains("port = 8080"));
}

// 14.2 — Method Missing

#[test]
fn test_metaprogramming_method_missing_execution() {
    let expected = r#"=== Dynamic Attribute Access ===
Alice
30
engineer
unknown: email

=== Ghost Methods ===
Called hello with 0 arg(s)
Called add with 2 arg(s)
Called greet with 3 arg(s)

=== Flexible Calculator ===
6
30
unknown operation: multiply

=== Selective ===
I am real
ghost: fake_method

=== Inherited method_missing ===
Base caught: anything
Base caught: whatever
"#;
    let output = run_example("metaprogramming/method_missing/method_missing.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_method_missing_no_parens_execution() {
    let expected = r#"=== Dynamic Attribute Access ===
Alice
30
engineer
unknown: email

=== Ghost Methods ===
Called hello with 0 arg(s)
Called add with 2 arg(s)
Called greet with 3 arg(s)

=== Flexible Calculator ===
6
30
unknown operation: multiply

=== Selective ===
I am real
ghost: fake_method

=== Inherited method_missing ===
Base caught: anything
Base caught: whatever
"#;
    let output = run_example("metaprogramming/method_missing/method_missing_no_parens.rb");
    assert_eq!(output, expected);
}

// 14.3 — Runtime Class Modification

#[test]
fn test_metaprogramming_class_modification_execution() {
    let expected = r#"=== alias_method ===
Hello, Alice!
Hello, Bob!

=== remove_method ===
moving
no method: speak

=== undef_method ===
Base farewell
undefined: greet

=== multiple aliases ===
HELLO
HELLO
HELLO

=== module_function ===
14
12

=== remove_method with inheritance ===
Parent greet
"#;
    let output = run_example("metaprogramming/class_modification/class_modification.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_class_modification_no_parens_execution() {
    let expected = r#"=== alias_method ===
Hello, Alice!
Hello, Bob!

=== remove_method ===
moving
no method: speak

=== undef_method ===
Base farewell
undefined: greet

=== multiple aliases ===
HELLO
HELLO
HELLO

=== module_function ===
14
12

=== remove_method with inheritance ===
Parent greet
"#;
    let output = run_example("metaprogramming/class_modification/class_modification_no_parens.rb");
    assert_eq!(output, expected);
}

// 14.4 — Reflection and Introspection

#[test]
fn test_metaprogramming_reflection_execution() {
    let expected = r#"=== class ===
Dog
String
Integer
Array

=== instance_of? ===
true
false

=== is_a? ===
true
true

=== respond_to? ===
true
true
false

=== methods ===
2

=== send ===
Woof!
Fetching ball

=== send with symbol ===
Woof!
Fetching stick

=== send on built-in ===
HELLO
5
"#;
    let output = run_example("metaprogramming/reflection/reflection.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_reflection_no_parens_execution() {
    let expected = r#"=== class ===
Dog
String
Integer
Array

=== instance_of? ===
true
false

=== is_a? ===
true
true

=== respond_to? ===
true
true
false

=== methods ===
2

=== send ===
Woof!
Fetching ball

=== send with symbol ===
Woof!
Fetching stick

=== send on built-in ===
HELLO
5
"#;
    let output = run_example("metaprogramming/reflection/reflection_no_parens.rb");
    assert_eq!(output, expected);
}

// 14.5 — AST Manipulation

#[test]
fn test_metaprogramming_ast_manipulation_execution() {
    let expected = r#"=== eval ===
6
50
30

=== eval define method ===
14

=== eval define class ===
(3, 4)

=== parse ===
Array
1

=== code generation ===
7
7
30

=== runtime modification ===
8
15
"#;
    let output = run_example("metaprogramming/ast/manipulation.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_instance_exec_execution() {
    let expected = "10\n20\n105\n";
    let output = run_example("metaprogramming/instance_exec/instance_exec.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_instance_exec_parens_execution() {
    let expected = "10\n20\n105\n";
    let output = run_example("metaprogramming/instance_exec/instance_exec_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_ast_manipulation_no_parens_execution() {
    let expected = r#"=== eval ===
6
50
30

=== eval define method ===
14

=== eval define class ===
(3, 4)

=== parse ===
Array
1

=== code generation ===
7
7
30

=== runtime modification ===
8
15
"#;
    let output = run_example("metaprogramming/ast/manipulation_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_refinement_execution() {
    let output = run_example("metaprogramming/refinement.rb");
    assert_eq!(output, "HELLO!\n");
}
