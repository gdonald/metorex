use super::run_example;

#[test]
fn test_basics_heredoc_with_args_execution() {
    let expected = "first line\nsecond line\n|second\nhello\n world\n";
    let output = run_example("basics/heredoc_with_args.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_array_delete_execution() {
    let expected = "2\n3\ntrue\n1\n2\nfalse\n";
    let output = run_example("basics/array_delete.rb");
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
fn test_basics_simple_range_execution() {
    let expected = "1..5\n1...5\n1\n2\n3\n4\n5\n1\n2\n3\n4\n";
    let output = run_example("basics/simple_range.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_each_block_execution() {
    let expected = "Range iteration:\n1\n2\n3\nArray iteration:\n10\n20\n30\n";
    let output = run_example("basics/each_block.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_array_execution() {
    let expected = "1\n2\n3\n";
    let output = run_example("basics/for_loop_array.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_range_execution() {
    let expected = "1\n2\n3\n4\n5\n";
    let output = run_example("basics/for_loop_range.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_break_execution() {
    let expected = "1\n2\n3\n4\n";
    let output = run_example("basics/for_loop_break.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_for_loop_continue_execution() {
    let expected = "1\n2\n4\n5\n";
    let output = run_example("basics/for_loop_continue.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_elsif_basic_execution() {
    let expected = "small positive\n";
    let output = run_example("basics/elsif_basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_elsif_without_else_execution() {
    let expected = "C\n";
    let output = run_example("basics/elsif_without_else.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_elsif_no_parens_execution() {
    let expected = "warm\n";
    let output = run_example("basics/elsif_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_spaceship_operator_execution() {
    let expected = "-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n";
    let output = run_example("basics/spaceship_operator.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_spaceship_operator_parens_execution() {
    let expected = "-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n-1\n0\n1\n";
    let output = run_example("basics/spaceship_operator_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_power_operator_execution() {
    let expected = "1024\n27\n1\n0.1\n2.0\n25\n256\n";
    let output = run_example("basics/power_operator.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_power_operator_parens_execution() {
    let expected = "1024\n27\n1\n0.1\n2.0\n25\n256\n";
    let output = run_example("basics/power_operator_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_bitwise_ops_execution() {
    let expected = "8\n14\n6\n170\n255\n85\n0\n255\n255\n";
    let output = run_example("basics/bitwise_ops.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_bitwise_ops_parens_execution() {
    let expected = "8\n14\n6\n170\n255\n85\n0\n255\n255\n";
    let output = run_example("basics/bitwise_ops_parens.rb");
    assert_eq!(output, expected);
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
fn test_type_annotations_collection_types_execution() {
    let output = run_example("type_annotations/collection_types.rb");
    let valid_output1 = "numbers = [1, 2, 3, 4, 5]\nscores = {\"Bob\" => 85, \"Alice\" => 90}\nlength of numbers: 5\nAlice's score: 90\n";
    let valid_output2 = "numbers = [1, 2, 3, 4, 5]\nscores = {\"Alice\" => 90, \"Bob\" => 85}\nlength of numbers: 5\nAlice's score: 90\n";
    assert!(
        output == valid_output1 || output == valid_output2,
        "Expected either '{}' or '{}', but got '{}'",
        valid_output1,
        valid_output2,
        output
    );
}

#[test]
fn test_scientific_notation_execution() {
    let expected = "2000.0\n2000.0\n2000.0\n0.0015\nFloat\n2001.0\n-2000.0\n10\n";
    let output = run_example("basics/scientific_notation.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_radix_literals_execution() {
    let expected = "31\n31\n10\n10\n15\n15\n15\n99\n99\n0\n0.5\n1000000\n65535\n";
    let output = run_example("basics/radix_literals.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_float_division_execution() {
    let expected = "Infinity\n-Infinity\nNaN\nInfinity\nInfinity\n2.0\ninteger division raises\n";
    let output = run_example("basics/float_division.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_then_yield_self_tap_execution() {
    let expected = "6\n10\n5\n5\nVALUE\n3\n42\n";
    let output = run_example("basics/then_yield_self_tap.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_imaginary_literals_execution() {
    let expected = "Complex\nComplex\ntrue\nComplex\n2\nComplex\nRational\n";
    let output = run_example("basics/imaginary_literals.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_not_match_operator() {
    let expected = concat!(
        "false\ntrue\nfalse\nfalse\ntrue\n:custom\n",
        "NoMethodError: undefined method '=~' for an instance of Object\n",
        "undefined method '=~' for an instance of Integer\n"
    );
    let output = run_example("basics/not_match_operator.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_not_match_operator_no_parens() {
    let expected = concat!(
        "false\ntrue\nfalse\nfalse\ntrue\n:custom\n",
        "NoMethodError: undefined method '=~' for an instance of Object\n",
        "undefined method '=~' for an instance of Integer\n"
    );
    let output = run_example("basics/not_match_operator_no_parens.rb");
    assert_eq!(output, expected);
}

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
fn test_basics_numeric_equality_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "false\n",
        "false\n",
        "true\n",
        "true\n",
        "\"method\"\n",
        "nil\n",
        "\"method\"\n",
        "nil\n",
    );
    let output = run_example("basics/numeric_equality.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_numeric_equality_parens_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "false\n",
        "false\n",
        "true\n",
        "true\n",
        "\"method\"\n",
        "nil\n",
        "\"method\"\n",
        "nil\n",
    );
    let output = run_example("basics/numeric_equality_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_puts_array_execution() {
    let expected = "a\nb\n1\n2\n3\n\nwith newline\nplain\n1\na\nb\n";
    let output = run_example("basics/puts_array.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_puts_array_parens_execution() {
    let expected = "a\nb\n1\n2\n3\n\nwith newline\nplain\n1\na\nb\n";
    let output = run_example("basics/puts_array_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_heredoc_bare_execution() {
    let expected = concat!(
        "\"first line\\nsecond line\\n\"\n",
        "\"SHOUT\\n\"\n",
        "\"hello world\\n\"\n",
        "\"no #{interpolation}\\n\"\n",
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
        "\"no #{interpolation}\\n\"\n",
        "[\"shovel still works\"]\n",
        "8\n",
    );
    let output = run_example("basics/heredoc_bare_parens.rb");
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
        "printed\n",
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
        "printed\n",
        "{key: :value2}\n",
        "1\n",
        "true\n"
    );
    let output = run_example("basics/warn/categories_parens.rb");
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
fn test_basics_integer_bit_predicates_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "no implicit conversion of String into Integer\n"
    );
    let output = run_example("basics/integers/bit_predicates.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_bit_predicates_parens_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "no implicit conversion of String into Integer\n"
    );
    let output = run_example("basics/integers/bit_predicates_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_rounding_execution() {
    let expected = concat!(
        "15\n",
        "15\n",
        "15\n",
        "15\n",
        "20\n",
        "10\n",
        "10\n",
        "-10\n",
        "-20\n",
        "200\n",
        "300\n",
        "30\n",
        "20\n",
        "20\n",
        "40\n",
        "-30\n",
        "-20\n",
        "invalid rounding mode: foo\n",
        "1\n",
        "-4\n",
        "200\n"
    );
    let output = run_example("basics/integers/rounding.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_rounding_parens_execution() {
    let expected = concat!(
        "15\n",
        "15\n",
        "15\n",
        "15\n",
        "20\n",
        "10\n",
        "10\n",
        "-10\n",
        "-20\n",
        "200\n",
        "300\n",
        "30\n",
        "20\n",
        "20\n",
        "40\n",
        "-30\n",
        "-20\n",
        "invalid rounding mode: foo\n",
        "1\n",
        "-4\n",
        "200\n"
    );
    let output = run_example("basics/integers/rounding_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_coercion_execution() {
    let expected = concat!(
        "2\n",
        "7\n",
        "5\n",
        "18\n",
        "3\n",
        "true\n",
        "1\n",
        "will not coerce\n",
        "false\n",
        "true\n",
        "6\n",
        "true\n"
    );
    let output = run_example("basics/integers/coercion.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_coercion_parens_execution() {
    let expected = concat!(
        "2\n",
        "7\n",
        "5\n",
        "18\n",
        "3\n",
        "true\n",
        "1\n",
        "will not coerce\n",
        "false\n",
        "true\n",
        "6\n",
        "true\n"
    );
    let output = run_example("basics/integers/coercion_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_division_execution() {
    let expected = concat!(
        "2\n",
        "-3\n",
        "2\n",
        "1\n",
        "[1, 2]\n",
        "[-2, 1]\n",
        "2\n",
        "-2\n",
        "2\n",
        "2.5\n",
        "2\n",
        "-1\n",
        "10.0\n",
        "divided by 0\n",
        "Infinity\n",
        "4\n",
        "true\n",
        "[5, 4, 3, 2, 1]\n",
        "[2, 13, 4]\n",
        "[2, 1]\n",
        "[2.5, 1.0]\n"
    );
    let output = run_example("basics/integers/division.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_division_parens_execution() {
    let expected = concat!(
        "2\n",
        "-3\n",
        "2\n",
        "1\n",
        "[1, 2]\n",
        "[-2, 1]\n",
        "2\n",
        "-2\n",
        "2\n",
        "2.5\n",
        "2\n",
        "-1\n",
        "10.0\n",
        "divided by 0\n",
        "Infinity\n",
        "4\n",
        "true\n",
        "[5, 4, 3, 2, 1]\n",
        "[2, 13, 4]\n",
        "[2, 1]\n",
        "[2.5, 1.0]\n"
    );
    let output = run_example("basics/integers/division_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_comparison_and_scope_execution() {
    let expected = concat!(
        "true\n",
        "[Integer, Comparable, Numeric, Object]\n",
        "true\n",
        "true\n",
        "false\n",
        "nil\n",
        "3 metric\n",
        "metric\n",
        "34\n",
        "1\n",
        "(34/1)\n",
        "8\n"
    );
    let output = run_example("basics/integers/comparison_and_scope.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_integer_comparison_and_scope_parens_execution() {
    let expected = concat!(
        "true\n",
        "[Integer, Comparable, Numeric, Object]\n",
        "true\n",
        "true\n",
        "false\n",
        "nil\n",
        "3 metric\n",
        "metric\n",
        "34\n",
        "1\n",
        "(34/1)\n",
        "8\n"
    );
    let output = run_example("basics/integers/comparison_and_scope_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_float_methods_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "true\n",
        "2.5\n",
        "0\n",
        "true\n",
        "true\n",
        "true\n",
        "0.0\n",
        "Infinity\n",
        "[1.0, 1.2]\n",
        "[2.5, 1.0]\n",
        "1\n",
        "2\n",
        "(12/1)\n",
        "3.1\n",
        "3.2\n",
        "34.5\n",
        "1200\n",
        "false\n",
        "false\n",
        "true\n",
        "divided by 0\n",
        "[2, 1.0]\n",
        "3.5\n"
    );
    let output = run_example("basics/floats/methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_float_methods_parens_execution() {
    let expected = concat!(
        "true\n",
        "true\n",
        "true\n",
        "2.5\n",
        "0\n",
        "true\n",
        "true\n",
        "true\n",
        "0.0\n",
        "Infinity\n",
        "[1.0, 1.2]\n",
        "[2.5, 1.0]\n",
        "1\n",
        "2\n",
        "(12/1)\n",
        "3.1\n",
        "3.2\n",
        "34.5\n",
        "1200\n",
        "false\n",
        "false\n",
        "true\n",
        "divided by 0\n",
        "[2, 1.0]\n",
        "3.5\n"
    );
    let output = run_example("basics/floats/methods_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_numeric_protocol_execution() {
    let expected = concat!(
        "true\n",
        "false\n",
        "false\n",
        "250\n",
        "-250\n",
        "false\n",
        "3\n",
        "2\n",
        "3.0\n",
        "-2\n",
        "2\n",
        "false\n",
        "true\n",
        "-250\n",
        "[200, 100]\n",
        "[3.0, 1.0]\n",
        "true\n",
        "false\n"
    );
    let output = run_example("basics/floats/numeric_protocol.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_numeric_protocol_parens_execution() {
    let expected = concat!(
        "true\n",
        "false\n",
        "false\n",
        "250\n",
        "-250\n",
        "false\n",
        "3\n",
        "2\n",
        "3.0\n",
        "-2\n",
        "2\n",
        "false\n",
        "true\n",
        "-250\n",
        "[200, 100]\n",
        "[3.0, 1.0]\n",
        "true\n",
        "false\n"
    );
    let output = run_example("basics/floats/numeric_protocol_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_rational_rounding_execution() {
    let expected = concat!(
        "315\n",
        "314\n",
        "314\n",
        "314\n",
        "(3143/10)\n",
        "(7857/25)\n",
        "(157143/500)\n",
        "400\n",
        "310\n",
        "3\n",
        "2\n",
        "2\n",
        "-2\n",
        "10.0\n",
        "(1/1)\n",
        "1\n"
    );
    let output = run_example("basics/rationals/rounding.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_rational_rounding_parens_execution() {
    let expected = concat!(
        "315\n",
        "314\n",
        "314\n",
        "314\n",
        "(3143/10)\n",
        "(7857/25)\n",
        "(157143/500)\n",
        "400\n",
        "310\n",
        "3\n",
        "2\n",
        "2\n",
        "-2\n",
        "10.0\n",
        "(1/1)\n",
        "1\n"
    );
    let output = run_example("basics/rationals/rounding_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_complex_arithmetic_execution() {
    let expected = concat!(
        "(4+6i)\n",
        "(-2-2i)\n",
        "(-5+10i)\n",
        "((11/25)+(2/25)*i)\n",
        "(-3+4i)\n",
        "(2+2i)\n",
        "(2+2i)\n",
        "(2+4i)\n",
        "5.0\n",
        "25\n",
        "true\n",
        "5.0\n",
        "[3, 4]\n",
        "(1-2i)\n",
        "(-1-2i)\n",
        "3\n",
        "can't convert 3+1i into Float\n",
        "can't convert 3+0.0i into Integer\n",
        "true\n",
        "false\n",
        "\"1+2i\"\n"
    );
    let output = run_example("basics/complexes/arithmetic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_complex_arithmetic_parens_execution() {
    let expected = concat!(
        "(4+6i)\n",
        "(-2-2i)\n",
        "(-5+10i)\n",
        "((11/25)+(2/25)*i)\n",
        "(-3+4i)\n",
        "(2+2i)\n",
        "(2+2i)\n",
        "(2+4i)\n",
        "5.0\n",
        "25\n",
        "true\n",
        "5.0\n",
        "[3, 4]\n",
        "(1-2i)\n",
        "(-1-2i)\n",
        "3\n",
        "can't convert 3+1i into Float\n",
        "can't convert 3+0.0i into Integer\n",
        "true\n",
        "false\n",
        "\"1+2i\"\n"
    );
    let output = run_example("basics/complexes/arithmetic_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_complex_protocol_execution() {
    let expected = concat!(
        "(0+1i)\n",
        "(-1+0i)\n",
        "(7+16i)\n",
        "8\n",
        "1\n",
        "1\n",
        "nil\n",
        "true\n",
        "false\n",
        "false\n",
        "undefined method 'new' for class 'Float'\n",
        "can't unfreeze Integer\n",
        "3\n",
        "2\n"
    );
    let output = run_example("basics/complexes/protocol.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_basics_complex_protocol_parens_execution() {
    let expected = concat!(
        "(0+1i)\n",
        "(-1+0i)\n",
        "(7+16i)\n",
        "8\n",
        "1\n",
        "1\n",
        "nil\n",
        "true\n",
        "false\n",
        "false\n",
        "undefined method 'new' for class 'Float'\n",
        "can't unfreeze Integer\n",
        "3\n",
        "2\n"
    );
    let output = run_example("basics/complexes/protocol_parens.rb");
    assert_eq!(output, expected);
}

/// The expected output of both `basics/string_bytes` variants, which differ
/// only in whether the calls are written with parentheses.
const STRING_BYTES_OUTPUT: &str = "5\n6\n[104, 195, 169, 108, 108, 111]\n104\n195\n111\nnil\n\"h\"\nfalse\ntrue\ntrue\n[97, 98, 99]\n[97, 98, 99]\n10\n31\n2880289470\n725008\n11259375\n0\n-4660\n511\n10\n15\n99\n255\n-511\n0\n";

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
