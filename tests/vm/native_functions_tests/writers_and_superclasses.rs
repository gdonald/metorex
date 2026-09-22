// Naming a superclass from the top level, and the attribute writers a
// body defines.

use super::*;

#[test]
fn a_superclass_can_be_named_from_the_top_level() {
    let result = run(r#"
class Basis
  def label
    :root
  end
end

module Nested
  class Basis < ::Basis
  end
end

Nested::Basis.new.label
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some(":root".to_string())
    );
}

#[test]
fn a_superclass_named_from_the_top_level_can_be_built_by_a_call() {
    let result = run(r#"
module Holder
  class Pair < ::Struct.new(:left, :right)
  end
end

Holder::Pair.new(1, 2).left
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("1".to_string()));
}

#[test]
fn a_block_on_super_reaches_the_parent_method() {
    let result = run(r#"
class Source
  def read
    yield
  end
end

class Doubled < Source
  def read
    super { 21 } * 2
  end
end

Doubled.new.read
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("42".to_string())
    );
}

#[test]
fn a_do_block_on_super_with_an_argument_list_reaches_the_parent_method() {
    let result = run(r#"
class Source
  def read(offset)
    offset + yield
  end
end

class Shifted < Source
  def read(offset)
    super(offset) do
      5
    end
  end
end

Shifted.new.read(10)
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("15".to_string())
    );
}

#[test]
fn a_writer_takes_its_parameter_without_parentheses() {
    let result = run(r#"
class Gauge
  def level
    @level
  end

  def level= reading
    @level = reading
  end
end

gauge = Gauge.new
gauge.level = 7
gauge.level
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("7".to_string()));
}

#[test]
fn a_space_before_the_equals_makes_the_definition_an_endless_one() {
    let result = run(r#"
class Gauge
  def level = 41
end

Gauge.new.level + 1
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("42".to_string())
    );
}

#[test]
fn a_class_level_writer_takes_its_parameter_without_parentheses() {
    let result = run(r#"
class Gauge
  def self.limit= ceiling
    @@limit = ceiling
  end

  def self.limit
    @@limit
  end
end

Gauge.limit = 3
Gauge.limit
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("3".to_string()));
}

#[test]
fn a_writer_written_on_one_object_answers_for_that_object() {
    let result = run(r#"
class Plain
  def held
    @held
  end
end

plain = Plain.new
def plain.held= value
  @held = value * 2
end

plain.held = 6
plain.held
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("12".to_string())
    );
}

#[test]
fn a_module_level_writer_answers_for_the_module() {
    let result = run(r#"
module Settings
  def self.depth= value
    @depth = value
  end

  def self.depth
    @depth
  end
end

Settings.depth = 4
Settings.depth
"#);
    assert_eq!(result.map(|value| value.to_string()), Some("4".to_string()));
}
