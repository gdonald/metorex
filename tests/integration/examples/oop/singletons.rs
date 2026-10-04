// The class one object alone has, and what BasicObject answers.

use super::super::run_example;
use super::*;
#[test]
fn test_oop_singleton_method_execution() {
    let output = run_example("oop/singleton_method/singleton_method.rb");
    assert_eq!(output, "hello from singleton\n");
}

#[test]
fn test_oop_singleton_method_no_parens_execution() {
    let output = run_example("oop/singleton_method/singleton_method_no_parens.rb");
    assert_eq!(output, "hello from singleton\n");
}

#[test]
fn test_oop_singleton_class_predicate_execution() {
    let expected = "false\ntrue\ntrue\nfalse\nfalse\nfalse\nfalse\ntrue\n";
    let output = run_example("oop/singleton_class_predicate.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_singleton_class_predicate_parens_execution() {
    let expected = "false\ntrue\ntrue\nfalse\nfalse\nfalse\nfalse\ntrue\n";
    let output = run_example("oop/singleton_class_predicate_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_singleton_on_variable_execution() {
    let expected = "the one\nwrote data\nfalse\n";
    let output = run_example("oop/singleton_on_variable.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_freeze_singleton_class_execution() {
    let expected =
        "false\ntrue\ntrue\nFrozenError\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\ntrue\ntrue\n";
    let output = run_example("oop/freeze_singleton_class.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_singleton_class_assignment() {
    let expected = concat!(
        ":hello\n:HELLO\n[:greet, :shout]\n:built\n:yes\n",
        "[:alpha, :beta, :gamma]\nSymbol\nString\n[:one, :two]\n"
    );
    let output = run_example("oop/singleton_class_assignment.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_singleton_class_assignment_no_parens() {
    let expected = concat!(
        ":hello\n:HELLO\n[:greet, :shout]\n:built\n:yes\n",
        "[:alpha, :beta, :gamma]\nSymbol\nString\n[:one, :two]\n"
    );
    let output = run_example("oop/singleton_class_assignment_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_singleton_class_of() {
    let expected = concat!(
        "true\ntrue\ntrue\ntrue\n",
        "TypeError: can't define singleton\n",
        "TypeError: can't define singleton\n",
        "TypeError: can't define singleton\n",
        "true\nfalse\ndeduplicated\nmutable\n"
    );
    let output = run_example("oop/singleton_class_of.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_singleton_class_of_parens() {
    let expected = concat!(
        "true\ntrue\ntrue\ntrue\n",
        "TypeError: can't define singleton\n",
        "TypeError: can't define singleton\n",
        "TypeError: can't define singleton\n",
        "true\nfalse\ndeduplicated\nmutable\n"
    );
    let output = run_example("oop/singleton_class_of_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_basic_object_constants_execution() {
    let expected = concat!(
        "nil\n",
        "Kernel\n",
        "uninitialized constant Kernel\n",
        ":value\n",
        "true\n",
        "false\n",
        "[:BasicObject]\n",
        "BasicObject\n",
        "true\n",
        "true\n",
        "Class\n",
        "TopLevelBinding\n"
    );
    let output = run_example("oop/basic_object/constants.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_basic_object_constants_parens_execution() {
    let expected = concat!(
        "nil\n",
        "Kernel\n",
        "uninitialized constant Kernel\n",
        ":value\n",
        "true\n",
        "false\n",
        "[:BasicObject]\n",
        "BasicObject\n",
        "true\n",
        "true\n",
        "Class\n",
        "TopLevelBinding\n"
    );
    let output = run_example("oop/basic_object/constants_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_singleton_on_a_collection_execution() {
    let output = run_example("oop/singleton_on_a_collection.rb");
    assert_eq!(output, SINGLETON_ON_A_COLLECTION_OUTPUT);
}

#[test]
fn test_oop_singleton_on_a_collection_no_parens_execution() {
    let output = run_example("oop/singleton_on_a_collection_no_parens.rb");
    assert_eq!(output, SINGLETON_ON_A_COLLECTION_OUTPUT);
}

#[test]
fn test_oop_basic_object_answers_nothing_execution() {
    let output = run_example("oop/basic_object/answers_nothing.rb");
    assert_eq!(output, BASIC_OBJECT_ANSWERS_NOTHING_OUTPUT);
}

#[test]
fn test_oop_basic_object_answers_nothing_no_parens_execution() {
    let output = run_example("oop/basic_object/answers_nothing_no_parens.rb");
    assert_eq!(output, BASIC_OBJECT_ANSWERS_NOTHING_OUTPUT);
}

/// The expected output of both `oop/singleton_class_kinds` variants.
const SINGLETON_CLASS_KINDS_OUTPUT: &str =
    concat!("true\n", "true\n", "true\n", "true\n", "false\n");

#[test]
fn test_oop_singleton_class_kinds_execution() {
    let output = run_example("oop/singleton_class_kinds.rb");
    assert_eq!(output, SINGLETON_CLASS_KINDS_OUTPUT);
}

#[test]
fn test_oop_singleton_class_kinds_no_parens_execution() {
    let output = run_example("oop/singleton_class_kinds_no_parens.rb");
    assert_eq!(output, SINGLETON_CLASS_KINDS_OUTPUT);
}

/// The expected output of both `oop/singleton_class_rules` variants.
const SINGLETON_CLASS_RULES_OUTPUT: &str = concat!(
    "true\n",
    "false\n",
    "true\n",
    "true\n",
    "false\n",
    "String\n",
    "can't create instance of singleton class\n",
    "can't create instance of singleton class\n",
    "undefined method 'total' for class Invoice\n",
);

#[test]
fn test_oop_singleton_class_rules_execution() {
    let output = run_example("oop/singleton_class_rules.rb");
    assert_eq!(output, SINGLETON_CLASS_RULES_OUTPUT);
}

#[test]
fn test_oop_singleton_class_rules_no_parens_execution() {
    let output = run_example("oop/singleton_class_rules_no_parens.rb");
    assert_eq!(output, SINGLETON_CLASS_RULES_OUTPUT);
}

/// The expected output of both `oop/undefining_new` variants.
const UNDEFINING_NEW_OUTPUT: &str = concat!(
    "\"undefined method 'new' for class Fixed\"\n",
    "false\n",
    "[:build]\n",
    ":built\n",
    "\"private method 'new' called for class Hidden\"\n",
    "\"private method 'assist' called for module Helpers\"\n",
);

#[test]
fn test_oop_undefining_new_execution() {
    let output = run_example("oop/undefining_new.rb");
    assert_eq!(output, UNDEFINING_NEW_OUTPUT);
}

#[test]
fn test_oop_undefining_new_no_parens_execution() {
    let output = run_example("oop/undefining_new_no_parens.rb");
    assert_eq!(output, UNDEFINING_NEW_OUTPUT);
}
