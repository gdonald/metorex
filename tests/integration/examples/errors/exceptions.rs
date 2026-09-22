// The exception classes, and the ones a program defines.

use super::super::run_example;
use super::*;
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
fn test_errors_error_location_execution() {
    let output = run_example("errors/error_location.rb");
    assert!(output.contains("Error:"));
    assert!(output.contains("Type:"));
}

#[test]
fn test_rescue_class_method_scope() {
    let expected = "rescued: location=test_loc\n";
    let output = run_example("rescue/class_method_test.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_rescue_param_scope() {
    let expected = "caught: loc=my_location\n";
    let output = run_example("rescue/param_scope_test.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_rescue_reraise_scope() {
    let expected = "outer rescue: location=nil, exc class=NoMethodError\n";
    let output = run_example("rescue/reraise_scope_test.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_rescue_rerescue() {
    // location is a String "test_loc"; .inspect quotes it.
    let expected = "rescue caught: location=\"test_loc\"\n";
    let output = run_example("rescue/rerescue_test.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_rescue_instance_exec_scope() {
    let expected = "rescued: location=my_location, exc=NoMethodError\nfalse\n";
    let output = run_example("rescue/instance_exec_scope_test.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_rescue_mspec_flow() {
    let expected = "......rescued: location=nil, exc=NoMethodError\n\ndone\n";
    let output = run_example("rescue/mspec_flow_test.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_rescue_modifier_fallbacks_execution() {
    let expected = concat!(
        "nil\n",
        "caught\n",
        "1\n",
        ":inline\n",
        "boom\n",
        "propagated: fatal\n"
    );
    let output = run_example("rescue/modifier/fallbacks.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_rescue_modifier_fallbacks_parens_execution() {
    let expected = concat!(
        "nil\n",
        "caught\n",
        "1\n",
        ":inline\n",
        "boom\n",
        "propagated: fatal\n"
    );
    let output = run_example("rescue/modifier/fallbacks_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_exception_message_to_s_execution() {
    let expected = concat!(
        "something went wrong\n",
        "Exceptional\n",
        "Exception\n",
        "boom\n",
        "a described message\n",
        "raised message\n",
        "RuntimeError\n",
        "raised message\n",
        "RuntimeError\n"
    );
    let output = run_example("errors/exception_message/to_s.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_exception_message_to_s_parens_execution() {
    let expected = concat!(
        "something went wrong\n",
        "Exceptional\n",
        "Exception\n",
        "boom\n",
        "a described message\n",
        "raised message\n",
        "RuntimeError\n",
        "raised message\n",
        "RuntimeError\n"
    );
    let output = run_example("errors/exception_message/to_s_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_subclass_state_execution() {
    let expected = concat!(
        "first failure\n42\nfalse\n",
        ":mine\n",
        "first failure\n42\ntrue\nfalse\n",
        "the consequence\nthe cause\ntrue\n"
    );
    let output = run_example("errors/subclass/state.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_subclass_state_parens_execution() {
    let expected = concat!(
        "first failure\n42\nfalse\n",
        ":mine\n",
        "first failure\n42\ntrue\nfalse\n",
        "the consequence\nthe cause\ntrue\n"
    );
    let output = run_example("errors/subclass/state_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_equality_comparison_execution() {
    let expected = "true\ntrue\ntrue\ntrue\nfalse\nfalse\nfalse\ntrue\nfalse\ntrue\n";
    let output = run_example("errors/equality/comparison.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_equality_comparison_parens_execution() {
    let expected = "true\ntrue\ntrue\ntrue\nfalse\nfalse\nfalse\ntrue\nfalse\ntrue\n";
    let output = run_example("errors/equality/comparison_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_exception_method_copies_execution() {
    let expected = concat!(
        "true\ntrue\n",
        "RuntimeError\nsecond\nfirst\nfalse\n",
        "Tagged\n:boom\nmessage\n",
        "built\nException\nRuntimeError\n",
        "RuntimeError\n\"\"\n"
    );
    let output = run_example("errors/exception_method/copies.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_exception_method_copies_parens_execution() {
    let expected = concat!(
        "true\ntrue\n",
        "RuntimeError\nsecond\nfirst\nfalse\n",
        "Tagged\n:boom\nmessage\n",
        "built\nException\nRuntimeError\n",
        "RuntimeError\n\"\"\n"
    );
    let output = run_example("errors/exception_method/copies_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_hierarchy_classes_execution() {
    let expected = concat!(
        "Object\nClass\n",
        "Exception\nException\nException\nException\nException\nException\nException\n",
        "SignalException\nScriptError\nIOError\nIndexError\nIndexError\nStopIteration\n",
        "NameError\nRangeError\nRuntimeError\nArgumentError\nStandardError\nStandardError\n",
        "KeyError\ntrue\n"
    );
    let output = run_example("errors/hierarchy/classes.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_hierarchy_classes_parens_execution() {
    let expected = concat!(
        "Object\nClass\n",
        "Exception\nException\nException\nException\nException\nException\nException\n",
        "SignalException\nScriptError\nIOError\nIndexError\nIndexError\nStopIteration\n",
        "NameError\nRangeError\nRuntimeError\nArgumentError\nStandardError\nStandardError\n",
        "KeyError\ntrue\n"
    );
    let output = run_example("errors/hierarchy/classes_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_exception_reports_execution() {
    let output = run_example("errors/exception_reports.rb");
    assert_eq!(output, EXCEPTION_REPORTS_OUTPUT);
}

#[test]
fn test_errors_exception_reports_parens_execution() {
    let output = run_example("errors/exception_reports_parens.rb");
    assert_eq!(output, EXCEPTION_REPORTS_OUTPUT);
}
