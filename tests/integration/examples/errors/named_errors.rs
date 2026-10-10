// The errors named for what went wrong.

use super::super::run_example;
use super::*;
#[test]
fn test_errors_errno_classes_execution() {
    let expected = concat!(
        "Errno::EINVAL\n",
        "SystemCallError\n",
        "22\n2\ntrue\n",
        "Errno::EINVAL\n",
        "22\ntrue\ntrue\nfalse\n",
        "Errno::ENOENT\n",
        "No such file or directory - boom\ntrue\ntrue\nnil\n",
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
        "No such file or directory - boom\ntrue\ntrue\nnil\n",
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
fn test_errors_syntax_error_path_execution() {
    let expected = concat!(
        "nil\n",
        "nil\n",
        "SyntaxError\n",
        "\"speccing.rb\"\n",
        "\"(eval at tests/_examples/errors/syntax_error/path.rb:12)\"\n",
        "SyntaxError\n",
        "true\n"
    );
    let output = run_example("errors/syntax_error/path.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_syntax_error_path_parens_execution() {
    let expected = concat!(
        "nil\n",
        "nil\n",
        "SyntaxError\n",
        "\"speccing.rb\"\n",
        "\"(eval at tests/_examples/errors/syntax_error/path_parens.rb:12)\"\n",
        "SyntaxError\n",
        "true\n"
    );
    let output = run_example("errors/syntax_error/path_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_system_call_error_execution() {
    let expected = concat!(
        "Errno::EINVAL\n",
        "Invalid argument\n",
        "Invalid argument - custom message\n",
        "Invalid argument @ location - custom message\n",
        "Invalid argument\n",
        "SystemCallError\n",
        "16777216\n",
        "true\n",
        "nil\n",
        "message\n",
        "42\n",
        "Errno::ENOENT\n",
        "Errno::ENOENT\n",
        "-1\n",
        "ArgumentError\n",
        "no implicit conversion of Symbol into String\n",
        "no implicit conversion of String into Integer\n",
        "can't convert 2.9+1i into Integer\n"
    );
    let output = run_example("errors/errno/system_call_error.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_system_call_error_parens_execution() {
    let expected = concat!(
        "Errno::EINVAL\n",
        "Invalid argument\n",
        "Invalid argument - custom message\n",
        "Invalid argument @ location - custom message\n",
        "Invalid argument\n",
        "SystemCallError\n",
        "16777216\n",
        "true\n",
        "nil\n",
        "message\n",
        "42\n",
        "Errno::ENOENT\n",
        "Errno::ENOENT\n",
        "-1\n",
        "ArgumentError\n",
        "no implicit conversion of Symbol into String\n",
        "no implicit conversion of String into Integer\n",
        "can't convert 2.9+1i into Integer\n"
    );
    let output = run_example("errors/errno/system_call_error_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_name_and_method_messages_execution() {
    let output = run_example("errors/name_and_method_messages.rb");
    assert_eq!(output, NAME_AND_METHOD_MESSAGES_OUTPUT);
}

#[test]
fn test_errors_name_and_method_messages_no_parens_execution() {
    let output = run_example("errors/name_and_method_messages_no_parens.rb");
    assert_eq!(output, NAME_AND_METHOD_MESSAGES_OUTPUT);
}

const SYNTAX_SUGGEST_REPORTS: &str = concat!(
    "\"constant\"\n",
    "--> missing_end.rb\n",
    "\n",
    "Unmatched keyword, missing `end' ?\n",
    "\n",
    "  2  class Kennel\n",
    "> 3    def admit(dog)\n",
    "> 6    def release(dog)\n",
    "> 8    end\n",
    "  9  end\n",
    "true\n",
    "false\n",
    "--> extra_end.rb\n",
    "\n",
    "Unmatched `end', missing keyword (`do', `def`, `if`, etc.) ?\n",
    "\n",
    "  2  def invoice_total(items)\n",
    "> 3    items.sum(&:price)\n",
    "> 4    end\n",
    "  5  end\n",
    "true\n",
    "false\n",
    "true\n",
    "true\n",
);

#[test]
fn test_errors_syntax_suggest_reports_execution() {
    let output = run_example("errors/syntax_error/syntax_suggest_reports.rb");
    assert_eq!(output, SYNTAX_SUGGEST_REPORTS);
}

#[test]
fn test_errors_syntax_suggest_reports_no_parens_execution() {
    let output = run_example("errors/syntax_error/syntax_suggest_reports_no_parens.rb");
    assert_eq!(output, SYNTAX_SUGGEST_REPORTS);
}

/// What a program that does not parse writes to standard error, and its
/// exit status, run with `flags` before the program.
fn unparsable_program_report(path: &str, flags: &[&str]) -> (String, Option<i32>) {
    let binary = env!("CARGO_BIN_EXE_metorex");
    // syntax_suggest gives its search one second by default, which a debug
    // build under a loaded machine can run past, so the report is given
    // all the time it needs.
    let output = std::process::Command::new(binary)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("SYNTAX_SUGGEST_TIMEOUT", "60")
        .args(flags)
        .arg(format!("{}/{}", EXAMPLES_DIR, path))
        .output()
        .expect("failed to execute example");
    (
        String::from_utf8(output.stderr).expect("stderr was not utf8"),
        output.status.code(),
    )
}

#[test]
fn a_program_missing_an_end_reports_the_lines_around_it() {
    let (stderr, status) = unparsable_program_report("errors/syntax_error/missing_end.rb", &[]);
    let expected = concat!(
        "tests/_examples/errors/syntax_error/missing_end.rb: ",
        "--> tests/_examples/errors/syntax_error/missing_end.rb\n",
        "\n",
        "Unmatched keyword, missing `end' ?\n",
        "\n",
        "  2  class Kennel\n",
        "> 3    def admit(dog)\n",
        "> 6    def release(dog)\n",
        "> 8    end\n",
        "  9  end\n",
        "\n",
    );
    assert_eq!(
        (stderr.starts_with(expected), status),
        (true, Some(1)),
        "{stderr}"
    );
}

#[test]
fn a_program_with_an_extra_end_reports_the_lines_around_it() {
    let (stderr, status) = unparsable_program_report("errors/syntax_error/extra_end.rb", &[]);
    let expected = concat!(
        "tests/_examples/errors/syntax_error/extra_end.rb: ",
        "--> tests/_examples/errors/syntax_error/extra_end.rb\n",
        "\n",
        "Unmatched `end', missing keyword (`do', `def`, `if`, etc.) ?\n",
        "\n",
        "  2  def invoice_total(items)\n",
        "> 3    items.sum(&:price)\n",
        "> 4    end\n",
        "  5  end\n",
        "\n",
    );
    assert_eq!(
        (stderr.starts_with(expected), status),
        (true, Some(1)),
        "{stderr}"
    );
}

#[test]
fn disabling_syntax_suggest_leaves_the_report_to_the_syntax_error() {
    for flag in ["--disable=syntax_suggest", "--disable=gems"] {
        let (stderr, status) =
            unparsable_program_report("errors/syntax_error/extra_end.rb", &[flag]);
        assert_eq!(
            (stderr.contains("-->"), status),
            (false, Some(1)),
            "{flag}: {stderr}"
        );
    }
}
