// Who a protected method answers, and who it refuses.

use super::*;

#[test]
fn a_protected_method_answers_another_object_of_its_own_class() {
    let result = run(r#"
class Pair
  def initialize(value)
    @value = value
  end

  def bigger_than?(other)
    held > other.held
  end

  protected

  def held
    @value
  end
end

Pair.new(2).bigger_than? Pair.new(1)
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("true".to_string())
    );
}

#[test]
fn a_protected_method_is_refused_from_outside_and_named_as_protected() {
    let error = run_err(
        r#"
class Pair
  protected

  def held
    1
  end
end

Pair.new.held
"#,
    );
    assert!(error.contains("protected method 'held'"), "{error}");
}

#[test]
fn a_constant_that_names_a_value_is_compared_against_it() {
    let result = run(r#"
class Level
  LOW = 1
  HIGH = 2

  def named(value)
    case value
    when LOW then :low
    when HIGH then :high
    when String then :text
    else :other
    end
  end
end

held = Level.new
[held.named(1), held.named(2), held.named("a"), held.named(9)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:low, :high, :text, :other]".to_string())
    );
}

#[test]
fn a_number_too_wide_for_a_machine_word_matches_the_integer_pattern() {
    let result = run(r#"
def named(value)
  case value
  when Integer then :whole
  else :other
  end
end

[named(1), named(12345678901234567890123), named("a")]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:whole, :whole, :other]".to_string())
    );
}

#[test]
fn a_number_written_in_exponent_notation_carries_a_signed_power() {
    let result = run(r#"["%e" % [1234.5678], "%.2e" % [0.000123], "%E" % [-5.0]]"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[1.234568e+03, 1.23e-04, -5.000000E+00]".to_string())
    );
}

#[test]
fn the_percent_method_is_named_by_a_symbol_and_reached_through_a_dot() {
    let result = run(r#"
class Divided
  def %(other)
    [:remainder, other]
  end
end

held = Divided.new
[:%, held.%(3), held.send(:%, 4)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:%, [:remainder, 3], [:remainder, 4]]".to_string())
    );
}

#[test]
fn a_percent_literal_after_a_ternary_colon_is_still_a_literal() {
    let result = run(r#"[true ? 1 : %w[a b], false ? 1 : %(text)]"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[1, text]".to_string())
    );
}

#[test]
fn reopening_the_class_of_an_immediate_adds_a_method_it_answers_to() {
    let result = run(r#"
class NilClass
  def held
    :from_nil
  end
end

class TrueClass
  def held
    :from_true
  end
end

[nil.held, true.held]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:from_nil, :from_true]".to_string())
    );
}

#[test]
fn a_handle_sends_what_was_written_through_it_on_when_asked_to_flush() {
    let result = run(r#"
held = "/tmp/metorex_flush_test.txt"
handle = File.open(held, "w+")
handle.write "kept"
handle.flush
size = File.size(held)
handle.close
File.delete(held)
size
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("4".to_string()));
}

#[test]
fn a_protected_method_answers_a_class_method_of_the_same_class() {
    let result = run(r#"
class Counted
  def self.compare(left, right)
    left.held <=> right.held
  end

  def initialize(value)
    @value = value
  end

  protected

  def held
    @value
  end
end

Counted.compare Counted.new(2), Counted.new(1)
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("1".to_string()));
}

#[test]
fn a_protected_method_answers_a_subclass_of_the_class_that_defines_it() {
    let result = run(r#"
class Base
  def initialize(value)
    @value = value
  end

  def sees(other)
    other.held
  end

  protected

  def held
    @value
  end
end

class Grown < Base
end

Grown.new(7).sees Grown.new(9)
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("9".to_string()));
}

#[test]
fn a_protected_method_an_included_module_supplies_answers_the_including_class() {
    let result = run(r#"
module Carried
  def sees(other)
    other.held
  end

  protected

  def held
    :carried
  end
end

class Holder
  include Carried
end

Holder.new.sees Holder.new
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some(":carried".to_string())
    );
}

#[test]
fn a_protected_method_answers_a_value_of_a_reopened_core_class() {
    let result = run(r#"
class Integer
  def sees(other)
    other.doubled
  end

  protected

  def doubled
    self * 2
  end
end

3.sees 5
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("10".to_string())
    );
}
