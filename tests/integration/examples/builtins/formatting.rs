// Writing a value out in a named format.

use super::super::run_example;
#[test]
fn test_keyword_symbols_execution() {
    let expected = "def\nclass\nif\nelse\nend\ndo\nnil\ntrue\nfalse\nreturn\nbegin\nrescue\nensure\nwhile\nfor\ncase\nwhen\nmodule\ninclude\nyield\nsuper\nlambda\nbreak\nnext\nraise\n@ivar\n@@cvar\n";
    let output = run_example("builtins/keyword_symbols.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_keyword_symbols_parens_execution() {
    let expected = "def\nclass\n@ivar\n@@cvar\nyield\n";
    let output = run_example("builtins/keyword_symbols_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_sprintf_and_float_constants() {
    let expected = concat!(
        "one and two\n42\nsymbol\nconverted format\n",
        "TypeError: no implicit conversion of Integer into String\n",
        "TypeError\nInfinity\ntrue\ntrue\nfalse\n15\n53\ntrue\ntrue\n"
    );
    let output = run_example("builtins/sprintf_and_float_constants.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_sprintf_and_float_constants_no_parens() {
    let expected = concat!(
        "one and two\n42\nsymbol\nconverted format\n",
        "TypeError: no implicit conversion of Integer into String\n",
        "TypeError\nInfinity\ntrue\ntrue\nfalse\n15\n53\ntrue\ntrue\n"
    );
    let output = run_example("builtins/sprintf_and_float_constants_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_line_loop_options_execution() {
    let expected = concat!(
        "\"a b c d\"\n",
        "\"1-2-3\"\n",
        "\"1,2\"\n",
        "\"\"\n",
        "0\n",
        "1\n",
        "2\n",
        "0\n",
        "\"closed stream\"\n",
    );
    let output = run_example("builtins/line_loop_options.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_line_loop_options_parens_execution() {
    let expected = concat!(
        "\"a b c d\"\n",
        "\"1-2-3\"\n",
        "\"1,2\"\n",
        "\"\"\n",
        "0\n",
        "1\n",
        "2\n",
        "0\n",
        "\"closed stream\"\n",
    );
    let output = run_example("builtins/line_loop_options_parens.rb");
    assert_eq!(output, expected);
}
