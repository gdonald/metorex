// The accessors an attribute declaration defines.

use super::super::run_example;
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
    let expected = "OK reader raised: protected method 'foo' called for an instance of #<Class:0xADDRESS>\nOK writer raised: protected method 'foo=' called for an instance of #<Class:0xADDRESS>\n";
    let output = run_example("oop/attr/protected_attr.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_writer_methods_execution() {
    let expected = concat!(
        "12\n",
        "24\n",
        "36\n",
        "[:total, :total=]\n",
        "[:doubled, :reading, :reading=, :scaled=]\n",
    );
    let output = run_example("oop/writer_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_oop_writer_methods_parens_execution() {
    let expected = concat!(
        "12\n",
        "24\n",
        "36\n",
        "[:total, :total=]\n",
        "[:doubled, :reading, :reading=, :scaled=]\n",
    );
    let output = run_example("oop/writer_methods_parens.rb");
    assert_eq!(output, expected);
}
