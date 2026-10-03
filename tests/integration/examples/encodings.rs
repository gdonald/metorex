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

/// The expected output of both `encodings/missing_converters` variants.
const MISSING_CONVERTERS_OUTPUT: &str = concat!(
    "#<Encoding:Emacs-Mule>\n",
    "#<Encoding:EUC-TW>\n",
    "\"code converter not found (ASCII-8BIT to Emacs-Mule)\"\n",
    "\"code converter not found (UTF-8 to EUC-TW)\"\n",
    "\"code converter not found (Emacs-Mule to ASCII-8BIT)\"\n",
    "\"code converter not found (UTF-8 to UTF-7)\"\n",
    "\"code converter not found (UTF-8 to xyz)\"\n",
    "#<Encoding:EUC-JP>\n",
    "false\n",
    "\"あ\"\n",
    "#<Encoding:UTF-8>\n",
    "#<Encoding:Emacs-Mule>\n",
    "\"code converter not found (ASCII-8BIT to Emacs-Mule)\"\n"
);

#[test]
fn test_encodings_missing_converters_execution() {
    let output = run_example("encodings/missing_converters.rb");
    assert_eq!(output, MISSING_CONVERTERS_OUTPUT);
}

#[test]
fn test_encodings_missing_converters_parens_execution() {
    let output = run_example("encodings/missing_converters_parens.rb");
    assert_eq!(output, MISSING_CONVERTERS_OUTPUT);
}

/// The expected output of both `encodings/replacing_while_encoding` variants.
const REPLACING_WHILE_ENCODING_OUTPUT: &str = concat!(
    "\"ab?c\"\n",
    "\"こ�\"\n",
    "\"ち��\"\n",
    "\"ちfoofoo\"\n",
    "\"B?\"\n",
    "\"Bfoo\"\n",
    "[164, 162, 63, 164, 162]\n",
    "[164, 162, 63, 164, 162]\n",
    "\"ab?c\"\n",
    "[Encoding::InvalidByteSequenceError, \"\\\"\\\\xFF\\\" on UTF-8\"]\n",
    "[Encoding::InvalidByteSequenceError, \"incomplete \\\"\\\\xE3\\\\x81\\\" on UTF-8\"]\n",
    "[Encoding::InvalidByteSequenceError, \"\\\"\\\\xE3\\\" followed by \\\"A\\\" on UTF-8\"]\n",
    "[Encoding::UndefinedConversionError, \"U+FFFD from UTF-8 to US-ASCII\"]\n",
    "\"あ\"\n",
    "#<Encoding:UTF-8>\n",
    "false\n",
    "\"?\"\n",
    "[Encoding::UndefinedConversionError, \"\\\"\\\\xC3\\\" from ASCII-8BIT to UTF-8\"]\n",
    "[Encoding::UndefinedConversionError, \"\\\"\\\\xC3\\\" to UTF-8 in conversion from ASCII-8BIT to UTF-8 to UTF-16LE\"]\n",
    "\"a�\"\n"
);

#[test]
fn test_encodings_replacing_while_encoding_execution() {
    let output = run_example("encodings/replacing_while_encoding.rb");
    assert_eq!(output, REPLACING_WHILE_ENCODING_OUTPUT);
}

#[test]
fn test_encodings_replacing_while_encoding_parens_execution() {
    let output = run_example("encodings/replacing_while_encoding_parens.rb");
    assert_eq!(output, REPLACING_WHILE_ENCODING_OUTPUT);
}

/// The expected output of both `encodings/encoding_fallbacks` variants.
const ENCODING_FALLBACKS_OUTPUT: &str = concat!(
    "\"Bbar\"\n",
    "\"Bdflt\"\n",
    "\"B[239, 191, 189]\"\n",
    "\"Bfffd\"\n",
    "\"BU+FFFD\"\n",
    "\"Blookup\"\n",
    "\"Bword\"\n",
    "\"Bfoo\"\n",
    "[Encoding::UndefinedConversionError, \"U+FFFD from UTF-8 to US-ASCII\"]\n",
    "[Encoding::UndefinedConversionError, \"U+FFFD from UTF-8 to US-ASCII\"]\n",
    "[TypeError, \"no implicit conversion of Object into String\"]\n",
    "[ArgumentError, \"too big fallback string\"]\n"
);

#[test]
fn test_encodings_encoding_fallbacks_execution() {
    let output = run_example("encodings/encoding_fallbacks.rb");
    assert_eq!(output, ENCODING_FALLBACKS_OUTPUT);
}

#[test]
fn test_encodings_encoding_fallbacks_parens_execution() {
    let output = run_example("encodings/encoding_fallbacks_parens.rb");
    assert_eq!(output, ENCODING_FALLBACKS_OUTPUT);
}

/// The expected output of both `encodings/xml_escaping` variants.
const XML_ESCAPING_OUTPUT: &str = concat!(
    "\"&lt;a &amp; b&gt;\"\n",
    "\"say \\\"hi\\\"\"\n",
    "\"&lt;ü&gt;\"\n",
    "\"\\\"&lt;&#xFC;&gt;&quot;\\\"\"\n",
    "\"&#x1F600;&amp;\"\n",
    "[34, 0, 38, 0, 108, 0, 116, 0, 59, 0, 34, 0]\n",
    "\"&#xFC;\"\n",
    "\"&#xFC;\"\n",
    "\"unexpected value for xml option: other\"\n",
    "\"unexpected value for xml option\"\n"
);

#[test]
fn test_encodings_xml_escaping_execution() {
    let output = run_example("encodings/xml_escaping.rb");
    assert_eq!(output, XML_ESCAPING_OUTPUT);
}

#[test]
fn test_encodings_xml_escaping_parens_execution() {
    let output = run_example("encodings/xml_escaping_parens.rb");
    assert_eq!(output, XML_ESCAPING_OUTPUT);
}

/// The expected output of both `encodings/newline_conversions` variants.
const NEWLINE_CONVERSIONS_OUTPUT: &str = concat!(
    "\"a\\nb\\nc\\nd\"\n",
    "\"a\\r\\r\\nb\\rc\\r\\nd\"\n",
    "\"a\\r\\rb\\rc\\rd\"\n",
    "\"a\\r\\nb\\rc\\nd\"\n",
    "\"a\\nb\\nc\\nd\"\n",
    "\"a\\r\\r\\nb\\rc\\r\\nd\"\n",
    "\"a\\nb\\nc\\nd\"\n",
    "[Encoding::ConverterNotFoundError, \"code converter not found (universal_newline,crlf_newline)\"]\n",
    "[Encoding::ConverterNotFoundError, \"code converter not found (crlf_newline,cr_newline)\"]\n",
    "[ArgumentError, \"unexpected value for newline option: other\"]\n",
    "[ArgumentError, \"unexpected value for newline option\"]\n"
);

#[test]
fn test_encodings_newline_conversions_execution() {
    let output = run_example("encodings/newline_conversions.rb");
    assert_eq!(output, NEWLINE_CONVERSIONS_OUTPUT);
}

#[test]
fn test_encodings_newline_conversions_parens_execution() {
    let output = run_example("encodings/newline_conversions_parens.rb");
    assert_eq!(output, NEWLINE_CONVERSIONS_OUTPUT);
}

/// The expected output of both `encodings/primitive_conversions` variants.
const PRIMITIVE_CONVERSIONS_OUTPUT: &str = concat!(
    ":undefined_conversion\n",
    "\"abcd\"\n",
    "[225]\n",
    "#<Encoding:ISO-8859-1>\n",
    ":destination_buffer_full\n",
    ":destination_buffer_full\n",
    ":finished\n",
    "\"aabbb\"\n",
    ":finished\n",
    "\"  \"\n",
    "\"output_byteoffset too big\"\n",
    ":invalid_byte_sequence\n",
    "\"bcd\"\n",
    "[:invalid_byte_sequence, \"UTF-8\", \"ISO-8859-1\", \"\\xF1\", \"a\"]\n",
    ":finished\n",
    "\"abcd\"\n",
    ":invalid_byte_sequence\n",
    "\"\\x80\"\n",
    ":source_buffer_empty\n",
    "\"\"\n",
    ":finished\n",
    "\"あ\"\n",
    ":destination_buffer_full\n",
    ":finished\n",
    "[27, 36, 66, 57, 97, 27, 40, 66]\n",
    "#<Encoding:UTF8-MAC>\n",
    "2\n",
    "[[164, 162], [97]]\n",
    "[225]\n"
);

#[test]
fn test_encodings_primitive_conversions_execution() {
    let output = run_example("encodings/primitive_conversions.rb");
    assert_eq!(output, PRIMITIVE_CONVERSIONS_OUTPUT);
}

#[test]
fn test_encodings_primitive_conversions_parens_execution() {
    let output = run_example("encodings/primitive_conversions_parens.rb");
    assert_eq!(output, PRIMITIVE_CONVERSIONS_OUTPUT);
}

/// The expected output of both `encodings/big5_source` variants.
const BIG5_SOURCE_OUTPUT: &str = "[167, 65, 166, 110]\n#<Encoding:Big5>\n#<Encoding:Big5>\n";

#[test]
fn test_encodings_big5_source_execution() {
    let output = run_example("encodings/big5_source.rb");
    assert_eq!(output, BIG5_SOURCE_OUTPUT);
}

#[test]
fn test_encodings_big5_source_no_parens_execution() {
    let output = run_example("encodings/big5_source_no_parens.rb");
    assert_eq!(output, BIG5_SOURCE_OUTPUT);
}

/// The expected output of both `encodings/regexp_encodings` variants.
const REGEXP_ENCODINGS_OUTPUT: &str = "3\n[195, 169]\n[195, 169, 98]\n[[195, 169]]\n1\n\"a\\x{82A0}\\xB1\"\n[177]\n[#<Encoding:EUC-JP>, #<Encoding:EUC-JP>, #<Encoding:Windows-31J>, #<Encoding:UTF-8>]\ntrue\n#<Encoding:BINARY (ASCII-8BIT)>\n\"incompatible encoding regexp match (US-ASCII regexp with UTF-16LE string)\"\n\"incompatible encoding regexp match (US-ASCII regexp with UTF-8 string)\"\n\"invalid byte sequence in UTF-8\"\n[\"historical binary regexp match /.../n against UTF-8 string\\n\"]\n[\"AB é \\xFF\\n\", false]\n[195, 169, 32, 255]\n[49, 32, 255]\n";

#[test]
fn test_encodings_regexp_encodings_execution() {
    let output = run_example("encodings/regexp_encodings.rb");
    assert_eq!(output, REGEXP_ENCODINGS_OUTPUT);
}

#[test]
fn test_encodings_regexp_encodings_no_parens_execution() {
    let output = run_example("encodings/regexp_encodings_no_parens.rb");
    assert_eq!(output, REGEXP_ENCODINGS_OUTPUT);
}

/// The expected output of both `encodings/symbol_encodings` variants.
const SYMBOL_ENCODINGS_OUTPUT: &str = concat!(
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:UTF-8>\n",
    "#<Encoding:UTF-16LE>\n",
    "#<Encoding:UTF-16LE>\n",
    "22\n",
);

#[test]
fn test_encodings_symbol_encodings_execution() {
    let output = run_example("encodings/symbol_encodings.rb");
    assert_eq!(output, SYMBOL_ENCODINGS_OUTPUT);
}

#[test]
fn test_encodings_symbol_encodings_parens_execution() {
    let output = run_example("encodings/symbol_encodings_parens.rb");
    assert_eq!(output, SYMBOL_ENCODINGS_OUTPUT);
}
