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

#[test]
fn test_block_as_arg() {
    let output = run_example("advanced/block_as_arg.rb");
    assert_eq!(output, "15\n");
}

#[test]
fn test_block_param_inspect() {
    let output = run_example("advanced/block_param_inspect.rb");
    assert_eq!(output, "Object\ntrue\n");
}

#[test]
fn test_block_param_nil() {
    let output = run_example("advanced/block_param_nil.rb");
    assert_eq!(output, "no block\n84\n");
}

#[test]
fn test_case_in_coverage() {
    let output = run_example("advanced/case_in_coverage.rb");
    assert_eq!(output, "one\ntwo\none\ntwo\n");
}

#[test]
fn test_case_when_value() {
    let output = run_example("advanced/case_when_value.rb");
    assert_eq!(output, "A\nB\nC\nF\n");
}

#[test]
fn test_multi_bracket_args() {
    let output = run_example("advanced/multi_bracket_args.rb");
    assert_eq!(output, "1,2,3\n");
}

#[test]
fn test_parenless_splat() {
    let output = run_example("advanced/parenless_splat.rb");
    assert_eq!(output, "4\n1\n2\n");
}

#[test]
fn test_stabby_expr_body() {
    let output = run_example("advanced/stabby_expr_body.rb");
    assert_eq!(output, "5\n");
}
