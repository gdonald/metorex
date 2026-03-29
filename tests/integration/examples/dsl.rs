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
