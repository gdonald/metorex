use super::run_example;

#[test]
fn test_oop_top_level_include_execution() {
    let expected = "true\nhi from greeter\n";
    let output = run_example("oop/top_level_include.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_basic_execution() {
    let expected = "Buddy\nGolden Retriever\nSome sound -> Woof!\nI am an animal named Buddy\n";
    let output = run_example("oop/super/basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_chain_basic_execution() {
    let expected = "GrandParent\nParent\nChild\n";
    let output = run_example("oop/super/chain_basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_simple_execution() {
    let expected = "AB\n";
    let output = run_example("oop/test/super_simple.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_reader_execution() {
    let expected = "Alice\n30\n";
    let output = run_example("oop/attr/reader.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_writer_execution() {
    let expected = "Unknown\n0\nBob\n25\n";
    let output = run_example("oop/attr/writer.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_accessor_execution() {
    let expected = "Charlie\n35\ncharlie@example.com\nCharles\n36\ncharles@example.com\n";
    let output = run_example("oop/attr/accessor.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_mixed_args_execution() {
    let expected = "1\n2\n10\n20\n42\n";
    let output = run_example("oop/attr/mixed_args.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_keyword_form_execution() {
    let expected = "42\nhello\n1\n2\n[:foo, :bar]\n[:baz]\n[:qux, :qux=]\n";
    let output = run_example("oop/attr/keyword_form.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_dynamic_arg_execution() {
    let expected = "true\nhello\n";
    let output = run_example("oop/attr/dynamic_arg.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_protected_visibility_execution() {
    let expected = "OK reader raised: private method 'foo' called for an instance of \nOK writer raised: private method 'foo=' called for an instance of \n";
    let output = run_example("oop/attr/protected_attr.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_str_execution() {
    let expected = "Person: Alice\n";
    let output = run_example("oop/test/str.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_repr_execution() {
    let expected = "Point(0, 0)\n";
    let output = run_example("oop/test/repr.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_special_methods_execution() {
    let expected = "Book: Ruby Guide\nMagazine: Tech Monthly\nnext_value\n";
    let output = run_example("oop/special_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_iter_execution() {
    let expected = "next\n";
    let output = run_example("oop/test/iter.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_method_missing_execution() {
    let expected = "bar\n42\n1\n2\n3\n";
    let output = run_example("oop/test/method_missing.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_scope_resolution_execution() {
    let expected = "1\n100\n";
    let output = run_example("oop/scope_resolution/scope_resolution.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_scope_resolution_parens_execution() {
    let expected = "1\n100\n";
    let output = run_example("oop/scope_resolution/scope_resolution_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_execution() {
    let expected = "(4, 6)\ntrue\nfalse\n";
    let output = run_example("oop/operator_methods/operator_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_parens_execution() {
    let expected = "(4, 6)\ntrue\nfalse\n";
    let output = run_example("oop/operator_methods/operator_methods_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_index_execution() {
    let expected = "42\n99\n0\n";
    let output = run_example("oop/operator_methods/index.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_index_parens_execution() {
    let expected = "42\n99\n0\n";
    let output = run_example("oop/operator_methods/index_parens.rb");
    assert_eq!(output, expected);
}

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
fn test_oop_super_keyword_execution() {
    let expected = "Rex makes a sound\nRex barks\nAnimal: Rex, Breed: Labrador\n";
    let output = run_example("oop/super/keyword.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_keyword_parens_execution() {
    let expected = "Rex makes a sound\nRex barks\nAnimal: Rex, Breed: Labrador\n";
    let output = run_example("oop/super/keyword_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_extended_execution() {
    let expected = "-1\n1\n0\n11\n";
    let output = run_example("oop/operator_methods/extended.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_extended_parens_execution() {
    let expected = "-1\n1\n0\n11\n";
    let output = run_example("oop/operator_methods/extended_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_multi_arg_bracket_execution() {
    let expected = "1\n2\n3\n4\n";
    let output = run_example("oop/multi_arg_bracket/multi_arg_bracket.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_multi_arg_bracket_parens_execution() {
    let expected = "1\n2\n3\n4\n";
    let output = run_example("oop/multi_arg_bracket/multi_arg_bracket_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_empty_bracket_call_execution() {
    let expected = "3\n";
    let output = run_example("oop/empty_bracket_call/empty_bracket_call.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_empty_bracket_call_parens_execution() {
    let expected = "3\n";
    let output = run_example("oop/empty_bracket_call/empty_bracket_call_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_class_self_new_execution() {
    let expected = "make: self=Foo\ninit called, @x=42\nmake: inst.class=Foo\nf.class=Foo\n";
    let output = run_example("oop/class/self_new.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_comparable_execution() {
    let expected = "true\nfalse\ntrue\nfalse\ntrue\ntrue\nfalse\ntrue\n";
    let output = run_example("oop/comparable/comparable.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_comparable_parens_execution() {
    let expected = "true\nfalse\ntrue\nfalse\ntrue\ntrue\nfalse\ntrue\n";
    let output = run_example("oop/comparable/comparable_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_case_compare_extend_execution() {
    let expected = "Basic === obj: true\nSuper === obj: true\nobj.is_a?(Basic): true\nobj.is_a?(Super): true\nBasic === Child.new: true\nSuper === Child.new: true\n";
    let output = run_example("oop/case_compare_extend.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_case_compare_extend_parens_execution() {
    let expected = "Basic === obj: true\nSuper === obj: true\nobj.is_a?(Basic): true\nobj.is_a?(Super): true\nBasic === Child.new: true\nSuper === Child.new: true\n";
    let output = run_example("oop/case_compare_extend_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_comparable_singleton_spaceship_execution() {
    let expected = "true\n1\n";
    let output = run_example("oop/comparable/singleton_spaceship.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_comparable_singleton_class_expr_execution() {
    let expected = "true\n";
    let output = run_example("oop/comparable/singleton_class_expr.rb");
    assert_eq!(output, expected);
}

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
fn test_oop_alias_method_strings_execution() {
    let output = run_example("oop/alias_method_strings.rb");
    assert_eq!(output, "original\noriginal\n");
}

#[test]
fn test_oop_class_reopen_execution() {
    let output = run_example("oop/class/reopen.rb");
    assert_eq!(output, "bar\nbaz\n");
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
    let expected = concat!(
        "[Basic]\n",
        "[Sup, Basic]\n",
        "---\n",
        "true\n",
        "true\n",
        "true\n",
        "---\n",
        "[Sup, Basic]\n",
    );
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
    let expected = concat!(
        "[AParent, Object, Kernel, BasicObject]\n",
        "[AParent, Object, Kernel, BasicObject]\n",
        "true\n",
        "---\n",
        "true\n",
        "true\n",
        "true\n",
        "true\n",
    );
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
fn test_oop_toplevel_class_reopen_execution() {
    let expected = ":reopened_toplevel\n:toplevel_module\nfalse\nfalse\nWrapper::Parent\n";
    let output = run_example("oop/toplevel_class_reopen.rb");
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
    let expected = "[:hello]\n[:hello]\n:hi\n[]\n[:blank]\n[:build]\n:built\n";
    let output = run_example("oop/module_dup_singleton.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_dup_singleton_parens_execution() {
    let expected = "[:hello]\n[:hello]\n:hi\n[]\n[:blank]\n[:build]\n:built\n";
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
    let expected = "nil\n\"Outer::Inner\"\nnil\nnil\ntrue\ntrue\n\"Outer::Inner::Bound\"\n\"Outer::Inner::Bound::Nested\"\n\"Outer::Conditional\"\n\"Outer::AlsoConditional\"\ntrue\ntrue\nUTF-8\n";
    let output = run_example("oop/module_name.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_module_name_parens_execution() {
    let expected = "nil\n\"Outer::Inner\"\nnil\nnil\ntrue\ntrue\n\"Outer::Inner::Bound\"\n\"Outer::Inner::Bound::Nested\"\n\"Outer::Conditional\"\n\"Outer::AlsoConditional\"\ntrue\ntrue\nUTF-8\n";
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
fn test_oop_keyword_method_names_execution() {
    let expected = ":original\n:original\n:original\n[:alias, :meth]\n";
    let output = run_example("oop/keyword_method_names.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_keyword_method_names_parens_execution() {
    let expected = ":original\n:original\n:original\n[:alias, :meth]\n";
    let output = run_example("oop/keyword_method_names_parens.rb");
    assert_eq!(output, expected);
}

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
fn test_oop_set_temporary_name_execution() {
    let expected = "nil\ntrue\n\"fake_name\"\nfake_name\n\"Template[\x27foo.rb\x27]\"\nnil\ntrue\n\"host::Inner\"\nnil\n\"\": empty class/module name\n\"Object\": the temporary name must not be a constant path to avoid confusion\n\"A::B\": the temporary name must not be a constant path to avoid confusion\n\"::A\": the temporary name must not be a constant path to avoid confusion\ncan\'t change permanent name\n";
    let output = run_example("oop/set_temporary_name.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_set_temporary_name_parens_execution() {
    let expected = "nil\ntrue\n\"fake_name\"\nfake_name\n\"Template[\x27foo.rb\x27]\"\nnil\ntrue\n\"host::Inner\"\nnil\n\"\": empty class/module name\n\"Object\": the temporary name must not be a constant path to avoid confusion\n\"A::B\": the temporary name must not be a constant path to avoid confusion\n\"::A\": the temporary name must not be a constant path to avoid confusion\ncan\'t change permanent name\n";
    let output = run_example("oop/set_temporary_name_parens.rb");
    assert_eq!(output, expected);
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
