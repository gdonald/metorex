use super::run_example;

#[test]
fn test_toplevel_binding_execution() {
    let expected = "false\nmain\nmain\nmain\n";
    let output = run_example("globals/toplevel_binding/toplevel_binding.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_toplevel_binding_receiver_execution() {
    let expected = "true\ntrue\n";
    let output = run_example("globals/toplevel_binding/receiver.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_special_globals_execution() {
    let expected = "Array\n0\n1\n1\nString\n\n\nNilClass\nFalseClass\n";
    let output = run_example("globals/special_globals.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_expand_path_execution() {
    let output = run_example("globals/expand_path/expand_path.rb");
    let lines: Vec<&str> = output.trim().split('\n').collect();
    assert!(lines[0].ends_with("/lib"));
    assert!(lines[1].ends_with("/tests/_examples/lib"));
}

#[test]
fn test_magic_file_and_line_execution() {
    let output = run_example("globals/magic_file.rb");
    let lines: Vec<&str> = output.trim().split('\n').collect();
    assert!(lines[0].ends_with("globals/magic_file.rb"));
    assert_eq!(lines[1], "2");
}

#[test]
fn test_parenless_method_call_execution() {
    let expected = "4\n0\n3\n";
    let output = run_example("globals/parenless_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_modifier_if_unless_execution() {
    let expected = "99\n42\n10\n";
    let output = run_example("globals/modifier_test.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_require_from_load_path_execution() {
    let expected = "hello from lib\n";
    let output = run_example("globals/require/test.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_expand_path_nonexistent_execution() {
    let output = run_example("globals/expand_path/nonexistent.rb");
    let lines: Vec<&str> = output.trim().split('\n').collect();
    assert!(lines[0].ends_with("/foo/bar") || lines[0].contains("foo/bar"));
    assert!(lines[1].ends_with("/tmp/base/baz") || lines[1].contains("base/baz"));
}

#[test]
fn test_expand_path_nonexistent_parens_execution() {
    let output = run_example("globals/expand_path/nonexistent_parens.rb");
    let lines: Vec<&str> = output.trim().split('\n').collect();
    assert!(lines[0].ends_with("/foo/bar") || lines[0].contains("foo/bar"));
    assert!(lines[1].ends_with("/tmp/base/baz") || lines[1].contains("base/baz"));
}

#[test]
fn test_globals_argf_gets_execution() {
    let expected = "stubbed line\nstubbed line\ntrue\nARGF.class\n";
    let output = run_example("globals/argf_gets.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_globals_global_variables_list_execution() {
    let expected = concat!(
        "true\ntrue\ntrue\nfalse\n1\ntrue\n",
        "[:$stderr, :$stdin, :$stdout]\n",
        "[\"apple\", \"avocado\"]\n[\"banana\"]\n[\"APPLE\", \"AVOCADO\"]\n",
        "[1, 4]\ntrue\n"
    );
    let output = run_example("globals/global_variables_list.rb");
    assert_eq!(output, expected);
}
