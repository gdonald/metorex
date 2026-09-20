use super::run_example;

const SCOPE_AND_RETURNS: &str = "\"Cabinet::Shelf\"\nKernel\ntrue\n6\nLocalJumpError\n:from_the_eval\nLocalJumpError\nnil\n#<Encoding:BINARY (ASCII-8BIT)>\ntrue\n12\n42\n\"wrong argument type proc (expected binding)\"\n";

#[test]
fn test_eval_scope_and_returns_execution() {
    let output = run_example("eval/eval_scope_and_returns.rb");
    assert_eq!(output, SCOPE_AND_RETURNS);
}

#[test]
fn test_eval_scope_and_returns_no_parens_execution() {
    let output = run_example("eval/eval_scope_and_returns_no_parens.rb");
    assert_eq!(output, SCOPE_AND_RETURNS);
}
