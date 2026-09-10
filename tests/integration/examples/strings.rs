//! Examples under `tests/_examples/strings`.

use super::run_example;

/// The expected output of both `strings/in_place/changes` variants.
const STRING_IN_PLACE_OUTPUT: &str = concat!(
    "\"abcdefg!\"\n",
    "\"oh, !hello\"\n",
    "\"oh, !\"\n",
    "\"hello\"\n",
    "\"HELLO\"\n",
    "nil\n",
    "\"HEy\"\n",
    "\"yEH\"\n",
    "\"\"\n",
    "true\n",
    "\"can't modify frozen String: \\\"kept\\\"\"\n",
    "false\n",
    "true\n",
    "\"cÅr\"\n",
    "\"i\"\n",
    "\"ss\"\n",
    "\"Sset\"\n",
    "[41, 0]\n"
);

#[test]
fn test_strings_in_place_changes_execution() {
    let output = run_example("strings/in_place/changes.rb");
    assert_eq!(output, STRING_IN_PLACE_OUTPUT);
}

#[test]
fn test_strings_in_place_changes_parens_execution() {
    let output = run_example("strings/in_place/changes_parens.rb");
    assert_eq!(output, STRING_IN_PLACE_OUTPUT);
}
