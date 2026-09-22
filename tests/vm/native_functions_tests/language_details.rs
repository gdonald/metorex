// Corners of the language the example scripts cannot reach.

use super::*;

#[test]
fn a_bang_method_answers_for_the_operator_written_in_front_of_it() {
    let result = run(r#"
class Negated
  def !
    :flipped
  end

  def ~
    :complemented
  end
end

[!Negated.new, ~Negated.new, Negated.new.!]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:flipped, :complemented, :flipped]".to_string())
    );
}

#[test]
fn a_pattern_can_name_a_constant_from_the_top_level() {
    let result = run(r#"
answers = []
["a", 1].each do |held|
  case held
  when ::String then answers << :string
  when ::Integer then answers << :integer
  end
end
answers
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:string, :integer]".to_string())
    );
}

#[test]
fn a_global_variable_is_not_answered_as_a_constant_of_the_same_name() {
    let result = run(r#"
class Level
  DEBUG = 3

  def named
    DEBUG
  end

  def defaulted(held: DEBUG)
    held
  end
end

[Level.new.named, Level.new.defaulted, $DEBUG]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[3, 3, false]".to_string())
    );
}

#[test]
fn a_yield_with_no_block_behind_it_is_a_jump_with_nowhere_to_land() {
    let result = run(r#"
def needs_one
  yield
end

begin
  needs_one
rescue LocalJumpError => problem
  [problem.class, problem.message]
end
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[LocalJumpError, no block given (yield)]".to_string())
    );
}

#[test]
fn methods_reports_what_an_included_module_supplies() {
    let result = run(r#"
module Carried
  def carried_name
    :carried
  end
end

class Holder
  include Carried

  def own_name
    :own
  end
end

held = Holder.new.methods
[held.include?(:carried_name), held.include?(:own_name)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true]".to_string())
    );
}

#[test]
fn indexing_reaches_method_missing_when_no_index_is_defined() {
    let result = run(r#"
class Asked
  def method_missing(name, *arguments)
    [name, arguments]
  end

  def respond_to_missing?(_name, _private = false)
    true
  end
end

Asked.new[7]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:[], [7]]".to_string())
    );
}

#[test]
fn a_double_splat_reaches_a_call_written_without_parentheses() {
    let result = run(r#"
def take(first, **rest)
  [first, rest]
end

extra = {second: 2}
take 1, **extra
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[1, {second: 2}]".to_string())
    );
}

#[test]
fn a_string_formatted_in_place_of_nil_says_nothing_at_all() {
    let result = run(r#"["[%s]" % [nil], "[%s]" % ["held"]]"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[[], [held]]".to_string())
    );
}

#[test]
fn an_open_handle_says_it_answers_to_what_it_can_do() {
    let result = run(r#"
held = "/tmp/metorex_handle_answers_test.txt"
handle = File.open(held, "w+")
answered = [
  handle.respond_to?(:write),
  handle.respond_to?(:readlines),
  handle.respond_to?(:no_such_thing)
]
handle.close
File.delete(held)
answered
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[true, true, false]".to_string())
    );
}

#[test]
fn a_name_may_be_moved_onto_another_one() {
    let result = run(r#"
from = "/tmp/metorex_rename_from.txt"
to = "/tmp/metorex_rename_to.txt"
File.write(from, "held")
File.rename(from, to)
answered = [File.exist?(from), File.read(to)]
File.delete(to)
answered
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[false, held]".to_string())
    );
}

#[test]
fn moving_a_name_no_file_answers_to_is_refused() {
    let error =
        run_err(r#"File.rename("/tmp/metorex_no_such_source.txt", "/tmp/metorex_dest.txt")"#);
    assert!(error.contains("No such file or directory"), "{error}");
}

#[test]
fn the_permissions_on_a_file_can_be_set_by_name() {
    let result = run(r#"
held = "/tmp/metorex_chmod_test.txt"
File.write(held, "x")
changed = File.chmod(0600, held)
mode = "%o" % File.stat(held).mode
File.delete(held)
[changed, mode]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[1, 100600]".to_string())
    );
}

#[test]
fn setting_permissions_on_a_name_that_is_not_there_raises() {
    let err = run_err(r#"File.chmod(0600, "/tmp/metorex_no_such_chmod_target.txt")"#);
    assert!(err.contains("No such file or directory"));
}

#[test]
fn the_scratch_directory_is_named_without_a_trailing_separator() {
    let result = run(r#"[Dir.tmpdir.end_with?("/"), Dir.exist?(Dir.tmpdir)]"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[false, true]".to_string())
    );
}

#[test]
fn digits_too_wide_for_a_machine_word_still_name_a_number() {
    let result = run(r#"
held = "33333333333333333333"
[held.to_i.to_s, "9223372036854775808".to_i.to_s, "ff".to_i(16)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[33333333333333333333, 9223372036854775808, 255]".to_string())
    );
}
