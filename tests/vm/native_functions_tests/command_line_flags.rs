// The command line's line-reading flags.

use super::*;

#[test]
fn short_flags_cluster_and_a_value_rides_on_the_end() {
    use metorex::split_short_flags;
    assert_eq!(
        split_short_flags("-naF:".to_string()),
        vec!["-n", "-a", "-F", ":"]
    );
    assert_eq!(split_short_flags("-rfoo".to_string()), vec!["-r", "foo"]);
    assert_eq!(split_short_flags("-np".to_string()), vec!["-n", "-p"]);
    // A bare flag and a name that only looks like one are left as written.
    assert_eq!(split_short_flags("-n".to_string()), vec!["-n"]);
    assert_eq!(
        split_short_flags("-zzz".to_string()),
        vec!["-zzz".to_string()]
    );
    assert_eq!(
        split_short_flags("--version".to_string()),
        vec!["--version".to_string()]
    );
    // `-i` carries its value attached and may be written with none at all,
    // which is the long form the argument parser reads.
    assert_eq!(
        split_short_flags("-i.bak".to_string()),
        vec!["--in-place=.bak".to_string()]
    );
    assert_eq!(
        split_short_flags("-i".to_string()),
        vec!["--in-place=".to_string()]
    );
}

#[test]
fn the_line_reading_flags_read_back_under_their_own_names() {
    let mut vm = VirtualMachine::new();
    vm.set_flag_global("a", true);
    vm.set_flag_global("p", false);
    vm.set_verbose(true);
    let tokens = Lexer::new("[$-a, $-p, $VERBOSE]").tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let answer = vm.execute_program(&statements).expect("execution failed");
    assert_eq!(
        answer.map(|value| value.to_string()),
        Some("[true, false, true]".to_string())
    );
}

#[test]
fn a_line_splits_into_fields_on_whitespace_or_a_named_pattern() {
    let mut vm = VirtualMachine::new();
    vm.set_split_fields("one two\n", None);
    let tokens = Lexer::new("$F").tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let answer = vm.execute_program(&statements).expect("execution failed");
    assert_eq!(
        answer.map(|value| value.to_string()),
        Some("[one, two]".to_string())
    );

    vm.set_split_fields("a:b\n", Some(":"));
    let tokens = Lexer::new("$F").tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let answer = vm.execute_program(&statements).expect("execution failed");
    assert_eq!(
        answer.map(|value| value.to_string()),
        Some("[a, b]".to_string())
    );
}

#[test]
fn the_line_separator_is_named_by_its_octal_code() {
    let mut vm = VirtualMachine::new();
    vm.set_line_separator("72");
    let tokens = Lexer::new("[$/, $-0]").tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let answer = vm.execute_program(&statements).expect("execution failed");
    assert_eq!(
        answer.map(|value| value.to_string()),
        Some("[:, :]".to_string())
    );
}

#[test]
fn a_bare_zero_flag_asks_for_paragraph_mode() {
    let mut vm = VirtualMachine::new();
    vm.set_line_separator("0");
    let tokens = Lexer::new("$/").tokenize();
    let statements = Parser::new(tokens).parse().expect("parse failed");
    let answer = vm.execute_program(&statements).expect("execution failed");
    assert_eq!(
        answer.map(|value| value.to_string()),
        Some("\n\n".to_string())
    );
}

#[test]
fn making_a_directory_hands_its_mode_to_the_system() {
    let result = run(r#"
held = "/tmp/metorex_mkdir_mode_test"
Dir.rmdir(held) if Dir.exist?(held)
Dir.mkdir(held, 01755)
answer = File.sticky?(held)
Dir.rmdir(held)
answer
"#);
    // Linux's mkdir keeps the sticky bit a mode asks for, and macOS's drops it.
    let kept = if cfg!(target_os = "macos") {
        "false"
    } else {
        "true"
    };
    assert_eq!(
        result.map(|value| value.to_string()),
        Some(kept.to_string())
    );
}

#[test]
fn a_plain_command_runs_without_a_shell_between() {
    let result = run(r#"IO.popen("/bin/echo plain") { |handle| handle.read }"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("plain\n".to_string())
    );
}

#[test]
fn a_command_written_in_shell_syntax_still_reaches_the_shell() {
    let result = run(r#"IO.popen("/bin/echo one; /bin/echo two") { |handle| handle.read }"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("one\ntwo\n".to_string())
    );
}

#[test]
fn a_command_of_only_spaces_is_left_for_the_shell_to_refuse() {
    let result = run(r#"IO.popen("  ") { |handle| handle.read }"#);
    assert_eq!(result.map(|value| value.to_string()), Some(String::new()));
}
