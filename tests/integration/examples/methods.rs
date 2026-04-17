use super::run_example;

#[test]
fn test_methods_keyword_args_execution() {
    let expected = "Hello, Alice!\nHi, Bob!\nHey, Carol!\n";
    let output = run_example("methods/keyword_args/keyword_args.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_keyword_args_parens_execution() {
    let expected = "Hello, Alice!\nHi, Bob!\nHey, Carol!\n";
    let output = run_example("methods/keyword_args/keyword_args_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_keyword_args_class_execution() {
    let expected = "I'm Alice, age 0\nI'm Bob, age 30\n";
    let output = run_example("methods/keyword_args/class.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_keyword_args_class_parens_execution() {
    let expected = "I'm Alice, age 0\nI'm Bob, age 30\n";
    let output = run_example("methods/keyword_args/class_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_global_variables_execution() {
    let expected = "3\nhello\n";
    let output = run_example("runtime/global/variables.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_global_variables_parens_execution() {
    let expected = "3\nhello\n";
    let output = run_example("runtime/global/variables_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_default_params_method_execution() {
    let expected = "Hello, Alice\nHi, Bob\n";
    let output = run_example("methods/default_params/method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_default_params_method_parens_execution() {
    let expected = "Hello, Alice\nHi, Bob\n";
    let output = run_example("methods/default_params/method_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_yield_qmark_call_execution() {
    let expected = "true\n";
    let output = run_example("methods/yield_qmark_call.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_splat_empty_call_execution() {
    let expected = "pong\n";
    let output = run_example("methods/splat_empty_call.rb");
    assert_eq!(output, expected);
}
