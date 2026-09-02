use crate::common::EXAMPLES_DIR;
use std::process::Command;

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
    let expected = "Division by zero!\nRuntimeError\nDivision by zero!\nArray\n";
    let output = run_example("errors/stack_trace_basic.rb");
    assert_eq!(output, expected);
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

#[test]
fn test_raise_two_arg_execution() {
    let expected = "caught two-arg raise\n";
    let output = run_example("errors/raise_two_arg.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_raise_two_arg_parens_execution() {
    let expected = "caught two-arg raise\n";
    let output = run_example("errors/raise_two_arg_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_begin_else_ensure_execution() {
    let expected = "try block\nno error, x = 42\nensure ran\n";
    let output = run_example("errors/begin_else_ensure.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_begin_else_ensure_parens_execution() {
    let expected = "try block\nno error, x = 42\nensure ran\n";
    let output = run_example("errors/begin_else_ensure_parens.rb");
    assert_eq!(output, expected);
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
fn test_errors_abort_rescued_execution() {
    let expected = "with parentheses omitted\n1\nwith parentheses\n1\nSystemExit\n1\nfrom the Kernel module\nfrom an instance\ncoerced with to_str\nno implicit conversion of Integer into String\ntrue\n";
    let output = run_example("errors/abort_rescued.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_abort_uncaught_execution() {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/errors/abort_uncaught.rb", EXAMPLES_DIR);
    let mut command = Command::new(binary);
    command.current_dir(manifest_dir).arg(&full_path);

    let output = command.output().expect("failed to execute example");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not utf8");

    assert_eq!(stdout, "captured: redirected message\n");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_errors_kernel_fail_execution() {
    let expected = "RuntimeError\nthe duck is not irish.\nMissingWidget\nno widget here\nStandardError: built by hand\nsent along\ntrue\n";
    let output = run_example("errors/kernel_fail.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_kernel_raise_method() {
    let expected = concat!(
        "keyword form\n",
        "ArgumentError: through send\n",
        "TypeError: with a receiver\n",
        "IndexError: on Kernel\n",
        "KeyError: through a Method object\n",
        "true\nRuntimeError\n\"\"\n"
    );
    let output = run_example("errors/kernel_raise_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_kernel_raise_method_parens() {
    let expected = concat!(
        "keyword form\n",
        "ArgumentError: through send\n",
        "TypeError: with a receiver\n",
        "IndexError: on Kernel\n",
        "KeyError: through a Method object\n",
        "true\nRuntimeError\n\"\"\n"
    );
    let output = run_example("errors/kernel_raise_method_parens.rb");
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
fn test_errors_backtrace_access_execution() {
    let expected = concat!(
        "nil\n",
        "Array\n",
        "true\n",
        "RuntimeError\n",
        "raised here\n",
        "Array\n",
        "[\"one\", \"two\"]\n",
        "true\n",
        "[\"single\"]\n",
        "nil\n",
        "backtrace must be an Array of String, got Symbol\n"
    );
    let output = run_example("errors/backtrace/access.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_backtrace_access_parens_execution() {
    let expected = concat!(
        "nil\n",
        "Array\n",
        "true\n",
        "RuntimeError\n",
        "raised here\n",
        "Array\n",
        "[\"one\", \"two\"]\n",
        "true\n",
        "[\"single\"]\n",
        "nil\n",
        "backtrace must be an Array of String, got Symbol\n"
    );
    let output = run_example("errors/backtrace/access_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_backtrace_locations_execution() {
    let expected = concat!(
        "nil\n",
        "Array\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "[\"0:a\", \"1:b\"]\n",
        "Enumerator\n"
    );
    let output = run_example("errors/backtrace/locations.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_backtrace_locations_parens_execution() {
    let expected = concat!(
        "nil\n",
        "Array\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "[\"0:a\", \"1:b\"]\n",
        "Enumerator\n"
    );
    let output = run_example("errors/backtrace/locations_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_errno_classes_execution() {
    let expected = concat!(
        "Errno::EINVAL\n",
        "SystemCallError\n",
        "22\n2\ntrue\n",
        "Errno::EINVAL\n",
        "22\ntrue\ntrue\nfalse\n",
        "Errno::ENOENT\n",
        "boom\ntrue\ntrue\nnil\n",
        "Invalid argument\n",
        "Invalid argument - custom message\n",
        "Invalid argument @ location - custom message\n",
        "No such file or directory\n",
        "No such file or directory - custom message\n"
    );
    let output = run_example("errors/errno/classes.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_errno_classes_parens_execution() {
    let expected = concat!(
        "Errno::EINVAL\n",
        "SystemCallError\n",
        "22\n2\ntrue\n",
        "Errno::EINVAL\n",
        "22\ntrue\ntrue\nfalse\n",
        "Errno::ENOENT\n",
        "boom\ntrue\ntrue\nnil\n",
        "Invalid argument\n",
        "Invalid argument - custom message\n",
        "Invalid argument @ location - custom message\n",
        "No such file or directory\n",
        "No such file or directory - custom message\n"
    );
    let output = run_example("errors/errno/classes_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_cause_chain_execution() {
    let expected = concat!(
        "nil\n",
        "the consequence\n",
        "Exception\n",
        "the cause\n",
        "ZeroDivisionError\n",
        "true\ntrue\ntrue\n",
        "nil\n"
    );
    let output = run_example("errors/cause/chain.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_cause_chain_parens_execution() {
    let expected = concat!(
        "nil\n",
        "the consequence\n",
        "Exception\n",
        "the cause\n",
        "ZeroDivisionError\n",
        "true\ntrue\ntrue\n",
        "nil\n"
    );
    let output = run_example("errors/cause/chain_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_message_detailed_execution() {
    let expected = "new error (RuntimeError)\nunhandled exception\nStandardError\nnew error (RuntimeError)\nmessage\ntrue\ntrue\na.rb:1: Some runtime error (RuntimeError)\n	from b.rb:2\nTraceback (most recent call last):\n	from b.rb:2\na.rb:1: Some runtime error (RuntimeError)\n<prefix>new error<suffix>\ntrue\n";
    let output = run_example("errors/message/detailed.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_message_detailed_parens_execution() {
    let expected = "new error (RuntimeError)\nunhandled exception\nStandardError\nnew error (RuntimeError)\nmessage\ntrue\ntrue\na.rb:1: Some runtime error (RuntimeError)\n	from b.rb:2\nTraceback (most recent call last):\n	from b.rb:2\na.rb:1: Some runtime error (RuntimeError)\n<prefix>new error<suffix>\ntrue\n";
    let output = run_example("errors/message/detailed_parens.rb");
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
fn test_errors_frozen_modification_execution() {
    let expected = concat!(
        "FrozenError\ntrue\ntrue\n",
        "true\n",
        "can't modify frozen Array: [1, 2]\n",
        "[1, 2]\n",
        "[1, 2]\nfalse\n",
        "true\n",
        "can't modify frozen Object: ...\n"
    );
    let output = run_example("errors/frozen/modification.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_frozen_modification_parens_execution() {
    let expected = concat!(
        "FrozenError\ntrue\ntrue\n",
        "true\n",
        "can't modify frozen Array: [1, 2]\n",
        "[1, 2]\n",
        "[1, 2]\nfalse\n",
        "true\n",
        "can't modify frozen Object: ...\n"
    );
    let output = run_example("errors/frozen/modification_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_full_message_rendering_execution() {
    let expected = "true\ntrue\ntrue\ntrue\na.rb:1: Some runtime error (RuntimeError)\nTraceback (most recent call last):\ntrue\ntrue\nkeywords ignored\npositionals ignored\nsecond\nfirst\nnil\nnil\n";
    let output = run_example("errors/full_message/rendering.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_full_message_rendering_parens_execution() {
    let expected = "true\ntrue\ntrue\ntrue\na.rb:1: Some runtime error (RuntimeError)\nTraceback (most recent call last):\ntrue\ntrue\nkeywords ignored\npositionals ignored\nsecond\nfirst\nnil\nnil\n";
    let output = run_example("errors/full_message/rendering_parens.rb");
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
fn test_errors_inspect_rendering_execution() {
    let expected = concat!(
        "#<Exception: Exception>\n",
        "#<Exception: boom>\n",
        "#<RuntimeError: boom>\n",
        "RuntimeError\n",
        "#<Described: this is from to_s>\n",
        "Silent\n",
        "#<Plain: Plain>\n",
        "true\n"
    );
    let output = run_example("errors/inspect/rendering.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_inspect_rendering_parens_execution() {
    let expected = concat!(
        "#<Exception: Exception>\n",
        "#<Exception: boom>\n",
        "#<RuntimeError: boom>\n",
        "RuntimeError\n",
        "#<Described: this is from to_s>\n",
        "Silent\n",
        "#<Plain: Plain>\n",
        "true\n"
    );
    let output = run_example("errors/inspect/rendering_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_io_wait_constants_execution() {
    let expected = "Errno::EAGAIN\ntrue\ntrue\nErrno::EAGAIN\ntrue\ntrue\nfalse\ntrue\nIO::EAGAINWaitReadable\ntrue\ntrue\n";
    let output = run_example("errors/io_wait/constants.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_io_wait_constants_parens_execution() {
    let expected = "Errno::EAGAIN\ntrue\ntrue\nErrno::EAGAIN\ntrue\ntrue\nfalse\ntrue\nIO::EAGAINWaitReadable\ntrue\ntrue\n";
    let output = run_example("errors/io_wait/constants_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_key_error_named_arguments_execution() {
    let expected = "\"lookup source\"\n:b\nKeyError\nkey not found: :b\n:b\nno key is available\nno receiver is available\n\"text\"\ncan't modify\n";
    let output = run_example("errors/key_error/named_arguments.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_key_error_named_arguments_parens_execution() {
    let expected = "\"lookup source\"\n:b\nKeyError\nkey not found: :b\n:b\nno key is available\nno receiver is available\n\"text\"\ncan't modify\n";
    let output = run_example("errors/key_error/named_arguments_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_load_error_path_execution() {
    let expected = "nil\nnil\n\"file_that_does_not_exist\"\ncannot load such file -- file_that_does_not_exist\nLoadError\ntrue\nfalse\n";
    let output = run_example("errors/load_error/path.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_load_error_path_parens_execution() {
    let expected = "nil\nnil\n\"file_that_does_not_exist\"\ncannot load such file -- file_that_does_not_exist\nLoadError\ntrue\nfalse\n";
    let output = run_example("errors/load_error/path_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_message_dispatch_execution() {
    let expected =
        "Exception\nOuch!\nthis is from to_s\nQuiet\nplain\nfrom a singleton\nthis is from to_s\n";
    let output = run_example("errors/message/dispatch.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_message_dispatch_parens_execution() {
    let expected =
        "Exception\nOuch!\nthis is from to_s\nQuiet\nplain\nfrom a singleton\nthis is from to_s\n";
    let output = run_example("errors/message/dispatch_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_name_error_names_execution() {
    let expected = ":doesnt_exist\n:DoesntExist\n:DoesntExist\n\"invalid_ivar_name\"\n\"invalid_cvar_name\"\n7\n7\nuninitialized class variable @@never_set in Counter\n:@@never_set\n";
    let output = run_example("errors/name_error/names.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_name_error_names_parens_execution() {
    let expected = ":doesnt_exist\n:DoesntExist\n:DoesntExist\n\"invalid_ivar_name\"\n\"invalid_cvar_name\"\n7\n7\nuninitialized class variable @@never_set in Counter\n:@@never_set\n";
    let output = run_example("errors/name_error/names_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_name_error_construction_execution() {
    let expected = "msg\n\"name\"\nno receiver is available\n:name\n\"the receiver\"\njust a message\nnil\n:missing_helper\n\"Caller\"\n:missing_helper\n\"Caller\"\n";
    let output = run_example("errors/name_error/construction.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_name_error_construction_parens_execution() {
    let expected = "msg\n\"name\"\nno receiver is available\n:name\n\"the receiver\"\njust a message\nnil\n:missing_helper\n\"Caller\"\n:missing_helper\n\"Caller\"\n";
    let output = run_example("errors/name_error/construction_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_no_method_error_args_execution() {
    let expected = "[\"args\"]\n\"name\"\nmsg\nnil\n:missing\n[]\n[1, :two, \"three\"]\n:missing\n[1, :two, \"three\"]\n\"Receiver\"\nababab\n-----\nnegative argument\n";
    let output = run_example("errors/no_method_error/args.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_no_method_error_args_parens_execution() {
    let expected = "[\"args\"]\n\"name\"\nmsg\nnil\n:missing\n[]\n[1, :two, \"three\"]\n:missing\n[1, :two, \"three\"]\n\"Receiver\"\nababab\n-----\nnegative argument\n";
    let output = run_example("errors/no_method_error/args_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_receiver_lookup_execution() {
    let expected = "true\ntrue\ntrue\ntrue\ntrue\ntrue\nno receiver is available\n";
    let output = run_example("errors/receiver/lookup.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_receiver_lookup_parens_execution() {
    let expected = "true\ntrue\ntrue\ntrue\ntrue\ntrue\nno receiver is available\n";
    let output = run_example("errors/receiver/lookup_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_stop_iteration_result_execution() {
    let expected = "3\n2\n1\niteration reached an end\n:liftoff\n3\nnil\nnil\n";
    let output = run_example("errors/stop_iteration/result.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_stop_iteration_result_parens_execution() {
    let expected = "3\n2\n1\niteration reached an end\n:liftoff\n3\nnil\nnil\n";
    let output = run_example("errors/stop_iteration/result_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_backtrace_locations_fields_execution() {
    let expected = "true\nObject#inner\n2\ntrue\ntrue\ntrue\n<main>\nObject#inner\nnil\ntrue\nObject#inner\ntrue\n";
    let output = run_example("errors/backtrace_locations/fields.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_backtrace_locations_fields_parens_execution() {
    let expected = "true\nObject#inner\n2\ntrue\ntrue\ntrue\n<main>\nObject#inner\nnil\ntrue\nObject#inner\ntrue\n";
    let output = run_example("errors/backtrace_locations/fields_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_rescue_scope_bare_rescue_execution() {
    let expected =
        "caught\ncaught\ncaught\ncaught\ncaught\ncaught\nException\npassed the bare rescue\n";
    let output = run_example("errors/rescue_scope/bare_rescue.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_rescue_scope_bare_rescue_parens_execution() {
    let expected =
        "caught\ncaught\ncaught\ncaught\ncaught\ncaught\nException\npassed the bare rescue\n";
    let output = run_example("errors/rescue_scope/bare_rescue_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_exit_rescued_execution() {
    let expected = concat!(
        "exit\n0\ntrue\n",
        "42\nfalse\n",
        "-1\nfalse\n",
        "1\nfalse\n",
        "0\ntrue\n",
        "ensure ran\n7\n",
        "SystemExit is not a StandardError\nSystemExit\n",
    );
    let output = run_example("errors/exit_rescued.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_exit_uncaught_execution() {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/errors/exit_uncaught.rb", EXAMPLES_DIR);
    let mut command = Command::new(binary);
    command.current_dir(manifest_dir).arg(&full_path);

    let output = command.output().expect("failed to execute example");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not utf8");

    assert_eq!(stdout, "before exit\n");
    assert_eq!(output.status.code(), Some(5));
}

#[test]
fn test_errors_exit_bang_execution() {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/errors/exit_bang.rb", EXAMPLES_DIR);
    let mut command = Command::new(binary);
    command.current_dir(manifest_dir).arg(&full_path);

    let output = command.output().expect("failed to execute example");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not utf8");

    assert_eq!(stdout, "before exit!\n");
    assert_eq!(output.status.code(), Some(9));
}

#[test]
fn test_errors_exit_status_coercion_execution() {
    let expected = concat!(
        "0\n",
        "8\n",
        "-1\n",
        "0\n",
        "1\n",
        "5\n",
        "-2\n",
        "5\n",
        "no implicit conversion of String into Integer\n",
        "no implicit conversion from nil to integer\n",
        "no implicit conversion of Array into Integer\n",
        "no implicit conversion of Object into Integer\n",
        "3\n",
        "4\n",
        "true\n",
        "true\n"
    );
    let output = run_example("errors/exit_status_coercion.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_exit_status_coercion_parens_execution() {
    let expected = concat!(
        "0\n",
        "8\n",
        "-1\n",
        "0\n",
        "1\n",
        "5\n",
        "-2\n",
        "5\n",
        "no implicit conversion of String into Integer\n",
        "no implicit conversion from nil to integer\n",
        "no implicit conversion of Array into Integer\n",
        "no implicit conversion of Object into Integer\n",
        "3\n",
        "4\n",
        "true\n",
        "true\n"
    );
    let output = run_example("errors/exit_status_coercion_parens.rb");
    assert_eq!(output, expected);
}
