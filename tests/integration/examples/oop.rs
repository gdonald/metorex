use super::run_example;

#[test]
fn test_oop_super_basic_execution() {
    let expected = "Buddy\nGolden Retriever\nSome sound -> Woof!\nI am an animal named Buddy\n";
    let output = run_example("oop/super_basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_chain_basic_execution() {
    let expected = "GrandParent\nParent\nChild\n";
    let output = run_example("oop/super_chain_basic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_simple_execution() {
    let expected = "AB\n";
    let output = run_example("oop/test_super_simple.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_reader_execution() {
    let expected = "Alice\n30\n";
    let output = run_example("oop/attr_reader.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_writer_execution() {
    let expected = "Unknown\n0\nBob\n25\n";
    let output = run_example("oop/attr_writer.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_attr_accessor_execution() {
    let expected = "Charlie\n35\ncharlie@example.com\nCharles\n36\ncharles@example.com\n";
    let output = run_example("oop/attr_accessor.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_str_execution() {
    let expected = "Person: Alice\n";
    let output = run_example("oop/test_str.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_repr_execution() {
    let expected = "Point(0, 0)\n";
    let output = run_example("oop/test_repr.rb");
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
    let output = run_example("oop/test_iter.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_test_method_missing_execution() {
    let expected = "bar\n42\n1\n2\n3\n";
    let output = run_example("oop/test_method_missing.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_scope_resolution_execution() {
    let expected = "1\n100\n";
    let output = run_example("oop/scope_resolution.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_scope_resolution_parens_execution() {
    let expected = "1\n100\n";
    let output = run_example("oop/scope_resolution_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_execution() {
    let expected = "(4, 6)\ntrue\nfalse\n";
    let output = run_example("oop/operator_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_parens_execution() {
    let expected = "(4, 6)\ntrue\nfalse\n";
    let output = run_example("oop/operator_methods_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_index_execution() {
    let expected = "42\n99\n0\n";
    let output = run_example("oop/operator_methods_index.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_index_parens_execution() {
    let expected = "42\n99\n0\n";
    let output = run_example("oop/operator_methods_index_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_modules_execution() {
    let expected = "Hello, I am Alice\nGoodbye from Alice\nI am a class with module methods\n";
    let output = run_example("oop/modules.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_modules_parens_execution() {
    let expected = "Hello, I am Alice\nGoodbye from Alice\nI am a class with module methods\n";
    let output = run_example("oop/modules_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_keyword_execution() {
    let expected = "Rex makes a sound\nRex barks\nAnimal: Rex, Breed: Labrador\n";
    let output = run_example("oop/super_keyword.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_super_keyword_parens_execution() {
    let expected = "Rex makes a sound\nRex barks\nAnimal: Rex, Breed: Labrador\n";
    let output = run_example("oop/super_keyword_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_extended_execution() {
    let expected = "-1\n1\n0\n11\n";
    let output = run_example("oop/operator_methods_extended.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_operator_methods_extended_parens_execution() {
    let expected = "-1\n1\n0\n11\n";
    let output = run_example("oop/operator_methods_extended_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_multi_arg_bracket_execution() {
    let expected = "1\n2\n3\n4\n";
    let output = run_example("oop/multi_arg_bracket.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_multi_arg_bracket_parens_execution() {
    let expected = "1\n2\n3\n4\n";
    let output = run_example("oop/multi_arg_bracket_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_empty_bracket_call_execution() {
    let expected = "3\n";
    let output = run_example("oop/empty_bracket_call.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_empty_bracket_call_parens_execution() {
    let expected = "3\n";
    let output = run_example("oop/empty_bracket_call_parens.rb");
    assert_eq!(output, expected);
}
