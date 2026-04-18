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
