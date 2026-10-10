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

const OBJECT_MIXIN: &str = concat!(
    "\"disk full\"\n",
    "\"disk full\"\n",
    "true\n",
    "\"bad size\"\n",
    "\"own message\"\n",
    "\"reported ab\"\n",
    "\"reported \"\n",
    "true\n",
    "[\"\", \"true\", \"false\"]\n",
    "nil\n",
    "true\n",
    "false\n",
    "reported text\n",
    "|true|false\n",
);

#[test]
fn test_errors_message_object_mixin_execution() {
    let output = run_example("errors/message/object_mixin.rb");
    assert_eq!(output, OBJECT_MIXIN);
}

#[test]
fn test_errors_message_object_mixin_parens_execution() {
    let output = run_example("errors/message/object_mixin_parens.rb");
    assert_eq!(output, OBJECT_MIXIN);
}

/// The expected output of both `errors/backtrace/native_frames` variants,
/// with `FILE` standing for the example's own name.
const NATIVE_FRAMES: &str = concat!(
    "[\"FILE:9:in 'Kernel#Integer'\", \"FILE:9:in 'Plain#converts'\", \"FILE:15:in '<main>'\"]\n",
    "[\"FILE:21:in 'Kernel#Integer'\", \"FILE:21:in 'block in <main>'\", \"FILE:21:in 'BasicObject#instance_exec'\", \"FILE:21:in '<main>'\"]\n",
    "[\"FILE:27:in 'Kernel#Integer'\", \"FILE:27:in 'block in <main>'\", \"FILE:27:in 'BasicObject#instance_eval'\", \"FILE:27:in '<main>'\"]\n",
    "[\"FILE:33:in 'Kernel#Integer'\", \"FILE:33:in 'block in <main>'\", \"FILE:33:in 'Module#class_eval'\", \"FILE:33:in '<main>'\"]\n",
    "[\"FILE:39:in 'Kernel#Integer'\", \"FILE:39:in 'block (2 levels) in <main>'\", \"FILE:39:in 'Module#class_exec'\", \"FILE:39:in 'block in <main>'\", \"FILE:38:in 'Array#each'\", \"FILE:38:in '<main>'\"]\n",
    "[\"FILE:12:in 'Object#calls_missing'\", \"FILE:45:in 'block (2 levels) in <main>'\", \"FILE:45:in 'Array#each'\", \"FILE:45:in 'block in <main>'\", \"FILE:44:in 'Array#each'\", \"FILE:44:in '<main>'\"]\n",
    "[\"FILE:51:in 'Integer#/'\", \"FILE:51:in 'block in <main>'\", \"FILE:51:in 'Array#map'\", \"FILE:51:in '<main>'\"]\n",
    "[\"FILE:56:in 'Kernel#Integer'\", \"FILE:56:in 'block (2 levels) in <main>'\", \"FILE:56:in 'BasicObject#instance_exec'\", \"FILE:56:in 'block in <main>'\"]\n",
    "5\n",
    "[\"FILE:71:in 'Kernel#Integer'\", \"FILE:71:in 'block in <main>'\"]\n",
);

#[test]
fn test_errors_backtrace_native_frames_execution() {
    let output = run_example("errors/backtrace/native_frames.rb");
    assert_eq!(output, NATIVE_FRAMES.replace("FILE", "native_frames.rb"));
}

#[test]
fn test_errors_backtrace_native_frames_no_parens_execution() {
    let output = run_example("errors/backtrace/native_frames_no_parens.rb");
    assert_eq!(
        output,
        NATIVE_FRAMES.replace("FILE", "native_frames_no_parens.rb")
    );
}

/// The expected output of `errors/backtrace/names_as_given`, where `FILE`
/// stands for the path the example was run by.
const NAMES_AS_GIVEN: &str = concat!(
    "[\"FILE:4:in '<main>'\"]\n",
    "[\"FILE:10:in 'Object#go'\", \"FILE:14:in '<main>'\"]\n",
);

#[test]
fn test_errors_backtrace_names_as_given_execution() {
    let output = run_example("errors/backtrace/names_as_given.rb");
    assert_eq!(
        output,
        NAMES_AS_GIVEN.replace("FILE", "tests/_examples/errors/backtrace/names_as_given.rb")
    );
}

#[test]
fn test_errors_backtrace_names_as_given_no_parens_execution() {
    let output = run_example("errors/backtrace/names_as_given_no_parens.rb");
    assert_eq!(
        output,
        NAMES_AS_GIVEN.replace(
            "FILE",
            "tests/_examples/errors/backtrace/names_as_given_no_parens.rb"
        )
    );
}

/// What metorex writes to stderr running `arguments`.
fn stderr_running(arguments: &[&str]) -> String {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_metorex"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(arguments)
        .output()
        .expect("failed to run metorex");
    String::from_utf8(output.stderr).expect("stderr was not utf8")
}

#[test]
fn test_an_uncaught_name_error_in_a_file_names_its_line() {
    let written = stderr_running(&["tests/_examples/errors/backtrace/uncaught_name.rb"]);
    assert_eq!(
        written,
        concat!(
            "tests/_examples/errors/backtrace/uncaught_name.rb:3:in '<main>': ",
            "undefined local variable or method 'nope' for main (NameError)\n",
            "\n",
            "nope\n",
            "^^^^\n",
        )
    );
}

#[test]
fn test_an_uncaught_name_error_in_code_given_with_e_names_its_line_alone() {
    let written = stderr_running(&["-e", "x = 1\nnope"]);
    assert_eq!(
        written,
        "-e:2:in '<main>': undefined local variable or method 'nope' for main (NameError)\n"
    );
}
