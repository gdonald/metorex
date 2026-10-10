// Mixing modules into classes, and the ancestry that follows.

use super::super::run_example;
use super::*;
#[test]
fn test_oop_modules_execution() {
    let expected = "Hello, I am Alice\nGoodbye from Alice\nI am a class with module methods\n";
    let output = run_example("oop/module/modules.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_modules_parens_execution() {
    let expected = "Hello, I am Alice\nGoodbye from Alice\nI am a class with module methods\n";
    let output = run_example("oop/module/modules_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_spaceship_execution() {
    let expected = "-1\n-1\n-1\n0\n1\n1\nnil\nnil\n";
    let output = run_example("oop/module_spaceship.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_spaceship_parens_execution() {
    let expected = "-1\n-1\n-1\n0\n1\n1\nnil\nnil\n";
    let output = run_example("oop/module_spaceship_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_append_features_hook() {
    let expected = "true\nfrozen ok\ncyclic ok\nrebind ok\n";
    let output = run_example("oop/module/append_features_hook.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_nested_execution() {
    let output = run_example("oop/module/nested.rb");
    assert_eq!(output, "hello from Inner\nwidget\n");
}

#[test]
fn test_oop_module_include_in_module_execution() {
    let output = run_example("oop/module/include_in_module.rb");
    assert_eq!(output, "hello\n");
}

#[test]
fn test_oop_module_reopen_execution() {
    let output = run_example("oop/module/reopen.rb");
    assert_eq!(output, "a\nb\n");
}

#[test]
fn test_oop_module_self_method_execution() {
    let output = run_example("oop/module/self_method.rb");
    assert_eq!(output, "from module\nalso from module\n");
}

#[test]
fn test_oop_ancestors_basics() {
    let expected = concat!(
        "[BasicObject]\n",
        "[Object, Kernel, BasicObject]\n",
        "[Kernel]\n",
        "[MSpecsAncestors]\n",
        "[MSABasic, Object, Kernel, BasicObject]\n",
        "[MSASuper, MSABasic, Object, Kernel, BasicObject]\n",
        "[MSAParent, Object, Kernel, BasicObject]\n",
        "[MSAChild, MSAParent, Object, Kernel, BasicObject]\n",
    );
    let output = run_example("oop/ancestors_basics.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_ancestors_module_include() {
    let expected = "[Basic]\n[Sup, Basic]\n---\ntrue\ntrue\ntrue\n---\n[Sup, Basic]\n";
    let output = run_example("oop/ancestors_module_include.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_ancestors_nested_include() {
    let expected = concat!(
        "[NIBasic]\n",
        "[NISuper, NIBasic]\n",
        "[NIParent, Object, Kernel, BasicObject]\n",
        "[NIChild, NISuper, NIBasic, NIParent, Object, Kernel, BasicObject]\n",
        "---\n",
        "true\n",
    );
    let output = run_example("oop/ancestors_nested_include.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_ancestors_parent_class() {
    // `puts [a, b].inspect` inspects the array and prints one line, since the
    // paren-less argument carries its trailing method call.
    let expected = "[AParent, Object, Kernel, BasicObject]\n[AParent, Object, Kernel, BasicObject]\ntrue\n---\ntrue\ntrue\ntrue\ntrue\n";
    let output = run_example("oop/ancestors_parent_class.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_ancestors_singleton() {
    let expected = concat!(
        "[#<Class:ASChild>, ASInternal, #<Class:ASParent>, #<Class:Object>, ",
        "#<Class:BasicObject>, Class, Module, Object, Kernel, BasicObject]\n",
        "---\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
    );
    let output = run_example("oop/ancestors_singleton.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_ancestors_standalone_module_singleton() {
    let expected = concat!(
        "[#<Class:ASMStandalone>, Module, Object, Kernel, BasicObject]\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
    );
    let output = run_example("oop/ancestors_standalone_module_singleton.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_extend_object_hook_execution() {
    let expected = "hello test\n:hello\n[:private_hook, :public_hook]\nfalse\nFrozenError\nfalse\n";
    let output = run_example("oop/extend_object_hook.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_extend_object_hook_parens_execution() {
    let expected = "hello test\n:hello\n[:private_hook, :public_hook]\nfalse\nFrozenError\nfalse\n";
    let output = run_example("oop/extend_object_hook_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_extended_hook_execution() {
    let expected = "[:extend_object, :extended, [:plain_extended, Object]]\ntrue\ntrue\ntrue\n";
    let output = run_example("oop/extended_hook.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_extended_hook_parens_execution() {
    let expected = "[:extend_object, :extended, [:plain_extended, Object]]\ntrue\ntrue\ntrue\n";
    let output = run_example("oop/extended_hook_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_comparison_operators_execution() {
    let expected = "false\ntrue\nfalse\nnil\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\nnil\ncompared with non class/module\n";
    let output = run_example("oop/module_comparison_operators.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_comparison_operators_parens_execution() {
    let expected = "false\ntrue\nfalse\nnil\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\nnil\ncompared with non class/module\n";
    let output = run_example("oop/module_comparison_operators_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_include_multiple_modules_execution() {
    let expected = ":first\n:second\n:third\n[Host, First, Second, Wrapper::Third]\ntrue\nfalse\nTypeError\nArgumentError\n";
    let output = run_example("oop/include_multiple_modules.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_include_multiple_modules_parens_execution() {
    let expected = ":first\n:second\n:third\n[Host, First, Second, Wrapper::Third]\ntrue\nfalse\nTypeError\nArgumentError\n";
    let output = run_example("oop/include_multiple_modules_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_include_nested_modules_execution() {
    let expected = "\"trunk\"\n\"trunk\"\n:leaf\n[:leaf_name]\n[Seedling, Sapling, Leaf]\n";
    let output = run_example("oop/include_nested_modules.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_include_nested_modules_parens_execution() {
    let expected = "\"trunk\"\n\"trunk\"\n:leaf\n[:leaf_name]\n[Seedling, Sapling, Leaf]\n";
    let output = run_example("oop/include_nested_modules_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_included_modules_execution() {
    let expected = "[]\n[Base]\n[Middle, Base, Kernel]\n[Kernel]\n";
    let output = run_example("oop/included_modules.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_included_modules_parens_execution() {
    let expected = "[]\n[Base]\n[Middle, Base, Kernel]\n[Kernel]\n";
    let output = run_example("oop/included_modules_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_included_hook_execution() {
    let expected = "[[:included, \"Host\"], :chained]\n:helped\ntrue\n";
    let output = run_example("oop/included_hook.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_included_hook_parens_execution() {
    let expected = "[[:included, \"Host\"], :chained]\n:helped\ntrue\n";
    let output = run_example("oop/included_hook_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_dup_singleton_execution() {
    let expected = "[:hello]\n[:hello]\n:hi\n[]\n[:&, :===, :=~, :^, :blank, :inspect, :nil?, :rationalize, :to_a, :to_c, :to_f, :to_h, :to_i, :to_r, :to_s, :|]\n[:build]\n:built\n";
    let output = run_example("oop/module_dup_singleton.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_dup_singleton_parens_execution() {
    let expected = "[:hello]\n[:hello]\n:hi\n[]\n[:&, :===, :=~, :^, :blank, :inspect, :nil?, :rationalize, :to_a, :to_c, :to_f, :to_h, :to_i, :to_r, :to_s, :|]\n[:build]\n:built\n";
    let output = run_example("oop/module_dup_singleton_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_subclass_instance_execution() {
    let expected = ":named\nNamespace\ntrue\n[]\n[:LIMIT]\n10\n\"A\"\n";
    let output = run_example("oop/module_subclass_instance.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_subclass_instance_parens_execution() {
    let expected = ":named\nNamespace\ntrue\n[]\n[:LIMIT]\n10\n\"A\"\n";
    let output = run_example("oop/module_subclass_instance_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_function_execution() {
    let expected = ":named\n:toggled\nfalse\nfalse\n[:named_form, :toggled]\ntrue\n:named\nNoMethodError\n[\"layered\", \"base\"]\ntrue\n";
    let output = run_example("oop/module_function.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_function_parens_execution() {
    let expected = ":named\n:toggled\nfalse\nfalse\n[:named_form, :toggled]\ntrue\n:named\nNoMethodError\n[\"layered\", \"base\"]\ntrue\n";
    let output = run_example("oop/module_function_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_name_execution() {
    let expected = "nil\n\"Outer::Inner\"\nnil\nnil\ntrue\ntrue\n\"Outer::Inner::Bound\"\n\"Outer::Inner::Bound::Nested\"\n\"Outer::Conditional\"\n\"Outer::AlsoConditional\"\ntrue\ntrue\n#<Encoding:UTF-8>\n";
    let output = run_example("oop/module_name.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_name_parens_execution() {
    let expected = "nil\n\"Outer::Inner\"\nnil\nnil\ntrue\ntrue\n\"Outer::Inner::Bound\"\n\"Outer::Inner::Bound::Nested\"\n\"Outer::Conditional\"\n\"Outer::AlsoConditional\"\ntrue\ntrue\n#<Encoding:UTF-8>\n";
    let output = run_example("oop/module_name_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_nesting_execution() {
    let expected = "[]\n[Outer]\n[Outer::Inner, Outer]\n[Outer::Inner::Nested, Outer::Inner, Outer]\ntrue\n[Outer::Inner, Outer]\n";
    let output = run_example("oop/module_nesting.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_nesting_parens_execution() {
    let expected = "[]\n[Outer]\n[Outer::Inner, Outer]\n[Outer::Inner::Nested, Outer::Inner, Outer]\ntrue\n[Outer::Inner, Outer]\n";
    let output = run_example("oop/module_nesting_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_prepend_features_hook_execution() {
    let expected = "[[:prepend_features, \"Prepender\"], [:prepended, \"Prepender\"], [:append_features, \"Includer\"], [:included, \"Includer\"]]\n:greeted\n:greeted\ntrue\n";
    let output = run_example("oop/prepend_features_hook.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_prepend_features_hook_parens_execution() {
    let expected = "[[:prepend_features, \"Prepender\"], [:prepended, \"Prepender\"], [:append_features, \"Includer\"], [:included, \"Includer\"]]\n:greeted\n:greeted\ntrue\n";
    let output = run_example("oop/prepend_features_hook_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_refinements_execution() {
    let expected = "2\ntrue\n[]\n[]\n:any\n[4, 5]\n:parens\n";
    let output = run_example("oop/module_refinements.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_refinements_parens_execution() {
    let expected = "2\ntrue\n[]\n[]\n:any\n[4, 5]\n:parens\n";
    let output = run_example("oop/module_refinements_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_to_s_execution() {
    let expected = "Named\nString\ntrue\ntrue\n#<Class:Named>\n#<Class:String>\ntrue\ntrue\n\"Refiner::Upcase\"\n#<refinement:String@Refiner>\n";
    let output = run_example("oop/module_to_s.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_to_s_parens_execution() {
    let expected = "Named\nString\ntrue\ntrue\n#<Class:Named>\n#<Class:String>\ntrue\ntrue\n\"Refiner::Upcase\"\n#<refinement:String@Refiner>\n";
    let output = run_example("oop/module_to_s_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_used_refinements_execution() {
    let expected = "[]\n2\ntrue\n[]\n[]\n";
    let output = run_example("oop/used_refinements.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_used_refinements_parens_execution() {
    let expected = "[]\n2\ntrue\n[]\n[]\n";
    let output = run_example("oop/used_refinements_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_using_execution() {
    let expected = "true\n\"plain\"\n\"refined\"\n\"refined\"\n\"plain\"\n";
    let output = run_example("oop/module_using.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_using_parens_execution() {
    let expected = "true\n\"plain\"\n\"refined\"\n\"refined\"\n\"plain\"\n";
    let output = run_example("oop/module_using_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_extend_module_execution() {
    let expected = "tagged\ntrue\ntagged\ntrue\ntagged\ntrue\nfalse\nTypeError\n-1\n";
    let output = run_example("oop/extend_module.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_reopen_instance_methods_execution() {
    let expected = concat!(
        "module method on Mixin\n",
        "module method on Widget\n",
        "class method on Widget\n",
        "true\n",
    );
    let output = run_example("oop/module_reopen_instance_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_reopen_instance_methods_parens_execution() {
    let expected = concat!(
        "module method on Mixin\n",
        "module method on Widget\n",
        "class method on Widget\n",
        "true\n",
    );
    let output = run_example("oop/module_reopen_instance_methods_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_prepend_ancestry_execution() {
    let expected = concat!(
        "logged: report\n",
        "Logging\n",
        "Report\n",
        "true\n",
        // A constant read inside `def self.channel` is the one the class
        // itself holds, not the one the prepended module does.
        "report\n",
        "true\n",
        "detailed: logged: report\n",
        "2\n",
        "ArgumentError\n",
        "super: no superclass method 'describe' for an instance of Alone\n",
    );
    let output = run_example("oop/prepend_ancestry.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_prepend_ancestry_parens_execution() {
    let expected = concat!(
        "logged: report\n",
        "Logging\n",
        "Report\n",
        "true\n",
        // A constant read inside `def self.channel` is the one the class
        // itself holds, not the one the prepended module does.
        "report\n",
        "true\n",
        "detailed: logged: report\n",
        "2\n",
        "ArgumentError\n",
        "super: no superclass method 'describe' for an instance of Alone\n",
    );
    let output = run_example("oop/prepend_ancestry_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_prepend_private_visibility_execution() {
    let expected = concat!(
        "2\n",
        "private method 'hidden' called for an instance of Vault\n",
    );
    let output = run_example("oop/prepend_private_visibility.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_prepend_private_visibility_parens_execution() {
    let expected = concat!(
        "2\n",
        "private method 'hidden' called for an instance of Vault\n",
    );
    let output = run_example("oop/prepend_private_visibility_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_refinement_names_target_execution() {
    let output = run_example("oop/refinement_names/target.rb");
    assert_eq!(output, REFINEMENT_NAMES_OUTPUT);
}

#[test]
fn test_oop_refinement_names_target_parens_execution() {
    let output = run_example("oop/refinement_names/target_parens.rb");
    assert_eq!(output, REFINEMENT_NAMES_OUTPUT);
}

#[test]
fn test_oop_module_reopening_execution() {
    let output = run_example("oop/module_reopening.rb");
    assert_eq!(output, MODULE_REOPENING_OUTPUT);
}

#[test]
fn test_oop_module_reopening_parens_execution() {
    let output = run_example("oop/module_reopening_parens.rb");
    assert_eq!(output, MODULE_REOPENING_OUTPUT);
}

#[test]
fn test_oop_refinement_indirect_calls_execution() {
    let output = run_example("oop/refinement_indirect_calls.rb");
    assert_eq!(output, REFINEMENT_INDIRECT_CALLS_OUTPUT);
}

#[test]
fn test_oop_refinement_indirect_calls_no_parens_execution() {
    let output = run_example("oop/refinement_indirect_calls_no_parens.rb");
    assert_eq!(output, REFINEMENT_INDIRECT_CALLS_OUTPUT);
}

const AUTOLOAD_ENCLOSING_SUPERCLASS_OUTPUT: &str =
    concat!("\"truck after carrier\"\n", "Shipping::Carrier\n",);

#[test]
fn test_oop_autoload_enclosing_superclass_execution() {
    let output = run_example("oop/autoload_enclosing_superclass.rb");
    assert_eq!(output, AUTOLOAD_ENCLOSING_SUPERCLASS_OUTPUT);
}

#[test]
fn test_oop_autoload_enclosing_superclass_no_parens_execution() {
    let output = run_example("oop/autoload_enclosing_superclass_no_parens.rb");
    assert_eq!(output, AUTOLOAD_ENCLOSING_SUPERCLASS_OUTPUT);
}

const AUTOLOAD_REOPENING_OUTPUT: &str = concat!(
    "\"constant\"\n",
    "\"constant\"\n",
    "75\n",
    "true\n",
    "1042\n",
);

#[test]
fn test_oop_autoload_reopening_execution() {
    let output = run_example("oop/autoload_reopening.rb");
    assert_eq!(output, AUTOLOAD_REOPENING_OUTPUT);
}

#[test]
fn test_oop_autoload_reopening_no_parens_execution() {
    let output = run_example("oop/autoload_reopening_no_parens.rb");
    assert_eq!(output, AUTOLOAD_REOPENING_OUTPUT);
}
