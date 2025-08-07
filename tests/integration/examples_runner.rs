// Examples runner

use std::process::Command;

use super::common::EXAMPLES_DIR;

fn run_example(path: &str) -> String {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/{}", EXAMPLES_DIR, path);
    let mut cmd = Command::new(binary);
    cmd.current_dir(manifest_dir).arg(&full_path);

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
    let output = run_example("basics/greeting_line.mx");
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

    let output = run_example("basics/string_methods.mx");
    assert_eq!(output, expected.to_string());
}

#[test]
fn test_data_structures_simple_dict_execution() {
    let output = run_example("data-structures/simple_dict.mx");
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
    let output = run_example("data-structures/dict_access.mx");
    assert_eq!(output, "Ada lives in London\n");
}

#[test]
fn test_data_structures_hash_methods_execution() {
    let output = run_example("data-structures/hash_methods.mx");
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
    let output = run_example("type-annotations/collection_types.mx");
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
    let output = run_example("basics/simple_range.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_each_block_execution() {
    let expected = "Range iteration:\n1\n2\n3\nArray iteration:\n10\n20\n30\n";
    let output = run_example("basics/each_block.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_factorial_iterative_execution() {
    let expected = "720\n";
    let output = run_example("algorithms/factorial_iterative.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_average_temperature_execution() {
    let expected = "69.9\n";
    let output = run_example("algorithms/average_temperature.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_primes_under_fifty_execution() {
    let expected = "[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47]\n";
    let output = run_example("algorithms/primes_under_fifty.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_functions_closures_nested_execution() {
    let expected = "10\n12\n";
    let output = run_example("functions/closures_nested.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_functions_nonlocal_counter_execution() {
    let expected = "1\n2\n3\n3\n0\n1\n";
    let output = run_example("functions/nonlocal_counter.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_functions_locals_scope_execution() {
    let expected = "20\n[0, 2, 4, 6, 8]\n";
    let output = run_example("functions/locals_scope.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_parser_lambdas_execution() {
    let expected = "10\n10\n42\n30\n13\n13\n18\n11\n14\n21\n24\n10\n";
    let output = run_example("functions/test_lambdas.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_parser_pattern_matching_execution() {
    let expected = "two\nstopping\nother number\none point zero\nfive\n";
    let output = run_example("control-flow/test_pattern_matching.mx");
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

    let output = run_example("metaprogramming/blocks_as_objects.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_filter_even_numbers_execution() {
    let expected = "[2, 4, 6]\n";
    let output = run_example("algorithms/filter_even_numbers.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_character_counter_execution() {
    let output = run_example("algorithms/character_counter.mx");
    // Hash map iteration order is non-deterministic, so check for all expected keys and values
    assert!(
        output.contains("b")
            && output.contains("a")
            && output.contains("n")
            && output.contains(": 1")
            && output.contains(": 3")
            && output.contains(": 2"),
        "Expected output to contain all characters (b:1, a:3, n:2), but got: {}",
        output
    );
}

#[test]
fn test_algorithms_zip_merger_execution() {
    let expected = "[[Ann, 88], [Ben, 93]]\n";
    let output = run_example("algorithms/zip_merger.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_matrix_transpose_execution() {
    let expected = "[[1, 4], [2, 5], [3, 6]]\n";
    let output = run_example("algorithms/matrix_transpose.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_matrix_transpose_comprehensive_execution() {
    let expected = r#"Basic 2x3 matrix:
[[1, 4], [2, 5], [3, 6]]
Double transpose (3x2 matrix):
[[1, 2], [3, 4], [5, 6]]
Single row matrix:
[[1], [2], [3], [4]]
Single column matrix:
[[1, 2, 3]]
Square 3x3 matrix:
[[1, 4, 7], [2, 5, 8], [3, 6, 9]]
"#;
    let output = run_example("algorithms/matrix_transpose_comprehensive.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_algorithms_matrix_nested_ops_execution() {
    let expected = r#"Original matrix:
[[1, 2, 3], [4, 5, 6]]
Element at [0][0]:
1
Element at [1][2]:
6
Doubled matrix:
[[2, 4, 6], [8, 10, 12]]
Sum of each column:
[5, 7, 9]
Rows where first element > 2:
[[3, 4], [5, 6]]
"#;
    let output = run_example("algorithms/matrix_nested_ops.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_basic_execution() {
    let expected = "Buddy\nGolden Retriever\nSome sound -> Woof!\nI am an animal named Buddy\n";
    let output = run_example("oop/super_basic.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_chain_basic_execution() {
    let expected = "GrandParent\nParent\nChild\n";
    let output = run_example("oop/super_chain_basic.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_simple_execution() {
    let expected = "AB\n";
    let output = run_example("oop/test_super_simple.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_reader_execution() {
    let expected = "Alice\n30\n";
    let output = run_example("oop/attr_reader.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_writer_execution() {
    let expected = "Unknown\n0\nBob\n25\n";
    let output = run_example("oop/attr_writer.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_accessor_execution() {
    let expected = "Charlie\n35\ncharlie@example.com\nCharles\n36\ncharles@example.com\n";
    let output = run_example("oop/attr_accessor.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_str_execution() {
    let expected = "Person: Alice\n";
    let output = run_example("oop/test_str.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_repr_execution() {
    let expected = "Point(0, 0)\n";
    let output = run_example("oop/test_repr.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_special_methods_execution() {
    let expected = "Book: Ruby Guide\nMagazine: Tech Monthly\nnext_value\n";
    let output = run_example("oop/special_methods.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_iter_execution() {
    let expected = "next\n";
    let output = run_example("oop/test_iter.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_method_missing_execution() {
    let expected = "bar\n42\n1\n2\n3\n";
    let output = run_example("oop/test_method_missing.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_array_execution() {
    let expected = "1\n2\n3\n";
    let output = run_example("basics/for_loop_array.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_range_execution() {
    let expected = "1\n2\n3\n4\n5\n";
    let output = run_example("basics/for_loop_range.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_break_execution() {
    let expected = "1\n2\n3\n4\n";
    let output = run_example("basics/for_loop_break.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_continue_execution() {
    let expected = "1\n2\n4\n5\n";
    let output = run_example("basics/for_loop_continue.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_elsif_basic_execution() {
    let expected = "small positive\n";
    let output = run_example("basics/elsif_basic.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_elsif_without_else_execution() {
    let expected = "C\n";
    let output = run_example("basics/elsif_without_else.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_elsif_no_parens_execution() {
    let expected = "warm\n";
    let output = run_example("basics/elsif_no_parens.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_guard_execution() {
    let expected = "Warm\nLarge hundred\n";
    let output = run_example("control-flow/case_guard.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_array_destructure_execution() {
    let expected = "a=1, b=2, c=3\nFirst: 1\nRest: [2, 3, 4, 5]\nFirst: 1, Last: 5\nMiddle: [2, 3, 4]\nSum: 10\nFirst is 1, last is 4\n";
    let output = run_example("control-flow/case_array_destructure.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_object_destructure_execution() {
    let expected = "Point at (10, 20)\nName: Alice, Age: 30\nAlice is 30 years old\n";
    let output = run_example("control-flow/case_object_destructure.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_variable_binding_execution() {
    let expected = "Matched: 42\nNot Found\nWorking age: 25\n";
    let output = run_example("control-flow/case_variable_binding.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_type_basic_execution() {
    let expected = "It's an integer\nIt's a string\nIt's an array\nIt's a hash\nFloat\n";
    let output = run_example("control-flow/case_type_basic.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_type_custom_class_execution() {
    let expected =
        "It's a dog!\nBuddy says woof!\nIt's a cat!\nWhiskers says meow!\nIt's just a string\n";
    let output = run_example("control-flow/case_type_custom_class.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_type_mixed_execution() {
    let expected = "It's an integer: 42\nGeneric string\nProcessing integer: 20\nProcessing float: 4.71\nProcessing string: TEST\nProcessing array of 3 elements\nProcessing hash with 2 keys\n";
    let output = run_example("control-flow/case_type_mixed.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_simple_rescue_execution() {
    let expected = "Before exception\nCaught an exception\nAfter rescue block\nCaught exception with message: RuntimeError: An error message\nIn try block\nIn rescue block\nIn ensure block\n";
    let output = run_example("errors/simple_rescue.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_advanced_exception_handling_execution() {
    let expected = "risky operation!\nGeneral error: Oops...\ncleanup\n";
    let output = run_example("advanced/exception_handling.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_exception_hierarchy_execution() {
    let expected = "Example 1: Different exception types\nCaught RuntimeError: Runtime error occurred\nCaught TypeError: Type mismatch\nCaught ValueError: Invalid value\n\nExample 2: Catching StandardError\nCaught as StandardError: A runtime error\nCaught as StandardError: A type error\n\nExample 3: Specific to general exception handling\nSpecific handler for RuntimeError: Runtime issue\nSpecific handler for TypeError: Type issue\nGeneral handler for StandardError: Value issue\n\nExample 4: Exception type checking\nRuntimeError is a StandardError: true\nError message: Test error\n";
    let output = run_example("errors/exception_hierarchy.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_custom_exceptions_execution() {
    let expected = "Example 1: Custom exception types\nCaught DatabaseError: Database connection failed\nCaught ConnectionError: Could not connect to database\nCaught QueryError: Invalid SQL query\n\nExample 2: Catching via parent class\nCaught as DatabaseError: Connection timeout\nCaught as DatabaseError: Table not found\n\nExample 3: Multiple rescue clauses\nConnection issue: Connection failed\nQuery issue: Query syntax error\nValidation issue: Invalid input data\n\nExample 4: Re-raising exceptions\nCaught in attempt_operation: Failed to execute query\nCaught in outer scope: Failed to execute query\n\nExample 5: Exception hierarchy in action\nSpecific handler: Database unreachable\n";
    let output = run_example("errors/custom_exceptions.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_exception_chaining_execution() {
    let expected = "Example 1: Catching and re-raising\nCaught NetworkError: Network connection failed\nRe-raising as DatabaseError...\nCaught DatabaseError: Database initialization failed\n\nExample 2: Multi-level exception handling\nLevel 2 caught: Error at level 1\nLevel 3 caught: Type error in level 2\nTop level caught: Value error in level 3\n\nExample 3: Accessing current exception with $!\nCaught exception: Original error\nException binding and $! both reference the current exception\n\nExample 4: Error context preservation\nFile error occurred: config.txt not found\nConfiguration error: Failed to load configuration\nApplication cannot start\n\nExample 5: Conditional re-raising\nRecovered from error: Something went wrong\nCannot recover, re-raising...\nCaught re-raised error: Something went wrong\n";
    let output = run_example("errors/exception_chaining.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_stack_trace_basic_execution() {
    let output = run_example("errors/stack_trace_basic.mx");
    // Check that the output contains the error message and backtrace info
    assert!(output.contains("RuntimeError: Division by zero!"));
    assert!(output.contains("Backtrace:"));
}

#[test]
fn test_errors_stack_trace_deep_execution() {
    let output = run_example("errors/stack_trace_deep.mx");
    assert!(output.contains("Error at level 4!"));
    // Check that there are multiple stack frames
    assert!(output.contains("Stack trace has"));
}

#[test]
fn test_errors_error_location_execution() {
    let output = run_example("errors/error_location.mx");
    // Division by zero should be caught
    assert!(output.contains("Error:"));
    assert!(output.contains("Type:"));
}

#[test]
fn test_errors_backtrace_method_execution() {
    let output = run_example("errors/backtrace_method.mx");
    assert!(output.contains("Caught: Error in inner method"));
    assert!(output.contains("Backtrace array length:"));
    assert!(output.contains("First frame:"));
}

#[test]
fn test_introspection_function_name_execution() {
    let expected = r#"greet
calculate
"#;
    let output = run_example("introspection/function_name.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_function_module_execution() {
    let expected = r#"main
main
"#;
    let output = run_example("introspection/function_module.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_code_object_execution() {
    let expected = r#"greet.source_location = 1:1
calculate.source_location = 5:1
"#;
    let output = run_example("introspection/code_object.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_closure_namespace_execution() {
    let expected = r#"simple_func
simple_func
nil
Object
Object
<Binding with 21 vars>
18
"#;
    let output = run_example("introspection/closure_namespace.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_basic_attributes_execution() {
    let expected = r#"greet.name = greet
calculate.name = calculate
greet.doc = nil
calculate.doc = nil
"#;
    let output = run_example("introspection/basic_attributes.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_annotations_execution() {
    let expected = r#"add.parameters = [x, y]
greet.parameters = [name]
process.parameters = [data, count, flag]
no_annotations.parameters = [a, b]
"#;
    let output = run_example("introspection/annotations.mx");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_default_parameters_execution() {
    let expected = r#"no_defaults
[a, b]
with_defaults
[a, b, c]
all_defaults
[x, y, z]
greet
[name, greeting, punctuation]
"#;
    let output = run_example("introspection/default_parameters.mx");
    assert_eq!(output, expected);
}
