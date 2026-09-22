// A method held as an object, and the names an object answers.

use super::*;

#[test]
fn method_converts_its_name_argument_with_to_str() {
    let result = run(r#"
class Named
  def to_str
    "upcase"
  end
end
"shout".method(Named.new).call
"#);
    assert_eq!(result, Some(Object::string("SHOUT".to_string())));
}

#[test]
fn method_raises_type_error_for_a_name_it_cannot_convert() {
    let error = run_err("Object.new.method([])");
    assert!(error.contains("no implicit conversion of Array into String"));
}

#[test]
fn method_propagates_an_error_raised_inside_to_str() {
    let error = run_err(
        r#"
class Exploding
  def to_str
    raise NoMethodError, "from to_str"
  end
end
Object.new.method(Exploding.new)
"#,
    );
    assert!(error.contains("from to_str"));
}

#[test]
fn method_answers_a_name_claimed_by_respond_to_missing() {
    let result = run(r#"
class Ghost
  def respond_to_missing?(name, include_private = false)
    name == :haunt
  end

  def method_missing(name, *args)
    "called #{name} with #{args.inspect}"
  end
end
Ghost.new.method(:haunt).call(1, 2)
"#);
    assert_eq!(
        result,
        Some(Object::string("called haunt with [1, 2]".to_string()))
    );
}

#[test]
fn method_asks_respond_to_missing_with_private_allowed() {
    let result = run(r#"
class Ghost
  def respond_to_missing?(name, include_private = false)
    name == :whisper && include_private
  end

  def method_missing(name, *args)
    name
  end
end
Ghost.new.method(:whisper).call.inspect
"#);
    assert_eq!(result, Some(Object::string(":whisper".to_string())));
}

#[test]
fn method_missing_dispatcher_keeps_its_own_arity() {
    let error = run_err(
        r#"
class OneArgument
  def respond_to_missing?(name, include_private = false)
    name == :only_name
  end

  def method_missing(name)
    name
  end
end
OneArgument.new.method(:only_name).call(1)
"#,
    );
    assert!(error.contains("wrong number of arguments (given 2, expected 1)"));
}

#[test]
fn method_still_raises_name_error_when_respond_to_missing_says_no() {
    let error = run_err(
        r#"
class Ghost
  def respond_to_missing?(name, include_private = false)
    false
  end
end
Ghost.new.method(:unknown)
"#,
    );
    assert!(error.contains("undefined method 'unknown' for class 'Ghost'"));
}

// ── Object#methods ───────────────────────────────────────────────────────────

#[test]
fn methods_lists_a_def_on_the_object_itself() {
    let result = run(r#"
class Widget
end
widget = Widget.new
def widget.polish
  :shiny
end
widget.methods(false).inspect
"#);
    assert_eq!(result, Some(Object::string("[:polish]".to_string())));
}

#[test]
fn methods_without_ancestors_leaves_out_a_module_attached_by_extend() {
    // `methods(false)` is `singleton_methods(false)`, which reports only the
    // methods defined directly on the object.
    let result = run(r#"
module Greeting
  def greet
    "hello"
  end
end
class Widget
end
widget = Widget.new
widget.extend(Greeting)
[widget.methods(false).length, widget.methods.include?(:greet)].inspect
"#);
    assert_eq!(result, Some(Object::string("[0, true]".to_string())));
}

#[test]
fn methods_omits_a_private_singleton_method() {
    let result = run(r#"
class Widget
end
widget = Widget.new
class << widget
  def buff
    :buffed
  end

  private

  def secret
    :hidden
  end
end
widget.methods(false).inspect
"#);
    assert_eq!(result, Some(Object::string("[:buff]".to_string())));
}

#[test]
fn methods_omits_a_singleton_method_that_was_undefined() {
    let result = run(r#"
class Widget
end
widget = Widget.new
def widget.polish
  :shiny
end
singleton = class << widget
  self
end
singleton.send(:undef_method, :polish)
widget.methods(false).inspect
"#);
    assert_eq!(result, Some(Object::string("[]".to_string())));
}

#[test]
fn methods_omits_an_inherited_method_the_class_undefined() {
    let result = run(r#"
class Parent
  def inherited_method
    :from_parent
  end
end
class Child < Parent
  undef_method :inherited_method
end
Child.new.methods.include?(:inherited_method)
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn methods_omits_class_methods_from_an_instance() {
    let result = run(r#"
class Widget
  def self.build
    :built
  end
end
Widget.new.methods.any? { |name| name.to_s.start_with?("__class__") }
"#);
    assert_eq!(result, Some(Object::Bool(false)));
}
