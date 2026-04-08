use super::run_example;

#[test]
fn test_functions_closures_nested_execution() {
    let expected = "10\n12\n";
    let output = run_example("functions/closures_nested.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_functions_nonlocal_counter_execution() {
    let expected = "1\n2\n3\n3\n0\n1\n";
    let output = run_example("functions/nonlocal_counter.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_functions_locals_scope_execution() {
    let expected = "20\n[0, 2, 4, 6, 8]\n";
    let output = run_example("functions/locals_scope.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_parser_lambdas_execution() {
    let expected = "10\n10\n42\n30\n13\n13\n18\n11\n14\n21\n24\n10\n";
    let output = run_example("functions/test_lambdas.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_parenless_dotted_symbol_args() {
    let expected = ":start\nhandler_one\n:finish\nhandler_two\n";
    let output = run_example("functions/parenless_symbol_args.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_parenless_dotted_symbol_args_parens() {
    let expected = ":start\nhandler_one\n:finish\nhandler_two\n";
    let output = run_example("functions/parenless_symbol_args_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_parenless_bare_call_symbol_args() {
    let expected = ":start\nhandler_one\n:finish\nhandler_two\n";
    let output = run_example("functions/parenless_symbol_bare_call.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_parenless_bare_call_symbol_args_parens() {
    let expected = ":start\nhandler_one\n:finish\nhandler_two\n";
    let output = run_example("functions/parenless_symbol_bare_call_parens.rb");
    assert_eq!(output, expected);
}
