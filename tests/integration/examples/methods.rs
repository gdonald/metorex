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

#[test]
fn test_methods_ruby2_keywords_flag_execution() {
    let expected = concat!(
        "2\n",
        "2\n",
        "undefined method 'missing' for class 'Forwarder'\n",
        "warned and carried on\n",
    );
    let output = run_example("methods/ruby2_keywords_flag.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_ruby2_keywords_flag_parens_execution() {
    let expected = concat!(
        "2\n",
        "2\n",
        "undefined method 'missing' for class 'Forwarder'\n",
        "warned and carried on\n",
    );
    let output = run_example("methods/ruby2_keywords_flag_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_argument_forwarding_execution() {
    let expected = "[1, 2, 3]\n[1, 2, 3]\n[1, 2]\n:done\n5\n";
    let output = run_example("methods/argument_forwarding.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_argument_forwarding_parens_execution() {
    let expected = "[1, 2, 3]\n[1, 2, 3]\n[1, 2]\n:done\n5\n";
    let output = run_example("methods/argument_forwarding_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_endless_definitions_execution() {
    let expected = "42\n0\n42\n3\n43\nnamed\n";
    let output = run_example("methods/endless_definitions.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_methods_endless_definitions_parens_execution() {
    let expected = "42\n0\n42\n3\n43\nnamed\n";
    let output = run_example("methods/endless_definitions_parens.rb");
    assert_eq!(output, expected);
}

/// The expected output of both `methods/argument_binding` variants, which
/// differ only in whether the calls are written with parentheses.
const ARGUMENT_BINDING_OUTPUT: &str = concat!(
    "[1, 9, [], 2]\n",
    "[1, 2, [], 3]\n",
    "[1, 2, [3], 4]\n",
    "[1, 2, [], 3, nil]\n",
    "[1, 2, [], 3, 4]\n",
    "[1, 2, [3], 4, 5]\n",
    "[1, 2, 3, 4]\n",
    "SyntaxError\n",
    "no implicit conversion of Object into Proc\n",
);

#[test]
fn test_methods_argument_binding_execution() {
    let output = run_example("methods/argument_binding.rb");
    assert_eq!(output, ARGUMENT_BINDING_OUTPUT);
}

#[test]
fn test_methods_argument_binding_no_parens_execution() {
    let output = run_example("methods/argument_binding_no_parens.rb");
    assert_eq!(output, ARGUMENT_BINDING_OUTPUT);
}

/// The expected output of both `methods/default_that_redefines` variants.
const DEFAULT_THAT_REDEFINES_OUTPUT: &str = "42\n1\n\"hello\"\n[true, true]\nfalse\n";

#[test]
fn test_methods_default_that_redefines_execution() {
    let output = run_example("methods/default_that_redefines.rb");
    assert_eq!(output, DEFAULT_THAT_REDEFINES_OUTPUT);
}

#[test]
fn test_methods_default_that_redefines_no_parens_execution() {
    let output = run_example("methods/default_that_redefines_no_parens.rb");
    assert_eq!(output, DEFAULT_THAT_REDEFINES_OUTPUT);
}

/// The expected output of both `methods/nested_definitions` variants.
const NESTED_DEFINITIONS_OUTPUT: &str = "true\ntrue\n:entry\ntrue\n:prepared\nfalse\n:configured\nfalse\n:labeled\nFrozenError\n\"wrong number of arguments (given 0, expected 1..2)\"\nnil\nSyntaxError\nfalse\n";

#[test]
fn test_methods_nested_definitions_execution() {
    let output = run_example("methods/nested_definitions.rb");
    assert_eq!(output, NESTED_DEFINITIONS_OUTPUT);
}

#[test]
fn test_methods_nested_definitions_no_parens_execution() {
    let output = run_example("methods/nested_definitions_no_parens.rb");
    assert_eq!(output, NESTED_DEFINITIONS_OUTPUT);
}

/// The expected output of both `methods/call_arguments` variants.
const CALL_ARGUMENTS_OUTPUT: &str = "\"can't convert Refuses to Array (Refuses#to_a gives Integer)\"\nDeclines\n{\"name\" => 1, size: 2}\n{a: 4, b: 2, c: 7}\n[1, {\"a\" => 1, b: 2}]\n{a: 1}\n\"no keywords accepted\"\n[nil]\n[1, 3]\nSyntaxError\n[1, 2]\n";

#[test]
fn test_methods_call_arguments_execution() {
    let output = run_example("methods/call_arguments.rb");
    assert_eq!(output, CALL_ARGUMENTS_OUTPUT);
}

#[test]
fn test_methods_call_arguments_no_parens_execution() {
    let output = run_example("methods/call_arguments_no_parens.rb");
    assert_eq!(output, CALL_ARGUMENTS_OUTPUT);
}

/// The expected output of both `methods/super_forms` variants.
const SUPER_FORMS_OUTPUT: &str = "[[10, 2, :added, 5], {flag: :off}]\n[[10, 3, 4, :added, 5], {flag: :on}]\n[:changed, :b]\n[[], {}]\n[3, 4]\n\"implicit argument passing of super from method defined by define_method() is not supported. Specify all arguments explicitly.\"\n[:missing, :frozen?]\n[:relabeled, :named]\n[:wrapped, :plain]\n42\n";

#[test]
fn test_methods_super_forms_execution() {
    let output = run_example("methods/super_forms.rb");
    assert_eq!(output, SUPER_FORMS_OUTPUT);
}

#[test]
fn test_methods_super_forms_no_parens_execution() {
    let output = run_example("methods/super_forms_no_parens.rb");
    assert_eq!(output, SUPER_FORMS_OUTPUT);
}

const BARE_NAMES_OVER_KERNEL: &str = concat!(
    "report \n",
    "\"report \"\n",
    "\"wrong number of arguments (given 0, expected 1)\"\n",
);

#[test]
fn test_methods_bare_names_over_kernel_execution() {
    let output = run_example("methods/bare_names_over_kernel.rb");
    assert_eq!(output, BARE_NAMES_OVER_KERNEL);
}

#[test]
fn test_methods_bare_names_over_kernel_no_parens_execution() {
    let output = run_example("methods/bare_names_over_kernel_no_parens.rb");
    assert_eq!(output, BARE_NAMES_OVER_KERNEL);
}

const PRIVATE_TOP_LEVEL_METHODS: &str = concat!(
    "\"private method 'helper' called for an instance of Object\"\n",
    "\"private method 'helper' called for an instance of String\"\n",
    "\"private method 'helper' called for an instance of Integer\"\n",
    "\"private method 'helper' called for an instance of Array\"\n",
    "\"private method 'helper' called for nil\"\n",
    "\"private method 'helper' called for true\"\n",
    "\"private method 'helper' called for an instance of Symbol\"\n",
    "\"private method 'helper' called for an instance of Float\"\n",
    "\"private method 'helper' called for an instance of Hash\"\n",
    ":top\n",
    "[false, true]\n",
    "NoMethodError\n",
    ":top_level_mkdir\n",
    "0\n",
    "0\n",
    "true\n",
);

#[test]
fn test_methods_private_top_level_methods_execution() {
    let output = run_example("methods/private_top_level_methods.rb");
    assert_eq!(output, PRIVATE_TOP_LEVEL_METHODS);
}

#[test]
fn test_methods_private_top_level_methods_no_parens_execution() {
    let output = run_example("methods/private_top_level_methods_no_parens.rb");
    assert_eq!(output, PRIVATE_TOP_LEVEL_METHODS);
}
