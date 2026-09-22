// What the command line reports for what it cannot run.

use super::*;
#[test]
fn cli_debug_flag_turns_on_ruby_debug() {
    assert_eq!(
        run_cli_flags_example(&["--debug"], "show_debug.rb"),
        "$DEBUG true\n$VERBOSE true\n$-d true\n"
    );
}

/// Run one of the `cli_flags` examples with the flags and environment given,
/// answering what it wrote to standard output.
fn cli_flags_example_with_env(flags: &[&str], env: &[(&str, &str)], script: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{}/tests/_examples/cli_flags/{}", manifest_dir, script);
    let mut arguments: Vec<&str> = flags.to_vec();
    arguments.push(&path);
    let mut command = metorex_cmd();
    command.current_dir(manifest_dir).args(arguments);
    for (name, value) in env {
        command.env(name, value);
    }
    let output = command.output().expect("failed to execute");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn cli_features_are_on_unless_a_flag_turns_them_off() {
    for script in ["show_features.rb", "show_features_parens.rb"] {
        assert_eq!(
            run_cli_flags_example(&[], script),
            "\"constant\"\n\"constant\"\nfalse\nfrozen false\n"
        );
        assert_eq!(
            cli_flags_example_with_env(&["--disable=all"], &[("RUBYOPT", "-w")], script),
            "nil\nnil\nfalse\nfrozen false\n"
        );
        assert_eq!(
            run_cli_flags_example(&["--enable=frozen-string-literal"], script),
            "\"constant\"\n\"constant\"\nfalse\nfrozen true\n"
        );
    }
}

#[test]
fn cli_rubyopt_is_read_unless_it_is_turned_off() {
    assert_eq!(
        cli_flags_example_with_env(&[], &[("RUBYOPT", "-w")], "show_features.rb"),
        "\"constant\"\n\"constant\"\ntrue\nfrozen false\n"
    );
    assert_eq!(
        cli_flags_example_with_env(
            &["--disable=rubyopt"],
            &[("RUBYOPT", "-w")],
            "show_features.rb"
        ),
        "\"constant\"\n\"constant\"\nfalse\nfrozen false\n"
    );
}

#[test]
fn cli_unknown_feature_names_are_reported() {
    let stderr = cli_flags_example_stderr(&["--enable=no-such-feature"], "show_features.rb");
    assert!(
        stderr.contains("warning: unknown argument for --enable: 'no_such_feature'"),
        "stderr: {}",
        stderr
    );
}

#[test]
fn cli_top_level_return_with_an_argument_warns() {
    for script in ["top_level_return.rb", "top_level_return_parens.rb"] {
        assert_eq!(run_cli_flags_example(&[], script), "before\n");
        assert_eq!(
            cli_flags_example_stderr(&[], script),
            format!(
                "tests/_examples/cli_flags/{}: \
warning: argument of top-level return is ignored\n",
                script
            )
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
