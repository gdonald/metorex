// Running a program, from a file and from a switch.

use super::*;
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
fn cli_debug_frozen_string_literal_names_where_a_refused_string_was_written() {
    for script in [
        "frozen_literal_birthplace.rb",
        "frozen_literal_birthplace_parens.rb",
    ] {
        assert_eq!(
            run_cli_flags_example(
                &[
                    "--enable-frozen-string-literal",
                    "--debug-frozen-string-literal"
                ],
                script
            ),
            format!(
                "can't modify frozen String: \"written here\", created at \
{}/tests/_examples/cli_flags/{}:3\n",
                env!("CARGO_MANIFEST_DIR"),
                script
            )
        );
    }
}

#[test]
fn cli_debug_frozen_string_literal_names_where_a_chilled_string_was_written() {
    assert_eq!(
        cli_flags_example_stderr(
            &["-w", "--debug-frozen-string-literal"],
            "frozen_literal_birthplace.rb"
        ),
        concat!(
            "warning: literal string will be frozen in the future\n",
            "tests/_examples/cli_flags/frozen_literal_birthplace.rb:3: \
info: the string was created here\n",
        )
    );
}
