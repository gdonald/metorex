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

/// The expected output of both `strings/byte_searches` variants.
const BYTE_SEARCHES_OUTPUT: &str = concat!(
    "0\n6\n3\n3\n6\n6\nnil\n2\n2\n5\n",
    "offset 1 does not land on character boundary\n",
    "no implicit conversion of Integer into String\n"
);

#[test]
fn test_strings_byte_searches_execution() {
    let output = run_example("strings/byte_searches.rb");
    assert_eq!(output, BYTE_SEARCHES_OUTPUT);
}

#[test]
fn test_strings_byte_searches_parens_execution() {
    let output = run_example("strings/byte_searches_parens.rb");
    assert_eq!(output, BYTE_SEARCHES_OUTPUT);
}

/// The expected output of both `strings/byte_splices` variants.
const BYTE_SPLICES_OUTPUT: &str = concat!(
    "\"jello\"\n",
    "\"jello\"\n",
    "\"hHElo\"\n",
    "\"say hello\"\n",
    "\"xxx\u{3093}\u{306b}\u{3061}\u{306f}\"\n",
    "offset 1 does not land on character boundary\n",
    "index 6 out of string\n",
    "-6...-6 out of range\n",
    "can't modify frozen String: \"frozen\"\n"
);

#[test]
fn test_strings_byte_splices_execution() {
    let output = run_example("strings/byte_splices.rb");
    assert_eq!(output, BYTE_SPLICES_OUTPUT);
}

#[test]
fn test_strings_byte_splices_parens_execution() {
    let output = run_example("strings/byte_splices_parens.rb");
    assert_eq!(output, BYTE_SPLICES_OUTPUT);
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

/// The expected output of both `strings/copies_and_dumps` variants, which
/// differ only in whether the calls are written with parentheses.
const COPIES_AND_DUMPS_OUTPUT: &str = concat!(
    "\"xtring\"\n",
    "\"string\"\n",
    "true\n",
    "false\n",
    "Counted\n",
    "1\n",
    "\"string\"\n",
    "\"\\\"\\\\a\\\\b\\\\t\\\\n\\\\v\\\\f\\\\r\\\\e\\\"\"\n",
    "\"\\\"\\\\x00\\\"\"\n",
    "\"\\\"caf\\\\u00E9\\\"\"\n",
    "\"\\\"\\\\u{10FFFF}\\\"\"\n",
    "\"\\\"interp 1 and \\\"\"\n",
);

#[test]
fn test_strings_copies_and_dumps_execution() {
    let output = run_example("strings/copies_and_dumps.rb");
    assert_eq!(output, COPIES_AND_DUMPS_OUTPUT);
}

#[test]
fn test_strings_copies_and_dumps_no_parens_execution() {
    let output = run_example("strings/copies_and_dumps_no_parens.rb");
    assert_eq!(output, COPIES_AND_DUMPS_OUTPUT);
}

/// The expected output of both `strings/bytes_and_encodings` variants, which differ only in whether the
/// calls are written with parentheses.
const BYTES_AND_ENCODINGS_OUTPUT: &str = concat!(
    "[120, 156]\n",
    "ASCII-8BIT\n",
    "ASCII-8BIT\n",
    "another string\n",
    "254\n",
    "MiqkFWCm1fNJI\n",
);

#[test]
fn test_strings_bytes_and_encodings_execution() {
    let output = run_example("strings/bytes_and_encodings.rb");
    assert_eq!(output, BYTES_AND_ENCODINGS_OUTPUT);
}

#[test]
fn test_strings_bytes_and_encodings_parens_execution() {
    let output = run_example("strings/bytes_and_encodings_parens.rb");
    assert_eq!(output, BYTES_AND_ENCODINGS_OUTPUT);
}

/// The expected output of both `strings/line_walks` variants, which differ
/// only in whether the calls are written with parentheses.
const LINE_WALKS_OUTPUT: &str = "[\"one\\n\", \"two\\n\", \"three\"]\n[\"one\\ntwo\\nthree\"]\n[\"hello\\nworld\\n\\n\", \"and\\nuniverse\\n\\n\"]\n[\"hello \", \"world\"]\n[\"hello \", \"world\"]\n[\"hello\", \"world\"]\n[\"hel\", \"l\", \"o\\nworl\", \"d\"]\n[\"one\\n\", \"two\"]\n\"x\\ny\"\n[\"x\", \"y\"]\n[\"ax\", \"bx\", \"c\"]\n";

#[test]
fn test_strings_line_walks_execution() {
    let output = run_example("strings/line_walks.rb");
    assert_eq!(output, LINE_WALKS_OUTPUT);
}

#[test]
fn test_strings_line_walks_parens_execution() {
    let output = run_example("strings/line_walks_parens.rb");
    assert_eq!(output, LINE_WALKS_OUTPUT);
}

/// The expected output of both `strings/written_formats` variants, which
/// differ only in whether the calls are written with parentheses.
const WRITTEN_FORMATS_OUTPUT: &str = "\"1010 127 c4 C4\"\n\"112 112 112\"\n\"1.095200e+02 1.095200E+02 10.952000\"\n\"1.23456e-05 1.23457E+06\"\n\"0x1.88p+7 0X1.88P+7\"\n\"a [1] abc\"\n\"..10110 ..7651 ..f3c ..F3C\"\n\"..11110110 ..11011\"\n\"+5  5 5     | 000005\"\n\"0b1010 0127 0xc4 0XC4\"\n\"hel   3.14     42\"\n\"hello world\"\n\"      42\"\n\"00042 and rest\"\n\"Inf -Inf NaN\"\n";

#[test]
fn test_strings_written_formats_execution() {
    let output = run_example("strings/written_formats.rb");
    assert_eq!(output, WRITTEN_FORMATS_OUTPUT);
}

#[test]
fn test_strings_written_formats_parens_execution() {
    let output = run_example("strings/written_formats_parens.rb");
    assert_eq!(output, WRITTEN_FORMATS_OUTPUT);
}

/// The expected output of both `strings/split_shapes` variants, which differ only in
/// whether the calls are written with parentheses.
const SPLIT_SHAPES_OUTPUT: &str = "[\"now's\", \"the\", \"time\"]\n[\"now's\", \"the\", \"time  \"]\n[\"now's\", \"the\", \"time\", \"\"]\n[\"1\", \"2\", \"\", \"3\", \"4\"]\n[\"1\", \"2\", \"\", \"3\", \"4\", \"\", \"\"]\n[\"1\", \"2.3.4\"]\n[\"h\", \"e\", \"l\", \"l\", \"o\"]\n[\"h\", \"ello\"]\n[\"h\", \"el\", \"lo\"]\n[\"h\", \"\", \"i\", \"\", \"!\"]\n[\"h\", \"el\", \"lo\"]\n[\"a\", \"B\", \"\", \"\", \"aBa\"]\n[\"h\", \"e\", \"l\", \"l\", \"o\", \"\"]\n[\"AA\", \"BCC\", \"BAA\"]\n[\"\", \"a\", \"b\", \"c\", \"d\"]\n[\"Chunky\", \"Bacon\"]\n\"chunky-bacon\"\n";

#[test]
fn test_strings_split_shapes_execution() {
    let output = run_example("strings/split_shapes.rb");
    assert_eq!(output, SPLIT_SHAPES_OUTPUT);
}

#[test]
fn test_strings_split_shapes_parens_execution() {
    let output = run_example("strings/split_shapes_parens.rb");
    assert_eq!(output, SPLIT_SHAPES_OUTPUT);
}

/// The expected output of both `strings/short_interpolation_and_escapes`
/// variants, which differ only in whether the calls are written with
/// parentheses.
const SHORT_INTERPOLATION_AND_ESCAPES_OUTPUT: &str = concat!(
    "[\"held\", \"gee\", \"again\", \"shared\", \"held[\", \"heldheld\", ",
    "\"\\#@\", \"\\#@ \", \"\\#@@\", \"\\#$%\"]\n",
    "[24]\n",
    "[24]\n",
    "[248]\n",
    "[152]\n",
    "[26]\n",
    "Encoding::CompatibilityError\n",
    "main's own\n",
    "nil\n",
);

#[test]
fn test_strings_short_interpolation_and_escapes_execution() {
    let output = run_example("strings/short_interpolation_and_escapes.rb");
    assert_eq!(output, SHORT_INTERPOLATION_AND_ESCAPES_OUTPUT);
}

#[test]
fn test_strings_short_interpolation_and_escapes_parens_execution() {
    let output = run_example("strings/short_interpolation_and_escapes_parens.rb");
    assert_eq!(output, SHORT_INTERPOLATION_AND_ESCAPES_OUTPUT);
}

/// The expected output of both `strings/element_assignment` variants.
const STRING_ELEMENT_ASSIGNMENT_OUTPUT: &str = concat!(
    "Jello\n",
    "JELLo\n",
    "JEllo\n",
    "JEll0\n",
    "Jal0\n",
    "Jars\n",
    "Bars\n",
    "y\n",
    "2024-07-01\n",
    "2025-07-01\n",
    "2025-07-15\n",
    "2025-07-20\n",
    "IndexError: regexp not matched\n",
    "IndexError: index 2 out of regexp\n",
    "IndexError: index -2 out of regexp\n",
    "IndexError: regexp group 2 not matched\n",
    "IndexError: string not matched\n",
    "IndexError: index 5 out of string\n",
    "IndexError: negative length -1\n",
    "RangeError: -4..-2 out of range\n",
    "RangeError: 4..5 out of range\n",
    "TypeError: no implicit conversion of Integer into String\n",
    "ArgumentError: wrong number of arguments (given 1, expected 2..3)\n",
    "ASCII-8BIT\n",
    "Encoding::CompatibilityError: incompatible character encodings: UTF-8 and EUC-JP\n",
    "atagb\n",
    "first\n",
    "z\n",
    "ztagb\n",
);

#[test]
fn test_strings_element_assignment_execution() {
    let output = run_example("strings/element_assignment.rb");
    assert_eq!(output, STRING_ELEMENT_ASSIGNMENT_OUTPUT);
}

#[test]
fn test_strings_element_assignment_no_parens_execution() {
    let output = run_example("strings/element_assignment_no_parens.rb");
    assert_eq!(output, STRING_ELEMENT_ASSIGNMENT_OUTPUT);
}

/// The expected output of both `strings/inspect_interpolation_marks` variants.
const INSPECT_INTERPOLATION_MARKS_OUTPUT: &str = concat!(
    "\"\\#{total}\"\n",
    "\"\\#$stdout\"\n",
    "\"\\#@count\"\n",
    "\"#plain\"\n",
    "\"\\#{total}\"\n",
    "\"\\#{\\x{A4A2}\"\n",
    "\"\\#$stdout\"\n",
    "\"\\#{total}\"\n",
);

#[test]
fn test_strings_inspect_interpolation_marks_execution() {
    let output = run_example("strings/inspect_interpolation_marks.rb");
    assert_eq!(output, INSPECT_INTERPOLATION_MARKS_OUTPUT);
}

#[test]
fn test_strings_inspect_interpolation_marks_no_parens_execution() {
    let output = run_example("strings/inspect_interpolation_marks_no_parens.rb");
    assert_eq!(output, INSPECT_INTERPOLATION_MARKS_OUTPUT);
}
