use super::run_example;

// 10.4.11 — Advanced Features (partial — traits.rb works)

#[test]
fn test_advanced_traits() {
    // traits.rb defines a module and class but produces no output
    // Verify it runs without error
    let output = run_example("advanced/traits.rb");
    assert_eq!(output, "");
}

#[test]
fn test_method_rescue_ensure_execution() {
    let expected = "rescued\nbody\nensure ran\ncaught\ncleanup\n";
    let output = run_example("advanced/method_rescue_ensure.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_method_rescue_ensure_parens_execution() {
    let expected = "rescued\nbody\nensure ran\ncaught\ncleanup\n";
    let output = run_example("advanced/method_rescue_ensure_parens.rb");
    assert_eq!(output, expected);
}
