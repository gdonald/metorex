//! Examples under `tests/_examples/strings`.

use super::run_example;

/// The expected output of both `strings/in_place/changes` variants.
const STRING_IN_PLACE_OUTPUT: &str = concat!(
    "\"abcdefg!\"\n",
    "\"oh, !hello\"\n",
    "\"oh, !\"\n",
    "\"hello\"\n",
    "\"HELLO\"\n",
    "nil\n",
    "\"HEy\"\n",
    "\"yEH\"\n",
    "\"\"\n",
    "true\n",
    "\"can't modify frozen String: \\\"kept\\\"\"\n",
    "false\n",
    "true\n",
    "\"cÅr\"\n",
    "\"i\"\n",
    "\"ss\"\n",
    "\"Sset\"\n",
    "[41, 0]\n"
);

#[test]
fn test_strings_in_place_changes_execution() {
    let output = run_example("strings/in_place/changes.rb");
    assert_eq!(output, STRING_IN_PLACE_OUTPUT);
}

#[test]
fn test_strings_in_place_changes_parens_execution() {
    let output = run_example("strings/in_place/changes_parens.rb");
    assert_eq!(output, STRING_IN_PLACE_OUTPUT);
}

/// The expected output of both `strings/byte_sets` variants.
const BYTE_SETS_OUTPUT: &str = "[0, 32, 45, 32, 255]\n#<Encoding:BINARY (ASCII-8BIT)>\ntrue\nfalse\ninvalid byte sequence in UTF-8\ninvalid range \"h-e\" in string transliteration\n\"heo\"\n";

#[test]
fn test_strings_byte_sets_execution() {
    let output = run_example("strings/byte_sets.rb");
    assert_eq!(output, BYTE_SETS_OUTPUT);
}

#[test]
fn test_strings_byte_sets_parens_execution() {
    let output = run_example("strings/byte_sets_parens.rb");
    assert_eq!(output, BYTE_SETS_OUTPUT);
}

/// The expected output of both `strings/unpack_offsets` variants, which differ only in whether
/// the calls are written with parentheses.
const UNPACK_OFFSETS_OUTPUT: &str = "68\n[\"ABCD\"]\n\"hogefuga\"\n[216, 136]\n136\nnil\n\"offset can't be negative\"\n\"offset outside of string\"\n";

#[test]
fn test_strings_unpack_offsets_execution() {
    let output = run_example("strings/unpack_offsets.rb");
    assert_eq!(output, UNPACK_OFFSETS_OUTPUT);
}

#[test]
fn test_strings_unpack_offsets_parens_execution() {
    let output = run_example("strings/unpack_offsets_parens.rb");
    assert_eq!(output, UNPACK_OFFSETS_OUTPUT);
}

/// The expected output of both `strings/coercion_and_padding` variants, which
/// differ only in whether the calls are written with parentheses.
const COERCION_AND_PADDING_OUTPUT: &str = "\"held-spelled\"\n\"ab-spe\"\n\"-speab\"\n\"no implicit conversion of Integer into String\"\n\"zero width padding\"\n\"ab  \"\n#<Encoding:UTF-8>\n\"1, 2, 3\"\n[1, 2, 1, 2, 1, 2]\n\"1:2:3\"\n\"no implicit conversion of Object into Integer\"\n";

#[test]
fn test_strings_coercion_and_padding_execution() {
    let output = run_example("strings/coercion_and_padding.rb");
    assert_eq!(output, COERCION_AND_PADDING_OUTPUT);
}

#[test]
fn test_strings_coercion_and_padding_no_parens_execution() {
    let output = run_example("strings/coercion_and_padding_no_parens.rb");
    assert_eq!(output, COERCION_AND_PADDING_OUTPUT);
}

/// The expected output of both `strings/symbol_slices_and_bytes` variants,
/// which differ only in whether the calls are written with parentheses.
const SYMBOL_SLICES_AND_BYTES_OUTPUT: &str = "\"y\"\n\"mb\"\n\"\"\nnil\n\"mb\"\nnil\n\"mb\"\n\"hé\"\n\"é\"\nnil\nfalse\ntrue\n\"hello€\"\n[49, 255, 97, 98]\n";

#[test]
fn test_strings_symbol_slices_and_bytes_execution() {
    let output = run_example("strings/symbol_slices_and_bytes.rb");
    assert_eq!(output, SYMBOL_SLICES_AND_BYTES_OUTPUT);
}

#[test]
fn test_strings_symbol_slices_and_bytes_no_parens_execution() {
    let output = run_example("strings/symbol_slices_and_bytes_no_parens.rb");
    assert_eq!(output, SYMBOL_SLICES_AND_BYTES_OUTPUT);
}

/// The expected output of both `strings/encoding_aware_length` variants, which
/// differ only in whether the calls are written with parentheses.
const ENCODING_AWARE_LENGTH_OUTPUT: &str = "4\n12\n12\n#<Encoding:UTF-8>\n4\n2\n4\n1\n1\n1\n4\n6\n";

#[test]
fn test_strings_encoding_aware_length_execution() {
    let output = run_example("strings/encoding_aware_length.rb");
    assert_eq!(output, ENCODING_AWARE_LENGTH_OUTPUT);
}

#[test]
fn test_strings_encoding_aware_length_no_parens_execution() {
    let output = run_example("strings/encoding_aware_length_no_parens.rb");
    assert_eq!(output, ENCODING_AWARE_LENGTH_OUTPUT);
}

/// The expected output of both `strings/encoding_aware_characters` variants,
/// which differ only in whether the calls are written with parentheses.
const ENCODING_AWARE_CHARACTERS_OUTPUT: &str = concat!(
    "[\"\u{24B62}\"]\n",
    "[\"UTF-8\"]\n",
    "[\"\\xF0\", \"\\xA4\", \"\\xAD\", \"\\xA2\"]\n",
    "[[240, 164], [173], [162]]\n",
    "4\n",
    "[[97], [98]]\n",
    "true\n",
    "[\"h\", \"e\", \"l\", \"l\", \"o\"]\n",
    "true\n",
    "[164]\n",
    "#<Encoding:ISO-8859-15>\n",
    "\"\u{20AC}\"\n",
);

#[test]
fn test_strings_encoding_aware_characters_execution() {
    let output = run_example("strings/encoding_aware_characters.rb");
    assert_eq!(output, ENCODING_AWARE_CHARACTERS_OUTPUT);
}

#[test]
fn test_strings_encoding_aware_characters_no_parens_execution() {
    let output = run_example("strings/encoding_aware_characters_no_parens.rb");
    assert_eq!(output, ENCODING_AWARE_CHARACTERS_OUTPUT);
}

/// The expected output of both `strings/chilled_literals` variants, which
/// differ only in whether the calls are written with parentheses.
const CHILLED_LITERALS_OUTPUT: &str = concat!(
    "false\n",
    "false\n",
    "false\n",
    "true\n",
    "true\n",
    "\"still+chilled\"\n",
    "\"abc\"\n",
);

#[test]
fn test_strings_chilled_literals_execution() {
    let output = run_example("strings/chilled_literals.rb");
    assert_eq!(output, CHILLED_LITERALS_OUTPUT);
}

#[test]
fn test_strings_chilled_literals_no_parens_execution() {
    let output = run_example("strings/chilled_literals_no_parens.rb");
    assert_eq!(output, CHILLED_LITERALS_OUTPUT);
}

/// The expected output of both `strings/ordering_and_case` variants, which
/// differ only in whether the calls are written with parentheses.
const ORDERING_AND_CASE_OUTPUT: &str = concat!(
    "0\n", "1\n", "1\n", "nil\n", "-1\n", "1\n", "0\n", "nil\n", "true\n", "true\n", "false\n",
);

#[test]
fn test_strings_ordering_and_case_execution() {
    let output = run_example("strings/ordering_and_case.rb");
    assert_eq!(output, ORDERING_AND_CASE_OUTPUT);
}

#[test]
fn test_strings_ordering_and_case_no_parens_execution() {
    let output = run_example("strings/ordering_and_case_no_parens.rb");
    assert_eq!(output, ORDERING_AND_CASE_OUTPUT);
}

/// The expected output of both `strings/radix_reading` variants, which differ
/// only in whether the calls are written with parentheses.
const RADIX_READING_OUTPUT: &str = concat!(
    "123\n",
    "0\n",
    "56\n",
    "250\n",
    "3\n",
    "127\n",
    "2833\n",
    "-18306744\n",
    "22452257707354557240087211123792674815\n",
    "245789127594125924165923648312749312749327482\n",
    "\"invalid radix 37\"\n",
    "\"ff\"\n",
    "\"5gv2rma270x9hhj4\"\n"
);

#[test]
fn test_strings_radix_reading_execution() {
    let output = run_example("strings/radix_reading.rb");
    assert_eq!(output, RADIX_READING_OUTPUT);
}

#[test]
fn test_strings_radix_reading_no_parens_execution() {
    let output = run_example("strings/radix_reading_no_parens.rb");
    assert_eq!(output, RADIX_READING_OUTPUT);
}
