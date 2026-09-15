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
