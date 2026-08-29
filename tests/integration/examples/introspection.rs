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
    let expected = r#"greet.source_location = 1:1
calculate.source_location = 5:1
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
fn test_introspection_instance_variable_queries_execution() {
    let expected = concat!(
        "true\ntrue\nfalse\nfalse\nTypeError\n",
        "false\nfalse\nfalse\n",
        "true\nfalse\nfalse\nfalse\n"
    );
    let output = run_example("introspection/instance_variable_queries.rb");
    assert_eq!(output, expected);
}
