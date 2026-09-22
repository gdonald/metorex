// Ending the program, from inside and out.

use super::super::run_example;
use super::*;
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

#[test]
fn test_errors_main_using_scope_execution() {
    let expected = "main.using is permitted only at toplevel\ntoplevel using allowed\n";
    let output = run_example("errors/using_scope/main_using.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_main_using_scope_parens_execution() {
    let expected = "main.using is permitted only at toplevel\ntoplevel using allowed\n";
    let output = run_example("errors/using_scope/main_using_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_system_exit_new_execution() {
    let expected = concat!(
        "42\n",
        "message\n",
        "false\n",
        "0\n",
        "1\n",
        "SystemExit\n",
        "42\n",
        "0\n",
        "message\n",
        "0\n",
        "SystemExit\n",
        "8\n",
        "false\n"
    );
    let output = run_example("errors/system_exit_new.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_system_exit_new_parens_execution() {
    let expected = concat!(
        "42\n",
        "message\n",
        "false\n",
        "0\n",
        "1\n",
        "SystemExit\n",
        "42\n",
        "0\n",
        "message\n",
        "0\n",
        "SystemExit\n",
        "8\n",
        "false\n"
    );
    let output = run_example("errors/system_exit_new_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_errors_system_exit_subclass_exits_silently() {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/errors/system_exit_subclass.rb", EXAMPLES_DIR);
    let mut command = Command::new(binary);
    command.current_dir(manifest_dir).arg(&full_path);

    let output = command.output().expect("failed to execute example");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not utf8");
    let stderr = String::from_utf8(output.stderr).expect("stderr was not utf8");

    assert_eq!(stdout, "before raise\n");
    assert_eq!(stderr, "");
    assert_eq!(output.status.code(), Some(8));
}
