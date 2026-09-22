// The protected methods an object answers.

use super::*;

const PROTECTED_FIXTURE: &str = r#"
module Helpers
  def mixed_in
  end
  protected :mixed_in
end

class Parent
  def parent_guard
  end
  protected :parent_guard

  class << self
    def parent_class_guard
    end
    protected :parent_class_guard
  end
end

class Child < Parent
  include Helpers

  def child_guard
  end
  protected :child_guard

  class << self
    def child_class_guard
    end
    protected :child_class_guard
  end
end
"#;

fn guards(code: &str) -> Option<Object> {
    run(&format!(
        "{PROTECTED_FIXTURE}\n({code}).select {{ |n| n.to_s.end_with?(\"guard\") }}.sort.inspect"
    ))
}

#[test]
fn protected_methods_without_ancestors_lists_only_the_objects_own() {
    let result = guards("Child.new.protected_methods(false)");
    assert_eq!(result, Some(Object::string("[:child_guard]".to_string())));
}

#[test]
fn protected_methods_with_ancestors_reaches_the_superclass() {
    let result = guards("Child.new.protected_methods");
    assert_eq!(
        result,
        Some(Object::string("[:child_guard, :parent_guard]".to_string()))
    );
}

#[test]
fn protected_methods_treats_nil_like_false() {
    let result = guards("Child.new.protected_methods(nil)");
    assert_eq!(result, Some(Object::string("[:child_guard]".to_string())));
}

#[test]
fn a_classs_protected_methods_come_from_its_singleton_chain() {
    let result = guards("Child.protected_methods(false)");
    assert_eq!(
        result,
        Some(Object::string(
            "[:child_class_guard, :parent_class_guard]".to_string()
        ))
    );
}

#[test]
fn protected_methods_includes_a_module_attached_by_extend() {
    let result = run(&format!(
        "{PROTECTED_FIXTURE}\nobj = Object.new\nobj.extend(Helpers)\nobj.protected_methods.include?(:mixed_in)"
    ));
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn protected_methods_excludes_a_mixin_when_ancestors_are_excluded() {
    let result = run(&format!(
        "{PROTECTED_FIXTURE}\nChild.new.protected_methods(false).include?(:mixed_in)"
    ));
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn protected_methods_includes_a_singleton_method() {
    let result = run(r#"
widget = Object.new
class << widget
  def singleton_guard
  end
  protected :singleton_guard
end
widget.protected_methods(false).inspect
"#);
    assert_eq!(
        result,
        Some(Object::string("[:singleton_guard]".to_string()))
    );
}

#[test]
fn protected_methods_rejects_extra_arguments() {
    let error = run_err("Object.new.protected_methods(true, false)");
    assert!(error.contains("wrong number of arguments (given 2, expected 1)"));
}
