// Drawing numbers, and the seed a sequence repeats from.

use super::super::run_example;
#[test]
fn test_builtins_rand_and_numeric() {
    let expected = concat!(
        "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n",
        "nil\n42\n1.5\ntrue\n",
        "TypeError: no implicit conversion of String into Integer\n",
        "true\ntrue\nNumeric\nNumeric\n",
        "true\ntrue\ntrue\n1\n-1\ntrue\n"
    );
    let output = run_example("builtins/rand_and_numeric.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_rand_and_numeric_parens() {
    let expected = concat!(
        "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n",
        "nil\n42\n1.5\ntrue\n",
        "TypeError: no implicit conversion of String into Integer\n",
        "true\ntrue\nNumeric\nNumeric\n",
        "true\ntrue\ntrue\n1\n-1\ntrue\n"
    );
    let output = run_example("builtins/rand_and_numeric_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_srand_seeding() {
    let expected = concat!(
        "10\n20\n0\ntrue\ntrue\n3\n7\ntrue\ntrue\n",
        "TypeError\nTypeError\ntrue\n"
    );
    let output = run_example("builtins/srand_seeding.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_srand_seeding_parens() {
    let expected = concat!(
        "10\n20\n0\ntrue\ntrue\n3\n7\ntrue\ntrue\n",
        "TypeError\nTypeError\ntrue\n"
    );
    let output = run_example("builtins/srand_seeding_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_random_source_generator_execution() {
    let expected = concat!(
        "\"\\x14\\\\\"\n",
        "\"_\\x91\"\n",
        "0.1915194503788923\n",
        "37.454011884736246\n",
        "true\n",
        "42\n",
        "8\n",
        "true\n",
        "true\n",
        "true\n",
        "\"bad value for range\"\n"
    );
    let output = run_example("builtins/random_source/generator.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_random_source_generator_parens_execution() {
    let expected = concat!(
        "\"\\x14\\\\\"\n",
        "\"_\\x91\"\n",
        "0.1915194503788923\n",
        "37.454011884736246\n",
        "true\n",
        "42\n",
        "8\n",
        "true\n",
        "true\n",
        "true\n",
        "\"bad value for range\"\n"
    );
    let output = run_example("builtins/random_source/generator_parens.rb");
    assert_eq!(output, expected);
}
