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
    let expected = "true\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nNumeric\nBasicObject\n3\ntrue\ntrue\nAnimal\n2\nRex\n3\n4\n";
    let output = run_example("builtins/type_introspection.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_type_introspection_parens_execution() {
    let expected = "true\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\nNumeric\nBasicObject\n3\ntrue\ntrue\nAnimal\n2\nRex\n3\n4\n";
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
    let expected = "/hello/\n/foo/\n/bar/i\n/[a-z]+/\n/path\\/to\\/file/\n5\n4\n5\n";
    let output = run_example("stdlib/regex/contexts.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_regex_contexts_parens_execution() {
    let expected = "/hello/\n/foo/\n/bar/i\n/[a-z]+/\n/path\\/to\\/file/\n5\n4\n5\n";
    let output = run_example("stdlib/regex/contexts_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_regex_literals_execution() {
    let expected = "/hello/\n/world/i\n5\n/[a-z]+\\d+/\n/hello\\/world/\n";
    let output = run_example("stdlib/regex/literals.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_regex_literals_parens_execution() {
    let expected = "/hello/\n/world/i\n5\n/[a-z]+\\d+/\n/hello\\/world/\n";
    let output = run_example("stdlib/regex/literals_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_slice_edge_execution() {
    let expected = "ell\nhe\nll\n\ndone\nfoobar\n";
    let output = run_example("stdlib/string/slice_edge.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_slice_edge_parens_execution() {
    let expected = "ell\nhe\nll\n\ndone\nfoobar\n";
    let output = run_example("stdlib/string/slice_edge_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_edge_execution() {
    let expected = "hello\nhello\nhello\nhello\nA\nZ\n      hi!\n7\n5.000000\n5.00\ntest%\n";
    let output = run_example("stdlib/string/format_edge.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_stdlib_string_format_edge_parens_execution() {
    let expected = "hello\nhello\nhello\nhello\nA\nZ\n      hi!\n7\n5.000000\n5.00\ntest%\n";
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
    let expected = "15\n25\n12345\n3\n4\n3\n1\n2\n3\n4\n1\n2\n3\ntrue\nfalse\n10\n30\nnil\nnil\ntrue\nfalse\ntrue\n1\n8\n1.2\n3.5\nnil\nnil\n3\n1\n2\n";
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
    let expected = "3.14\n-3.14\n4\n3\n4.0\n3\n3.14\n42.0\n42\n42\n2\nell\ntrue\ntrue\ntrue\nhello\nhello\nHELLO\nolleh\n1, 2, 3\n2, 1, 3\n3\n0\n0\n123\nnil\nfalse\ntrue\n3\n15\n3\nerror_paths_test passed\n";
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
        "\"caf\\u{e9}\"\n",
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
        "\"caf\\u{e9}\"\n",
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

#[test]
fn test_stdlib_string_format_keywords_execution() {
    let expected = concat!(
        "a and b\n",
        "test value\n",
        "hello, world!\n",
        "00042\n",
        "3.14\n",
        "through the module\n",
        "key<missing> not found\n",
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
        "key<missing> not found\n",
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
