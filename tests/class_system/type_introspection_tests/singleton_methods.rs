// The methods one object alone answers.

use super::*;

#[test]
fn singleton_method_finds_a_def_on_the_object() {
    let result = run(r#"
widget = Object.new
def widget.polish
  :shiny
end
widget.singleton_method(:polish).call.inspect
"#);
    assert_eq!(result, Some(Object::string(":shiny".to_string())));
}

#[test]
fn singleton_method_answers_a_method_object() {
    let result = run(r#"
widget = Object.new
def widget.polish
end
widget.singleton_method(:polish).class.name
"#);
    assert_eq!(result, Some(Object::string("Method".to_string())));
}

#[test]
fn singleton_method_finds_a_module_included_in_the_singleton_class() {
    let result = run(r#"
included = Module.new do
  def from_include
    :included
  end
end
widget = Object.new
widget.singleton_class.include(included)
widget.singleton_method(:from_include).call.inspect
"#);
    assert_eq!(result, Some(Object::string(":included".to_string())));
}

#[test]
fn singleton_method_finds_a_module_attached_by_extend() {
    let result = run(r#"
extension = Module.new do
  def from_extend
    :extended
  end
end
widget = Object.new
widget.extend(extension)
widget.singleton_method(:from_extend).call.inspect
"#);
    assert_eq!(result, Some(Object::string(":extended".to_string())));
}

#[test]
fn singleton_method_finds_a_class_method() {
    let result = run(r#"
class Registry
  def self.lookup
    :found
  end
end
Registry.singleton_method(:lookup).call.inspect
"#);
    assert_eq!(result, Some(Object::string(":found".to_string())));
}

#[test]
fn singleton_method_does_not_look_at_the_objects_class() {
    let error = run_err(
        r#"
class Widget
  def instance_level
  end
end
Widget.new.singleton_method(:instance_level)
"#,
    );
    assert!(error.contains("undefined singleton method 'instance_level'"));
}

#[test]
fn singleton_method_raises_for_a_name_nothing_defines() {
    let error = run_err("Object.new.singleton_method(:never_defined)");
    assert!(error.contains("undefined singleton method 'never_defined'"));
}

// ── Kernel#singleton_methods ─────────────────────────────────────────────────

#[test]
fn singleton_methods_is_empty_for_a_plain_object() {
    let result = run("Object.new.singleton_methods.inspect");
    assert_eq!(result, Some(Object::string("[]".to_string())));
}

#[test]
fn singleton_methods_lists_a_def_on_the_object() {
    let result = run(r#"
widget = Object.new
def widget.polish
end
widget.singleton_methods.inspect
"#);
    assert_eq!(result, Some(Object::string("[:polish]".to_string())));
}

#[test]
fn singleton_methods_includes_an_extended_module_by_default() {
    let result = run(r#"
module Greeting
  def greet
  end
end
widget = Object.new
widget.extend(Greeting)
widget.singleton_methods.inspect
"#);
    assert_eq!(result, Some(Object::string("[:greet]".to_string())));
}

#[test]
fn singleton_methods_without_ancestors_leaves_out_an_extended_module() {
    let result = run(r#"
module Greeting
  def greet
  end
end
widget = Object.new
widget.extend(Greeting)
widget.singleton_methods(false).inspect
"#);
    assert_eq!(result, Some(Object::string("[]".to_string())));
}

const SINGLETON_CLASSES: &str = r#"
class Parent
  def self.parent_class_method
  end
end

class Child < Parent
  def self.child_class_method
  end

  class << self
    def opened_on_child
    end

    private

    def hidden_class_method
    end
  end
end
"#;

#[test]
fn singleton_methods_reaches_an_inherited_class_method() {
    let result = run(&format!(
        "{SINGLETON_CLASSES}\nChild.singleton_methods.sort.inspect"
    ));
    assert_eq!(
        result,
        Some(Object::string(
            "[:child_class_method, :opened_on_child, :parent_class_method]".to_string()
        ))
    );
}

#[test]
fn singleton_methods_without_ancestors_stops_at_the_class() {
    let result = run(&format!(
        "{SINGLETON_CLASSES}\nChild.singleton_methods(false).sort.inspect"
    ));
    assert_eq!(
        result,
        Some(Object::string(
            "[:child_class_method, :opened_on_child]".to_string()
        ))
    );
}

#[test]
fn singleton_methods_leaves_out_a_private_class_method() {
    let result = run(&format!(
        "{SINGLETON_CLASSES}\nChild.singleton_methods.include?(:hidden_class_method)"
    ));
    assert_eq!(result, Some(Object::Bool(false)));
}

// ── extend self ──────────────────────────────────────────────────────────────

#[test]
fn extend_self_makes_a_modules_methods_callable_on_it() {
    let result = run(r#"
module Helper
  extend self

  def assist
    :assisted
  end
end
Helper.assist.inspect
"#);
    assert_eq!(result, Some(Object::string(":assisted".to_string())));
}

#[test]
fn top_level_extend_makes_the_methods_callable() {
    let result = run(r#"
module Helper
  def assist
    :assisted
  end
end
extend Helper
assist.inspect
"#);
    assert_eq!(result, Some(Object::string(":assisted".to_string())));
}

// ── Array slicing with a Range ───────────────────────────────────────────────

#[test]
fn an_array_slices_with_an_inclusive_range() {
    let result = run("[1, 2, 3, 4, 5][1..3].inspect");
    assert_eq!(result, Some(Object::string("[2, 3, 4]".to_string())));
}

#[test]
fn an_array_slices_with_an_exclusive_range() {
    let result = run("[1, 2, 3, 4, 5][1...3].inspect");
    assert_eq!(result, Some(Object::string("[2, 3]".to_string())));
}

#[test]
fn an_array_slices_with_an_endless_range() {
    let result = run("[1, 2, 3, 4, 5][2..].inspect");
    assert_eq!(result, Some(Object::string("[3, 4, 5]".to_string())));
}

#[test]
fn an_array_slices_with_a_beginless_range() {
    let result = run("[1, 2, 3, 4, 5][..2].inspect");
    assert_eq!(result, Some(Object::string("[1, 2, 3]".to_string())));
}

#[test]
fn an_array_slice_counts_a_negative_bound_from_the_end() {
    let result = run("[1, 2, 3, 4, 5][-2..].inspect");
    assert_eq!(result, Some(Object::string("[4, 5]".to_string())));
}

#[test]
fn an_array_slice_past_the_end_is_nil() {
    let result = run("[1, 2, 3][9..].inspect");
    assert_eq!(result, Some(Object::string("nil".to_string())));
}

#[test]
fn an_array_slice_at_the_end_is_empty() {
    let result = run("[1, 2, 3][3..].inspect");
    assert_eq!(result, Some(Object::string("[]".to_string())));
}
