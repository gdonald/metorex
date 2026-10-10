// The public methods an object answers.

use super::*;

const VAULT: &str = r#"
class Vault
  def open_door
    :opened
  end

  def hidden
    :hidden
  end
  private :hidden

  def guarded
    :guarded
  end
  protected :guarded

  def self.build
    :built
  end
end
"#;

#[test]
fn public_method_returns_a_bound_method_for_a_public_name() {
    let result = run(&format!(
        "{VAULT}\nVault.new.public_method(:open_door).call.inspect"
    ));
    assert_eq!(result, Some(Object::string(":opened".to_string())));
}

#[test]
fn public_method_reaches_a_class_method() {
    let result = run(&format!(
        "{VAULT}\nVault.public_method(:build).call.inspect"
    ));
    assert_eq!(result, Some(Object::string(":built".to_string())));
}

#[test]
fn public_method_refuses_a_private_name() {
    let error = run_err(&format!("{VAULT}\nVault.new.public_method(:hidden)"));
    assert!(error.contains("method 'hidden' for class 'Vault' is private"));
}

#[test]
fn public_method_refuses_a_protected_name() {
    let error = run_err(&format!("{VAULT}\nVault.new.public_method(:guarded)"));
    assert!(error.contains("method 'guarded' for class 'Vault' is protected"));
}

#[test]
fn method_still_answers_a_private_name() {
    let result = run(&format!("{VAULT}\nVault.new.method(:hidden).call.inspect"));
    assert_eq!(result, Some(Object::string(":hidden".to_string())));
}

const GHOST: &str = r#"
class Ghost
  def respond_to_missing?(name, include_private = false)
    return true if name == :publicly_handled
    include_private && name == :privately_handled
  end

  def method_missing(name, *args)
    "called #{name}"
  end
end
"#;

#[test]
fn public_method_asks_respond_to_missing_without_private() {
    let result = run(&format!(
        "{GHOST}\nGhost.new.public_method(:publicly_handled).call"
    ));
    assert_eq!(
        result,
        Some(Object::string("called publicly_handled".to_string()))
    );
}

#[test]
fn public_method_refuses_a_name_only_claimed_privately() {
    let error = run_err(&format!(
        "{GHOST}\nGhost.new.public_method(:privately_handled)"
    ));
    assert!(error.contains("undefined method 'privately_handled'"));
}

#[test]
fn method_accepts_a_name_claimed_privately() {
    let result = run(&format!(
        "{GHOST}\nGhost.new.method(:privately_handled).call"
    ));
    assert_eq!(
        result,
        Some(Object::string("called privately_handled".to_string()))
    );
}

#[test]
fn a_class_method_named_public_method_wins_over_the_native() {
    let result = run(r#"
class Parent
  def self.public_method
    :its_own
  end
end
Parent.public_method.inspect
"#);
    assert_eq!(result, Some(Object::string(":its_own".to_string())));
}

// ── Object#public_methods ────────────────────────────────────────────────────

const PUBLIC_FIXTURE: &str = r#"
module Helpers
  def mixed_in_open
  end
end

class Parent
  def parent_open
  end

  def parent_shut
  end
  private :parent_shut

  def self.parent_class_open
  end
end

class Child < Parent
  include Helpers

  def child_open
  end

  def child_guarded
  end
  protected :child_guarded

  def self.child_class_open
  end
end
"#;

fn opens(code: &str) -> Option<Object> {
    run(&format!(
        "{PUBLIC_FIXTURE}\n({code}).select {{ |n| n.to_s.include?(\"open\") }}.sort.inspect"
    ))
}

#[test]
fn public_methods_without_ancestors_lists_only_the_objects_own() {
    let result = opens("Child.new.public_methods(false)");
    assert_eq!(result, Some(Object::string("[:child_open]".to_string())));
}

#[test]
fn public_methods_with_ancestors_reaches_the_superclass_and_mixins() {
    let result = opens("Child.new.public_methods");
    assert_eq!(
        result,
        Some(Object::string(
            "[:child_open, :mixed_in_open, :parent_open]".to_string()
        ))
    );
}

#[test]
fn public_methods_treats_nil_like_false() {
    let result = opens("Child.new.public_methods(nil)");
    assert_eq!(result, Some(Object::string("[:child_open]".to_string())));
}

#[test]
fn a_classs_public_methods_are_its_class_methods() {
    let result = opens("Child.public_methods(false)");
    assert_eq!(
        result,
        Some(Object::string(
            "[:child_class_open, :parent_class_open]".to_string()
        ))
    );
}

#[test]
fn public_methods_leaves_out_a_protected_name() {
    let result = run(&format!(
        "{PUBLIC_FIXTURE}\nChild.new.public_methods.include?(:child_guarded)"
    ));
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn public_methods_leaves_out_a_private_name() {
    let result = run(&format!(
        "{PUBLIC_FIXTURE}\nChild.new.public_methods.include?(:parent_shut)"
    ));
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn public_methods_lists_an_immediates_native_methods() {
    let result = run("1.public_methods.include?(:divmod)");
    assert_eq!(result, Some(Object::Bool(true)));
}
