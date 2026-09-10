// Examples covering StringScanner

use super::run_example;

/// The expected output of both `scanning/reading_a_string` variants, which
/// differ only in whether the calls are written with parentheses.
const READING_A_STRING_OUTPUT: &str = "\"This\"\n4\n\" \"\nnil\n\"is\"\n5\n2\n2\n\" a test\"\n7\nfalse\n0\n\"This\"\n\"Th\"\n\" is a test\"\n\"is\"\ntrue\n2\n0\n\"This\"\n10\n\" is a\"\n5\n\" test\"\n\"2024-05\"\n\"2024-05\"\n\"2024\"\n\"2024\"\n[\"2024\", \"05\"]\n{\"year\" => \"2024\", \"month\" => \"05\"}\n[\"2024-05\", \"2024\", \"05\"]\n3\n\"a\"\n\"bc\"\n98\n98\nfalse\ntrue\n\"#<StringScanner fin>\"\nTypeError\nScanError\n";

#[test]
fn test_scanning_reading_a_string_execution() {
    let output = run_example("scanning/reading_a_string.rb");
    assert_eq!(output, READING_A_STRING_OUTPUT);
}

#[test]
fn test_scanning_reading_a_string_no_parens_execution() {
    let output = run_example("scanning/reading_a_string_no_parens.rb");
    assert_eq!(output, READING_A_STRING_OUTPUT);
}

/// The expected output of both `scanning/byte_positions/cursor` variants.
const BYTE_POSITIONS_OUTPUT: &str = concat!(
    "10\n",
    "10\n",
    "8\n",
    "3\n",
    "\"ädeföghi\"\n",
    "\"T\"\n",
    "\"T\"\n",
    "true\n",
    "\"東京\"\n",
    "[230, 157, 177, 228, 186, 172]\n"
);

#[test]
fn test_scanning_byte_positions_cursor_execution() {
    let output = run_example("scanning/byte_positions/cursor.rb");
    assert_eq!(output, BYTE_POSITIONS_OUTPUT);
}

#[test]
fn test_scanning_byte_positions_cursor_parens_execution() {
    let output = run_example("scanning/byte_positions/cursor_parens.rb");
    assert_eq!(output, BYTE_POSITIONS_OUTPUT);
}

/// The expected output of both `scanning/buffered_stream/reading` variants.
const BUFFERED_STREAM_OUTPUT: &str = concat!(
    "true\n",
    "\"this\"\n",
    "\"this is\\n\"\n",
    "\"this is\\n\"\n",
    "\"an example\"\n",
    "[\"this \", \"is\\nan \", \"example\\n\"]\n",
    "[\"this is\\n\", \"an example\\n\"]\n",
    "\"1234\"\n",
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:US-ASCII>\n"
);

#[test]
fn test_scanning_buffered_stream_reading_execution() {
    let output = run_example("scanning/buffered_stream/reading.rb");
    assert_eq!(output, BUFFERED_STREAM_OUTPUT);
}

#[test]
fn test_scanning_buffered_stream_reading_parens_execution() {
    let output = run_example("scanning/buffered_stream/reading_parens.rb");
    assert_eq!(output, BUFFERED_STREAM_OUTPUT);
}
