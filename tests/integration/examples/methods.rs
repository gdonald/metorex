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

#[test]
fn test_callee_and_method_execution() {
    let expected = "[:plain, :plain]\n[:aliased, :plain]\n[:in_block, :in_block]\n:defined\n:from_send\nnil\nnil\nnil\nnil\nsuper-sub\n";
    let output = run_example("methods/callee_and_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_anonymous_block_parameter_execution() {
    let expected = "plain\nlabeled: value\n42\n";
    let output = run_example("methods/anonymous_block_parameter.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_singleton_keyword_method_names() {
    let expected = "Integer\nopened\nnot really\n42\nshoveled log\ntrue\nFloat\ntrue\n";
    let output = run_example("methods/singleton_keyword_method_names.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_singleton_keyword_method_names_no_parens() {
    let expected = "Integer\nopened\nnot really\n42\nshoveled log\ntrue\nFloat\ntrue\n";
    let output = run_example("methods/singleton_keyword_method_names_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_optional_before_required_binding_execution() {
    let expected = concat!(
        "<only\n",
        "[both\n",
        "(-)\n",
        "=-)\n",
        "{=}\n",
        "[1, 2, 3, 9]\n",
        "[1, 8, 3, 9]\n",
        "[1, 7, 8, 9]\n"
    );
    let output = run_example("methods/optional_before_required/binding.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_optional_before_required_binding_no_parens_execution() {
    let expected = concat!(
        "<only\n",
        "[both\n",
        "(-)\n",
        "=-)\n",
        "{=}\n",
        "[1, 2, 3, 9]\n",
        "[1, 8, 3, 9]\n",
        "[1, 7, 8, 9]\n"
    );
    let output = run_example("methods/optional_before_required/binding_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_scope_locals_stay_local_execution() {
    let expected = "inner\nouter\nouter\nouter\nclass body\nouter\n6\n";
    let output = run_example("methods/scope/locals_stay_local.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_scope_locals_stay_local_parens_execution() {
    let expected = "inner\nouter\nouter\nouter\nclass body\nouter\n6\n";
    let output = run_example("methods/scope/locals_stay_local_parens.rb");
    assert_eq!(output, expected);
}
