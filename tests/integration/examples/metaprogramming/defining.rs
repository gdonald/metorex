// Defining and renaming methods while the program runs.

use super::super::run_example;
use super::*;
#[test]
fn test_metaprogramming_define_method_execution() {
    let expected = "Hello, World!\n10\n12\nHi there!\nzero\none\ntwo\ntrue\n";
    let output = run_example("metaprogramming/define_method.rb");
    assert_eq!(output, expected);
}

// 14.2 — Method Missing

#[test]
fn test_metaprogramming_method_missing_execution() {
    let expected = r#"=== Dynamic Attribute Access ===
Alice
30
engineer
unknown: email

=== Ghost Methods ===
Called hello with 0 arg(s)
Called add with 2 arg(s)
Called greet with 3 arg(s)

=== Flexible Calculator ===
6
30
unknown operation: multiply

=== Selective ===
I am real
ghost: fake_method

=== Inherited method_missing ===
Base caught: anything
Base caught: whatever
"#;
    let output = run_example("metaprogramming/method_missing/method_missing.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_method_missing_no_parens_execution() {
    let expected = r#"=== Dynamic Attribute Access ===
Alice
30
engineer
unknown: email

=== Ghost Methods ===
Called hello with 0 arg(s)
Called add with 2 arg(s)
Called greet with 3 arg(s)

=== Flexible Calculator ===
6
30
unknown operation: multiply

=== Selective ===
I am real
ghost: fake_method

=== Inherited method_missing ===
Base caught: anything
Base caught: whatever
"#;
    let output = run_example("metaprogramming/method_missing/method_missing_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_method_from_callable_execution() {
    let expected = concat!(
        "[1, 2]\n",
        ":bar\n",
        ":module_method\n",
        "3\n",
        ":named\n",
        "[:public_one]\n",
        "[:initialize, :private_one]\n",
        "wrong argument type String (expected Proc/Method/UnboundMethod)\n",
        "FrozenError\n"
    );
    let output = run_example("metaprogramming/define_method_from_callable.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_method_from_callable_no_parens_execution() {
    let expected = "[1, 2]\n:bar\n:module_method\n3\n:named\n";
    let output = run_example("metaprogramming/define_method_from_callable_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_singleton_method_execution() {
    let expected = "42\n20\nbuilt\nhi\nfalse\n";
    let output = run_example("metaprogramming/define_singleton_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_singleton_method_no_parens_execution() {
    let expected = "42\nhey\n";
    let output = run_example("metaprogramming/define_singleton_method_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_method_lambda_control_flow_execution() {
    let expected = "42\n42\n42\n[:first, :second]\n1\n[1, 2]\n";
    let output = run_example("metaprogramming/define_method_lambda_control_flow.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_method_lambda_control_flow_no_parens_execution() {
    let expected = "42\n42\n[:first, :second]\n";
    let output = run_example("metaprogramming/define_method_lambda_control_flow_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_define_singleton_method_sources_execution() {
    let expected =
        "[:LIMIT]\ntrue\nfrom parent\nnot defined on the parent\nhello\ntrue\nFrozenError\n";
    let output = run_example("metaprogramming/define_singleton_method_sources.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_method_missing_visibility_execution() {
    let expected = concat!(
        "handled hidden with []\n",
        "handled shielded with [1, 2]\n",
        "handled absent with [:arg]\n",
        "private method 'hidden' called for an instance of Plain\n",
        ":hidden\n",
        "true\n",
        "undefined method 'absent' for an instance of Passthrough\n",
        ":absent\n"
    );
    let output = run_example("metaprogramming/method_missing/visibility.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_method_missing_visibility_parens_execution() {
    let expected = concat!(
        "handled hidden with []\n",
        "handled shielded with [1, 2]\n",
        "handled absent with [:arg]\n",
        "private method 'hidden' called for an instance of Plain\n",
        ":hidden\n",
        "true\n",
        "undefined method 'absent' for an instance of Passthrough\n",
        ":absent\n"
    );
    let output = run_example("metaprogramming/method_missing/visibility_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_alias_over_native_execution() {
    let output = run_example("metaprogramming/alias_over_native.rb");
    assert_eq!(output, ALIAS_OVER_NATIVE_OUTPUT);
}

#[test]
fn test_metaprogramming_alias_over_native_no_parens_execution() {
    let output = run_example("metaprogramming/alias_over_native_no_parens.rb");
    assert_eq!(output, ALIAS_OVER_NATIVE_OUTPUT);
}

#[test]
fn test_metaprogramming_alias_definee_execution() {
    let output = run_example("metaprogramming/alias_definee.rb");
    assert_eq!(output, ALIAS_DEFINEE_OUTPUT);
}

#[test]
fn test_metaprogramming_alias_definee_parens_execution() {
    let output = run_example("metaprogramming/alias_definee_parens.rb");
    assert_eq!(output, ALIAS_DEFINEE_OUTPUT);
}

/// The expected output of both `metaprogramming/define_method_super_block`
/// variants, which differ only in whether the calls are written with parentheses.
const DEFINE_METHOD_SUPER_BLOCK_OUTPUT: &str =
    ":no_block\n[1, 10]\n[2, 20]\n[3, 30]\n[1, 2]\n[2, 3]\n[3, 4]\n";

#[test]
fn test_metaprogramming_define_method_super_block_execution() {
    let output = run_example("metaprogramming/define_method_super_block.rb");
    assert_eq!(output, DEFINE_METHOD_SUPER_BLOCK_OUTPUT);
}

#[test]
fn test_metaprogramming_define_method_super_block_no_parens_execution() {
    let output = run_example("metaprogramming/define_method_super_block_no_parens.rb");
    assert_eq!(output, DEFINE_METHOD_SUPER_BLOCK_OUTPUT);
}
