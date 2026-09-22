// The rest of what a program can ask about itself.

use super::super::run_example;
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
    let expected = "=== each_with_index ===\n0: apple\n1: banana\n2: cherry\n\n=== times ===\nIteration 0\nIteration 1\nIteration 2\nIteration 3\nIteration 4\n\n=== reduce ===\nSum: 15\nProduct: 120\n\n=== flat_map ===\n1\n2\n3\n4\n5\n6\n\n=== take_while ===\n2\n4\n6\n";
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
