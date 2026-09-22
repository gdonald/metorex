// Files, streams and the rest of the core classes.

use super::super::run_example;
#[test]
fn test_builtins_core_class_additions_execution() {
    let expected = concat!(
        "\"/dev/null\"\n",
        "true\n",
        "IO\n",
        "true\n",
        "true\n",
        "\"EUC-JP\"\n",
        "\"ISO-8859-1\"\n",
        "\"#<Encoding:UTF-8>\"\n",
        "true\n",
        "true\n",
        "1\n",
        "true\n",
        "12\n",
        "true\n",
        "true\n",
        "false\n",
        "\"\"\n",
        "false\n",
        "true\n",
        "true\n",
        "false\n",
        "[1, 2]\n",
        "[1, 2]\n",
        "true\n",
    );
    let output = run_example("builtins/core_class_additions.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_core_class_additions_parens_execution() {
    let expected = concat!(
        "\"/dev/null\"\n",
        "true\n",
        "IO\n",
        "true\n",
        "true\n",
        "\"EUC-JP\"\n",
        "\"ISO-8859-1\"\n",
        "\"#<Encoding:UTF-8>\"\n",
        "true\n",
        "true\n",
        "1\n",
        "true\n",
        "12\n",
        "true\n",
        "true\n",
        "false\n",
        "\"\"\n",
        "false\n",
        "true\n",
        "true\n",
        "false\n",
        "[1, 2]\n",
        "[1, 2]\n",
        "true\n",
    );
    let output = run_example("builtins/core_class_additions_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_file_handle_reads_execution() {
    let expected = concat!(
        "\"a\"\n",
        "1\n",
        "\"b\"\n",
        "3\n",
        "true\n",
        "\"end of file reached\"\n",
        "0\n",
        "0\n",
        "4\n",
        "false\n",
        "true\n",
        "false\n",
        "\"abc\\n\"\n",
        "false\n",
        "true\n",
        "\"fifo\"\n",
        "Errno::EEXIST\n",
        "true\n",
    );
    let output = run_example("builtins/file_handle_reads.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_file_handle_reads_parens_execution() {
    let expected = concat!(
        "\"a\"\n",
        "1\n",
        "\"b\"\n",
        "3\n",
        "true\n",
        "\"end of file reached\"\n",
        "0\n",
        "0\n",
        "4\n",
        "false\n",
        "true\n",
        "false\n",
        "\"abc\\n\"\n",
        "false\n",
        "true\n",
        "\"fifo\"\n",
        "Errno::EEXIST\n",
        "true\n",
    );
    let output = run_example("builtins/file_handle_reads_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_core_state_and_time_execution() {
    let expected = concat!(
        "false\n",
        "true\n",
        "false\n",
        "false\n",
        "true\n",
        "false\n",
        "false\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "true\n",
        "ClosedQueueError\n",
        "true\n",
        "true\n",
        "true\n",
        "(13/25)\n",
        "\"1991-01-01 00:00:00 +0000\"\n",
        "\"1976-08-26 14:30:00 -0400\"\n",
        "\"1997-11-21 09:55:06 -0600\"\n",
        "[Infinity, 1]\n",
        "[Infinity, -1]\n",
        "(1.0000000000000002+1.7320508075688772i)\n",
        "2.8284271247461903\n",
        "true\n",
        "Vector[(1/1), (2/1), (-1/1)]\n",
        "false\n",
        "true\n",
    );
    let output = run_example("builtins/core_state_and_time.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_core_state_and_time_parens_execution() {
    let expected = concat!(
        "false\n",
        "true\n",
        "false\n",
        "false\n",
        "true\n",
        "false\n",
        "false\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "true\n",
        "ClosedQueueError\n",
        "true\n",
        "true\n",
        "true\n",
        "(13/25)\n",
        "\"1991-01-01 00:00:00 +0000\"\n",
        "\"1976-08-26 14:30:00 -0400\"\n",
        "\"1997-11-21 09:55:06 -0600\"\n",
        "[Infinity, 1]\n",
        "[Infinity, -1]\n",
        "(1.0000000000000002+1.7320508075688772i)\n",
        "2.8284271247461903\n",
        "true\n",
        "Vector[(1/1), (2/1), (-1/1)]\n",
        "false\n",
        "true\n",
    );
    let output = run_example("builtins/core_state_and_time_parens.rb");
    assert_eq!(output, expected);
}
