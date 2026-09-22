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
        "nil\n",
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
        "nil\n",
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
