// Which methods a receiver may reach from outside.

use super::super::run_example;
#[test]
fn test_oop_private_class_method_execution() {
    let expected = "inherited_secret hidden\nfirst hidden\nsecond hidden\n:first\nonly hidden\nNameError for a missing method\nNameError for an instance method\n";
    let output = run_example("oop/private_class_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_private_class_method_parens_execution() {
    let expected = "inherited_secret hidden\nfirst hidden\nsecond hidden\n:first\nonly hidden\nNameError for a missing method\nNameError for an instance method\n";
    let output = run_example("oop/private_class_method_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_private_constant_execution() {
    let expected = ":visible\n:hidden\nHIDDEN is private\nALSO_HIDDEN is private\nNameError for an inherited constant\nNameError for a missing constant\n:hidden\n";
    let output = run_example("oop/private_constant.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_private_constant_parens_execution() {
    let expected = ":visible\n:hidden\nHIDDEN is private\nALSO_HIDDEN is private\nNameError for an inherited constant\nNameError for a missing constant\n:hidden\n";
    let output = run_example("oop/private_constant_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_visibility_modifiers_execution() {
    let expected = "[:hidden]\n[:guarded]\n[:open]\n:first\n[:first, :second]\n[:first, :second]\nnil\n[:in_eval]\n[:after_closure]\n[]\ntrue\n";
    let output = run_example("oop/visibility_modifiers.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_visibility_modifiers_parens_execution() {
    let expected = "[:hidden]\n[:guarded]\n[:open]\n:first\n[:first, :second]\n[:first, :second]\nnil\n[:in_eval]\n[:after_closure]\n[]\ntrue\n";
    let output = run_example("oop/visibility_modifiers_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_public_instance_method_execution() {
    let expected = "Base\ntrue\ntrue\ntrue\nguarded: :guarded\nhidden: :hidden\nmissing: :missing\nnil is not a symbol nor a string\n1\n";
    let output = run_example("oop/public_instance_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_public_instance_method_parens_execution() {
    let expected = "Base\ntrue\ntrue\ntrue\nguarded: :guarded\nhidden: :hidden\nmissing: :missing\nnil is not a symbol nor a string\n1\n";
    let output = run_example("oop/public_instance_method_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_public_visibility_execution() {
    let expected = ":after\n[:redefined_later]\n[]\n[:redefined_later]\n[]\n";
    let output = run_example("oop/public_visibility.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_public_visibility_parens_execution() {
    let expected = ":after\n[:redefined_later]\n[]\n[:redefined_later]\n[]\n";
    let output = run_example("oop/public_visibility_parens.rb");
    assert_eq!(output, expected);
}
