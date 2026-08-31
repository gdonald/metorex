use super::run_example;

#[test]
fn test_dsl_test_framework_execution() {
    let expected = "Suite: Math Tests\n  PASS: 1 + 1 equals 2\n  PASS: 10 / 2 equals 5\n  PASS: 3 * 4 equals 12\n  PASS: 5 - 3 equals 2\n  FAIL: impossible math\nResults: 4 passed, 1 failed\n";
    let output = run_example("dsl/test_framework.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_dsl_html_builder_execution() {
    let expected = "<h1>Welcome to Metorex</h1>\n<p class=\"intro\">A meta-object programming language.</p>\n<span class=\"highlight\" id=\"main\">Highlighted text</span>\n\n";
    let output = run_example("dsl/html_builder.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_dsl_query_builder_execution() {
    let expected = "SELECT name, email FROM users WHERE age > 18 AND active = true ORDER BY name LIMIT 10\nSELECT * FROM products\n";
    let output = run_example("dsl/query_builder.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_dsl_config_execution() {
    let expected = "localhost\n8080\ndb.local\n5432\ntrue\n";
    let output = run_example("dsl/config.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_dsl_keyword_arg_options_execution() {
    // Trailing `key: value` syntax fills the last positional `options = nil`
    // parameter when the callee declares no explicit keyword params.
    let expected = "options.class = Hash\noptions[:shared] = true\noptions[\"shared\"] = nil\noptions.class = Hash\noptions[:shared] = true\noptions[\"shared\"] = nil\noptions is nil\n";
    let output = run_example("dsl/keyword_arg_options.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_dsl_eval_define_method_execution() {
    let expected = "eval result: :boom\nObject has boom? true\ncalling: :bam\n";
    let output = run_example("dsl/eval_define_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_dsl_method_call_with_symbol_arg_execution() {
    // Paren-less method call where the sole arg is a symbol literal:
    // `have_method :boom` must parse as `have_method(:boom)`, not as a
    // `have_method:` keyword arg.
    let expected = "boom\nboom\nmatcher for :boom\nmatcher for :boom\n";
    let output = run_example("dsl/method_call_with_symbol_arg.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_dsl_object_method_call_chain_execution() {
    // Same parse rule must hold inside a chained call: the arg to `.my_should`
    // is the result of `have_method :boom`, never a keyword Hash.
    // matcher is a String, so .inspect quotes it (Ruby-correct).
    let expected = "matcher class: String\nmatcher value: \"matcher for :boom\"\n";
    let output = run_example("dsl/object_method_call_chain.rb");
    assert_eq!(output, expected);
}
