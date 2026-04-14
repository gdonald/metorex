use super::run_example;

#[test]
fn test_introspection_function_name_execution() {
    let expected = r#"greet
calculate
"#;
    let output = run_example("introspection/function_name.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_function_module_execution() {
    let expected = r#"main
main
"#;
    let output = run_example("introspection/function_module.rb");
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
Object
Object
<Binding with 129 vars>
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
    let expected = "Array\nString\nInteger\nFloat\nObject\nObject\nRange\nHash\nSet\n";
    let output = run_example("introspection/builtin_class_method.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_introspection_builtin_class_method_no_parens() {
    let expected = "Array\nString\nInteger\nFloat\nObject\nObject\nRange\nHash\nSet\n";
    let output = run_example("introspection/builtin_class_method_no_parens.rb");
    assert_eq!(output, expected);
}
