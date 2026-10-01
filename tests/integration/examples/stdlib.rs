use crate::common::EXAMPLES_DIR;
use std::process::Command;

use super::run_example;

#[test]
fn test_stdlib_strings_execution() {
    let expected = "11\n11\nHELLO WORLD\nhello world\ndlrow olleh\nhello\nhello\n2\nhello\nworld\none, two, three\nonetwothree\nhello\nworld\nworld\ntrue\nfalse\ntrue\ntrue\nfalse\ntrue\nfalse\n";
    let output = run_example("stdlib/string/strings.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_strings_parens_execution() {
    let expected = "11\n11\nHELLO WORLD\nhello world\ndlrow olleh\nhello\nhello\n2\nhello\nworld\none, two, three\nonetwothree\nhello\nworld\nworld\ntrue\nfalse\ntrue\ntrue\nfalse\ntrue\nfalse\n";
    let output = run_example("stdlib/string/strings_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_arrays_execution() {
    let expected = "8\n8\n1\n2\n3\n4\n4\n1\n2\n3\n1\n2\n3\n0\n2\n3\n1\n1\n2\n3\n4\n5\n6\n9\n6\n2\n9\n5\n1\n4\n1\n3\n2\n4\n6\n2\n4\n6\n15\na, b, c\nabc\n";
    let output = run_example("stdlib/array/arrays.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_arrays_parens_execution() {
    let expected = "8\n8\n1\n2\n3\n4\n4\n1\n2\n3\n1\n2\n3\n0\n2\n3\n1\n1\n2\n3\n4\n5\n6\n9\n6\n2\n9\n5\n1\n4\n1\n3\n2\n4\n6\n2\n4\n6\n15\na, b, c\nabc\n";
    let output = run_example("stdlib/array/arrays_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_hashes_execution() {
    let expected = "alice\nbob\ncharlie\n25\n30\n35\ntrue\nfalse\n3\n30\n0\n25\n4\n90\n3\n";
    let output = run_example("stdlib/hash/hashes.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_hashes_parens_execution() {
    let expected = "alice\nbob\ncharlie\n25\n30\n35\ntrue\nfalse\n3\n30\n0\n25\n4\n90\n3\n";
    let output = run_example("stdlib/hash/hashes_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_numbers_execution() {
    let expected = "42\n7\n42.0\n42\n42\n10\n3.14\n2.5\n4\n3\n3.14\n3\n3.14\n3.14\n";
    let output = run_example("stdlib/numbers/numbers.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_numbers_parens_execution() {
    let expected = "42\n7\n42.0\n42\n42\n10\n3.14\n2.5\n4\n3\n3.14\n3\n3.14\n3.14\n";
    let output = run_example("stdlib/numbers/numbers_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_io_execution() {
    let expected = "Hello from puts\nHello from print\n42\nHello from file!\ntrue\n";
    let output = run_example("stdlib/io/io.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_io_parens_execution() {
    let expected = "Hello from puts\nHello from print\n42\nHello from file!\ntrue\n";
    let output = run_example("stdlib/io/io_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_sets_execution() {
    let expected = "3\n3\ntrue\nfalse\n2\ntrue\nfalse\n6\n2\n2\n3\n3\n";
    let output = run_example("stdlib/sets/sets.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_sets_parens_execution() {
    let expected = "3\n3\ntrue\nfalse\n2\ntrue\nfalse\n6\n2\n2\n3\n3\n";
    let output = run_example("stdlib/sets/sets_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_testing_framework_execution() {
    let expected = "\x1b[1mMath operations\x1b[0m\n  \x1b[32mPASS\x1b[0m: adds numbers\n  \x1b[32mPASS\x1b[0m: multiplies numbers\n  \x1b[32mPASS\x1b[0m: divides numbers\n\x1b[32m3 passed\x1b[0m, 0 failed\n\x1b[1mString operations\x1b[0m\n  \x1b[32mPASS\x1b[0m: concatenates strings\n  \x1b[32mPASS\x1b[0m: gets length\n\x1b[32m2 passed\x1b[0m, 0 failed\n\x1b[1mType checking\x1b[0m\n  \x1b[32mPASS\x1b[0m: checks integer type\n  \x1b[32mPASS\x1b[0m: checks truthiness\n  \x1b[32mPASS\x1b[0m: checks nil\n\x1b[32m3 passed\x1b[0m, 0 failed\n\x1b[1mAssertions\x1b[0m\n  \x1b[32mPASS\x1b[0m: assert_equal catches mismatches\n  \x1b[32mPASS\x1b[0m: assert catches false\n\x1b[32m2 passed\x1b[0m, 0 failed\n\x1b[1mFiltered suite\x1b[0m\n  \x1b[32mPASS\x1b[0m: add test\n\x1b[32m1 passed\x1b[0m, 0 failed\n";
    let output = run_example("stdlib/testing/framework.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_testing_framework_parens_execution() {
    let expected = "\x1b[1mMath operations\x1b[0m\n  \x1b[32mPASS\x1b[0m: adds numbers\n  \x1b[32mPASS\x1b[0m: multiplies numbers\n  \x1b[32mPASS\x1b[0m: divides numbers\n\x1b[32m3 passed\x1b[0m, 0 failed\n\x1b[1mString operations\x1b[0m\n  \x1b[32mPASS\x1b[0m: concatenates strings\n  \x1b[32mPASS\x1b[0m: gets length\n\x1b[32m2 passed\x1b[0m, 0 failed\n\x1b[1mType checking\x1b[0m\n  \x1b[32mPASS\x1b[0m: checks integer type\n  \x1b[32mPASS\x1b[0m: checks truthiness\n  \x1b[32mPASS\x1b[0m: checks nil\n\x1b[32m3 passed\x1b[0m, 0 failed\n\x1b[1mAssertions\x1b[0m\n  \x1b[32mPASS\x1b[0m: assert_equal catches mismatches\n  \x1b[32mPASS\x1b[0m: assert catches false\n\x1b[32m2 passed\x1b[0m, 0 failed\n\x1b[1mFiltered suite\x1b[0m\n  \x1b[32mPASS\x1b[0m: add test\n\x1b[32m1 passed\x1b[0m, 0 failed\n";
    let output = run_example("stdlib/testing/framework_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_type_introspection_execution() {
    let expected = "true\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nNumeric\nBasicObject\n6\ntrue\ntrue\nAnimal\n2\nRex\n3\n4\n";
    let output = run_example("builtins/type_introspection.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_type_introspection_parens_execution() {
    let expected = "true\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nNumeric\nBasicObject\n6\ntrue\ntrue\nAnimal\n2\nRex\n3\n4\n";
    let output = run_example("builtins/type_introspection_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_execution() {
    let expected = "hello world\nnum: 42\npi: 3.14\nhex: ff\ncart has 5 items\n100% complete: done\n\"test\"\n00042\nleft      |\n";
    let output = run_example("stdlib/string/format.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_parens_execution() {
    let expected = "hello world\nnum: 42\npi: 3.14\nhex: ff\ncart has 5 items\n100% complete: done\n\"test\"\n00042\nleft      |\n";
    let output = run_example("stdlib/string/format_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_extended_execution() {
    let expected = "42\n+42\n-5\n 42\n-5\n+3.140000\n 3.140000\n3\n+3\n 3\nFF\n10\n1010\nA\nh\nnil\n42\nhel\n     right|\n0000000042\n100%\n";
    let output = run_example("stdlib/string/format_extended.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_extended_parens_execution() {
    let expected = "42\n+42\n-5\n 42\n-5\n+3.140000\n 3.140000\n3\n+3\n 3\nFF\n10\n1010\nA\nh\nnil\n42\nhel\n     right|\n0000000042\n100%\n";
    let output = run_example("stdlib/string/format_extended_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_regex_contexts_execution() {
    let expected = "(?-mix:hello)\n(?-mix:foo)\n(?i-mx:bar)\n(?-mix:[a-z]+)\n(?-mix:path\\/to\\/file)\n5\n4\n5\n";
    let output = run_example("stdlib/regex/contexts.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_regex_contexts_parens_execution() {
    let expected = "(?-mix:hello)\n(?-mix:foo)\n(?i-mx:bar)\n(?-mix:[a-z]+)\n(?-mix:path\\/to\\/file)\n5\n4\n5\n";
    let output = run_example("stdlib/regex/contexts_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_regex_literals_execution() {
    let expected = "(?-mix:hello)\n(?i-mx:world)\n5\n(?-mix:[a-z]+\\d+)\n(?-mix:hello\\/world)\n";
    let output = run_example("stdlib/regex/literals.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_regex_literals_parens_execution() {
    let expected = "(?-mix:hello)\n(?i-mx:world)\n5\n(?-mix:[a-z]+\\d+)\n(?-mix:hello\\/world)\n";
    let output = run_example("stdlib/regex/literals_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_slice_edge_execution() {
    let expected = "ell\nhe\nll\nnil\n\"\"\ndone\nfoobar\n";
    let output = run_example("stdlib/string/slice_edge.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_slice_edge_parens_execution() {
    let expected = "ell\nhe\nll\nnil\n\"\"\ndone\nfoobar\n";
    let output = run_example("stdlib/string/slice_edge_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_edge_execution() {
    let expected = concat!(
        "c4\n",
        "C4\n",
        "127\n",
        "1010\n",
        "invalid value for Integer(): \"hello\"\n",
        "A\n",
        "Z\n",
        "      hi!\n",
        "7\n",
        "5.000000\n",
        "5.00\n",
        "test%\n",
    );
    let output = run_example("stdlib/string/format_edge.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_edge_parens_execution() {
    let expected = concat!(
        "c4\n",
        "C4\n",
        "127\n",
        "1010\n",
        "invalid value for Integer(): \"hello\"\n",
        "A\n",
        "Z\n",
        "      hi!\n",
        "7\n",
        "5.000000\n",
        "5.00\n",
        "test%\n",
    );
    let output = run_example("stdlib/string/format_edge_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_to_i_execution() {
    let expected = "42\n99\n0\n-7\n3\n3.14\n0.0\nworld\nherro\nherlo\ntrue\nfalse\n";
    let output = run_example("stdlib/string/to_i.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_to_i_parens_execution() {
    let expected = "42\n99\n0\n-7\n3\n3.14\n0.0\nworld\nherro\nherlo\ntrue\nfalse\n";
    let output = run_example("stdlib/string/to_i_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_extended_execution() {
    let expected =
        "15\n25\n5\n6\n1\n2\n3\n1\n2\n3\n4\n5\n3\n1\n2\n1\n8\ntrue\nfalse\n10\n30\ntrue\nfalse\n";
    let output = run_example("stdlib/array/extended.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_extended_parens_execution() {
    let expected =
        "15\n25\n5\n6\n1\n2\n3\n1\n2\n3\n4\n5\n3\n1\n2\n1\n8\ntrue\nfalse\n10\n30\ntrue\nfalse\n";
    let output = run_example("stdlib/array/extended_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_new_methods_execution() {
    let expected = "15\n25\n12345\n3\n4\n3\n1\n2\n3\n4\n1\n2\n3\ntrue\nfalse\n10\n30\n\n\ntrue\nfalse\ntrue\n1\n8\n1.2\n3.5\n\n\n3\n1\n2\n";
    let output = run_example("stdlib/array/new_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_new_methods_parens_execution() {
    let expected =
        "15\n25\n3\n4\n1\n2\n3\n4\n1\n2\n3\ntrue\nfalse\n10\n30\ntrue\nfalse\n1\n8\n3\n1\n2\n";
    let output = run_example("stdlib/array/new_methods_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_new_methods_execution() {
    let expected =
        "42\n99\n0\n-7\n3\n3.14\n0.0\n-2.5\noriginal\nhell0 w0rld\nbbbbbb\nhi hello\ntrue\nfalse\n";
    let output = run_example("stdlib/string/new_methods.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_new_methods_parens_execution() {
    let expected =
        "42\n99\n0\n-7\n3\n3.14\n0.0\n-2.5\noriginal\nhell0 w0rld\nbbbbbb\nhi hello\ntrue\nfalse\n";
    let output = run_example("stdlib/string/new_methods_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_error_paths_test_execution() {
    let expected = "3.14\n3.14\n4\n3\n4\n3\n3.14\n42.0\n42\n42\n2\nell\ntrue\ntrue\ntrue\nhello\nhello\nHELLO\nolleh\n1, 2, 3\n2, 1, 3\n3\n0\n0\n123\n\nfalse\ntrue\n3\n15\n3\nerror_paths_test passed\n";
    let output = run_example("stdlib/error_paths_test.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_functional_execution() {
    let expected = "true\nfalse\ntrue\nfalse\ntrue\nfalse\n2\n2\n4\n2\n3\n15\n120\n";
    let output = run_example("stdlib/array/functional.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_array_functional_parens_execution() {
    let expected = "true\nfalse\ntrue\nfalse\ntrue\nfalse\n2\n2\n4\n2\n3\n15\n120\n";
    let output = run_example("stdlib/array/functional_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_hash_shorthand_execution() {
    let expected = "Alice\n30\nlocalhost\n8080\ntrue\n3\n2\n";
    let output = run_example("stdlib/hash/shorthand.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_hash_shorthand_parens_execution() {
    let expected = "Alice\n30\nlocalhost\n8080\ntrue\n3\n2\n";
    let output = run_example("stdlib/hash/shorthand_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_file_and_string_reads_execution() {
    let expected = "[\"alpha\", \"beta\"]\n2\n11\n\"beta\"\nnil\n";
    let output = run_example("stdlib/file_and_string_reads.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_file_and_string_reads_parens_execution() {
    let expected = "[\"alpha\", \"beta\"]\n2\n11\n\"beta\"\nnil\n";
    let output = run_example("stdlib/file_and_string_reads_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_io_popen_execution() {
    let expected = concat!(
        "hello\n", "true\n", "true\n", "false\n", "true\n", "true\n", "0\n", "false\n", "nil\n",
        "true\n", "0\n", "one\n", "err\n", "3\n", "false\n",
    );
    let output = run_example("runtime/io_popen.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_io_popen_parens_execution() {
    let expected = concat!(
        "hello\n", "true\n", "true\n", "false\n", "true\n", "true\n", "0\n", "false\n", "nil\n",
        "true\n", "0\n", "one\n", "err\n", "3\n", "false\n",
    );
    let output = run_example("runtime/io_popen_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_dump_execution() {
    let expected = concat!(
        "\"plain\"\n",
        "\"with \\\"quotes\\\"\"\n",
        "\"tab\\there\"\n",
        "\"line\\nbreak\"\n",
        "\"back\\\\slash\"\n",
        "\"interp \\#{x} and \\#@ivar\"\n",
        "\"caf\\u00E9\"\n",
    );
    let output = run_example("stdlib/string/dump.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_dump_parens_execution() {
    let expected = concat!(
        "\"plain\"\n",
        "\"with \\\"quotes\\\"\"\n",
        "\"tab\\there\"\n",
        "\"line\\nbreak\"\n",
        "\"back\\\\slash\"\n",
        "\"interp \\#{x} and \\#@ivar\"\n",
        "\"caf\\u00E9\"\n",
    );
    let output = run_example("stdlib/string/dump_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_dir_chdir_execution() {
    let expected = "true\ntrue\ntrue\ntrue\ntrue\n42\n";
    let output = run_example("runtime/dir_chdir.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_dir_chdir_parens_execution() {
    let expected = "true\ntrue\ntrue\ntrue\ntrue\n42\n";
    let output = run_example("runtime/dir_chdir_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_magic_dir_execution() {
    let expected = "true\ntrue\n\".\"\n\"foo\"\nnil\n";
    let output = run_example("runtime/magic_dir.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_magic_dir_parens_execution() {
    let expected = "true\ntrue\n\".\"\n\"foo\"\nnil\n";
    let output = run_example("runtime/magic_dir_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_at_exit_handlers_execution() {
    let expected = concat!(
        "called without a block\n",
        "true\n",
        "main body\n",
        "registered last\n",
        "outer\n",
        "outer done\n",
        "nested\n",
        "registered first\n",
    );
    let output = run_example("runtime/at_exit_handlers.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_at_exit_handlers_parens_execution() {
    let expected = concat!(
        "called without a block\n",
        "true\n",
        "main body\n",
        "registered last\n",
        "outer\n",
        "outer done\n",
        "nested\n",
        "registered first\n",
    );
    let output = run_example("runtime/at_exit_handlers_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_at_exit_exit_status_execution() {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/runtime/at_exit_exit_status.rb", EXAMPLES_DIR);
    let mut command = Command::new(binary);
    command.current_dir(manifest_dir).arg(&full_path);

    let output = command.output().expect("failed to execute example");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not utf8");

    assert_eq!(stdout, "main body\nfirst handler\nlast handler\n");
    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn test_runtime_at_exit_last_exception_execution() {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/runtime/at_exit_last_exception.rb", EXAMPLES_DIR);
    let mut command = Command::new(binary);
    command.current_dir(manifest_dir).arg(&full_path);

    let output = command.output().expect("failed to execute example");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not utf8");

    assert_eq!(stdout, "RuntimeError\nboom\n");
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_runtime_backtick_command_execution() {
    let expected = concat!(
        "\"hello world\\n\"\n",
        "0\n",
        "true\n",
        "true\n",
        "false\n",
        "Process::Status\n",
        "7\n",
        "false\n",
        "\"through the module\\n\"\n",
        "\"coerced\\n\"\n",
        "No such file or directory - nonexistent_command_xyz 2>/dev/null\n",
        "true\n",
        ":`\n"
    );
    let output = run_example("runtime/backtick_command.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_backtick_command_parens_execution() {
    let expected = concat!(
        "\"hello world\\n\"\n",
        "0\n",
        "true\n",
        "true\n",
        "false\n",
        "Process::Status\n",
        "7\n",
        "false\n",
        "\"through the module\\n\"\n",
        "\"coerced\\n\"\n",
        "No such file or directory - nonexistent_command_xyz 2>/dev/null\n",
        "true\n",
        ":`\n"
    );
    let output = run_example("runtime/backtick_command_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_dash_n_chomp_execution() {
    let expected = concat!(
        "abc\n",
        "abc\n",
        "abc\n",
        "\"abc\\n\"\n",
        "ab\n",
        "\"abc\"\n",
        "abc\n",
        "true\n",
        "true\n",
    );
    let output = run_example("runtime/dash_n_chomp.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_dash_n_chomp_parens_execution() {
    let expected = concat!(
        "abc\n",
        "abc\n",
        "abc\n",
        "\"abc\\n\"\n",
        "ab\n",
        "\"abc\"\n",
        "abc\n",
        "true\n",
        "true\n",
    );
    let output = run_example("runtime/dash_n_chomp_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_io_popen_argv_execution() {
    let expected = "got: a line\nno input\n0\n";
    let output = run_example("runtime/io_popen_argv.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_io_popen_argv_parens_execution() {
    let expected = "got: a line\nno input\n0\n";
    let output = run_example("runtime/io_popen_argv_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_exec_replaces_process_execution() {
    let expected = "before exec\nreplaced\n";
    let output = run_example("runtime/exec_replaces_process.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_exec_replaces_process_parens_execution() {
    let expected = "before exec\nreplaced\n";
    let output = run_example("runtime/exec_replaces_process_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_exec_missing_command_execution() {
    let expected = "true\nNo such file or directory - definitely_not_a_command_xyz\n";
    let output = run_example("runtime/exec_missing_command.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_exec_missing_command_parens_execution() {
    let expected = "true\nNo such file or directory - definitely_not_a_command_xyz\n";
    let output = run_example("runtime/exec_missing_command_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_exit_bang_skips_handlers_execution() {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/runtime/exit_bang_skips_handlers.rb", EXAMPLES_DIR);
    let mut command = Command::new(binary);
    command.current_dir(manifest_dir).arg(&full_path);

    let output = command.output().expect("failed to execute example");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not utf8");

    assert_eq!(stdout, "before\n");
    assert_eq!(output.status.code(), Some(21));
}

#[test]
fn test_runtime_at_exit_overrides_error_execution() {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/runtime/at_exit_overrides_error.rb", EXAMPLES_DIR);
    let mut command = Command::new(binary);
    command.current_dir(manifest_dir).arg(&full_path);

    let output = command.output().expect("failed to execute example");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not utf8");

    assert_eq!(stdout, "in at_exit\n$! is RuntimeError:original error\n");
    assert_eq!(output.status.code(), Some(21));
}

#[test]
fn test_runtime_fork_child_process_execution() {
    let expected = "42\n0\n7\nwritten by child\ntrue\ntrue\nfalse\nfalse\ntrue\n";
    let output = run_example("runtime/fork_child_process.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_fork_child_process_parens_execution() {
    let expected = "42\n0\n7\nwritten by child\ntrue\ntrue\nfalse\nfalse\ntrue\n";
    let output = run_example("runtime/fork_child_process_parens.rb");
    assert_eq!(output, expected);
}

/// The expected output of both `runtime/forked_popen` variants.
const FORKED_POPEN_OUTPUT: &str = concat!(
    "\"hello from child\\n\"\n",
    "true\n",
    "false\n",
    "\"child read \\\"to child\\\\n\\\"\\n\"\n",
    "3\n",
    "\"from the child's block\\n\"\n",
    "IOError\n",
    "true\n",
    "true\n",
    "5\n",
    "\"I'm shared!\"\n",
    "\"I'm shared!\"\n",
    "1\n"
);

#[test]
fn test_runtime_forked_popen_execution() {
    let output = run_example("runtime/forked_popen.rb");
    assert_eq!(output, FORKED_POPEN_OUTPUT);
}

#[test]
fn test_runtime_forked_popen_parens_execution() {
    let output = run_example("runtime/forked_popen_parens.rb");
    assert_eq!(output, FORKED_POPEN_OUTPUT);
}

#[test]
fn test_stdlib_string_format_keywords_execution() {
    let expected = concat!(
        "a and b\n",
        "test value\n",
        "hello, world!\n",
        "00042\n",
        "3.14\n",
        "through the module\n",
        "key{missing} not found\n",
        "true\n",
        "true\n",
    );
    let output = run_example("stdlib/string/format_keywords.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_keywords_parens_execution() {
    let expected = concat!(
        "a and b\n",
        "test value\n",
        "hello, world!\n",
        "00042\n",
        "3.14\n",
        "through the module\n",
        "key{missing} not found\n",
        "true\n",
        "true\n",
    );
    let output = run_example("stdlib/string/format_keywords_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_verbose_warning_execution() {
    let binary = env!("CARGO_BIN_EXE_metorex");
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{}/stdlib/string/format_verbose_warning.rb", EXAMPLES_DIR);
    let mut command = Command::new(binary);
    command.current_dir(manifest_dir).arg(&full_path);

    let output = command.output().expect("failed to execute example");
    let stdout = String::from_utf8(output.stdout).expect("stdout was not utf8");
    let stderr = String::from_utf8(output.stderr).expect("stderr was not utf8");

    assert_eq!(stdout, "no placeholders\nstill quiet\n");
    assert_eq!(stderr, "warning: too many arguments for format string\n");
}

#[test]
fn test_runtime_stdout_redirect_execution() {
    let expected = concat!(
        "\"\\\"captured\"\n",
        "\"\nand this\n",
        "[<described>, \"text\", :symbol, nil]\n",
        "<described>\n",
    );
    let output = run_example("runtime/stdout_redirect.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_runtime_stdout_redirect_parens_execution() {
    let expected = concat!(
        "\"\\\"captured\"\n",
        "\"\nand this\n",
        "[<described>, \"text\", :symbol, nil]\n",
        "<described>\n",
    );
    let output = run_example("runtime/stdout_redirect_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_stringio_printf_execution() {
    let expected = concat!(
        "\"start: value-7 shovelled printed\\n\"\n",
        "\"first\\n\"\n",
        "\"second\\n\"\n",
        "\"\"\n",
        "\"first\"\n",
        "one and two\n",
        "42\n",
        "false\n",
        "true\n",
        "written\n"
    );
    let output = run_example("stdlib/string/stringio_printf.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_stringio_printf_parens_execution() {
    let expected = concat!(
        "\"start: value-7 shovelled printed\\n\"\n",
        "\"first\\n\"\n",
        "\"second\\n\"\n",
        "\"\"\n",
        "\"first\"\n",
        "one and two\n",
        "42\n",
        "false\n",
        "true\n",
        "written\n"
    );
    let output = run_example("stdlib/string/stringio_printf_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_math_functions_execution() {
    let expected = concat!(
        "2.0\n",
        "3.0\n",
        "5.0\n",
        "0.0\n",
        "3.0\n",
        "3.0\n",
        "3.0\n",
        "1.0\n",
        "[0.6025390625, 11]\n",
        "1234.0\n",
        "5.0\n",
        "true\n",
        "10001.0\n",
        "Numerical argument is out of domain - \"sqrt\"\n",
        "can't convert String into Float\n",
        "true\n"
    );
    let output = run_example("stdlib/math/functions.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_math_functions_parens_execution() {
    let expected = concat!(
        "2.0\n",
        "3.0\n",
        "5.0\n",
        "0.0\n",
        "3.0\n",
        "3.0\n",
        "3.0\n",
        "1.0\n",
        "[0.6025390625, 11]\n",
        "1234.0\n",
        "5.0\n",
        "true\n",
        "10001.0\n",
        "Numerical argument is out of domain - \"sqrt\"\n",
        "can't convert String into Float\n",
        "true\n"
    );
    let output = run_example("stdlib/math/functions_parens.rb");
    assert_eq!(output, expected);
}

/// The expected output of both `stdlib/string/character_sets` variants.
const STRING_CHARACTER_SETS_OUTPUT: &str = "3\n1\n3\n\"heo word\"\n\"abc\"\n\"abbbccc\"\n\"hippo\"\n\"ifmmp\"\n\"*e**o\"\n\"hero\"\n\"invalid range \\\"h-e\\\" in string transliteration\"\n\"hello\"\n\"hello\"\n\"llo\"\n\"hel\"\n[\"he\", \"l\", \"lo\"]\n[\"hel\", \"l\", \"o\"]\n[\"he\", \"ll\", \"o\"]\n:hello\n294\n6\n\"   hi    \"\n\"121hi1212\"\n0\ntrue\nnil\n\"x\"\nnil\n[\"a\", \"b\", \"c\", \"d\", \"e\"]\n[\"9\", \":\", \";\", \"<\", \"=\", \">\", \"?\", \"@\", \"A\"]\n[\"8\", \"9\", \"10\", \"11\"]\n[\"a\", \"b\", \"c\"]\n[]\n";

#[test]
fn test_stdlib_string_character_sets_execution() {
    let output = run_example("stdlib/string/character_sets.rb");
    assert_eq!(output, STRING_CHARACTER_SETS_OUTPUT);
}

#[test]
fn test_stdlib_string_character_sets_no_parens_execution() {
    let output = run_example("stdlib/string/character_sets_no_parens.rb");
    assert_eq!(output, STRING_CHARACTER_SETS_OUTPUT);
}

/// The expected output of both `stdlib/dir/reading_a_directory` variants.
const READING_A_DIRECTORY_OUTPUT: &str = "true\nfalse\nfalse\n[\".\", \"..\", \"held.txt\"]\n[\"held.txt\"]\n[\".\", \"..\", \"held.txt\"]\n[\"held.txt\"]\nEnumerator\nnil\nErrno::ENOTEMPTY\nErrno::ENOENT\n0\nfalse\ntrue\n";

#[test]
fn test_stdlib_dir_reading_a_directory_execution() {
    let output = run_example("stdlib/dir/reading_a_directory.rb");
    assert_eq!(output, READING_A_DIRECTORY_OUTPUT);
}

#[test]
fn test_stdlib_dir_reading_a_directory_no_parens_execution() {
    let output = run_example("stdlib/dir/reading_a_directory_no_parens.rb");
    assert_eq!(output, READING_A_DIRECTORY_OUTPUT);
}

/// The expected output of both `stdlib/marshal_round_trip` variants, which
/// differ only in whether the calls are written with parentheses.
const MARSHAL_ROUND_TRIP_OUTPUT: &str = concat!(
    "[4, 8, 102, 8, 48, 46, 53]\n",
    "[4, 8, 105, 47]\n",
    "[1, \"two\", :three, {four: 4}]\n",
    "nil\n",
    "true\n",
    "Waypoint\n",
    "\"summit\"\n",
    "12\n",
    "#<Encoding:UTF-8>\n"
);

#[test]
fn test_stdlib_marshal_round_trip_execution() {
    let output = run_example("stdlib/marshal_round_trip.rb");
    assert_eq!(output, MARSHAL_ROUND_TRIP_OUTPUT);
}

#[test]
fn test_stdlib_marshal_round_trip_parens_execution() {
    let output = run_example("stdlib/marshal_round_trip_parens.rb");
    assert_eq!(output, MARSHAL_ROUND_TRIP_OUTPUT);
}

/// The expected output of both `stdlib/erb_trim_modes` variants, which differ
/// only in whether the calls are written with parentheses.
const ERB_TRIM_MODES_OUTPUT: &str = concat!(
    "\"<ul>\\n\\n<li>1</li>\\n\\n<li>2</li>\\n\\n</ul>\\n\"\n",
    "\"<ul>\\n<li>1</li>\\n<li>2</li>\\n</ul>\\n\"\n",
    "\"<ul>\\n<li>1</li>\\n<li>2</li>\\n</ul>\\n\"\n",
    "\"<ul>\\n<li>1</li>\\n<li>2</li>\\n</ul>\\n\"\n",
    "\"<ul>\\n<li>1</li>\\n<li>2</li>\\n</ul>\\n%done\\n\"\n"
);

#[test]
fn test_stdlib_erb_trim_modes_execution() {
    let output = run_example("stdlib/erb_trim_modes.rb");
    assert_eq!(output, ERB_TRIM_MODES_OUTPUT);
}

#[test]
fn test_stdlib_erb_trim_modes_parens_execution() {
    let output = run_example("stdlib/erb_trim_modes_parens.rb");
    assert_eq!(output, ERB_TRIM_MODES_OUTPUT);
}

/// The expected output of both `stdlib/string_streams` variants, which differ
/// only in whether the calls are written with parentheses.
const STRING_STREAMS_OUTPUT: &str = "false\ntrue\ntrue\nfalse\n\"\"\nErrno::EACCES\n\"second\"\n";

#[test]
fn test_stdlib_string_streams_execution() {
    let output = run_example("stdlib/string_streams.rb");
    assert_eq!(output, STRING_STREAMS_OUTPUT);
}

#[test]
fn test_stdlib_string_streams_parens_execution() {
    let output = run_example("stdlib/string_streams_parens.rb");
    assert_eq!(output, STRING_STREAMS_OUTPUT);
}

const ZLIB_DEFLATE_OUTPUT: &str = concat!(
    "120,156,99,96,128,1,0,0,10,0,1\n",
    "120,156,51,52,132,1,0,10,145,1,235\n",
    "52\n",
    "120,187,20,225,3,203,75,76,74,78,73,77,75,207,200,204,2,0,21,134,3,248\n",
    "12 -> 13\n",
    "true\n",
    "2250 -> 72\n",
    "true\n",
    "157,5,0,36,10,0,0,0\n",
    "true\n",
);

#[test]
fn test_stdlib_zlib_deflate_execution() {
    let output = run_example("stdlib/zlib_deflate.rb");
    assert_eq!(output, ZLIB_DEFLATE_OUTPUT);
}

#[test]
fn test_stdlib_zlib_deflate_no_parens_execution() {
    let output = run_example("stdlib/zlib_deflate_no_parens.rb");
    assert_eq!(output, ZLIB_DEFLATE_OUTPUT);
}

const JSON_DOCUMENTS_OUTPUT: &str = concat!(
    "Ada\n[1815, 1852]\nnil\n2.5\n{a: 1}\n",
    "{\"a\":1,\"b\":[true,false,null]}\n",
    "{\n  \"a\": 1,\n  \"b\": [\n    1,\n    2\n  ]\n}\n",
    "{\"k\":\"v\"}\n[1,\"two\",null]\n\"a\\nbA\"\n[]\n{}\n",
    "refused: JSON::ParserError\n1.1111111111111112\n",
);

#[test]
fn test_stdlib_json_documents_execution() {
    let output = run_example("stdlib/json_documents.rb");
    assert_eq!(output, JSON_DOCUMENTS_OUTPUT);
}

#[test]
fn test_stdlib_json_documents_no_parens_execution() {
    let output = run_example("stdlib/json_documents_no_parens.rb");
    assert_eq!(output, JSON_DOCUMENTS_OUTPUT);
}

const OBJSPACE_DUMP_OUTPUT: &str = concat!(
    "STRING\nabc\n3\nUTF-8\nARRAY\nHASH\nString\ntrue\n",
    "wrong output option: #<Object:0x>\n",
);

#[test]
fn test_stdlib_objspace_dump_execution() {
    let output = run_example("stdlib/objspace_dump.rb");
    assert_eq!(output, OBJSPACE_DUMP_OUTPUT);
}

#[test]
fn test_stdlib_objspace_dump_no_parens_execution() {
    let output = run_example("stdlib/objspace_dump_no_parens.rb");
    assert_eq!(output, OBJSPACE_DUMP_OUTPUT);
}

/// The expected output of both `runtime/popen_options` variants.
const POPEN_OPTIONS_OUTPUT: &str = concat!(
    "\"hello\\n\"\n",
    "\"hi\\n\"\n",
    "\"to_err\\n\"\n",
    "\"/\\n\"\n",
    "\"named\\n\"\n",
    "true\n",
    "\"positional\\n\"\n",
    "\"bar\"\n",
    "Stream\n",
    "#<Encoding:EUC-JP>\n",
    "IOError\n",
    "\"foo\\n\"\n",
    "true\n",
    "\"logged\"\n",
    "\"metorex: No such file or directory -- does_not_exist (LoadError)\\n\"\n"
);

#[test]
fn test_runtime_popen_options_execution() {
    let output = run_example("runtime/popen_options.rb");
    assert_eq!(output, POPEN_OPTIONS_OUTPUT);
}

#[test]
fn test_runtime_popen_options_parens_execution() {
    let output = run_example("runtime/popen_options_parens.rb");
    assert_eq!(output, POPEN_OPTIONS_OUTPUT);
}

/// The expected output of both `runtime/process_title` variants.
const PROCESS_TITLE_OUTPUT: &str = concat!("\"metorex-title-example\"\n", "false\n");

#[test]
fn test_runtime_process_title_execution() {
    let output = run_example("runtime/process_title.rb");
    assert_eq!(output, PROCESS_TITLE_OUTPUT);
}

#[test]
fn test_runtime_process_title_no_parens_execution() {
    let output = run_example("runtime/process_title_no_parens.rb");
    assert_eq!(output, PROCESS_TITLE_OUTPUT);
}

/// The expected output of both `runtime/status_wait` variants.
const STATUS_WAIT_OUTPUT: &str = concat!("-1\n", "true\n", "Process::Status\n", "true\n");

#[test]
fn test_runtime_status_wait_execution() {
    let output = run_example("runtime/status_wait.rb");
    assert_eq!(output, STATUS_WAIT_OUTPUT);
}

#[test]
fn test_runtime_status_wait_no_parens_execution() {
    let output = run_example("runtime/status_wait_no_parens.rb");
    assert_eq!(output, STATUS_WAIT_OUTPUT);
}

#[test]
fn test_runtime_signal_zero_execution() {
    let output = run_example("runtime/signal_zero.rb");
    assert_eq!(output, "1\n");
}

#[test]
fn test_runtime_signal_zero_no_parens_execution() {
    let output = run_example("runtime/signal_zero_no_parens.rb");
    assert_eq!(output, "1\n");
}

/// The expected output of both `runtime/exec_with_options` variants.
const EXEC_WITH_OPTIONS_OUTPUT: &str = concat!(
    "Errno::ENOENT\n",
    "Errno::EACCES\n",
    "\"string contains null byte\"\n",
    "\"wrong first argument\"\n",
    "named_shell from the environment /\n",
);

#[test]
fn test_runtime_exec_with_options_execution() {
    let output = run_example("runtime/exec_with_options.rb");
    assert_eq!(output, EXEC_WITH_OPTIONS_OUTPUT);
}

#[test]
fn test_runtime_exec_with_options_no_parens_execution() {
    let output = run_example("runtime/exec_with_options_no_parens.rb");
    assert_eq!(output, EXEC_WITH_OPTIONS_OUTPUT);
}

#[test]
fn test_runtime_writing_to_a_closed_stderr_execution() {
    let output = run_example("runtime/writing_to_a_closed_stderr.rb");
    assert_eq!(output, "\"rescued Errno::EBADF\\n\"\n");
}

#[test]
fn test_runtime_writing_to_a_closed_stderr_no_parens_execution() {
    let output = run_example("runtime/writing_to_a_closed_stderr_no_parens.rb");
    assert_eq!(output, "\"rescued Errno::EBADF\\n\"\n");
}

/// The expected output of both `runtime/spawn_with_options` variants.
const SPAWN_WITH_OPTIONS_OUTPUT: &str = concat!(
    "\"hello\\ngone\\n\"\n",
    "[ArgumentError, \"negative process group ID : -1\"]\n",
    "[TypeError, \"no implicit conversion of Symbol into Integer\"]\n",
    "[ArgumentError, \"wrong exec option symbol: nonesuch\"]\n",
    "[ArgumentError, \"wrong exec option\"]\n",
    "[ArgumentError, \"environment name contains a equal : A=B\"]\n",
    "[TypeError, \"no implicit conversion of Symbol into String\"]\n",
    "[ArgumentError, \"wrong number of arguments (given 0, expected 1+)\"]\n",
);

#[test]
fn test_runtime_spawn_with_options_execution() {
    let output = run_example("runtime/spawn_with_options.rb");
    assert_eq!(output, SPAWN_WITH_OPTIONS_OUTPUT);
}

#[test]
fn test_runtime_spawn_with_options_no_parens_execution() {
    let output = run_example("runtime/spawn_with_options_no_parens.rb");
    assert_eq!(output, SPAWN_WITH_OPTIONS_OUTPUT);
}

/// The expected output of both `runtime/daemonizing` variants.
const DAEMONIZING_OUTPUT: &str = concat!("\"before\\n\"\n", "\"[true, true, \\\"/\\\", true]\"\n",);

#[test]
fn test_runtime_daemonizing_execution() {
    let output = run_example("runtime/daemonizing.rb");
    assert_eq!(output, DAEMONIZING_OUTPUT);
}

#[test]
fn test_runtime_daemonizing_no_parens_execution() {
    let output = run_example("runtime/daemonizing_no_parens.rb");
    assert_eq!(output, DAEMONIZING_OUTPUT);
}

/// The expected output of both `stdlib/marshal_wrapped_values` variants.
const MARSHAL_WRAPPED_VALUES_OUTPUT: &str = concat!(
    "\"\\x04\\x08I\\\"\\ttext\\x06:\\x06ET\"\n",
    "\"\\x04\\x08\\\"\\ttext\"\n",
    "\"\\x04\\x08I:\\x08\\xE2\\x86\\x92\\x06:\\x06ET\"\n",
    "\"\\x04\\x08I/\\x07a.\\x01\\x06:\\x06EF\"\n",
    "\"\\x04\\x08e:\\x0BTagged[\\x00\"\n",
    "\"\\x04\\x08C:\\nWords[\\x00\"\n",
    "\"\\x04\\x08}\\x00i\\x00\"\n",
    "\"\\x04\\x08C:\\tHash{\\x00\"\n",
    "\"\\x04\\x08C:\\nTable{\\x00\"\n",
    "\"\\x04\\x08I\\\"\\tnote\\x07:\\x06ET:\\x08@byI\\\"\\x07me\\x06;\\x00T\"\n",
    "\"can't dump hash with default proc\"\n",
);

#[test]
fn test_stdlib_marshal_wrapped_values_execution() {
    let output = run_example("stdlib/marshal_wrapped_values.rb");
    assert_eq!(output, MARSHAL_WRAPPED_VALUES_OUTPUT);
}

#[test]
fn test_stdlib_marshal_wrapped_values_no_parens_execution() {
    let output = run_example("stdlib/marshal_wrapped_values_no_parens.rb");
    assert_eq!(output, MARSHAL_WRAPPED_VALUES_OUTPUT);
}

/// The expected output of both `stdlib/marshal_objects` variants.
const MARSHAL_OBJECTS_OUTPUT: &str = concat!(
    "\"\\x04\\x08o:\\x0CAccount\\x06:\\x0B@owner\\\"\\x08ann\"\n",
    "\"\\x04\\x08e:\\x0CLabeledo:\\x0CAccount\\x06:\\x0B@owner\\\"\\x08ann\"\n",
    "\"\\x04\\x08S:\\nPoint\\x07:\\x06xi\\x06:\\x06yi\\x07\"\n",
    "\"\\x04\\x08S:\\tSize\\x07:\\nwidthi\\x08:\\x0Bheighti\\t\"\n",
    "\"\\x04\\x08c\\x0CAccount\"\n",
    "\"\\x04\\x08m\\x0CLabeled\"\n",
    "\"\\x04\\x08U:\\rSnapshot[\\x07i\\x06i\\x07\"\n",
    "\"\\x04\\x08u:\\x0BPacked\\x0Bpacked\"\n",
    "[TypeError, \"can't dump anonymous class #<Class:0x...>\"]\n",
    "[TypeError, \"singleton class can't be dumped\"]\n",
    "[TypeError, \"no _dump_data is defined for class Proc\"]\n",
    "[ArgumentError, \"exceed depth limit\"]\n",
    "[TypeError, \"instance of IO needed\"]\n",
    "[:binmode, \"\\x04\\x08:\\tnote\"]\n",
);

#[test]
fn test_stdlib_marshal_objects_execution() {
    let output = run_example("stdlib/marshal_objects.rb");
    assert_eq!(output, MARSHAL_OBJECTS_OUTPUT);
}

#[test]
fn test_stdlib_marshal_objects_no_parens_execution() {
    let output = run_example("stdlib/marshal_objects_no_parens.rb");
    assert_eq!(output, MARSHAL_OBJECTS_OUTPUT);
}

/// The expected output of both `stdlib/marshal_core_values` variants.
const MARSHAL_CORE_VALUES_OUTPUT: &str = concat!(
    "\"\\x04\\x08o:\\x11RuntimeError\\x08:\\tmesgI\\\"\\x0Edisk full\\x06:\\x06ET:\\x07bt[\\x06I\\\"\\x10store.rb:12\\x06;\\x07T:\\r@retriesi\\x08\"\n",
    "[RuntimeError, \"write failed\", ArgumentError, \"bad size\"]\n",
    "\"\\x04\\x08o:\\nRange\\x08:\\texclF:\\nbegini\\x06:\\x08endi\\x07\"\n",
    "\"\\x04\\x08o:\\nRange\\x08:\\texclT:\\nbegini\\x06:\\x08endi\\x07\"\n",
    "3...9\n",
    "\"\\x04\\x08Iu:\\tTime\\r \\x00\\x1C\\xC0\\x00\\x00\\x00\\x00\\x06:\\tzoneI\\\"\\x08UTC\\x06:\\x06EF\"\n",
    "true\n",
    "\"\\x04\\x08[\\x07l+\\n\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x00\\x01\\x00@\\x06\"\n",
    "\"Exception\"\n",
);

#[test]
fn test_stdlib_marshal_core_values_execution() {
    let output = run_example("stdlib/marshal_core_values.rb");
    assert_eq!(output, MARSHAL_CORE_VALUES_OUTPUT);
}

#[test]
fn test_stdlib_marshal_core_values_no_parens_execution() {
    let output = run_example("stdlib/marshal_core_values_no_parens.rb");
    assert_eq!(output, MARSHAL_CORE_VALUES_OUTPUT);
}

/// The expected output of both `stdlib/marshal_load_handler` variants.
const MARSHAL_LOAD_HANDLER_OUTPUT: &str = concat!(
    "[\"foo\"]\n",
    "FOO\n",
    "false\n",
    "true\n",
    "true\n",
    "42\n",
    "false\n",
    "\"\\x04\\x08Iu:\\tTime\\r \\x80\\x11\\xC0@\\xE2\\x01\\x00\\t:\\rnano_numi\\x02\\x15\\x03:\\rnano_deni\\x06:\\rsubmicro\\\"\\x07x\\x90:\\tzoneI\\\"\\x08UTC\\x06:\\x06EF\"\n",
    "123456789\n",
    "true\n",
    "(1/3000000000)\n",
);

#[test]
fn test_stdlib_marshal_load_handler_execution() {
    let output = run_example("stdlib/marshal_load_handler.rb");
    assert_eq!(output, MARSHAL_LOAD_HANDLER_OUTPUT);
}

#[test]
fn test_stdlib_marshal_load_handler_no_parens_execution() {
    let output = run_example("stdlib/marshal_load_handler_no_parens.rb");
    assert_eq!(output, MARSHAL_LOAD_HANDLER_OUTPUT);
}

/// The expected output of both `stdlib/weakref_lifetime` variants.
const WEAKREF_LIFETIME_OUTPUT: &str =
    "true\ntrue\nfalse\n\"Invalid Reference - probably recycled\"\ntrue\n";

#[test]
fn test_stdlib_weakref_lifetime_execution() {
    let output = run_example("stdlib/weakref_lifetime.rb");
    assert_eq!(output, WEAKREF_LIFETIME_OUTPUT);
}

#[test]
fn test_stdlib_weakref_lifetime_no_parens_execution() {
    let output = run_example("stdlib/weakref_lifetime_no_parens.rb");
    assert_eq!(output, WEAKREF_LIFETIME_OUTPUT);
}

/// The expected output of both `stdlib/rubygems_interaction` variants.
const RUBYGEMS_INTERACTION_OUTPUT: &str = "\".]2;title. and . text\"\n[[:say, \"plain\"], [:say, \".[31mshown\"]]\n[[:say, \".]2;ok.\"], [:say, \"push: gone.\"], [:exit, 1]]\n[[:error, \"Invalid option: --.]2;x.. See 'gem --help'.\"], [:exit, 1], [:error, \"While executing gem ... (RuntimeError)\\n    bad.\"], [:exit, 1]]\nnil\n";

#[test]
fn test_stdlib_rubygems_interaction_execution() {
    let output = run_example("stdlib/rubygems_interaction.rb");
    assert_eq!(output, RUBYGEMS_INTERACTION_OUTPUT);
}

#[test]
fn test_stdlib_rubygems_interaction_no_parens_execution() {
    let output = run_example("stdlib/rubygems_interaction_no_parens.rb");
    assert_eq!(output, RUBYGEMS_INTERACTION_OUTPUT);
}

/// The expected output of both `stdlib/prime_generators` variants.
const PRIME_GENERATORS_OUTPUT: &str = "[2, 3, 5, 7, 11, 13, 17, 19]\n19\n[2, 3, 5, 7, 11, 13]\n[2, 3, 5]\n2\n2\n[2, 3, 5, 7, 11, 13, 17, 19, 23, 29]\n[2, 3, 5, 7, 11, 13, 17, 19]\n[[2, 1], [3, 2], [5, 3]]\ntrue\nfalse\n[[2, 3], [3, 2], [5, 1]]\n360\ntrue\n[true, true, false]\n[1, 2]\n[2, 3, 4, 5]\n[20, 30]\n";

#[test]
fn test_stdlib_prime_generators_execution() {
    let output = run_example("stdlib/prime_generators.rb");
    assert_eq!(output, PRIME_GENERATORS_OUTPUT);
}

#[test]
fn test_stdlib_prime_generators_no_parens_execution() {
    let output = run_example("stdlib/prime_generators_no_parens.rb");
    assert_eq!(output, PRIME_GENERATORS_OUTPUT);
}

/// The expected output of both `stdlib/bigdecimal_arithmetic` variants.
const BIGDECIMAL_ARITHMETIC_OUTPUT: &str = "0.33333333333333333333333333333333e0\n0.162000001474200013415220122078503110914378309320842614819667794858976933e46\n56\n3\n[4, Integer, 0.6e1]\n[-1, Infinity]\nFloatDomainError\n0.1234567e94\n0.1e1\n\"invalid value for BigDecimal(): \\\"1__2\\\"\"\nnil\n0.1e0\n0.33333e0\n-1\n0.15e1\n[\"loud\", \"0.4444e2\"]\n0.693147180559945309417232121458e0\n0.271828182845904523536028747135e1\n0.22314354220170971436e0\n[1.5, 0.75]\n[(10/1), (3/4)]\n0.25\n";

#[test]
fn test_stdlib_bigdecimal_arithmetic_execution() {
    let output = run_example("stdlib/bigdecimal_arithmetic.rb");
    assert_eq!(output, BIGDECIMAL_ARITHMETIC_OUTPUT);
}

#[test]
fn test_stdlib_bigdecimal_arithmetic_no_parens_execution() {
    let output = run_example("stdlib/bigdecimal_arithmetic_no_parens.rb");
    assert_eq!(output, BIGDECIMAL_ARITHMETIC_OUTPUT);
}

/// The expected output of both `stdlib/zlib_streaming` variants.
const ZLIB_STREAMING_OUTPUT: &str = "[16396, 16392, 7233]\n[16384, 16384, 7253]\n[nil, [16384, 16384, 7253], true]\n[[16384], 3632, true]\n[43759, 52790]\n";

#[test]
fn test_stdlib_zlib_streaming_execution() {
    let output = run_example("stdlib/zlib_streaming.rb");
    assert_eq!(output, ZLIB_STREAMING_OUTPUT);
}

#[test]
fn test_stdlib_zlib_streaming_no_parens_execution() {
    let output = run_example("stdlib/zlib_streaming_no_parens.rb");
    assert_eq!(output, ZLIB_STREAMING_OUTPUT);
}

/// The expected output of both `stdlib/matrix_eigen` variants.
const MATRIX_EIGEN_OUTPUT: &str = "[1.8548973088, 3.4760236029, 6.6690790883]\n3\ntrue\n[(0.0+1.0i), (0.0-1.0i)]\n[[(-1.0+0.0i), (0.0+1.0i)], [(-1.0-0.0i), (0.0-1.0i)]]\n5\nExceptionForMatrix::ErrDimensionMismatch\n\"Expected Matrix but got Integer\"\n";

#[test]
fn test_stdlib_matrix_eigen_execution() {
    let output = run_example("stdlib/matrix_eigen.rb");
    assert_eq!(output, MATRIX_EIGEN_OUTPUT);
}

#[test]
fn test_stdlib_matrix_eigen_no_parens_execution() {
    let output = run_example("stdlib/matrix_eigen_no_parens.rb");
    assert_eq!(output, MATRIX_EIGEN_OUTPUT);
}

/// The expected output of both `stdlib/allocation_tracing` variants.
const ALLOCATION_TRACING_OUTPUT: &str = "[true, 14, \"Shelf\", :stock]\n[true, 14, \"Shelf\", :stock]\n[true, 14, \"Shelf\", :stock]\n[true, 14, \"Shelf\", :stock]\n[true, 18, nil, :build]\n[true, 24, \"Labeled\", :label]\n[true, 36, nil, nil]\ntrue\n[false, nil, nil, nil]\n[false, nil, nil, nil]\n[false, nil, nil, nil]\n[false, nil, nil, nil]\n:finished\n50\nnil\n";

#[test]
fn test_stdlib_allocation_tracing_execution() {
    let output = run_example("stdlib/allocation_tracing.rb");
    assert_eq!(output, ALLOCATION_TRACING_OUTPUT);
}

#[test]
fn test_stdlib_allocation_tracing_no_parens_execution() {
    let output = run_example("stdlib/allocation_tracing_no_parens.rb");
    assert_eq!(output, ALLOCATION_TRACING_OUTPUT);
}
