// The copies an object hands back, and how it is set up.

use super::super::run_example;
#[test]
fn test_oop_dup_initialize_copy_execution() {
    let expected = "original\ncopy of original\n1\n2\nfalse\ntrue\ntrue\ntrue\n";
    let output = run_example("oop/dup_initialize_copy.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_copy_hooks_execution() {
    let expected = concat!(
        "[\"copied\"]\ntrue\ntrue\ntrue\n",
        "FrozenError\nFrozenError\n",
        "initialize_copy should take same class object\ntrue\n"
    );
    let output = run_example("oop/copy_hooks.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_initialize_default_arity_execution() {
    let expected = concat!(
        "Plain\n",
        "wrong number of arguments (given 2, expected 0)\n",
        "wrong number of arguments (given 1, expected 0)\n",
        "true\n",
        "[1, 2]\n",
        "[1, 9]\n",
        "wrong number of arguments (given 0, expected 1..2)\n",
        "[1, 2, 3]\n",
        "wrong number of arguments (given 0, expected 1+)\n",
        "wrong number of arguments (given 1, expected 2)\n",
        "wrong number of arguments (given 3, expected 1..2)\n"
    );
    let output = run_example("oop/initialize/default_arity.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_initialize_default_arity_parens_execution() {
    let expected = concat!(
        "Plain\n",
        "wrong number of arguments (given 2, expected 0)\n",
        "wrong number of arguments (given 1, expected 0)\n",
        "true\n",
        "[1, 2]\n",
        "[1, 9]\n",
        "wrong number of arguments (given 0, expected 1..2)\n",
        "[1, 2, 3]\n",
        "wrong number of arguments (given 0, expected 1+)\n",
        "wrong number of arguments (given 1, expected 2)\n",
        "wrong number of arguments (given 3, expected 1..2)\n"
    );
    let output = run_example("oop/initialize/default_arity_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_clone_semantics_execution() {
    let expected = concat!(
        "the_one\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "false\n",
        "unexpected value for freeze: Integer\n",
        "true\n",
        "wrong number of arguments (given 2, expected 1)\n",
        "[\"singleton\", \"base\"]\n",
        "[\"singleton\", \"base\"]\n",
    );
    let output = run_example("oop/clone_semantics.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_clone_semantics_parens_execution() {
    let expected = concat!(
        "the_one\n",
        "false\n",
        "true\n",
        "false\n",
        "true\n",
        "false\n",
        "unexpected value for freeze: Integer\n",
        "true\n",
        "wrong number of arguments (given 2, expected 1)\n",
        "[\"singleton\", \"base\"]\n",
        "[\"singleton\", \"base\"]\n",
    );
    let output = run_example("oop/clone_semantics_parens.rb");
    assert_eq!(output, expected);
}

/// The expected output of both `oop/undefined_copy_hook` variants.
const UNDEFINED_COPY_HOOK_OUTPUT: &str = concat!(
    "undefined method 'initialize_copy' for an instance of Uncopyable\n",
    "undefined method 'initialize_copy' for an instance of Uncopyable\n",
);

#[test]
fn test_oop_undefined_copy_hook_execution() {
    let output = run_example("oop/undefined_copy_hook.rb");
    assert_eq!(output, UNDEFINED_COPY_HOOK_OUTPUT);
}

#[test]
fn test_oop_undefined_copy_hook_no_parens_execution() {
    let output = run_example("oop/undefined_copy_hook_no_parens.rb");
    assert_eq!(output, UNDEFINED_COPY_HOOK_OUTPUT);
}

/// The expected output of both `oop/module_superclass/undefined` variants.
const MODULE_SUPERCLASS_OUTPUT: &str = concat!(
    "false\n",
    "undefined method 'superclass' for module Greetings\n",
    "Object\n",
    "nil\n",
);

#[test]
fn test_oop_module_superclass_undefined_execution() {
    let output = run_example("oop/module_superclass/undefined.rb");
    assert_eq!(output, MODULE_SUPERCLASS_OUTPUT);
}

#[test]
fn test_oop_module_superclass_undefined_parens_execution() {
    let output = run_example("oop/module_superclass/undefined_parens.rb");
    assert_eq!(output, MODULE_SUPERCLASS_OUTPUT);
}
