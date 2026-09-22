// What a program reports about the call that is running.

use super::super::run_example;
#[test]
fn test_metaprogramming_ast_inspection_execution() {
    let expected = "add\n2\n1\nBinaryOp\n+\n1\nBinaryOp\n*\n1\n1\n";
    let output = run_example("metaprogramming/ast/inspection.rb");
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
[:fetch, :speak]

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
[:fetch, :speak]

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
fn test_metaprogramming_caller_locations_lineno_execution() {
    let expected = "true\ntrue\n";
    let output = run_example("metaprogramming/caller_locations_lineno.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_caller_locations_slicing_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "nil\n",
        "[]\n",
        "Thread::Backtrace::Location\n",
        "true\n",
        "true\n",
        "true\n",
    );
    let output = run_example("metaprogramming/caller_locations_slicing.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_caller_locations_slicing_parens_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "nil\n",
        "[]\n",
        "Thread::Backtrace::Location\n",
        "true\n",
        "true\n",
        "true\n",
    );
    let output = run_example("metaprogramming/caller_locations_slicing_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_caller_entries_execution() {
    let expected = concat!(
        "true\n", "true\n", "true\n", "true\n", "true\n", "true\n", "nil\n", "[]\n", "true\n",
        "true\n",
    );
    let output = run_example("metaprogramming/caller_entries.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_caller_entries_parens_execution() {
    let expected = concat!(
        "true\n", "true\n", "true\n", "true\n", "true\n", "true\n", "nil\n", "[]\n", "true\n",
        "true\n",
    );
    let output = run_example("metaprogramming/caller_entries_parens.rb");
    assert_eq!(output, expected);
}
