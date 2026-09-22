// The constants a pattern compares against, wherever they are defined.

use super::*;

#[test]
fn a_constant_an_included_module_supplies_is_compared_against_in_a_pattern() {
    let result = run(r#"
module Levels
  QUIET = 5
end

class Reader
  include Levels

  def named(value)
    case value
    when QUIET then :quiet
    else :other
    end
  end
end

[Reader.new.named(5), Reader.new.named(6)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:quiet, :other]".to_string())
    );
}

#[test]
fn a_constant_a_superclass_supplies_is_compared_against_in_a_pattern() {
    let result = run(r#"
class Above
  MARK = 4
end

class Below < Above
  def named(value)
    case value
    when MARK then :marked
    else :other
    end
  end
end

[Below.new.named(4), Below.new.named(5)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:marked, :other]".to_string())
    );
}

#[test]
fn a_constant_named_in_a_pattern_from_a_class_method_is_compared_against() {
    let result = run(r#"
class Gauge
  STEP = 3

  def self.named(value)
    case value
    when STEP then :step
    else :other
    end
  end
end

[Gauge.named(3), Gauge.named(4)]
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some("[:step, :other]".to_string())
    );
}

#[test]
fn a_pattern_naming_a_class_nothing_defines_matches_nothing() {
    let result = run(r#"
def named(value)
  case value
  when NoSuchClassAnywhere then :found
  else :other
  end
end

named 1
"#);
    assert_eq!(
        result.map(|value| value.to_string()),
        Some(":other".to_string())
    );
}
