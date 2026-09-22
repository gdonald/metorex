// Writing values out and reading a line in.

use super::super::run_example;
#[test]
fn test_basics_kernel_p() {
    let expected = concat!(
        "\"abcde\"\n42\n:symbol\nnil\n",
        "[1, :two, \"three\"]\ncustom inspect\n",
        "7\n7\n1\n2\n[1, 2]\nnil\ntrue\n"
    );
    let output = run_example("basics/kernel_p.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_kernel_p_parens() {
    let expected = concat!(
        "\"abcde\"\n42\n:symbol\nnil\n",
        "[1, :two, \"three\"]\ncustom inspect\n",
        "7\n7\n1\n2\n[1, 2]\nnil\ntrue\n"
    );
    let output = run_example("basics/kernel_p_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_stdout_redirection() {
    let expected = concat!(
        "\"through puts\\nthrough print\\\"through p\\\"\\n\\n\"\n",
        "symbol\nsymbol\n:symbol\nlast line\nspeaker to_s\n"
    );
    let output = run_example("basics/stdout_redirection.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_stdout_redirection_no_parens() {
    let expected = concat!(
        "\"through puts\\nthrough print\\\"through p\\\"\\n\\n\"\n",
        "symbol\nsymbol\n:symbol\nlast line\nspeaker to_s\n"
    );
    let output = run_example("basics/stdout_redirection_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_readline_and_readlines() {
    let expected = concat!(
        "true\ntrue\ntrue\nIOError\ntrue\n",
        "nil\n[]\n",
        "EOFError: end of file reached\n",
        "readline() expects 0 arguments, got 1\n"
    );
    let output = run_example("basics/readline_and_readlines.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_readline_and_readlines_no_parens() {
    let expected = concat!(
        "true\ntrue\ntrue\nIOError\ntrue\n",
        "nil\n[]\n",
        "EOFError: end of file reached\n",
        "readline() expects 0 arguments, got 1\n"
    );
    let output = run_example("basics/readline_and_readlines_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_warn_messages_execution() {
    let expected = concat!(
        "plain\n",
        "already ended\n",
        "first\n",
        "second\n",
        "from\n",
        "an array\n",
        "categorized\n",
        "with empty keywords\n",
        "warning: too far\n",
        "TypeError for an unconvertible category\n",
        "ArgumentError for a negative uplevel\n",
        "TypeError for a non-Integer uplevel\n"
    );
    let output = run_example("basics/warn/messages.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_warn_messages_parens_execution() {
    let expected = concat!(
        "plain\n",
        "already ended\n",
        "first\n",
        "second\n",
        "from\n",
        "an array\n",
        "categorized\n",
        "with empty keywords\n",
        "warning: too far\n",
        "TypeError for an unconvertible category\n",
        "ArgumentError for a negative uplevel\n",
        "TypeError for a non-Integer uplevel\n"
    );
    let output = run_example("basics/warn/messages_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_puts_array_execution() {
    let expected = "a\nb\n1\n2\n3\nwith newline\nplain\n1\na\nb\n";
    let output = run_example("basics/puts_array.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_puts_array_parens_execution() {
    let expected = "a\nb\n1\n2\n3\nwith newline\nplain\n1\na\nb\n";
    let output = run_example("basics/puts_array_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_warn_categories_execution() {
    let expected = concat!(
        "[:deprecated, :experimental, :performance, :strict_unused_block]\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "unknown category: noop\n",
        "no implicit conversion of String into Symbol\n",
        "no implicit conversion of Integer into Symbol\n",
        "true\n",
        "Warning\n",
        "nil\n",
        "{key: :value2}\n",
        "1\n",
        "true\n"
    );
    let output = run_example("basics/warn/categories.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_warn_categories_parens_execution() {
    let expected = concat!(
        "[:deprecated, :experimental, :performance, :strict_unused_block]\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "unknown category: noop\n",
        "no implicit conversion of String into Symbol\n",
        "no implicit conversion of Integer into Symbol\n",
        "true\n",
        "Warning\n",
        "nil\n",
        "{key: :value2}\n",
        "1\n",
        "true\n"
    );
    let output = run_example("basics/warn/categories_parens.rb");
    assert_eq!(output, expected);
}
