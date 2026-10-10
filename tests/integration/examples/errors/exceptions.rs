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
    let expected = "Example 1: Different exception types\nCaught RuntimeError: Runtime error occurred\nCaught TypeError: Type mismatch\nCaught ArgumentError: Invalid value\n\nExample 2: Catching StandardError\nCaught as StandardError: A runtime error\nCaught as StandardError: A type error\n\nExample 3: Specific to general exception handling\nSpecific handler for RuntimeError: Runtime issue\nSpecific handler for TypeError: Type issue\nGeneral handler for StandardError: Value issue\n\nExample 4: Exception type checking\nRuntimeError is a StandardError: true\nError message: Test error\n";
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
        "RuntimeError\n\"\"\n",
        "true\ntrue\n"
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
        "RuntimeError\n\"\"\n",
        "true\ntrue\n"
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

/// The expected output of both `errors/recursing_too_deep` variants.
const RECURSING_TOO_DEEP_OUTPUT: &str = concat!(
    "SystemStackError\n",
    "\"stack level too deep\"\n",
    ":carried_on\n"
);

#[test]
fn test_errors_recursing_too_deep_execution() {
    let output = run_example("errors/recursing_too_deep.rb");
    assert_eq!(output, RECURSING_TOO_DEEP_OUTPUT);
}

#[test]
fn test_errors_recursing_too_deep_no_parens_execution() {
    let output = run_example("errors/recursing_too_deep_no_parens.rb");
    assert_eq!(output, RECURSING_TOO_DEEP_OUTPUT);
}

const DID_YOU_MEAN_SUGGESTIONS: &str = concat!(
    "[NameError, [:first_name]]\n",
    "\"undefined local variable or method 'firts_name' for main (NameError)\\nDid you mean?  first_name\"\n",
    "[NameError, [:first_name]]\n",
    "\"undefined local variable or method 'firts_name' for main (NameError)\\nDid you mean?  first_name\"\n",
    "[NameError, [\"String\"]]\n",
    "\"uninitialized constant Strng (NameError)\\nDid you mean?  String\"\n",
    "[NoMethodError, [:capacity]]\n",
    "\"undefined method 'capcity' for an instance of Shelf (NoMethodError)\\nDid you mean?  capacity\"\n",
    "[NoMethodError, [:label]]\n",
    "\"undefined method 'lable' for an instance of Shelf (NoMethodError)\\nDid you mean?  label\"\n",
    "[KeyError, [\":name\"]]\n",
    "\"key not found: :nme (KeyError)\\nDid you mean?  :name\"\n",
    "[KeyError, [\"\\\"alpha\\\"\"]]\n",
    "\"key not found: \\\"alpah\\\" (KeyError)\\nDid you mean?  \\\"alpha\\\"\"\n",
    "[KeyError, [\":name\"]]\n",
    "\"key not found: \\\"name\\\" (KeyError)\\nDid you mean?  :name\"\n",
    "[LoadError, [\"fileutils\"]]\n",
    "\"cannot load such file -- fileutil (LoadError)\\nDid you mean?  fileutils\"\n",
    "[NoMatchingPatternKeyError, [\":name\"]]\n",
    "\"{name: 1}: key not found: :nmae (NoMatchingPatternKeyError)\\nDid you mean?  :name\"\n",
    "[NoMethodError, []]\n",
    "\"undefined method 'nothing_close' for an instance of Shelf (NoMethodError)\"\n",
    "\"undefined method 'capcity' for an instance of Shelf (NoMethodError)\"\n",
    "\"\\e[1mundefined method 'capcity' for an instance of Shelf (\\e[1;4mNoMethodError\\e[m\\e[1m)\\e[m\\n\\e[1mDid you mean?  capacity\\e[m\"\n",
    "\"undefined method 'capcity' for an instance of Shelf\"\n",
    "[\"apple\"]\n",
    "[\"banana\"]\n",
    "[]\n",
    "[\"net/http\", \"net/ftp\"]\n",
    "\"\\nDid you mean?  first\\n               second\"\n",
    "\"\"\n",
    "0.9611\n",
    "3\n",
    "missing (KeyError)\n",
    "Did you mean?  :name\n",
    "missing (KeyError)\n",
);

#[test]
fn test_errors_did_you_mean_suggestions_execution() {
    let output = run_example("errors/did_you_mean_suggestions.rb");
    assert_eq!(output, DID_YOU_MEAN_SUGGESTIONS);
}

#[test]
fn test_errors_did_you_mean_suggestions_no_parens_execution() {
    let output = run_example("errors/did_you_mean_suggestions_no_parens.rb");
    assert_eq!(output, DID_YOU_MEAN_SUGGESTIONS);
}

const NAME_ERROR_DETAILS: &str = concat!(
    "[main, [:total, :error, :pantry], true]\n",
    "\"label for misspelled_total\"\n",
    "\"[annotated] undefined local variable or method 'misspelled_total' for main (NameError)\"\n",
    "[:inside_block, :outer]\n",
    "[Pantry, []]\n",
    "[NoMethodError, \"undefined method 'missing_call' for main\", true, true]\n",
    "[\"undefined method 'missing_call' for main\", true]\n",
    "[\"undefined method 'counted' for an instance of Pantry\", true]\n",
    "[\"undefined method 'counted' for an instance of Pantry\", false]\n",
    "[\"undefined method 'missing_call' for an instance of String\", false, true]\n",
    "true\n",
    "false\n",
    "true\n",
    "true\n",
    "false\n",
);

#[test]
fn test_errors_name_error_details_execution() {
    let output = run_example("errors/name_error_details.rb");
    assert_eq!(output, NAME_ERROR_DETAILS);
}

#[test]
fn test_errors_name_error_details_no_parens_execution() {
    let output = run_example("errors/name_error_details_no_parens.rb");
    assert_eq!(output, NAME_ERROR_DETAILS);
}

const PRIVATE_KERNEL_CALLS: &str = concat!(
    "[NoMethodError, \"private method 'puts' called for an instance of Object\"]\n",
    "[NoMethodError, \"private method 'raise' called for nil\"]\n",
    "[NoMethodError, \"private method 'format' called for an instance of Array\"]\n",
    "spoken\n",
    "nil\n",
    "\"forwarded puts with [\\\"anything\\\"]\"\n",
    "nil\n",
    "[ArgumentError, \"wrong number of arguments (given 1, expected 0)\"]\n",
    "[NoMethodError, \"private method 'initialize' called for an instance of Object\"]\n",
    "nil\n",
    "[NoMethodError, \"private method 'absent' called for an instance of Object\"]\n",
    "[ArgumentError, \"no method name given\"]\n",
    "[ArgumentError, \"method name must be a Symbol but String is given\"]\n",
    "true\n",
    "false\n",
    "true\n",
    "[false, true]\n",
    "nil\n",
    "nil\n",
);

#[test]
fn test_errors_private_kernel_calls_execution() {
    let output = run_example("errors/private_kernel_calls.rb");
    assert_eq!(output, PRIVATE_KERNEL_CALLS);
}

#[test]
fn test_errors_private_kernel_calls_no_parens_execution() {
    let output = run_example("errors/private_kernel_calls_no_parens.rb");
    assert_eq!(output, PRIVATE_KERNEL_CALLS);
}

const ERROR_HIGHLIGHT_SPOTS: &str = concat!(
    "undefined local variable or method 'totl' for main (NameError)\n",
    "\n",
    "report { total = totl + 1 }\n",
    "                 ^^^^\n",
    "Did you mean?  total\n",
    "--\n",
    "undefined method 'length' for nil (NoMethodError)\n",
    "\n",
    "report { order = nil; order.length }\n",
    "                           ^^^^^^^\n",
    "--\n",
    "String can't be coerced into Integer (TypeError)\n",
    "\n",
    "report { sum = 1 + \"2\" }\n",
    "                   ^^^\n",
    "--\n",
    "wrong number of arguments (given 1, expected 2) (ArgumentError)\n",
    "\n",
    "    caller: FILE:18\n",
    "    | report { shipping_cost(4) }\n",
    "               ^^^^^^^^^^^^^\n",
    "    callee: FILE:8\n",
    "    | def shipping_cost(weight, zone) = weight * zone\n",
    "          ^^^^^^^^^^^^^\n",
    "--\n",
    "undefined method '[]' for nil (NoMethodError)\n",
    "\n",
    "report { rows = nil; rows[0] }\n",
    "                         ^^^\n",
    "--\n",
    "undefined method 'label=' for nil (NoMethodError)\n",
    "\n",
    "report { nil.label = \"fragile\" }\n",
    "            ^^^^^^^^\n",
    "--\n",
    "undefined method 'nope' for an instance of Integer (NoMethodError)\n",
    "\n",
    "report { [1].map(&:nope) }\n",
    "            ^^^^\n",
    "--\n",
    "undefined method 'round' for nil (NoMethodError)\n",
    "\n",
    "report { Parcel.new.weight.round(nil) }\n",
    "                          ^^^^^^\n",
    "--\n",
    "no receiver given (ArgumentError)\n",
    "--\n",
    "comparison of String with 2 failed (ArgumentError)\n",
    "\n",
    "report { Comparable.instance_method(:clamp).bind_call(1, \"x\", 2) }\n",
    "                                                      ^^^^^^^^^\n",
    "--\n",
    "[27, 12, 27, 19]\n",
    "\"  Parcel.new.volume\\n\"\n",
);

#[test]
fn test_errors_error_highlight_spots_execution() {
    let output = run_example("errors/error_highlight_spots.rb");
    assert_eq!(output, ERROR_HIGHLIGHT_SPOTS);
}

const ERROR_HIGHLIGHT_SPOTS_NO_PARENS: &str = concat!(
    "undefined local variable or method 'totl' for main (NameError)\n",
    "\n",
    "report { total = totl + 1 }\n",
    "                 ^^^^\n",
    "Did you mean?  total\n",
    "--\n",
    "undefined method 'length' for nil (NoMethodError)\n",
    "\n",
    "report { order = nil; order.length }\n",
    "                           ^^^^^^^\n",
    "--\n",
    "String can't be coerced into Integer (TypeError)\n",
    "\n",
    "report { sum = 1 + \"2\" }\n",
    "                   ^^^\n",
    "--\n",
    "wrong number of arguments (given 1, expected 2) (ArgumentError)\n",
    "\n",
    "    caller: FILE:20\n",
    "    | report { shipping_cost 4 }\n",
    "               ^^^^^^^^^^^^^\n",
    "    callee: FILE:8\n",
    "    | def shipping_cost weight, zone\n",
    "          ^^^^^^^^^^^^^\n",
    "--\n",
    "undefined method '[]' for nil (NoMethodError)\n",
    "\n",
    "report { rows = nil; rows[0] }\n",
    "                         ^^^\n",
    "--\n",
    "undefined method 'label=' for nil (NoMethodError)\n",
    "\n",
    "report { nil.label = \"fragile\" }\n",
    "            ^^^^^^^^\n",
    "--\n",
    "undefined method 'nope' for an instance of Integer (NoMethodError)\n",
    "\n",
    "report { [1].map &:nope }\n",
    "            ^^^^\n",
    "--\n",
    "undefined method 'round' for nil (NoMethodError)\n",
    "\n",
    "report { Parcel.new.weight.round nil }\n",
    "                          ^^^^^^\n",
    "--\n",
    "no receiver given (ArgumentError)\n",
    "--\n",
    "comparison of String with 2 failed (ArgumentError)\n",
    "\n",
    "report { Comparable.instance_method(:clamp).bind_call 1, \"x\", 2 }\n",
    "                                                      ^^^^^^^^^\n",
    "--\n",
    "[29, 12, 29, 19]\n",
    "\"  Parcel.new.volume\\n\"\n",
);

#[test]
fn test_errors_error_highlight_spots_no_parens_execution() {
    let output = run_example("errors/error_highlight_spots_no_parens.rb");
    assert_eq!(output, ERROR_HIGHLIGHT_SPOTS_NO_PARENS);
}

/// Run an example that ends on an uncaught error, answering what it printed
/// to standard output and to standard error, and its exit status.
fn run_failing_example(path: &str) -> (String, String, Option<i32>) {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = Command::new(binary)
        .current_dir(manifest_dir)
        .arg(format!("{}/{}", EXAMPLES_DIR, path))
        .output()
        .expect("failed to execute example");
    (
        String::from_utf8(output.stdout).expect("stdout was not utf8"),
        String::from_utf8(output.stderr).expect("stderr was not utf8"),
        output.status.code(),
    )
}

/// The report of an uncaught error past its first line, which names the
/// file and line the error was raised on.
fn report_past_header(stderr: &str) -> (&str, &str) {
    let (header, rest) = stderr.split_once('\n').expect("a report line");
    let message = header
        .split_once(": ")
        .map_or(header, |(_, message)| message);
    (message, rest)
}

#[test]
fn test_errors_error_highlight_uncaught_execution() {
    let (stdout, stderr, status) = run_failing_example("errors/error_highlight_uncaught.rb");
    assert_eq!(
        (stdout.as_str(), report_past_header(&stderr), status),
        (
            "packing\n",
            (
                "undefined method 'length' for nil (NoMethodError)",
                "\nputs(order.length)\n          ^^^^^^^\n"
            ),
            Some(1)
        )
    );
}

#[test]
fn test_errors_error_highlight_uncaught_no_parens_execution() {
    let (stdout, stderr, status) =
        run_failing_example("errors/error_highlight_uncaught_no_parens.rb");
    assert_eq!(
        (stdout.as_str(), report_past_header(&stderr), status),
        (
            "packing\n",
            (
                "undefined method 'length' for nil (NoMethodError)",
                "\nputs order.length\n          ^^^^^^^\n"
            ),
            Some(1)
        )
    );
}

const EVAL_AND_INTERNAL_FRAMES_OUTPUT: &str = concat!(
    "\"FILE:7:in 'Object#run_snippet'\"\n",
    "[\"FILE:10:in 'block in <main>'\", \"FILE:10:in 'Kernel#tap'\", \"FILE:10:in '<main>'\"]\n",
    "#<NameError: method 'secret' for class 'Vault' is private>\n",
    "#<NameError: method 'guarded' for class 'Vault' is protected>\n",
    "#<NameError: method 'exit' for class 'Object' is private>\n",
);

#[test]
fn test_errors_eval_and_internal_frames_execution() {
    let output = run_example("errors/eval_and_internal_frames.rb");
    assert_eq!(output, EVAL_AND_INTERNAL_FRAMES_OUTPUT);
}

const EVAL_AND_INTERNAL_FRAMES_NO_PARENS_OUTPUT: &str = concat!(
    "\"FILE:7:in 'Object#run_snippet'\"\n",
    "[\"FILE:12:in 'block in <main>'\", \"FILE:12:in 'Kernel#tap'\", \"FILE:12:in '<main>'\"]\n",
    "#<NameError: method 'secret' for class 'Vault' is private>\n",
    "#<NameError: method 'guarded' for class 'Vault' is protected>\n",
    "#<NameError: method 'exit' for class 'Object' is private>\n",
);

#[test]
fn test_errors_eval_and_internal_frames_no_parens_execution() {
    let output = run_example("errors/eval_and_internal_frames_no_parens.rb");
    assert_eq!(output, EVAL_AND_INTERNAL_FRAMES_NO_PARENS_OUTPUT);
}

/// The expected output of both `errors/builtin_error_ancestry` variants.
const BUILTIN_ERROR_ANCESTRY: &str = concat!("nil\n", "true\n", "true\n", "NotImplementedError\n");

#[test]
fn test_errors_builtin_error_ancestry_execution() {
    let output = run_example("errors/builtin_error_ancestry.rb");
    assert_eq!(output, BUILTIN_ERROR_ANCESTRY);
}

#[test]
fn test_errors_builtin_error_ancestry_no_parens_execution() {
    let output = run_example("errors/builtin_error_ancestry_no_parens.rb");
    assert_eq!(output, BUILTIN_ERROR_ANCESTRY);
}
