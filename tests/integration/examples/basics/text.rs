// Strings, symbols and the heredoc forms.

use super::super::run_example;
use super::*;
#[test]
fn test_basics_heredoc_with_args_execution() {
    let expected = "first line\nsecond line\n|second\nhello\n world\n";
    let output = run_example("basics/heredoc_with_args.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_greeting_line_execution() {
    let output = run_example("basics/greeting_line.rb");
    assert_eq!(output, "Hello, Ada!\n");
}

#[test]
fn test_basics_string_methods_execution() {
    let expected = r#"=== Basic String Methods ===
ALICE
alice
Hello, World!
xeroteM
11

=== String Inspection Methods ===
H
i
65
66
"#;

    let output = run_example("basics/string_methods.rb");
    assert_eq!(output, expected.to_string());
}

#[test]
fn test_heredoc_execution() {
    let expected = "Hello, World!\nThis is a heredoc.\nGood morning!\n";
    let output = run_example("basics/heredoc.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_heredoc_parens_execution() {
    let expected = "Hello, World!\nThis is a heredoc.\nGood morning!\n";
    let output = run_example("basics/heredoc_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_heredoc_bare_execution() {
    let expected = concat!(
        "\"first line\\nsecond line\\n\"\n",
        "\"SHOUT\\n\"\n",
        "\"hello world\\n\"\n",
        "\"no \\#{interpolation}\\n\"\n",
        "[\"shovel still works\"]\n",
        "8\n",
    );
    let output = run_example("basics/heredoc_bare.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_heredoc_bare_parens_execution() {
    let expected = concat!(
        "\"first line\\nsecond line\\n\"\n",
        "\"SHOUT\\n\"\n",
        "\"hello world\\n\"\n",
        "\"no \\#{interpolation}\\n\"\n",
        "[\"shovel still works\"]\n",
        "8\n",
    );
    let output = run_example("basics/heredoc_bare_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_operator_name_symbols_execution() {
    let expected = concat!(
        "3\n", "8\n", "2\n", "7\n", "-2\n", "2\n", "2\n", "-2.5\n", ":/\n", ":**\n", ":-@\n",
        ":!\n", "1\n", "[:key]\n"
    );
    let output = run_example("basics/symbols/operator_names.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_operator_name_symbols_parens_execution() {
    let expected = concat!(
        "3\n", "8\n", "2\n", "7\n", "-2\n", "2\n", "2\n", "-2.5\n", ":/\n", ":**\n", ":-@\n",
        ":!\n", "1\n", "[:key]\n"
    );
    let output = run_example("basics/symbols/operator_names_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_string_bytes_execution() {
    let output = run_example("basics/string_bytes.rb");
    assert_eq!(output, STRING_BYTES_OUTPUT);
}

#[test]
fn test_basics_string_bytes_no_parens_execution() {
    let output = run_example("basics/string_bytes_no_parens.rb");
    assert_eq!(output, STRING_BYTES_OUTPUT);
}

#[test]
fn test_basics_heredoc_details_execution() {
    let output = run_example("basics/heredoc_details.rb");
    assert_eq!(output, HEREDOC_DETAILS_OUTPUT);
}

#[test]
fn test_basics_heredoc_details_parens_execution() {
    let output = run_example("basics/heredoc_details_parens.rb");
    assert_eq!(output, HEREDOC_DETAILS_OUTPUT);
}
