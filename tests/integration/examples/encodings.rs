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
