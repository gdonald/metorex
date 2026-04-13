use super::run_example;

#[test]
fn test_stdlib_strings_execution() {
    let expected = "11\n11\nHELLO WORLD\nhello world\ndlrow olleh\nhello\nhello\n2\nhello\nworld\none, two, three\nonetwothree\nhello\nworld\nworld\ntrue\nfalse\ntrue\ntrue\nfalse\ntrue\nfalse\n";
    let output = run_example("stdlib/strings.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_strings_parens_execution() {
    let expected = "11\n11\nHELLO WORLD\nhello world\ndlrow olleh\nhello\nhello\n2\nhello\nworld\none, two, three\nonetwothree\nhello\nworld\nworld\ntrue\nfalse\ntrue\ntrue\nfalse\ntrue\nfalse\n";
    let output = run_example("stdlib/strings_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_arrays_execution() {
    let expected = "8\n8\n[1, 2, 3, 4]\n4\n[1, 2, 3]\n1\n[2, 3]\n[0, 2, 3]\n[1, 1, 2, 3, 4, 5, 6, 9]\n[6, 2, 9, 5, 1, 4, 1, 3]\n[2, 4, 6]\n[2, 4, 6]\n15\na, b, c\nabc\n";
    let output = run_example("stdlib/arrays.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_arrays_parens_execution() {
    let expected = "8\n8\n[1, 2, 3, 4]\n4\n[1, 2, 3]\n1\n[2, 3]\n[0, 2, 3]\n[1, 1, 2, 3, 4, 5, 6, 9]\n[6, 2, 9, 5, 1, 4, 1, 3]\n[2, 4, 6]\n[2, 4, 6]\n15\na, b, c\nabc\n";
    let output = run_example("stdlib/arrays_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_hashes_execution() {
    let expected = "[alice, bob, charlie]\n[25, 30, 35]\ntrue\nfalse\n3\n30\n0\n25\n4\n90\n3\n";
    let output = run_example("stdlib/hashes.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_hashes_parens_execution() {
    let expected = "[alice, bob, charlie]\n[25, 30, 35]\ntrue\nfalse\n3\n30\n0\n25\n4\n90\n3\n";
    let output = run_example("stdlib/hashes_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_numbers_execution() {
    let expected = "42\n7\n42\n42\n42\n10\n3.14\n2.5\n4\n3\n3.14\n3\n3.14\n3.14\n";
    let output = run_example("stdlib/numbers.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_numbers_parens_execution() {
    let expected = "42\n7\n42\n42\n42\n10\n3.14\n2.5\n4\n3\n3.14\n3\n3.14\n3.14\n";
    let output = run_example("stdlib/numbers_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_io_execution() {
    let expected = "Hello from puts\nHello from print\nInt(42)\nHello from file!\ntrue\n";
    let output = run_example("stdlib/io.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_io_parens_execution() {
    let expected = "Hello from puts\nHello from print\nInt(42)\nHello from file!\ntrue\n";
    let output = run_example("stdlib/io_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_sets_execution() {
    let expected = "3\n3\ntrue\nfalse\n2\ntrue\nfalse\n6\n2\n2\n3\n3\n";
    let output = run_example("stdlib/sets.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_sets_parens_execution() {
    let expected = "3\n3\ntrue\nfalse\n2\ntrue\nfalse\n6\n2\n2\n3\n3\n";
    let output = run_example("stdlib/sets_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_testing_framework_execution() {
    let expected = "\x1b[1mMath operations\x1b[0m\n  \x1b[32mPASS\x1b[0m: adds numbers\n  \x1b[32mPASS\x1b[0m: multiplies numbers\n  \x1b[32mPASS\x1b[0m: divides numbers\n\x1b[32m3 passed\x1b[0m, 0 failed\n\x1b[1mString operations\x1b[0m\n  \x1b[32mPASS\x1b[0m: concatenates strings\n  \x1b[32mPASS\x1b[0m: gets length\n\x1b[32m2 passed\x1b[0m, 0 failed\n\x1b[1mType checking\x1b[0m\n  \x1b[32mPASS\x1b[0m: checks integer type\n  \x1b[32mPASS\x1b[0m: checks truthiness\n  \x1b[32mPASS\x1b[0m: checks nil\n\x1b[32m3 passed\x1b[0m, 0 failed\n\x1b[1mAssertions\x1b[0m\n  \x1b[32mPASS\x1b[0m: assert_equal catches mismatches\n  \x1b[32mPASS\x1b[0m: assert catches false\n\x1b[32m2 passed\x1b[0m, 0 failed\n\x1b[1mFiltered suite\x1b[0m\n  \x1b[32mPASS\x1b[0m: add test\n\x1b[32m1 passed\x1b[0m, 0 failed\n";
    let output = run_example("stdlib/testing_framework.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_testing_framework_parens_execution() {
    let expected = "\x1b[1mMath operations\x1b[0m\n  \x1b[32mPASS\x1b[0m: adds numbers\n  \x1b[32mPASS\x1b[0m: multiplies numbers\n  \x1b[32mPASS\x1b[0m: divides numbers\n\x1b[32m3 passed\x1b[0m, 0 failed\n\x1b[1mString operations\x1b[0m\n  \x1b[32mPASS\x1b[0m: concatenates strings\n  \x1b[32mPASS\x1b[0m: gets length\n\x1b[32m2 passed\x1b[0m, 0 failed\n\x1b[1mType checking\x1b[0m\n  \x1b[32mPASS\x1b[0m: checks integer type\n  \x1b[32mPASS\x1b[0m: checks truthiness\n  \x1b[32mPASS\x1b[0m: checks nil\n\x1b[32m3 passed\x1b[0m, 0 failed\n\x1b[1mAssertions\x1b[0m\n  \x1b[32mPASS\x1b[0m: assert_equal catches mismatches\n  \x1b[32mPASS\x1b[0m: assert catches false\n\x1b[32m2 passed\x1b[0m, 0 failed\n\x1b[1mFiltered suite\x1b[0m\n  \x1b[32mPASS\x1b[0m: add test\n\x1b[32m1 passed\x1b[0m, 0 failed\n";
    let output = run_example("stdlib/testing_framework_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_type_introspection_execution() {
    let expected = "true\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nObject\nnil\n2\ntrue\ntrue\nAnimal\n2\nRex\n3\n4\n";
    let output = run_example("builtins/type_introspection.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_type_introspection_parens_execution() {
    let expected = "true\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nObject\nnil\n2\ntrue\ntrue\nAnimal\n2\nRex\n3\n4\n";
    let output = run_example("builtins/type_introspection_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_execution() {
    let expected = "hello world\nnum: 42\npi: 3.14\nhex: ff\ncart has 5 items\n100% complete: done\n\"test\"\n00042\nleft      |\n";
    let output = run_example("stdlib/string_format.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_parens_execution() {
    let expected = "hello world\nnum: 42\npi: 3.14\nhex: ff\ncart has 5 items\n100% complete: done\n\"test\"\n00042\nleft      |\n";
    let output = run_example("stdlib/string_format_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_extended_execution() {
    let expected = "42\n+42\n-5\n 42\n-5\n+3.140000\n 3.140000\n3\n+3\n 3\nFF\n10\n1010\nA\nh\nnil\n42\nhel\n     right|\n0000000042\n100%\n";
    let output = run_example("stdlib/string_format_extended.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_extended_parens_execution() {
    let expected = "42\n+42\n-5\n 42\n-5\n+3.140000\n 3.140000\n3\n+3\n 3\nFF\n10\n1010\nA\nh\nnil\n42\nhel\n     right|\n0000000042\n100%\n";
    let output = run_example("stdlib/string_format_extended_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_regex_contexts_execution() {
    let expected = "/hello/\n/foo/\n/bar/i\n/[a-z]+/\n/path\\/to\\/file/\n5\n4\n5\n";
    let output = run_example("stdlib/regex_contexts.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_regex_contexts_parens_execution() {
    let expected = "/hello/\n/foo/\n/bar/i\n/[a-z]+/\n/path\\/to\\/file/\n5\n4\n5\n";
    let output = run_example("stdlib/regex_contexts_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_regex_literals_execution() {
    let expected = "/hello/\n/world/i\n5\n/[a-z]+\\d+/\n/hello\\/world/\n";
    let output = run_example("stdlib/regex_literals.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_regex_literals_parens_execution() {
    let expected = "/hello/\n/world/i\n5\n/[a-z]+\\d+/\n/hello\\/world/\n";
    let output = run_example("stdlib/regex_literals_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_slice_edge_execution() {
    let expected = "ell\nhe\nll\n\ndone\nfoobar\n";
    let output = run_example("stdlib/string_slice_edge.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_slice_edge_parens_execution() {
    let expected = "ell\nhe\nll\n\ndone\nfoobar\n";
    let output = run_example("stdlib/string_slice_edge_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_edge_execution() {
    let expected = "hello\nhello\nhello\nhello\nA\nZ\n      hi!\n7\n5.000000\n5.00\ntest%\n";
    let output = run_example("stdlib/string_format_edge.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_edge_parens_execution() {
    let expected = "hello\nhello\nhello\nhello\nA\nZ\n      hi!\n7\n5.000000\n5.00\ntest%\n";
    let output = run_example("stdlib/string_format_edge_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_to_i_execution() {
    let expected = "42\n99\n0\n-7\n3\n3.14\n0\nworld\nherro\nherlo\ntrue\nfalse\n";
    let output = run_example("stdlib/string_to_i.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_to_i_parens_execution() {
    let expected = "42\n99\n0\n-7\n3\n3.14\n0\nworld\nherro\nherlo\ntrue\nfalse\n";
    let output = run_example("stdlib/string_to_i_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_extended_execution() {
    let expected = "15\n25\n5\n6\n[1, 2, 3]\n[1, 2, 3, 4, [5]]\n[3, 1, 2]\n1\n8\ntrue\nfalse\n10\n30\ntrue\nfalse\n";
    let output = run_example("stdlib/array_extended.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_extended_parens_execution() {
    let expected = "15\n25\n5\n6\n[1, 2, 3]\n[1, 2, 3, 4, [5]]\n[3, 1, 2]\n1\n8\ntrue\nfalse\n10\n30\ntrue\nfalse\n";
    let output = run_example("stdlib/array_extended_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_new_methods_execution() {
    let expected = "15\n25\n12345\n3\n4\n3\n[1, 2, 3, 4]\n[1, 2, 3]\ntrue\nfalse\n10\n30\nnil\nnil\ntrue\nfalse\ntrue\n1\n8\n1.2\n3.5\nnil\nnil\n[3, 1, 2]\n";
    let output = run_example("stdlib/array_new_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_new_methods_parens_execution() {
    let expected = "15\n25\n3\n4\n[1, 2, 3, 4]\n[1, 2, 3]\ntrue\nfalse\n10\n30\ntrue\nfalse\n1\n8\n[3, 1, 2]\n";
    let output = run_example("stdlib/array_new_methods_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_new_methods_execution() {
    let expected =
        "42\n99\n0\n-7\n3\n3.14\n0\n-2.5\noriginal\nhell0 w0rld\nbbbbbb\nhi hello\ntrue\nfalse\n";
    let output = run_example("stdlib/string_new_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_new_methods_parens_execution() {
    let expected =
        "42\n99\n0\n-7\n3\n3.14\n0\n-2.5\noriginal\nhell0 w0rld\nbbbbbb\nhi hello\ntrue\nfalse\n";
    let output = run_example("stdlib/string_new_methods_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_error_paths_test_execution() {
    let expected = "3.14\n-3.14\n4\n3\n4\n3\n3.14\n42\n42\n42\n2\nell\ntrue\ntrue\ntrue\nhello\nhello\nHELLO\nolleh\n1, 2, 3\n2, 1, 3\n3\n0\n0\n123\nnil\nfalse\ntrue\n3\n15\n3\nerror_paths_test passed\n";
    let output = run_example("stdlib/error_paths_test.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_functional_execution() {
    let expected = "true\nfalse\ntrue\nfalse\ntrue\nfalse\n2\n2\n4\n2\n3\n15\n120\n";
    let output = run_example("stdlib/array_functional.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_functional_parens_execution() {
    let expected = "true\nfalse\ntrue\nfalse\ntrue\nfalse\n2\n2\n4\n2\n3\n15\n120\n";
    let output = run_example("stdlib/array_functional_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_hash_shorthand_execution() {
    let expected = "Alice\n30\nlocalhost\n8080\ntrue\n3\n2\n";
    let output = run_example("stdlib/hash_shorthand.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_hash_shorthand_parens_execution() {
    let expected = "Alice\n30\nlocalhost\n8080\ntrue\n3\n2\n";
    let output = run_example("stdlib/hash_shorthand_parens.rb");
    assert_eq!(output, expected);
}
