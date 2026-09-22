// The private methods an object answers, and `=~` on a Symbol.

use super::*;

const PRIVATE_FIXTURE: &str = r#"
module Helpers
  def mixed_in
  end
  private :mixed_in
end

class Parent
  def parent_secret
  end
  private :parent_secret

  class << self
    def parent_class_secret
    end
    private :parent_class_secret
  end
end

class Child < Parent
  include Helpers

  def child_secret
  end
  private :child_secret

  class << self
    def child_class_secret
    end
    private :child_class_secret
  end
end
"#;

fn secrets(code: &str) -> Option<Object> {
    run(&format!(
        "{PRIVATE_FIXTURE}\n({code}).select {{ |n| n.to_s.end_with?(\"secret\") }}.sort.inspect"
    ))
}

#[test]
fn private_methods_without_ancestors_lists_only_the_objects_own() {
    let result = secrets("Child.new.private_methods(false)");
    assert_eq!(result, Some(Object::string("[:child_secret]".to_string())));
}

#[test]
fn private_methods_with_ancestors_reaches_the_superclass() {
    let result = secrets("Child.new.private_methods");
    assert_eq!(
        result,
        Some(Object::string(
            "[:child_secret, :parent_secret]".to_string()
        ))
    );
}

#[test]
fn private_methods_treats_nil_like_false() {
    let result = secrets("Child.new.private_methods(nil)");
    assert_eq!(result, Some(Object::string("[:child_secret]".to_string())));
}

#[test]
fn a_classs_private_methods_come_from_its_singleton_chain() {
    let result = secrets("Child.private_methods(false)");
    assert_eq!(
        result,
        Some(Object::string(
            "[:child_class_secret, :parent_class_secret]".to_string()
        ))
    );
}

#[test]
fn private_methods_includes_a_class_private_singleton_method() {
    let result = run(&format!(
        "{PRIVATE_FIXTURE}\nChild.private_methods.include?(:child_class_secret)"
    ));
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn private_methods_includes_a_module_attached_by_extend() {
    let result = run(&format!(
        "{PRIVATE_FIXTURE}\nobj = Object.new\nobj.extend(Helpers)\nobj.private_methods.include?(:mixed_in)"
    ));
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn private_methods_excludes_a_mixin_when_ancestors_are_excluded() {
    let result = run(&format!(
        "{PRIVATE_FIXTURE}\nChild.new.private_methods(false).include?(:mixed_in)"
    ));
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn private_methods_rejects_extra_arguments() {
    let error = run_err("Object.new.private_methods(true, false)");
    assert!(error.contains("wrong number of arguments (given 2, expected 1)"));
}

// ── =~ against a Symbol ──────────────────────────────────────────────────────

#[test]
fn a_regexp_matches_a_symbol_by_its_name() {
    let result = run(r"(/_secret\z/ =~ :child_secret)");
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn a_symbol_matches_a_regexp_on_the_left() {
    let result = run(r"(:child_secret =~ /_secret\z/)");
    assert_eq!(result, Some(Object::Int(5)));
}

#[test]
fn a_regexp_that_misses_a_symbol_is_nil() {
    let result = run(r"(/nope\z/ =~ :child_secret).inspect");
    assert_eq!(result, Some(Object::string("nil".to_string())));
}

#[test]
fn not_match_negates_a_symbol_match() {
    let result = run(r"(:child_secret !~ /_secret\z/)");
    assert_eq!(result, Some(Object::Bool(false)));
}
