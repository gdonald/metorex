use super::run_example;

#[test]
fn test_file_tracking_simple_execution() {
    let output = run_example("file_tracking/simple.rb");
    assert_eq!(output, "File tracking works\n");
}

#[test]
fn test_require_basic_execution() {
    let expected = "from helper\nhelper method called\n";
    let output = run_example("require/basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_require_deduplication_execution() {
    let expected = "counter loaded\n";
    let output = run_example("require/deduplication.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_require_nested_execution() {
    let expected = "util_a loaded\nutil_b loaded\nmain file\n";
    let output = run_example("require/nested.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_require_return_values_execution() {
    let expected = "true\nfalse\n";
    let output = run_example("require/return_values.rb");
    assert_eq!(output, expected);
}
