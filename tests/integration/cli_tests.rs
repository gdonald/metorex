// CLI integration tests

use std::process::Command;

fn metorex_cmd() -> Command {
    let binary = env!("CARGO_BIN_EXE_metorex");
    Command::new(binary)
}

// ============================================================================
// --version
// ============================================================================

#[test]
fn cli_version_flag() {
    let output = metorex_cmd()
        .arg("--version")
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("metorex"));
    assert!(stdout.contains("0.1.0"));
}

// ============================================================================
// --help
// ============================================================================

#[test]
fn cli_help_flag() {
    let output = metorex_cmd()
        .arg("--help")
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Metorex"));
    assert!(stdout.contains("--ast"));
    assert!(stdout.contains("--debug"));
    assert!(stdout.contains("--repl"));
}

// ============================================================================
// --ast
// ============================================================================

#[test]
fn cli_ast_flag_dumps_ast() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["--ast", "tests/_examples/basics/each_block.rb"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Expression"));
}

// ============================================================================
// --debug
// ============================================================================

#[test]
fn cli_debug_flag_prints_debug_info() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["--debug", "tests/_examples/basics/each_block.rb"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("[debug]"));
    assert!(stderr.contains("Tokens:"));
    assert!(stderr.contains("Statements:"));
}

// ============================================================================
// File execution
// ============================================================================

#[test]
fn cli_execute_file() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .arg("tests/_examples/basics/each_block.rb")
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
}

#[test]
fn cli_nonexistent_file_exits_with_error() {
    let output = metorex_cmd()
        .arg("nonexistent_file_xyz.rb")
        .output()
        .expect("failed to execute");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error"));
}

#[test]
fn cli_file_with_syntax_error_exits_with_error() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .arg("tests/_examples/execute_file/syntax_error.rb")
        .output()
        .expect("failed to execute");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("SyntaxError"));
}

// ============================================================================
// --test (test discovery)
// ============================================================================

#[test]
fn cli_test_flag_discovers_and_runs_tests() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["--test", "tests/_examples/test_discovery/nested"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("1 passed"));
    assert!(stdout.contains("0 failed"));
}

#[test]
fn cli_test_flag_shows_discovery_count() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["--test", "tests/_examples/test_discovery/nested"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Discovered 1 test file(s)"));
}

#[test]
fn cli_test_flag_exits_nonzero_on_failure() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["--test", "tests/_examples/test_discovery/failing"])
        .output()
        .expect("failed to execute");
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("FAIL"));
    assert!(stdout.contains("2 failed"));
}

#[test]
fn cli_test_flag_nonexistent_dir_fails() {
    let output = metorex_cmd()
        .args(["--test", "nonexistent_dir_xyz"])
        .output()
        .expect("failed to execute");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error") || stderr.contains("Error"));
}

#[test]
fn cli_test_flag_appears_in_help() {
    let output = metorex_cmd()
        .arg("--help")
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--test"));
}

// ============================================================================
// -v (Ruby-compatible version)
// ============================================================================

#[test]
fn cli_v_flag_prints_ruby_version() {
    let output = metorex_cmd().arg("-v").output().expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("4.0.1"));
    assert!(stdout.contains("metorex"));
    assert!(stdout.contains("(ruby-compatible)"));
}

// ============================================================================
// -e (evaluate inline code)
// ============================================================================

#[test]
fn cli_e_flag_evaluates_code() {
    let output = metorex_cmd()
        .args(["-e", "puts 1 + 2"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "3");
}

#[test]
fn cli_e_flag_syntax_error_exits_nonzero() {
    let output = metorex_cmd()
        .args(["-e", "def"])
        .output()
        .expect("failed to execute");
    assert!(!output.status.success());
}

// ============================================================================
// --disable (ignored Ruby flag)
// ============================================================================

#[test]
fn cli_disable_flag_ignored() {
    let output = metorex_cmd()
        .args(["--disable=gems", "-e", "puts 42"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "42");
}

// ============================================================================
// -I flag (prepend to $LOAD_PATH)
// ============================================================================

#[test]
fn cli_i_flag_prepends_load_path() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let lib_path = format!("{}/tests/_examples/cli_flags/lib", manifest_dir);
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["-I", &lib_path, "-e", "require \"greet\"\ngreet(\"world\")"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "hello world");
}

#[test]
fn cli_i_flag_with_file_execution() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let lib_path = format!("{}/tests/_examples/cli_flags/lib", manifest_dir);
    let script = format!("{}/tests/_examples/cli_flags/use_greet.rb", manifest_dir);
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["-I", &lib_path, &script])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "hello world");
}

#[test]
fn cli_multiple_i_flags() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let lib_path = format!("{}/tests/_examples/cli_flags/lib", manifest_dir);
    let script = format!("{}/tests/_examples/cli_flags/use_both.rb", manifest_dir);
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["-I", &lib_path, &script])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "hello world\n42");
}

// ============================================================================
// -r flag (require library before executing)
// ============================================================================

#[test]
fn cli_r_flag_requires_library() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let lib_path = format!("{}/tests/_examples/cli_flags/lib", manifest_dir);
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["-I", &lib_path, "-r", "greet", "-e", "greet(\"from -r\")"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "hello from -r");
}

#[test]
fn cli_r_flag_multiple_requires() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let lib_path = format!("{}/tests/_examples/cli_flags/lib", manifest_dir);
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args([
            "-I",
            &lib_path,
            "-r",
            "greet",
            "-r",
            "math_helpers",
            "-e",
            "greet(\"test\")\nputs double(7)",
        ])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "hello test\n14");
}

#[test]
fn cli_r_flag_with_file_execution() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let lib_path = format!("{}/tests/_examples/cli_flags/lib", manifest_dir);
    let script = format!("{}/tests/_examples/cli_flags/use_greet.rb", manifest_dir);
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["-I", &lib_path, "-r", "math_helpers", &script])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "hello world");
}

#[test]
fn cli_r_flag_missing_library_fails() {
    let output = metorex_cmd()
        .args(["-r", "nonexistent_lib", "-e", "puts 1"])
        .output()
        .expect("failed to execute");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("cannot load such file"));
    assert!(stderr.contains("nonexistent_lib"));
}

#[test]
fn cli_i_flag_no_parens_file_execution() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let lib_path = format!("{}/tests/_examples/cli_flags/lib", manifest_dir);
    let script = format!(
        "{}/tests/_examples/cli_flags/use_greet_no_parens.rb",
        manifest_dir
    );
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["-I", &lib_path, &script])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "hello world");
}

#[test]
fn cli_i_flag_no_parens_multiple_libs() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let lib_path = format!("{}/tests/_examples/cli_flags/lib", manifest_dir);
    let script = format!(
        "{}/tests/_examples/cli_flags/use_both_no_parens.rb",
        manifest_dir
    );
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["-I", &lib_path, &script])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "hello world\n42");
}

// ============================================================================
// -w, -d (ignored Ruby flags)
// ============================================================================

#[test]
fn cli_w_flag_ignored() {
    let output = metorex_cmd()
        .args(["-w", "-e", "puts 3"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "3");
}

#[test]
fn cli_d_flag_ignored() {
    let output = metorex_cmd()
        .args(["-d", "-e", "puts 4"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "4");
}

// ============================================================================
// ARGV (script arguments)
// ============================================================================

#[test]
fn cli_argv_passed_to_script() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["tests/_examples/argv/argv_basic.rb", "foo", "bar", "baz"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "3\nfoo\nbar\nbaz");
}

#[test]
fn cli_argv_empty_when_no_args() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["tests/_examples/argv/argv_basic.rb"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.trim(), "0");
}

// ============================================================================
// -c flag (syntax check)
// ============================================================================

/// Both `cli_flags/good_syntax` variants, which differ only in whether the
/// calls are written with parentheses.
const GOOD_SYNTAX_SCRIPTS: [&str; 2] = [
    "tests/_examples/cli_flags/good_syntax.rb",
    "tests/_examples/cli_flags/good_syntax_no_parens.rb",
];

#[test]
fn cli_c_flag_reports_a_file_that_parses() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    for script in GOOD_SYNTAX_SCRIPTS {
        let output = metorex_cmd()
            .current_dir(manifest_dir)
            .args(["-c", script])
            .output()
            .expect("failed to execute");
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout, "Syntax OK\n");
    }
}

#[test]
fn cli_c_flag_reports_inline_code_that_parses() {
    let output = metorex_cmd()
        .args(["-c", "-e", "puts 1", "-e", "puts 2"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, "Syntax OK\n");
}

#[test]
fn cli_c_flag_fails_on_code_that_does_not_parse() {
    let output = metorex_cmd()
        .args(["-c", "-e", "def broken("])
        .output()
        .expect("failed to execute");
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("Syntax OK"));
}

// ============================================================================
// -e flag written more than once
// ============================================================================

#[test]
fn cli_repeated_e_flags_join_with_newlines() {
    let output = metorex_cmd()
        .args(["-e", "greeting = \"hello\"", "-e", "puts greeting"])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, "hello\n");
}

// ============================================================================
// -d flag (debug mode)
// ============================================================================

#[test]
fn cli_d_flag_turns_on_debug_and_verbose() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    for script in [
        "tests/_examples/cli_flags/show_debug.rb",
        "tests/_examples/cli_flags/show_debug_no_parens.rb",
    ] {
        let output = metorex_cmd()
            .current_dir(manifest_dir)
            .args(["-d", script])
            .output()
            .expect("failed to execute");
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(stdout, "$DEBUG true\n$VERBOSE true\n$-d true\n");
    }
}

// ============================================================================
// -C and -X flags (the directory to work from)
// ============================================================================

#[test]
fn cli_c_and_x_flags_change_the_working_directory() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let directory = format!("{}/tests/_examples/cli_flags", manifest_dir);
    for flag in ["-C", "-X"] {
        for script in [
            "tests/_examples/cli_flags/show_directory.rb",
            "tests/_examples/cli_flags/show_directory_no_parens.rb",
        ] {
            let script = format!("{}/{}", manifest_dir, script);
            let output = metorex_cmd()
                .current_dir(manifest_dir)
                .args([flag, &directory, &script])
                .output()
                .expect("failed to execute");
            assert!(
                output.status.success(),
                "stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let stdout = String::from_utf8(output.stdout).unwrap();
            assert_eq!(stdout, directory);
        }
    }
}

#[test]
fn cli_c_flag_reports_a_directory_it_cannot_change_to() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(["-C", "no_such_directory_here", "-e", "puts 1"])
        .output()
        .expect("failed to execute");
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Error changing directory"));
}

// ============================================================================
// -I flag paths, which $LOAD_PATH holds expanded from the working directory
// ============================================================================

#[test]
fn cli_i_flag_expands_a_relative_path_from_the_working_directory() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    for script in [
        "tests/_examples/cli_flags/show_load_path.rb",
        "tests/_examples/cli_flags/show_load_path_no_parens.rb",
    ] {
        let output = metorex_cmd()
            .current_dir(manifest_dir)
            .args(["-I", "tests/_examples/cli_flags/lib", script])
            .output()
            .expect("failed to execute");
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert_eq!(
            stdout,
            format!("{}/tests/_examples/cli_flags/lib\n", manifest_dir)
        );
    }
}

#[test]
fn cli_i_flag_leaves_an_absolute_path_as_written() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let lib_path = format!("{}/tests/_examples/cli_flags/lib", manifest_dir);
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args([
            "-I",
            &lib_path,
            "tests/_examples/cli_flags/show_load_path.rb",
        ])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout, format!("{}\n", lib_path));
}

// ============================================================================
// -v alongside a program, which runs after the version line
// ============================================================================

#[test]
fn cli_v_flag_runs_the_program_after_printing_the_version() {
    let output = metorex_cmd()
        .args(["-v", "-e", "puts $VERBOSE"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("(ruby-compatible)"));
    assert!(stdout.ends_with("true\n"));
}

// ============================================================================
// -K, -E, -U and the long encoding flags
// ============================================================================

/// Run one of the `cli_flags/show_encodings` examples under the given flags.
fn show_encodings(flags: &[&str], script: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/tests/_examples/cli_flags/{}", manifest_dir, script);
    let mut arguments: Vec<&str> = flags.to_vec();
    arguments.push(&path);
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(arguments)
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn cli_encodings_default_to_utf8() {
    assert_eq!(
        show_encodings(&[], "show_encodings.rb"),
        "source UTF-8\nexternal UTF-8\ninternal none\n"
    );
}

#[test]
fn cli_encodings_default_to_utf8_no_parens() {
    assert_eq!(
        show_encodings(&[], "show_encodings_no_parens.rb"),
        "source UTF-8\nexternal UTF-8\ninternal none\n"
    );
}

/// Run one of the `cli_flags` examples with the flags given, answering what it
/// wrote to standard output.
fn run_cli_flags_example(flags: &[&str], script: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/tests/_examples/cli_flags/{}", manifest_dir, script);
    let mut arguments: Vec<&str> = flags.to_vec();
    arguments.push(&path);
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args(arguments)
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn cli_enable_frozen_string_literal_shares_one_frozen_string() {
    for script in ["frozen_literal_flags.rb", "frozen_literal_flags_parens.rb"] {
        assert_eq!(
            run_cli_flags_example(&["--enable-frozen-string-literal"], script),
            "frozen true\nshared true\n"
        );
    }
}

#[test]
fn cli_disable_frozen_string_literal_writes_a_string_each_time() {
    for script in ["frozen_literal_flags.rb", "frozen_literal_flags_parens.rb"] {
        assert_eq!(
            run_cli_flags_example(&["--disable-frozen-string-literal"], script),
            "frozen false\nshared false\n"
        );
    }
}

#[test]
fn cli_without_a_frozen_string_literal_flag_writes_a_string_each_time() {
    for script in ["frozen_literal_flags.rb", "frozen_literal_flags_parens.rb"] {
        assert_eq!(
            run_cli_flags_example(&[], script),
            "frozen false\nshared false\n"
        );
    }
}

#[test]
fn cli_k_flag_names_the_source_encoding() {
    assert_eq!(
        show_encodings(&["-KE"], "show_encodings.rb"),
        "source EUC-JP\nexternal EUC-JP\ninternal none\n"
    );
}

#[test]
fn cli_k_flag_names_the_source_encoding_no_parens() {
    assert_eq!(
        show_encodings(&["-KE"], "show_encodings_no_parens.rb"),
        "source EUC-JP\nexternal EUC-JP\ninternal none\n"
    );
}

#[test]
fn cli_upper_e_flag_names_both_encodings() {
    assert_eq!(
        show_encodings(&["-E", "big5:ISO-8859-1"], "show_encodings.rb"),
        "source UTF-8\nexternal Big5\ninternal ISO-8859-1\n"
    );
}

#[test]
fn cli_upper_e_flag_names_both_encodings_no_parens() {
    assert_eq!(
        show_encodings(&["-E", "big5:ISO-8859-1"], "show_encodings_no_parens.rb"),
        "source UTF-8\nexternal Big5\ninternal ISO-8859-1\n"
    );
}

#[test]
fn cli_upper_u_flag_reads_text_as_utf8() {
    assert_eq!(
        show_encodings(
            &["-U", "--external-encoding=Shift_JIS"],
            "show_encodings.rb"
        ),
        "source UTF-8\nexternal Shift_JIS\ninternal UTF-8\n"
    );
}

#[test]
fn cli_upper_u_flag_reads_text_as_utf8_no_parens() {
    assert_eq!(
        show_encodings(
            &["-U", "--external-encoding=Shift_JIS"],
            "show_encodings_no_parens.rb"
        ),
        "source UTF-8\nexternal Shift_JIS\ninternal UTF-8\n"
    );
}

#[test]
fn cli_long_encoding_flags_name_both_encodings() {
    assert_eq!(
        show_encodings(
            &["--external-encoding", "big5", "--internal-encoding", "big5"],
            "show_encodings.rb"
        ),
        "source UTF-8\nexternal Big5\ninternal Big5\n"
    );
}

#[test]
fn cli_upper_u_flag_conflicts_with_an_internal_encoding() {
    let output = metorex_cmd()
        .args(["-Eascii:ascii", "-U", "-e", "p 1"])
        .output()
        .expect("failed to execute");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("RuntimeError"), "stderr: {}", stderr);
}

#[test]
fn cli_upper_e_flag_refuses_a_third_encoding() {
    let output = metorex_cmd()
        .args(["--encoding", "big5:ISO-8859-1:utf-32le", "-e", "p 1"])
        .output()
        .expect("failed to execute");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("extra argument"), "stderr: {}", stderr);
}

#[test]
fn cli_k_flag_ignores_a_letter_that_names_no_encoding() {
    assert_eq!(
        show_encodings(&["-KZ"], "show_encodings.rb"),
        "source UTF-8\nexternal UTF-8\ninternal none\n"
    );
}

// ============================================================================
// -i flag (editing the files ARGF reads in place)
// ============================================================================

#[test]
fn cli_i_flag_edits_the_files_in_place_and_keeps_a_backup() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    for (script, mark) in [
        ("tests/_examples/cli_flags/edit_in_place.rb", "parens"),
        (
            "tests/_examples/cli_flags/edit_in_place_no_parens.rb",
            "plain",
        ),
    ] {
        let directory = std::env::temp_dir();
        let first = directory.join(format!("metorex_edit_one_{mark}.txt"));
        let second = directory.join(format!("metorex_edit_two_{mark}.txt"));
        std::fs::write(&first, "one\ntwo\n").expect("failed to write the first file");
        std::fs::write(&second, "three\n").expect("failed to write the second file");
        let output = metorex_cmd()
            .current_dir(manifest_dir)
            .args([
                "-i.bak",
                script,
                first.to_str().expect("a path of text"),
                second.to_str().expect("a path of text"),
            ])
            .output()
            .expect("failed to execute");
        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "");
        let backup_one = directory.join(format!("metorex_edit_one_{mark}.txt.bak"));
        let backup_two = directory.join(format!("metorex_edit_two_{mark}.txt.bak"));
        assert_eq!(std::fs::read_to_string(&first).unwrap(), "ONE\nTWO\n");
        assert_eq!(std::fs::read_to_string(&second).unwrap(), "THREE\n");
        assert_eq!(std::fs::read_to_string(&backup_one).unwrap(), "one\ntwo\n");
        assert_eq!(std::fs::read_to_string(&backup_two).unwrap(), "three\n");
        for path in [first, second, backup_one, backup_two] {
            std::fs::remove_file(path).expect("failed to clean up");
        }
    }
}

// ============================================================================
// -s flag (switches among the program's arguments become globals)
// ============================================================================

/// The expected output of both `cli_flags/switch_globals` variants.
const SWITCH_GLOBALS_OUTPUT: &str = "true\n\"ada\"\n\"held\"\n[\"rest\"]\n";

fn switch_globals_output(script: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/tests/_examples/cli_flags/{}", manifest_dir, script);
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args([
            "-s",
            &path,
            "-flag",
            "-name=ada",
            "--long--name=held",
            "rest",
        ])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn cli_s_flag_binds_switches_as_globals() {
    assert_eq!(
        switch_globals_output("switch_globals.rb"),
        SWITCH_GLOBALS_OUTPUT
    );
}

#[test]
fn cli_s_flag_binds_switches_as_globals_parens() {
    assert_eq!(
        switch_globals_output("switch_globals_parens.rb"),
        SWITCH_GLOBALS_OUTPUT
    );
}

// ============================================================================
// __END__ and the DATA constant
// ============================================================================

/// The expected output of both `cli_flags/end_data` variants.
const END_DATA_OUTPUT: &str = concat!(
    "File\n",
    "true\n",
    "\"first line\\nsecond line\\n\"\n",
    "\"# A line reading `__END__` closes the code, and `DATA` reads the text after it\\n\"\n"
);

fn end_data_output(script: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/tests/_examples/cli_flags/{}", manifest_dir, script);
    let output = metorex_cmd()
        .current_dir(manifest_dir)
        .args([&path])
        .output()
        .expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn cli_end_marker_opens_the_data_constant() {
    assert_eq!(end_data_output("end_data.rb"), END_DATA_OUTPUT);
}

#[test]
fn cli_end_marker_opens_the_data_constant_parens() {
    assert_eq!(end_data_output("end_data_parens.rb"), END_DATA_OUTPUT);
}
