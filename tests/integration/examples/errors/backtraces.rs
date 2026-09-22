// The frames an exception carries, and how they read.

use super::super::run_example;
use super::*;
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
fn test_errors_backtrace_method_execution() {
    let output = run_example("errors/backtrace_method.rb");
    assert!(output.contains("Caught: Error in inner method"));
    assert!(output.contains("Backtrace array length:"));
    assert!(output.contains("First frame:"));
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
fn test_errors_backtrace_locations_fields_execution() {
    let expected =
        "true\nObject#inner\n2\ntrue\ntrue\ntrue\n<main>\ninner\nnil\ntrue\nObject#inner\ntrue\n";
    let output = run_example("errors/backtrace_locations/fields.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_backtrace_locations_fields_parens_execution() {
    let expected =
        "true\nObject#inner\n2\ntrue\ntrue\ntrue\n<main>\ninner\nnil\ntrue\nObject#inner\ntrue\n";
    let output = run_example("errors/backtrace_locations/fields_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_backtrace_frame_labels_execution() {
    let output = run_example("errors/backtrace/frame_labels.rb");
    assert_eq!(output, BACKTRACE_FRAME_LABELS_OUTPUT);
}

#[test]
fn test_errors_backtrace_frame_labels_no_parens_execution() {
    let output = run_example("errors/backtrace/frame_labels_no_parens.rb");
    assert_eq!(output, BACKTRACE_FRAME_LABELS_OUTPUT);
}
