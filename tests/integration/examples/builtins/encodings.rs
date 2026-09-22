// The encoding a string is written in.

use super::super::run_example;
use super::*;
#[test]
fn test_builtins_string_encodings_execution() {
    let expected = concat!(
        "#<Encoding:UTF-8>\n",
        "#<Encoding:EUC-JP>\n",
        "#<Encoding:EUC-JP>\n",
        "#<Encoding:EUC-JP>\n",
        "#<Encoding:UTF-8>\n",
        "#<Encoding:BINARY (ASCII-8BIT)>\n",
        "true\n",
        "#<Encoding:BINARY (ASCII-8BIT)>\n",
        "#<Encoding:US-ASCII>\n",
        "#<Encoding:US-ASCII>\n",
        "true\n",
        "true\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "\"Named\"\n",
        "#<Encoding:EUC-JP>\n",
        "#<Encoding:US-ASCII>\n",
        "#<Encoding:BINARY (ASCII-8BIT)>\n",
        "#<Encoding:BINARY (ASCII-8BIT)>\n",
        "#<Encoding:US-ASCII>\n",
    );
    let output = run_example("builtins/string_encodings.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_string_encodings_parens_execution() {
    let expected = concat!(
        "#<Encoding:UTF-8>\n",
        "#<Encoding:EUC-JP>\n",
        "#<Encoding:EUC-JP>\n",
        "#<Encoding:EUC-JP>\n",
        "#<Encoding:UTF-8>\n",
        "#<Encoding:BINARY (ASCII-8BIT)>\n",
        "true\n",
        "#<Encoding:BINARY (ASCII-8BIT)>\n",
        "#<Encoding:US-ASCII>\n",
        "#<Encoding:US-ASCII>\n",
        "true\n",
        "true\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "\"Named\"\n",
        "#<Encoding:EUC-JP>\n",
        "#<Encoding:US-ASCII>\n",
        "#<Encoding:BINARY (ASCII-8BIT)>\n",
        "#<Encoding:BINARY (ASCII-8BIT)>\n",
        "#<Encoding:US-ASCII>\n",
    );
    let output = run_example("builtins/string_encodings_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_encoding_converter_paths_execution() {
    let output = run_example("builtins/encoding_converter/paths.rb");
    assert_eq!(output, ENCODING_CONVERTER_OUTPUT);
}

#[test]
fn test_builtins_encoding_converter_paths_parens_execution() {
    let output = run_example("builtins/encoding_converter/paths_parens.rb");
    assert_eq!(output, ENCODING_CONVERTER_OUTPUT);
}

#[test]
fn test_builtins_byte_views_reading_execution() {
    let output = run_example("builtins/byte_views/reading.rb");
    assert_eq!(output, BYTE_VIEWS_OUTPUT);
}

#[test]
fn test_builtins_byte_views_reading_parens_execution() {
    let output = run_example("builtins/byte_views/reading_parens.rb");
    assert_eq!(output, BYTE_VIEWS_OUTPUT);
}

#[test]
fn test_builtins_encoding_converting_execution() {
    let output = run_example("builtins/encoding_converter/converting.rb");
    assert_eq!(output, ENCODING_CONVERTING_OUTPUT);
}

#[test]
fn test_builtins_encoding_converting_no_parens_execution() {
    let output = run_example("builtins/encoding_converter/converting_no_parens.rb");
    assert_eq!(output, ENCODING_CONVERTING_OUTPUT);
}
