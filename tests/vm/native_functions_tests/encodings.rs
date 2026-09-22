// The encoding a string is tagged with, and the ids values answer.

use super::*;

#[test]
fn force_encoding_changes_what_a_string_says_it_is() {
    let result = run(r#"
held = "text"
before = held.encoding.name
held.force_encoding("EUC-JP")
[before, held.encoding.name, held]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[UTF-8, EUC-JP, text]".to_string())
    );
}

#[test]
fn every_reference_to_a_string_sees_the_encoding_it_was_given() {
    let result = run(r#"
held = "text"
alias_of_it = held
held.force_encoding("EUC-JP")
alias_of_it.encoding.name
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("EUC-JP".to_string())
    );
}

#[test]
fn packing_answers_a_run_of_bytes_and_an_empty_format_answers_ascii() {
    let result = run(r#"[[65].pack("C").encoding.name, [].pack("").encoding.name]"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[ASCII-8BIT, US-ASCII]".to_string())
    );
}

#[test]
fn encode_tags_text_with_the_encoding_asked_for() {
    // Metorex holds every string's characters as text, so `encode` answers a
    // copy tagged with the encoding asked for rather than rewriting it.
    let result = run(r#"
[ "plain".encode("US-ASCII").encoding.name,
  "é".encode("US-ASCII").encoding.name ]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[US-ASCII, US-ASCII]".to_string())
    );
}

#[test]
fn two_strings_holding_the_same_text_are_two_strings() {
    let result = run(r#"
first = "same"
second = "same"
[first == second, first.equal?(second), first.equal?(first)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, false, true]".to_string())
    );
}

#[test]
fn a_string_answers_the_same_id_every_time_it_is_asked() {
    let result = run(r#"
held = "same"
other = "same"
[held.object_id == held.object_id, held.object_id == other.object_id]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, false]".to_string())
    );
}

#[test]
fn the_values_that_write_themselves_answer_one_unchanging_string() {
    let result = run(r#"
module NamedOnce; end
[nil.to_s.equal?(nil.to_s),
 true.to_s.equal?(true.to_s),
 false.to_s.equal?(false.to_s),
 :held.name.equal?(:held.name),
 NamedOnce.name.equal?(NamedOnce.name),
 :held.id2name.equal?(:held.id2name)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true, true, true, true, false]".to_string())
    );
}

#[test]
fn a_module_that_gains_a_name_answers_the_new_one() {
    let result = run(r#"
made = Module.new
before = made.name
Gained = made
[before, made.name]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[nil, Gained]".to_string())
    );
}

#[test]
fn a_tie_answers_the_one_that_came_first() {
    let result = run(r#"
first = "2"
second = "2"
held = [first, second]
[held.max_by { |value| value.to_i }.equal?(first),
 held.min_by { |value| value.to_i }.equal?(first)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true]".to_string())
    );
}

#[test]
fn a_string_value_hashes_and_compares_by_the_text_it_holds() {
    // Two strings holding the same text are one hash key, even though they
    // are two objects.
    let result = run(r#"
held = {}
held["same"] = 1
held["same"] = 2
[held.size, held["same"], "same" == "same".dup]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[1, 2, true]".to_string())
    );
}

#[test]
fn a_wide_character_packs_as_the_bytes_its_encoding_needs() {
    let result = run(r#"[[960].pack("U").length, [960].pack("U").unpack("U")]"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[1, [960]]".to_string())
    );
}
