use super::run_example;

// 10.4.11 — Advanced Features (partial — traits.rb works)

#[test]
fn test_advanced_traits() {
    // traits.rb defines a module and class but produces no output
    // Verify it runs without error
    let output = run_example("advanced/traits.rb");
    assert_eq!(output, "");
}
