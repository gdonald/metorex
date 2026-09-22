// Reading and writing what an instance carries.

use super::super::run_example;
use super::*;
#[test]
fn test_metaprogramming_implicit_blocks_execution() {
    let expected = "Howdy, Alice!\nHey, Bob!\nHello, Charlie!\nIteration: 0\nIteration: 1\nIteration: 2\nno block\ngot a block\n10\n20\n1\n4\n9\n1\n4\n9\n16\n2\n4\n6\n1\n2\n3\n4\n5\n";
    let output = run_example("metaprogramming/implicit_blocks.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_get_source_execution() {
    let expected = "speak\nfetch\ntrue\nspeak\npurr\npurr\n";
    let output = run_example("metaprogramming/get_source.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_instance_exec_execution() {
    let expected = "10\n20\n105\n";
    let output = run_example("metaprogramming/instance_exec/instance_exec.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_instance_exec_parens_execution() {
    let expected = "10\n20\n105\n";
    let output = run_example("metaprogramming/instance_exec/instance_exec_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_instance_eval_source_execution() {
    let expected = concat!(
        "42\n42\nHOLA\n",
        "wrong number of arguments (given 2, expected 0)\n",
        "wrong number of arguments (given 0, expected 1..3)\n",
        "wrong number of arguments (given 4, expected 1..3)\n"
    );
    let output = run_example("metaprogramming/instance_eval_source/strings.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_instance_eval_source_parens_execution() {
    let expected = concat!(
        "42\n42\nHOLA\n",
        "wrong number of arguments (given 2, expected 0)\n",
        "wrong number of arguments (given 0, expected 1..3)\n",
        "wrong number of arguments (given 4, expected 1..3)\n"
    );
    let output = run_example("metaprogramming/instance_eval_source/strings_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_instance_exec_receivers_execution() {
    let expected = concat!(
        "7\n10\n3\n",
        "no block given (yield)\n",
        "can't define singleton\n",
        "can't define singleton\n",
        "-1\n-1\n"
    );
    let output = run_example("metaprogramming/instance_exec/receivers.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_instance_exec_receivers_parens_execution() {
    let expected = concat!(
        "7\n10\n3\n",
        "no block given (yield)\n",
        "can't define singleton\n",
        "can't define singleton\n",
        "-1\n-1\n"
    );
    let output = run_example("metaprogramming/instance_exec/receivers_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_metaprogramming_instance_exec_singleton_def_execution() {
    let output = run_example("metaprogramming/instance_exec/singleton_def.rb");
    assert_eq!(output, INSTANCE_EXEC_SINGLETON_DEF_OUTPUT);
}

#[test]
fn test_metaprogramming_instance_exec_singleton_def_parens_execution() {
    let output = run_example("metaprogramming/instance_exec/singleton_def_parens.rb");
    assert_eq!(output, INSTANCE_EXEC_SINGLETON_DEF_OUTPUT);
}

#[test]
fn test_metaprogramming_instance_exec_reading_a_string_execution() {
    let output = run_example("metaprogramming/instance_exec/reading_a_string.rb");
    assert_eq!(output, READING_A_STRING);
}

#[test]
fn test_metaprogramming_instance_exec_reading_a_string_no_parens_execution() {
    let output = run_example("metaprogramming/instance_exec/reading_a_string_no_parens.rb");
    assert_eq!(output, READING_A_STRING);
}
