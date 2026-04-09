use super::run_example;

#[test]
fn test_methods_keyword_args_execution() {
    let expected = "Hello, Alice!\nHi, Bob!\nHey, Carol!\n";
    let output = run_example("methods/keyword_args.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_keyword_args_parens_execution() {
    let expected = "Hello, Alice!\nHi, Bob!\nHey, Carol!\n";
    let output = run_example("methods/keyword_args_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_keyword_args_class_execution() {
    let expected = "I'm Alice, age 0\nI'm Bob, age 30\n";
    let output = run_example("methods/keyword_args_class.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_keyword_args_class_parens_execution() {
    let expected = "I'm Alice, age 0\nI'm Bob, age 30\n";
    let output = run_example("methods/keyword_args_class_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_global_variables_execution() {
    let expected = "3\nhello\n";
    let output = run_example("runtime/global_variables.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_global_variables_parens_execution() {
    let expected = "3\nhello\n";
    let output = run_example("runtime/global_variables_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_default_params_method_execution() {
    let expected = "Hello, Alice\nHi, Bob\n";
    let output = run_example("methods/default_params_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_default_params_method_parens_execution() {
    let expected = "Hello, Alice\nHi, Bob\n";
    let output = run_example("methods/default_params_method_parens.rb");
    assert_eq!(output, expected);
}
