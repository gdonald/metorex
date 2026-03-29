use super::run_example;

#[test]
fn test_errors_simple_rescue_execution() {
    let expected = "Before exception\nCaught an exception\nAfter rescue block\nCaught exception with message: RuntimeError: An error message\nIn try block\nIn rescue block\nIn ensure block\n";
    let output = run_example("errors/simple_rescue.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_advanced_exception_handling_execution() {
    let expected = "risky operation!\nGeneral error: Oops...\ncleanup\n";
    let output = run_example("advanced/exception_handling.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_exception_hierarchy_execution() {
    let expected = "Example 1: Different exception types\nCaught RuntimeError: Runtime error occurred\nCaught TypeError: Type mismatch\nCaught ValueError: Invalid value\n\nExample 2: Catching StandardError\nCaught as StandardError: A runtime error\nCaught as StandardError: A type error\n\nExample 3: Specific to general exception handling\nSpecific handler for RuntimeError: Runtime issue\nSpecific handler for TypeError: Type issue\nGeneral handler for StandardError: Value issue\n\nExample 4: Exception type checking\nRuntimeError is a StandardError: true\nError message: Test error\n";
    let output = run_example("errors/exception_hierarchy.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_custom_exceptions_execution() {
    let expected = "Example 1: Custom exception types\nCaught DatabaseError: Database connection failed\nCaught ConnectionError: Could not connect to database\nCaught QueryError: Invalid SQL query\n\nExample 2: Catching via parent class\nCaught as DatabaseError: Connection timeout\nCaught as DatabaseError: Table not found\n\nExample 3: Multiple rescue clauses\nConnection issue: Connection failed\nQuery issue: Query syntax error\nValidation issue: Invalid input data\n\nExample 4: Re-raising exceptions\nCaught in attempt_operation: Failed to execute query\nCaught in outer scope: Failed to execute query\n\nExample 5: Exception hierarchy in action\nSpecific handler: Database unreachable\n";
    let output = run_example("errors/custom_exceptions.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_exception_chaining_execution() {
    let expected = "Example 1: Catching and re-raising\nCaught NetworkError: Network connection failed\nRe-raising as DatabaseError...\nCaught DatabaseError: Database initialization failed\n\nExample 2: Multi-level exception handling\nLevel 2 caught: Error at level 1\nLevel 3 caught: Type error in level 2\nTop level caught: Value error in level 3\n\nExample 3: Accessing current exception with $!\nCaught exception: Original error\nException binding and $! both reference the current exception\n\nExample 4: Error context preservation\nFile error occurred: config.txt not found\nConfiguration error: Failed to load configuration\nApplication cannot start\n\nExample 5: Conditional re-raising\nRecovered from error: Something went wrong\nCannot recover, re-raising...\nCaught re-raised error: Something went wrong\n";
    let output = run_example("errors/exception_chaining.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_stack_trace_basic_execution() {
    let output = run_example("errors/stack_trace_basic.rb");
    assert!(output.contains("RuntimeError: Division by zero!"));
    assert!(output.contains("Backtrace:"));
}

#[test]
fn test_errors_stack_trace_deep_execution() {
    let output = run_example("errors/stack_trace_deep.rb");
    assert!(output.contains("Error at level 4!"));
    assert!(output.contains("Stack trace has"));
}

#[test]
fn test_errors_error_location_execution() {
    let output = run_example("errors/error_location.rb");
    assert!(output.contains("Error:"));
    assert!(output.contains("Type:"));
}

#[test]
fn test_errors_backtrace_method_execution() {
    let output = run_example("errors/backtrace_method.rb");
    assert!(output.contains("Caught: Error in inner method"));
    assert!(output.contains("Backtrace array length:"));
    assert!(output.contains("First frame:"));
}
