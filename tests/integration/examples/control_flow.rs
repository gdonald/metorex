use super::run_example;

#[test]
fn test_parser_pattern_matching_execution() {
    let expected = "two\nstopping\nother number\none point zero\nfive\n";
    let output = run_example("control_flow/test_pattern_matching.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_guard_execution() {
    let expected = "Warm\nLarge hundred\n";
    let output = run_example("control_flow/case_guard.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_array_destructure_execution() {
    let expected = "a=1, b=2, c=3\nFirst: 1\nRest: [2, 3, 4, 5]\nFirst: 1, Last: 5\nMiddle: [2, 3, 4]\nSum: 10\nFirst is 1, last is 4\n";
    let output = run_example("control_flow/case_array_destructure.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_object_destructure_execution() {
    let expected = "Point at (10, 20)\nName: Alice, Age: 30\nAlice is 30 years old\n";
    let output = run_example("control_flow/case_object_destructure.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_variable_binding_execution() {
    let expected = "Matched: 42\nNot Found\nWorking age: 25\n";
    let output = run_example("control_flow/case_variable_binding.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_type_basic_execution() {
    let expected = "It's an integer\nIt's a string\nIt's an array\nIt's a hash\nFloat\n";
    let output = run_example("control_flow/case_type_basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_type_custom_class_execution() {
    let expected =
        "It's a dog!\nBuddy says woof!\nIt's a cat!\nWhiskers says meow!\nIt's just a string\n";
    let output = run_example("control_flow/case_type_custom_class.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_type_mixed_execution() {
    let expected = "It's an integer: 42\nGeneric string\nProcessing integer: 20\nProcessing float: 4.71\nProcessing string: TEST\nProcessing array of 3 elements\nProcessing hash with 2 keys\n";
    let output = run_example("control_flow/case_type_mixed.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_expr_inline_execution() {
    let expected = "two\nten\nfirst\n";
    let output = run_example("control_flow/case_expr_inline.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_expr_block_execution() {
    let expected = "two\ngreeting\nit's an integer\n";
    let output = run_example("control_flow/case_expr_block.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_expr_mixed_execution() {
    let expected = "Hello, Alice!\ntwo\nB\nweekend\n";
    let output = run_example("control_flow/case_expr_mixed.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_expression_basic_execution() {
    let expected = "two\nThe answer!\nnil\na=1, b=2\n";
    let output = run_example("control_flow/case_expression_basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_expression_patterns_execution() {
    let expected = "6\n10\n84\nIt's an integer\nmatches anything\n30\n";
    let output = run_example("control_flow/case_expression_patterns.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_expression_nested_execution() {
    let expected = "25\nall match\n1\n20\n3\n30\nfruit\n120\n150\nundefined state\n";
    let output = run_example("control_flow/case_expression_nested.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_case_multi_value_execution() {
    let expected = "small\nmedium\nlarge\nGood\nsuccess\n";
    let output = run_example("control_flow/case_multi_value.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_logical_operators_execution() {
    let expected = "false\ntrue\nfalse\ntrue\n42\nnil\ntrue\nfalse\ntrue\n";
    let output = run_example("control_flow/logical_operators.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_logical_operators_parens_execution() {
    let expected = "false\ntrue\nfalse\ntrue\n42\nnil\ntrue\nfalse\ntrue\n";
    let output = run_example("control_flow/logical_operators_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_bang_operator_execution() {
    let expected = "false\ntrue\ntrue\ntrue\ntrue\n";
    let output = run_example("control_flow/bang_operator.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_bang_operator_parens_execution() {
    let expected = "false\ntrue\ntrue\ntrue\ntrue\n";
    let output = run_example("control_flow/bang_operator_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_in_basic_execution() {
    let expected = "42\nhello\n1\n2\n3\n99\n";
    let output = run_example("control_flow/case_in_basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_in_basic_parens_execution() {
    let expected = "42\nhello\n1\n2\n3\n99\n";
    let output = run_example("control_flow/case_in_basic_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_in_else_execution() {
    let expected = "no match\nnil or other\n";
    let output = run_example("control_flow/case_in_else.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_in_else_parens_execution() {
    let expected = "no match\nnil or other\n";
    let output = run_example("control_flow/case_in_else_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_if_expression_execution() {
    let expected = "42\n2\nnil\n10\npositive\n";
    let output = run_example("control_flow/if_expression.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_if_expression_parens_execution() {
    let expected = "42\n2\nnil\n10\npositive\n";
    let output = run_example("control_flow/if_expression_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_range_pattern_execution() {
    let expected = "B\nmid\ninclusive or out\n";
    let output = run_example("control_flow/case_range_pattern.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_range_pattern_parens_execution() {
    let expected = "B\nmid\ninclusive or out\n";
    let output = run_example("control_flow/case_range_pattern_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_as_expression_execution() {
    let expected = "two\n2\ntrue\n31\n";
    let output = run_example("control_flow/case_as_expression.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_as_expression_parens_execution() {
    let expected = "two\n2\ntrue\n31\n";
    let output = run_example("control_flow/case_as_expression_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_multi_when_execution() {
    let expected = "small\nsmall\nmedium\nlarge\nother\n";
    let output = run_example("control_flow/case_multi_when.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_case_multi_when_parens_execution() {
    let expected = "small\nsmall\nmedium\nlarge\nother\n";
    let output = run_example("control_flow/case_multi_when_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_ternary_operator_execution() {
    let expected = "yes\nno\nbig\nb\n";
    let output = run_example("control_flow/ternary.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_ternary_operator_parens_execution() {
    let expected = "yes\nno\nbig\nb\n";
    let output = run_example("control_flow/ternary_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_unless_else_execution() {
    let expected = "not big\ntruthy path\n";
    let output = run_example("control_flow/unless_else.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_unless_else_parens_execution() {
    let expected = "not big\ntruthy path\n";
    let output = run_example("control_flow/unless_else_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_catch_throw_execution() {
    let expected = "thrown value\nblock value\nunwound to the outer catch\n-4\nnil\nObject\nmatched by identity\nUncaughtThrowError\nuncaught throw :b\nLocalJumpError\n";
    let output = run_example("control_flow/catch_throw.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_kernel_loop() {
    let expected = concat!(
        "10\n123\nnil\n3\nnil\n1\n",
        "anonymous subclass ended the loop\n",
        "ArgumentError: not swallowed\ntrue\n"
    );
    let output = run_example("control_flow/kernel_loop.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_control_flow_kernel_loop_no_parens() {
    let expected = concat!(
        "10\n123\nnil\n3\nnil\n1\n",
        "anonymous subclass ended the loop\n",
        "ArgumentError: not swallowed\ntrue\n"
    );
    let output = run_example("control_flow/kernel_loop_no_parens.rb");
    assert_eq!(output, expected);
}
