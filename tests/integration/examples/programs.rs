use super::run_example;

// 10.1.1 — FizzBuzz

#[test]
fn test_programs_fizzbuzz() {
    let expected = "1\n2\nFizz\n4\nBuzz\nFizz\n7\n8\nFizz\nBuzz\n11\nFizz\n13\n14\nFizzBuzz\n16\n17\nFizz\n19\nBuzz\n";
    let output = run_example("programs/fizzbuzz.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_programs_fizzbuzz_no_parens() {
    let expected = "1\n2\nFizz\n4\nBuzz\nFizz\n7\n8\nFizz\nBuzz\n11\nFizz\n13\n14\nFizzBuzz\n16\n17\nFizz\n19\nBuzz\n";
    let output = run_example("programs/fizzbuzz_no_parens.rb");
    assert_eq!(output, expected);
}

// 10.1.2 — Fibonacci

#[test]
fn test_programs_fibonacci() {
    let expected = "Iterative:\n0\n1\n1\n2\n3\n5\n8\n13\n21\n34\nRecursive:\n0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n";
    let output = run_example("programs/fibonacci.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_programs_fibonacci_no_parens() {
    let expected = "Iterative:\n0\n1\n1\n2\n3\n5\n8\n13\n21\n34\nRecursive:\n0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n";
    let output = run_example("programs/fibonacci_no_parens.rb");
    assert_eq!(output, expected);
}

// 10.1.3 — Factorial

#[test]
fn test_programs_factorial() {
    let expected = "0! = 1\n1! = 1\n2! = 2\n3! = 6\n4! = 24\n5! = 120\n6! = 720\n7! = 5040\n8! = 40320\n9! = 362880\n10! = 3628800\n";
    let output = run_example("programs/factorial.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_programs_factorial_no_parens() {
    let expected = "0! = 1\n1! = 1\n2! = 2\n3! = 6\n4! = 24\n5! = 120\n6! = 720\n7! = 5040\n8! = 40320\n9! = 362880\n10! = 3628800\n";
    let output = run_example("programs/factorial_no_parens.rb");
    assert_eq!(output, expected);
}

// 10.1.4 — Calculator

#[test]
fn test_programs_calculator() {
    let expected = r#"Addition: 10 + 3 = 13
Subtraction: 10 - 3 = 7
Multiplication: 10 * 3 = 30
Division: 10 / 3 = 3.3333333333333335
Modulo: 10 % 3 = 1
Division by zero:
Error: Division by zero
Last result: 1
"#;
    let output = run_example("programs/calculator.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_programs_calculator_no_parens() {
    let expected = r#"Addition: 10 + 3 = 13
Subtraction: 10 - 3 = 7
Multiplication: 10 * 3 = 30
Division: 10 / 3 = 3.3333333333333335
Modulo: 10 % 3 = 1
Division by zero:
Error: Division by zero
Last result: 1
"#;
    let output = run_example("programs/calculator_no_parens.rb");
    assert_eq!(output, expected);
}

// 10.1.5 — Todo List

#[test]
fn test_programs_todo_list() {
    let expected = r#"Before completing tasks:
=== My Tasks ===
0. [ ] Buy groceries
1. [ ] Write tests
2. [ ] Read a book
3. [ ] Exercise
4 pending, 4 total

After completing tasks:
=== My Tasks ===
0. [x] Buy groceries
1. [x] Write tests
2. [ ] Read a book
3. [ ] Exercise
2 pending, 4 total
"#;
    let output = run_example("programs/todo_list.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_programs_todo_list_no_parens() {
    let expected = r#"Before completing tasks:
=== My Tasks ===
0. [ ] Buy groceries
1. [ ] Write tests
2. [ ] Read a book
3. [ ] Exercise
4 pending, 4 total

After completing tasks:
=== My Tasks ===
0. [x] Buy groceries
1. [x] Write tests
2. [ ] Read a book
3. [ ] Exercise
2 pending, 4 total
"#;
    let output = run_example("programs/todo_list_no_parens.rb");
    assert_eq!(output, expected);
}

// 10.1.7 — Data Processing Pipeline

#[test]
fn test_programs_data_pipeline() {
    let expected = r#"Passing students:
  Alice: 92
  Bob: 78
  Charlie: 95
  Diana: 88
  Frank: 91
Honor roll:
  Alice: 92
  Charlie: 95
  Frank: 91
Average grade: 85.16666666666667
Math students: 3
Science students: 3
All students: [Alice, Bob, Charlie, Diana, Eve, Frank]
"#;
    let output = run_example("programs/data_pipeline.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_programs_data_pipeline_no_parens() {
    let expected = r#"Passing students:
  Alice: 92
  Bob: 78
  Charlie: 95
  Diana: 88
  Frank: 91
Honor roll:
  Alice: 92
  Charlie: 95
  Frank: 91
Average grade: 85.16666666666667
Math students: 3
Science students: 3
All students: [Alice, Bob, Charlie, Diana, Eve, Frank]
"#;
    let output = run_example("programs/data_pipeline_no_parens.rb");
    assert_eq!(output, expected);
}

// 10.1.8 — OOP Design Patterns

#[test]
fn test_programs_oop_patterns() {
    let expected = r#"=== Strategy Pattern ===
HELLO WORLD
hello world

=== Template Method Pattern ===
--- Report ---
Total sales: $1000
--- End ---

*** Inventory ***
Items in stock: 42
*** End ***

=== Observer Pattern ===
Listener 1: event fired
Listener 2: event fired

=== Decorator Pattern ===
Basic coffee: $5
Basic coffee + milk: $7
Basic coffee + milk + sugar: $8
"#;
    let output = run_example("programs/oop_patterns.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_programs_oop_patterns_no_parens() {
    let expected = r#"=== Strategy Pattern ===
HELLO WORLD
hello world

=== Template Method Pattern ===
--- Report ---
Total sales: $1000
--- End ---

*** Inventory ***
Items in stock: 42
*** End ***

=== Observer Pattern ===
Listener 1: event fired
Listener 2: event fired

=== Decorator Pattern ===
Basic coffee: $5
Basic coffee + milk: $7
Basic coffee + milk + sugar: $8
"#;
    let output = run_example("programs/oop_patterns_no_parens.rb");
    assert_eq!(output, expected);
}
