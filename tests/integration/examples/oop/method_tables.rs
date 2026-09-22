// Adding, removing and renaming the methods of a class.

use super::super::run_example;
#[test]
fn test_oop_alias_method_strings_execution() {
    let output = run_example("oop/alias_method_strings.rb");
    assert_eq!(output, "original\noriginal\n");
}

#[test]
fn test_oop_unbound_method_execution() {
    let expected = ":from_module\n:from_base\n:from_base\n1\n:missing\n42 is not a symbol nor a string\n:label\n";
    let output = run_example("oop/unbound_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_unbound_method_parens_execution() {
    let expected = ":from_module\n:from_base\n:from_base\n1\n:missing\n42 is not a symbol nor a string\n:label\n";
    let output = run_example("oop/unbound_method_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_instance_methods_visibility_execution() {
    let expected = "[:protected_parent, :public_parent]\n[:public_parent]\n[:protected_parent]\n[:private_parent]\n[:public_child]\ntrue\nfalse\nNoMethodError\n";
    let output = run_example("oop/instance_methods_visibility.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_instance_methods_visibility_parens_execution() {
    let expected = "[:protected_parent, :public_parent]\n[:public_parent]\n[:protected_parent]\n[:private_parent]\n[:public_child]\ntrue\nfalse\nNoMethodError\n";
    let output = run_example("oop/instance_methods_visibility_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_method_added_hook_execution() {
    let expected = "[[:singleton, :singleton_method_added], [:added, :first], [:added, :aliased], [:added, :aliased_again], [:added, :inherited_method], [:added, :retired]]\nfalse\n[:aliased, :aliased_again, :first, :inherited_method]\nnil\ntrue\n";
    let output = run_example("oop/method_added_hook.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_method_added_hook_parens_execution() {
    let expected = "[[:singleton, :singleton_method_added], [:added, :first], [:added, :aliased], [:added, :aliased_again], [:added, :inherited_method], [:added, :retired]]\nfalse\n[:aliased, :aliased_again, :first, :inherited_method]\nnil\ntrue\n";
    let output = run_example("oop/method_added_hook_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_method_defined_visibility_execution() {
    let expected = "public_mixin true true false false\nprotected_mixin true false true false\nprivate_mixin false false false true\npublic_holder true true false false\nprivate_holder false false false true\n[:private_mixin]\n[:protected_mixin]\ntrue\nfalse\n42 is not a symbol nor a string\n";
    let output = run_example("oop/method_defined_visibility.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_method_defined_visibility_parens_execution() {
    let expected = "public_mixin true true false false\nprotected_mixin true false true false\nprivate_mixin false false false true\npublic_holder true true false false\nprivate_holder false false false true\n[:private_mixin]\n[:protected_mixin]\ntrue\nfalse\n42 is not a symbol nor a string\n";
    let output = run_example("oop/method_defined_visibility_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_method_removed_hook_execution() {
    let expected = "[[:removed, :doomed], [:undefined, :shadowed]]\n[]\nnil\ntrue\ncan\'t modify frozen Module: \n";
    let output = run_example("oop/method_removed_hook.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_method_removed_hook_parens_execution() {
    let expected = "[[:removed, :doomed], [:undefined, :shadowed]]\n[]\nnil\ntrue\ncan\'t modify frozen Module: \n";
    let output = run_example("oop/method_removed_hook_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_remove_class_variable_execution() {
    let expected = ":shared\nfalse\n:own\n@@shared: NameError\n@shared: NameError\nshared: NameError\n@@absent: NameError\nfalse\n:no_block\n";
    let output = run_example("oop/remove_class_variable.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_remove_class_variable_parens_execution() {
    let expected = ":shared\nfalse\n:own\n@@shared: NameError\n@shared: NameError\nshared: NameError\n@@absent: NameError\nfalse\n:no_block\n";
    let output = run_example("oop/remove_class_variable_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_remove_const_execution() {
    let expected = ":doomed\n:also\n[:KEPT]\nname: NameError\n__CONSTX__: NameError\n@Name: NameError\nName=: NameError\nMissing: NameError\ninherited: NameError\nnil\ntrue\ntrue\n";
    let output = run_example("oop/remove_const.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_remove_const_parens_execution() {
    let expected = ":doomed\n:also\n[:KEPT]\nname: NameError\n__CONSTX__: NameError\n@Name: NameError\nName=: NameError\nMissing: NameError\ninherited: NameError\nnil\ntrue\ntrue\n";
    let output = run_example("oop/remove_const_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_remove_method_execution() {
    let expected = "Child\n[]\n:parent\ninherited: NameError\nmissing: NameError\nChild\nfrozen: FrozenError\ntrue\n-1\ntrue\n";
    let output = run_example("oop/remove_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_remove_method_parens_execution() {
    let expected = "Child\n[]\n:parent\ninherited: NameError\nmissing: NameError\nChild\nfrozen: FrozenError\ntrue\n-1\ntrue\n";
    let output = run_example("oop/remove_method_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_undef_method_execution() {
    let expected = "Child\nfalse\n:parent\nundefined method \'never_defined\' for class \'Child\'\nundefined method \'not_exist\' for class \'String\'\nChild\nfrozen: FrozenError\n/Hello World/\ntrue\n\"a\\\\.b\\\\*c\"\n";
    let output = run_example("oop/undef_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_undef_method_parens_execution() {
    let expected = "Child\nfalse\n:parent\nundefined method \'never_defined\' for class \'Child\'\nundefined method \'not_exist\' for class \'String\'\nChild\nfrozen: FrozenError\n/Hello World/\ntrue\n\"a\\\\.b\\\\*c\"\n";
    let output = run_example("oop/undef_method_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_undefined_instance_methods_execution() {
    let expected = "[:retired]\n[:from_module, :kept, :own]\n[]\nfalse\nnil\n";
    let output = run_example("oop/undefined_instance_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_undefined_instance_methods_parens_execution() {
    let expected = "[:retired]\n[:from_module, :kept, :own]\n[]\nfalse\nnil\n";
    let output = run_example("oop/undefined_instance_methods_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_undef_keyword_execution() {
    let expected = "hello\nfalse\nfalse\ngreet undefined\n";
    let output = run_example("oop/undef_keyword.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_send_dispatch_execution() {
    let expected = concat!(
        "hello\nhello\nhello\n",
        "[:one]\n",
        "[:one, :two]\n",
        "42 is not a symbol nor a string\n",
        "no method name given\n",
        "NoMethodError for a name nothing defines\n",
        "[:!, :!=, :==, :__id__, :__send__, :equal?, :instance_eval, :instance_exec]\n",
        "true\n",
        "[:first, :second]\n"
    );
    let output = run_example("oop/send/dispatch.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_send_dispatch_parens_execution() {
    let expected = concat!(
        "hello\nhello\nhello\n",
        "[:one]\n",
        "[:one, :two]\n",
        "42 is not a symbol nor a string\n",
        "no method name given\n",
        "NoMethodError for a name nothing defines\n",
        "[:!, :!=, :==, :__id__, :__send__, :equal?, :instance_eval, :instance_exec]\n",
        "true\n",
        "[:first, :second]\n"
    );
    let output = run_example("oop/send/dispatch_parens.rb");
    assert_eq!(output, expected);
}
