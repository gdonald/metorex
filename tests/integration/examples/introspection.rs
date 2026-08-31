use super::run_example;

#[test]
fn test_introspection_function_name_execution() {
    let expected = r#"greet
calculate
"#;
    let output = run_example("introspection/function/name.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_function_module_execution() {
    let expected = r#"main
main
"#;
    let output = run_example("introspection/function/module.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_code_object_execution() {
    let expected = r#"greet defined in code_object.rb line 1
calculate defined in code_object.rb line 5
"#;
    let output = run_example("introspection/code_object.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_closure_namespace_execution() {
    let expected = r#"simple_func
simple_func
nil
Proc
Proc
Binding
18
"#;
    let output = run_example("introspection/closure_namespace.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_basic_attributes_execution() {
    let expected = r#"greet.name = greet
calculate.name = calculate
greet.doc = nil
calculate.doc = nil
"#;
    let output = run_example("introspection/basic_attributes.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_annotations_execution() {
    let expected = r#"add.parameters = [x, y]
greet.parameters = [name]
process.parameters = [data, count, flag]
no_annotations.parameters = [a, b]
"#;
    let output = run_example("introspection/annotations.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_default_parameters_execution() {
    let expected = r#"no_defaults
[a, b]
with_defaults
[a, b, c]
all_defaults
[x, y, z]
greet
[name, greeting, punctuation]
"#;
    let output = run_example("introspection/default_parameters.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_builtin_class_method() {
    let expected = "Array\nString\nInteger\nFloat\nTrueClass\nNilClass\nRange\nHash\nSet\n";
    let output = run_example("introspection/builtin_class_method/builtin_class_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_builtin_class_method_no_parens() {
    let expected = "Array\nString\nInteger\nFloat\nTrueClass\nNilClass\nRange\nHash\nSet\n";
    let output =
        run_example("introspection/builtin_class_method/builtin_class_method_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_instance_variable_set_frozen() {
    let expected = "true: FrozenError\nfalse: FrozenError\nnil: FrozenError\ninteger: FrozenError\nsymbol: FrozenError\ninstance: 99\nfrozen instance: FrozenError\nrescued via RuntimeError\n";
    let output = run_example("introspection/instance_variable_set_frozen.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_instance_variable_set_frozen_no_parens() {
    let expected = "true: FrozenError\nfalse: FrozenError\nnil: FrozenError\ninteger: FrozenError\nsymbol: FrozenError\ninstance: 99\nfrozen instance: FrozenError\nrescued via RuntimeError\n";
    let output = run_example("introspection/instance_variable_set_frozen_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_class_of_class_execution() {
    let expected = "Class\nClass\nClass\nClass\nClass\nModule\nGreeter\ntrue\ntrue\n";
    let output = run_example("introspection/class_of_class.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_binding_context_execution() {
    let expected = "Binding\ntrue\n99\ntrue\npassword\n4\n4\nfalse\ntrue\ntrue\n";
    let output = run_example("introspection/binding_context.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_block_given_context_execution() {
    let expected =
        "true\nfalse\ntrue\nfalse\ntrue\nfalse\ntrue\nfalse\ntrue\nfalse\nfalse\nfalse\n";
    let output = run_example("introspection/block_given_context.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_instance_variable_get_names() {
    let expected = concat!(
        "hello\nhello\nhello\nnil\nnil\n",
        "NameError: `@' is not allowed as an instance variable name\n",
        "NameError: `@0' is not allowed as an instance variable name\n",
        "NameError: `@@greeting' is not allowed as an instance variable name\n",
        "NameError: `greeting' is not allowed as an instance variable name\n",
        "TypeError: no implicit conversion of Integer into String\n"
    );
    let output = run_example("introspection/instance_variable_get_names.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_instance_variable_get_names_no_parens() {
    let expected = concat!(
        "hello\nhello\nhello\nnil\nnil\n",
        "NameError: `@' is not allowed as an instance variable name\n",
        "NameError: `@0' is not allowed as an instance variable name\n",
        "NameError: `@@greeting' is not allowed as an instance variable name\n",
        "NameError: `greeting' is not allowed as an instance variable name\n",
        "TypeError: no implicit conversion of Integer into String\n"
    );
    let output = run_example("introspection/instance_variable_get_names_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_instance_variable_set_names() {
    let expected = concat!(
        "hi\nhey\nhowdy\nhowdy\n42\n",
        "NameError: `@' is not allowed as an instance variable name\n",
        "NameError: `@0' is not allowed as an instance variable name\n",
        "NameError: `@@greeting' is not allowed as an instance variable name\n",
        "NameError: `greeting' is not allowed as an instance variable name\n",
        "NameError: `greeting' is not allowed as an instance variable name\n",
        "FrozenError: can't modify frozen NilClass: nil\n",
        "TypeError: no implicit conversion of Integer into String\n"
    );
    let output = run_example("introspection/instance_variable_set_names.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_instance_variable_set_names_no_parens() {
    let expected = concat!(
        "hi\nhey\nhowdy\nhowdy\n42\n",
        "NameError: `@' is not allowed as an instance variable name\n",
        "NameError: `@0' is not allowed as an instance variable name\n",
        "NameError: `@@greeting' is not allowed as an instance variable name\n",
        "NameError: `greeting' is not allowed as an instance variable name\n",
        "NameError: `greeting' is not allowed as an instance variable name\n",
        "FrozenError: can't modify frozen NilClass: nil\n",
        "TypeError: no implicit conversion of Integer into String\n"
    );
    let output = run_example("introspection/instance_variable_set_names_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_instance_variables_order() {
    let expected = concat!(
        "[:@name, :@servings, :@vegetarian]\n",
        "[:@name, :@servings, :@vegetarian, :@rating]\n",
        "[]\n[]\n[]\n"
    );
    let output = run_example("introspection/instance_variables_order.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_instance_variables_order_no_parens() {
    let expected = concat!(
        "[:@name, :@servings, :@vegetarian]\n",
        "[:@name, :@servings, :@vegetarian, :@rating]\n",
        "[]\n[]\n[]\n"
    );
    let output = run_example("introspection/instance_variables_order_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_itself() {
    let expected = concat!(
        "gear\ntrue\n42\ntext\n:symbol\nnil\n",
        "[3, 1, 2]\n[3, 1, 2]\nWidget\nArgumentError\n"
    );
    let output = run_example("introspection/itself.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_itself_no_parens() {
    let expected = concat!(
        "gear\ntrue\n42\ntext\n:symbol\nnil\n",
        "[3, 1, 2]\n[3, 1, 2]\nWidget\nArgumentError\n"
    );
    let output = run_example("introspection/itself_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_local_variables() {
    let expected = concat!(
        "[:top_level_one, :top_level_two]\n",
        "[:inside_one, :inside_two]\n",
        "[:shadowed]\n",
        "[:bound_one, :bound_two]\n",
        "[:in_block]\n",
        "[:collected, :evaluated_one, :evaluated_two, :top_level_one, :top_level_two]\n"
    );
    let output = run_example("introspection/local_variables.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_local_variables_no_parens() {
    let expected = concat!(
        "[:top_level_one, :top_level_two]\n",
        "[:inside_one, :inside_two]\n",
        "[:shadowed]\n",
        "[:bound_one, :bound_two]\n",
        "[:in_block]\n",
        "[:binding_locals, :collected, :evaluated_one, :evaluated_two, ",
        ":top_level_one, :top_level_two]\n"
    );
    let output = run_example("introspection/local_variables_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_method_via_respond_to_missing() {
    let expected = concat!(
        "Method\n",
        "called haunt with []\n",
        "called haunt with [1, 2]\n",
        "called whisper with [\"softly\"]\n",
        "NameError: undefined method 'unknown' for class 'Ghost'\n",
        ":only_name\nArgumentError\nSHOUT\n",
        "TypeError: no implicit conversion of Object into String\n"
    );
    let output = run_example("introspection/method_via_respond_to_missing.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_method_via_respond_to_missing_no_parens() {
    let expected = concat!(
        "Method\n",
        "called haunt with []\n",
        "called haunt with [1, 2]\n",
        "called whisper with [\"softly\"]\n",
        "NameError: undefined method 'unknown' for class 'Ghost'\n",
        ":only_name\nArgumentError\nSHOUT\n",
        "TypeError: no implicit conversion of Object into String\n"
    );
    let output = run_example("introspection/method_via_respond_to_missing_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_methods_listing() {
    let expected = concat!(
        "[]\n[:polish]\n[:polish]\ntrue\ntrue\n",
        "[:buff, :polish]\n[:buff]\n",
        "false\ntrue\nSymbol\nfalse\n6\n",
        "[2, 3]\n[1, 2, 3, 4]\n"
    );
    let output = run_example("introspection/methods_listing.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_methods_listing_no_parens() {
    let expected = concat!(
        "[]\n[:polish]\n[:polish]\ntrue\ntrue\n",
        "[:buff, :polish]\n[:buff]\n",
        "false\ntrue\nSymbol\nfalse\n6\n",
        "[2, 3]\n[1, 2, 3, 4]\n"
    );
    let output = run_example("introspection/methods_listing_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_object_id() {
    let expected = concat!(
        "true\ntrue\nfalse\nfalse\n",
        "true\nfalse\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\n",
        "4\n2\n0\n3\nfalse\nfalse\n"
    );
    let output = run_example("introspection/object_id.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_object_id_no_parens() {
    let expected = concat!(
        "true\ntrue\nfalse\nfalse\n",
        "true\nfalse\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\n",
        "4\n2\n0\n3\nfalse\nfalse\n"
    );
    let output = run_example("introspection/object_id_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_private_methods() {
    let expected = concat!(
        "[:child_secret]\n",
        "[:child_secret, :parent_secret]\n",
        "[:child_class_secret, :parent_class_secret]\n",
        "[:child_class_secret, :child_secret, :parent_class_secret, :parent_secret]\n",
        "[:child_secret]\n[:singleton_secret]\ntrue\n5\nnil\nfalse\n"
    );
    let output = run_example("introspection/private_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_private_methods_no_parens() {
    let expected = concat!(
        "[:child_secret]\n",
        "[:child_secret, :parent_secret]\n",
        "[:child_class_secret, :parent_class_secret]\n",
        "[:child_class_secret, :child_secret, :parent_class_secret, :parent_secret]\n",
        "[:child_secret]\n[:singleton_secret]\ntrue\n5\nnil\nfalse\n"
    );
    let output = run_example("introspection/private_methods_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_protected_methods() {
    let expected = concat!(
        "[:child_guard]\n",
        "[:child_guard, :mixed_in_guard, :parent_guard]\n",
        "[:child_class_guard, :parent_class_guard]\n",
        "[:child_guard]\n[:singleton_guard]\ntrue\nfalse\n"
    );
    let output = run_example("introspection/protected_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_protected_methods_no_parens() {
    let expected = concat!(
        "[:child_guard]\n",
        "[:child_guard, :mixed_in_guard, :parent_guard]\n",
        "[:child_class_guard, :parent_class_guard]\n",
        "[:child_guard]\n[:singleton_guard]\ntrue\nfalse\n"
    );
    let output = run_example("introspection/protected_methods_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_public_method() {
    let expected = concat!(
        ":opened\n:built\n",
        "NameError: undefined method 'hidden' for class 'Vault'\n",
        "NameError: undefined method 'guarded' for class 'Vault'\n",
        "called publicly_handled\ncalled privately_handled\nNameError\n"
    );
    let output = run_example("introspection/public_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_public_method_no_parens() {
    let expected = concat!(
        ":opened\n:built\n",
        "NameError: undefined method 'hidden' for class 'Vault'\n",
        "NameError: undefined method 'guarded' for class 'Vault'\n",
        "called publicly_handled\ncalled privately_handled\nNameError\n"
    );
    let output = run_example("introspection/public_method_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_public_methods() {
    let expected = concat!(
        "[:child_open]\n",
        "[:child_open, :mixed_in_open, :opens, :parent_open]\n",
        "[:child_class_open, :parent_class_open]\n",
        "[:child_open]\nfalse\nfalse\n",
        "[3, 1]\n[-4, -3]\n[-4, 3]\ntrue\n",
        "ZeroDivisionError: divided by 0\n"
    );
    let output = run_example("introspection/public_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_public_methods_no_parens() {
    let expected = concat!(
        "[:child_open]\n",
        "[:child_open, :mixed_in_open, :opens, :parent_open]\n",
        "[:child_class_open, :parent_class_open]\n",
        "[:child_open]\nfalse\nfalse\n",
        "[3, 1]\n[-4, -3]\n[-4, 3]\ntrue\n",
        "ZeroDivisionError: divided by 0\n"
    );
    let output = run_example("introspection/public_methods_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_remove_instance_variable() {
    let expected = concat!(
        "[:@greeting, :@name]\n\"hello\"\n[:@name]\nfalse\n",
        "\"world\"\n[]\n",
        "NameError: instance variable @unknown not defined\n",
        "NameError: `@0' is not allowed as an instance variable name\n",
        "TypeError\n\"hello\"\nFrozenError\nFrozenError\nNameError\ntrue\n"
    );
    let output = run_example("introspection/remove_instance_variable.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_remove_instance_variable_no_parens() {
    let expected = concat!(
        "[:@greeting, :@name]\n\"hello\"\n[:@name]\nfalse\n",
        "\"world\"\n[]\n",
        "NameError: instance variable @unknown not defined\n",
        "NameError: `@0' is not allowed as an instance variable name\n",
        "TypeError\n\"hello\"\nFrozenError\nFrozenError\nNameError\ntrue\n"
    );
    let output = run_example("introspection/remove_instance_variable_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_respond_to_missing() {
    let expected = concat!(
        "true\nfalse\ntrue\nfalse\n",
        "true\nfalse\ntrue\ntrue\nfalse\ntrue\n",
        "true\nfalse\ntrue\ntrue\n"
    );
    let output = run_example("introspection/respond_to_missing.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_respond_to_missing_no_parens() {
    let expected = concat!(
        "true\nfalse\ntrue\nfalse\n",
        "true\nfalse\ntrue\ntrue\nfalse\ntrue\n",
        "true\nfalse\ntrue\ntrue\n"
    );
    let output = run_example("introspection/respond_to_missing_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_respond_to() {
    let expected = concat!(
        "true\nfalse\ntrue\nfalse\ntrue\nfalse\ntrue\n",
        "false\nfalse\ntrue\n",
        "NoMethodError: private method 'new' called for Sealed\n",
        "true\ntrue\ntrue\n",
        "TypeError: 42 is not a symbol nor a string\n"
    );
    let output = run_example("introspection/respond_to.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_respond_to_no_parens() {
    let expected = concat!(
        "true\nfalse\ntrue\nfalse\ntrue\nfalse\ntrue\n",
        "false\nfalse\ntrue\n",
        "NoMethodError: private method 'new' called for Sealed\n",
        "true\ntrue\ntrue\n",
        "TypeError: 42 is not a symbol nor a string\n"
    );
    let output = run_example("introspection/respond_to_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_singleton_method() {
    let expected = concat!(
        "Method\n:shiny\n:included\n:prepended\n:extended\n",
        ":from_class\nNameError\nNameError\n:found\n"
    );
    let output = run_example("introspection/singleton_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_singleton_method_no_parens() {
    let expected = concat!(
        "Method\n:shiny\n:included\n:prepended\n:extended\n",
        ":from_class\nNameError\nNameError\n:found\n"
    );
    let output = run_example("introspection/singleton_method_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_singleton_methods() {
    let expected = concat!(
        "[]\n[:polish]\n[:greet, :polish]\n[:polish]\n",
        "[:child_class_method, :opened_on_child, :parent_class_method]\n",
        "[:child_class_method, :opened_on_child]\n",
        "[:parent_class_method]\n:assisted\n[:assist]\n",
        "[2, 3, 4]\n[2, 3]\n[3, 4, 5]\n[1, 2, 3]\n[4, 5]\nnil\n"
    );
    let output = run_example("introspection/singleton_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_singleton_methods_no_parens() {
    let expected = concat!(
        "[]\n[:polish]\n[:greet, :polish]\n[:polish]\n",
        "[:child_class_method, :opened_on_child, :parent_class_method]\n",
        "[:child_class_method, :opened_on_child]\n",
        "[:parent_class_method]\n:assisted\n[:assist]\n",
        "[2, 3, 4]\n[2, 3]\n[3, 4, 5]\n[1, 2, 3]\n[4, 5]\nnil\n"
    );
    let output = run_example("introspection/singleton_methods_no_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_instance_variable_queries_execution() {
    let expected = concat!(
        "true\ntrue\nfalse\nfalse\nTypeError\n",
        "false\nfalse\nfalse\n",
        "true\nfalse\nfalse\nfalse\n"
    );
    let output = run_example("introspection/instance_variable_queries.rb");
    assert_eq!(output, expected);
}
