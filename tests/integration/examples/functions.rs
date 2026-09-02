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
    let expected = "20\n0\n2\n4\n6\n8\n";
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
    let expected = "start\nhandler_one\nfinish\nhandler_two\n";
    let output = run_example("functions/parenless_symbol_args.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_parenless_dotted_symbol_args_parens() {
    let expected = "start\nhandler_one\nfinish\nhandler_two\n";
    let output = run_example("functions/parenless_symbol_args_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_parenless_bare_call_symbol_args() {
    let expected = "start\nhandler_one\nfinish\nhandler_two\n";
    let output = run_example("functions/parenless_symbol_bare_call.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_parenless_bare_call_symbol_args_parens() {
    let expected = "start\nhandler_one\nfinish\nhandler_two\n";
    let output = run_example("functions/parenless_symbol_bare_call_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_default_params_standalone_execution() {
    let expected = "once\ntwice\ntwice\n";
    let output = run_example("functions/default_params_standalone.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_default_params_standalone_parens_execution() {
    let expected = "once\ntwice\ntwice\n";
    let output = run_example("functions/default_params_standalone_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_multiple_return_execution() {
    let expected = "2\n1\n10\n20\n30\n";
    let output = run_example("functions/multiple_return.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_multiple_return_parens_execution() {
    let expected = "2\n1\n10\n20\n30\n";
    let output = run_example("functions/multiple_return_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_yield_basic_execution() {
    let expected = "before yield\nHello, Alice\nafter yield\n0\n1\n2\n20\nyielded with no args\n";
    let output = run_example("functions/yield_basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_yield_basic_parens_execution() {
    let expected = "before yield\nHello, Alice\nafter yield\n0\n1\n2\n20\nyielded with no args\n";
    let output = run_example("functions/yield_basic_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_yield_class_execution() {
    let expected = "10\n20\n30\n20\n40\n60\n";
    let output = run_example("functions/yield_class.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_yield_class_parens_execution() {
    let expected = "10\n20\n30\n20\n40\n60\n";
    let output = run_example("functions/yield_class_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_yield_block_given_execution() {
    let expected = "got: 42\nno block: 99\n";
    let output = run_example("functions/yield_block_given.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_yield_block_given_parens_execution() {
    let expected = "got: 42\nno block: 99\n";
    let output = run_example("functions/yield_block_given_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_splat_basic_execution() {
    let expected = "INFO: starting\nINFO: loading\nINFO: done\nonly\n0\n6\nHi, Alice\n";
    let output = run_example("functions/splat_basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_splat_basic_parens_execution() {
    let expected = "INFO: starting\nINFO: loading\nINFO: done\nonly\n0\n6\nHi, Alice\n";
    let output = run_example("functions/splat_basic_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_splat_class_execution() {
    let expected = "APP: start\nAPP: running\nAPP: stop\n3\n1\n2\n3\n";
    let output = run_example("functions/splat_class.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_splat_class_parens_execution() {
    let expected = "APP: start\nAPP: running\nAPP: stop\n3\n1\n2\n3\n";
    let output = run_example("functions/splat_class_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_lambda_bracket_call_execution() {
    let expected = "7\n30\n42\n99\n";
    let output = run_example("functions/lambda_bracket_call.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_lambda_bracket_call_parens_execution() {
    let expected = "7\n30\n42\n99\n";
    let output = run_example("functions/lambda_bracket_call_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_splat_variadic_coverage_execution() {
    let expected = "INFO: start\nINFO: done\na-b-c\n60\n0\n3\n";
    let output = run_example("functions/splat_variadic_coverage.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_splat_variadic_coverage_parens_execution() {
    let expected = "INFO: start\nINFO: done\na-b-c\n60\n0\n3\n";
    let output = run_example("functions/splat_variadic_coverage_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_double_splat_parameters_execution() {
    let expected = "[1, 0, nil, nil]\n[1, 2, 2, 3]\n[1, 2, 1, 3]\n[1, 5, 1, 3]\n1\n[1, 1]\n";
    let output = run_example("functions/double_splat_params.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_lambda_parenless_arg_execution() {
    let expected = "ran without parentheses\nran with parentheses\n42\n42\n";
    let output = run_example("functions/lambda_parenless_arg.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_lambda_and_proc_execution() {
    let expected = concat!(
        "true\nfalse\ntrue\nfalse\n",
        "3\n[1, nil]\n[1, 2]\nArgumentError\n",
        "[:from_lambda, :method_finished]\n:from_proc\ntrue\n",
        "ArgumentError: the lambda method requires a literal block\n",
        "ArgumentError: tried to create Proc object without a block\n",
        "false\n"
    );
    let output = run_example("procs/lambda_and_proc.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_lambda_and_proc_no_parens_execution() {
    let expected = concat!(
        "true\nfalse\ntrue\nfalse\n",
        "3\n[1, nil]\n[1, 2]\nArgumentError\n",
        "[:from_lambda, :method_finished]\n:from_proc\ntrue\n",
        "ArgumentError: the lambda method requires a literal block\n",
        "ArgumentError: tried to create Proc object without a block\n",
        "false\n"
    );
    let output = run_example("procs/lambda_and_proc_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_kernel_proc_execution() {
    let expected = concat!(
        "false\n42\ntrue\ntrue\n:from_send\n:early\n",
        "ArgumentError: tried to create Proc object without a block\n",
        "true\nfalse\ntrue\n"
    );
    let output = run_example("procs/kernel_proc.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_kernel_proc_no_parens_execution() {
    let expected = concat!(
        "false\n42\ntrue\ntrue\n:from_send\n:early\n",
        "ArgumentError: tried to create Proc object without a block\n",
        "true\nfalse\ntrue\n"
    );
    let output = run_example("procs/kernel_proc_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_procs_lambda_literal_chaining_execution() {
    let expected = "6\n8\n10\n7\n16\n9\n11\ntrue\n2\n";
    let output = run_example("procs/lambda_literal_chaining.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_procs_lambda_literal_chaining_parens_execution() {
    let expected = "6\n8\n10\n7\n16\n9\n11\ntrue\n2\n";
    let output = run_example("procs/lambda_literal_chaining_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_procs_lambda_chained_comparison_execution() {
    let expected = "true\ntrue\ntrue\ntrue\n";
    let output = run_example("procs/lambda_chained_comparison.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_procs_lambda_chained_comparison_parens_execution() {
    let expected = "true\ntrue\ntrue\ntrue\n";
    let output = run_example("procs/lambda_chained_comparison_parens.rb");
    assert_eq!(output, expected);
}
