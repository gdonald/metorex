use super::run_example;

/// The expected output of both `encodings/converter_stepwise` variants, which
/// differ only in whether the calls are written with parentheses.
const CONVERTER_STEPWISE_OUTPUT: &str = ":finished\n\"glark\"\n[:finished, nil, nil, nil, nil]\nnil\n:undefined_conversion\nEncoding::UndefinedConversionError\n\"abc\"\n:incomplete_input\n[:incomplete_input, \"EUC-JP\", \"UTF-8\", \"\\xA4\", \"\"]\n:finished\n\"!!123\"\nEncoding::UndefinedConversionError\n";

#[test]
fn test_encodings_converter_stepwise_execution() {
    let output = run_example("encodings/converter_stepwise.rb");
    assert_eq!(output, CONVERTER_STEPWISE_OUTPUT);
}

#[test]
fn test_encodings_converter_stepwise_no_parens_execution() {
    let output = run_example("encodings/converter_stepwise_no_parens.rb");
    assert_eq!(output, CONVERTER_STEPWISE_OUTPUT);
}

/// The expected output of both `encodings/negotiation` variants, which differ
/// only in whether the calls are written with parentheses.
const NEGOTIATION_OUTPUT: &str = concat!(
    "#<Encoding:UTF-8>\n",
    "#<Encoding:UTF-8>\n",
    "nil\n",
    "nil\n",
    "#<Encoding:UTF-8>\n",
    "nil\n",
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:UTF-8>\n",
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:BINARY (ASCII-8BIT)>\n",
    "#<Encoding:UTF-8>\n",
    "#<Encoding:EUC-JP>\n"
);

#[test]
fn test_encodings_negotiation_execution() {
    let output = run_example("encodings/negotiation.rb");
    assert_eq!(output, NEGOTIATION_OUTPUT);
}

#[test]
fn test_encodings_negotiation_parens_execution() {
    let output = run_example("encodings/negotiation_parens.rb");
    assert_eq!(output, NEGOTIATION_OUTPUT);
}

/// The expected output of both `encodings/source_encoding_and_new` variants,
/// which differ only in whether the calls are written with parentheses.
const SOURCE_ENCODING_AND_NEW_OUTPUT: &str = concat!(
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:BINARY (ASCII-8BIT)>\n",
    "#<Encoding:UTF-8>\n",
    "true\n",
    "false\n",
    "#<Encoding:BINARY (ASCII-8BIT)>\n",
    "#<Encoding:EUC-JP>\n",
    "\"abc\"\n",
    "TypeError\n",
);

#[test]
fn test_encodings_source_encoding_and_new_execution() {
    let output = run_example("encodings/source_encoding_and_new.rb");
    assert_eq!(output, SOURCE_ENCODING_AND_NEW_OUTPUT);
}

#[test]
fn test_encodings_source_encoding_and_new_no_parens_execution() {
    let output = run_example("encodings/source_encoding_and_new_no_parens.rb");
    assert_eq!(output, SOURCE_ENCODING_AND_NEW_OUTPUT);
}

/// The expected output of both `encodings/japanese_text` variants, which
/// differ only in whether the calls are written with parentheses.
const JAPANESE_TEXT_OUTPUT: &str = "[164, 162]\n[143, 171, 177]\n\"あ\"\n#<Encoding:EUC-JP>\n\"\\\"\\\\x{A4A2}\\\"\"\n\"\\\"a\\\\x{A4A2}b\\\"\"\n\"U+1F600 from UTF-8 to EUC-JP\"\n[128]\n[27, 36, 66, 57, 97]\n[27, 40, 66]\n";

#[test]
fn test_encodings_japanese_text_execution() {
    let output = run_example("encodings/japanese_text.rb");
    assert_eq!(output, JAPANESE_TEXT_OUTPUT);
}

#[test]
fn test_encodings_japanese_text_parens_execution() {
    let output = run_example("encodings/japanese_text_parens.rb");
    assert_eq!(output, JAPANESE_TEXT_OUTPUT);
}

/// The expected output of both `encodings/stream_encodings` variants, which
/// differ only in whether the calls are written with parentheses.
const STREAM_ENCODINGS_OUTPUT: &str =
    "#<Encoding:EUC-JP>\n[164, 162]\n\"あ\"\n#<Encoding:UTF-8>\n\"あ\"\n\"a.b\"\n";

#[test]
fn test_encodings_stream_encodings_execution() {
    let output = run_example("encodings/stream_encodings.rb");
    assert_eq!(output, STREAM_ENCODINGS_OUTPUT);
}

#[test]
fn test_encodings_stream_encodings_parens_execution() {
    let output = run_example("encodings/stream_encodings_parens.rb");
    assert_eq!(output, STREAM_ENCODINGS_OUTPUT);
}

/// The expected output of both `encodings/code_points` variants, which differ
/// only in whether the calls are written with parentheses.
const CODE_POINTS_OUTPUT: &str = "#<Encoding:US-ASCII>\n#<Encoding:BINARY (ASCII-8BIT)>\n[200]\n[227, 129, 130]\n[164, 162]\n[129, 64]\n[237, 160, 129, 237, 176, 128]\n[256, \"US-ASCII\", \"refused\"]\n[256, \"BINARY\", \"refused\"]\n[41376, \"EUC-JP\", \"refused\"]\n[128, \"SHIFT_JIS\", \"refused\"]\n[256, \"ISO-8859-9\", \"refused\"]\n[620, \"TIS-620\", \"refused\"]\n[55296, \"UTF-8\", \"refused\"]\nnil\n\"\"\nnil\nRangeError\n";

#[test]
fn test_encodings_code_points_execution() {
    let output = run_example("encodings/code_points.rb");
    assert_eq!(output, CODE_POINTS_OUTPUT);
}

#[test]
fn test_encodings_code_points_parens_execution() {
    let output = run_example("encodings/code_points_parens.rb");
    assert_eq!(output, CODE_POINTS_OUTPUT);
}

/// The expected output of both `encodings/byte_validity` variants, which
/// differ only in whether the calls are written with parentheses.
const BYTE_VALIDITY_OUTPUT: &str =
    "true\ntrue\ntrue\nfalse\nfalse\nfalse\nfalse\nfalse\nfalse\ntrue\ntrue\n[97, 221]\nfalse\n";

#[test]
fn test_encodings_byte_validity_execution() {
    let output = run_example("encodings/byte_validity.rb");
    assert_eq!(output, BYTE_VALIDITY_OUTPUT);
}

#[test]
fn test_encodings_byte_validity_parens_execution() {
    let output = run_example("encodings/byte_validity_parens.rb");
    assert_eq!(output, BYTE_VALIDITY_OUTPUT);
}

/// The expected output of both `encodings/shift_jis_text` variants, which
/// differ only in whether the calls are written with parentheses.
const SHIFT_JIS_TEXT_OUTPUT: &str = "#<Encoding:Shift_JIS>\n[147, 250, 150, 123, 140, 234]\n3\n6\ntrue\ntrue\n[162, 166, 163]\n3\ntrue\n\"日\"\n[130, 160]\nfalse\n";

#[test]
fn test_encodings_shift_jis_text_execution() {
    let output = run_example("encodings/shift_jis_text.rb");
    assert_eq!(output, SHIFT_JIS_TEXT_OUTPUT);
}

#[test]
fn test_encodings_shift_jis_text_parens_execution() {
    let output = run_example("encodings/shift_jis_text_parens.rb");
    assert_eq!(output, SHIFT_JIS_TEXT_OUTPUT);
}

/// The expected output of both `encodings/normalized_text` variants, which
/// differ only in whether the calls are written with parentheses.
const NORMALIZED_TEXT_OUTPUT: &str = "[7835, 803]\n[383, 803, 775]\n[7785]\n[115, 803, 775]\n[197]\n[937]\ntrue\nfalse\ntrue\ntrue\n[4352, 4449]\n[44032]\n[224]\n";

#[test]
fn test_encodings_normalized_text_execution() {
    let output = run_example("encodings/normalized_text.rb");
    assert_eq!(output, NORMALIZED_TEXT_OUTPUT);
}

#[test]
fn test_encodings_normalized_text_parens_execution() {
    let output = run_example("encodings/normalized_text_parens.rb");
    assert_eq!(output, NORMALIZED_TEXT_OUTPUT);
}

/// The expected output of both `encodings/japanese_streams` variants, which differ only in
/// whether the calls are written with parentheses.
const JAPANESE_STREAMS_OUTPUT: &str = "[97, 98, 99, 27, 36, 66, 36, 34, 27, 40, 66, 100, 101, 102]\ntrue\n[\"Shift_JIS\", :undefined]\n[\"EUC-JP\", [143, 171, 177]]\n[\"UTF-16BE\", [0, 233]]\n[\"IBM437\", [130]]\n[\"macCyrillic\", :undefined]\n[\"IBM720\", [130]]\n#<Encoding:EUC-JP>\n[164, 162]\nEncodingError\n";

#[test]
fn test_encodings_japanese_streams_execution() {
    let output = run_example("encodings/japanese_streams.rb");
    assert_eq!(output, JAPANESE_STREAMS_OUTPUT);
}

#[test]
fn test_encodings_japanese_streams_parens_execution() {
    let output = run_example("encodings/japanese_streams_parens.rb");
    assert_eq!(output, JAPANESE_STREAMS_OUTPUT);
}

/// The expected output of both `encodings/held_back_bytes` variants, which
/// differ only in whether the calls are written with parentheses.
const HELD_BACK_BYTES_OUTPUT: &str = concat!(
    ":invalid_byte_sequence\n",
    "[:invalid_byte_sequence, \"EUC-JP\", \"UTF-8\", \"\\xA1\", \"d\"]\n",
    "\"d\"\n\"\"\n\"d\"\n",
    ":invalid_byte_sequence\n",
    "[:invalid_byte_sequence, \"UTF-16LE\", \"UTF-8\", \"\\x00\\xD8\", \"a\\x00\"]\n",
    "[97, 0]\n"
);

#[test]
fn test_encodings_held_back_bytes_execution() {
    let output = run_example("encodings/held_back_bytes.rb");
    assert_eq!(output, HELD_BACK_BYTES_OUTPUT);
}

#[test]
fn test_encodings_held_back_bytes_parens_execution() {
    let output = run_example("encodings/held_back_bytes_parens.rb");
    assert_eq!(output, HELD_BACK_BYTES_OUTPUT);
}
