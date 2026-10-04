// Whether an object answers a name, including through
// `respond_to_missing?`.

use super::*;

const RESPOND_GHOST: &str = r#"
class Ghost
  def respond_to_missing?(name, include_private = false)
    return true if name == :publicly_handled
    include_private && name == :privately_handled
  end
end
"#;

#[test]
fn respond_to_answers_a_name_claimed_by_respond_to_missing() {
    let result = run(&format!(
        "{RESPOND_GHOST}\nGhost.new.respond_to?(:publicly_handled)"
    ));
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn respond_to_passes_false_for_the_private_flag_by_default() {
    let result = run(&format!(
        "{RESPOND_GHOST}\nGhost.new.respond_to?(:privately_handled)"
    ));
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn respond_to_passes_the_private_flag_it_was_given() {
    let result = run(&format!(
        "{RESPOND_GHOST}\nGhost.new.respond_to?(:privately_handled, true)"
    ));
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn respond_to_is_false_for_a_name_nothing_claims() {
    let result = run(&format!(
        "{RESPOND_GHOST}\nGhost.new.respond_to?(:not_handled)"
    ));
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn a_class_can_override_respond_to_missing_for_itself() {
    let result = run(r#"
class Registry
  def self.respond_to_missing?(name, include_private = false)
    name == :lookup
  end
end
[Registry.respond_to?(:lookup), Registry.respond_to?(:missing_entirely)].inspect
"#);
    assert_eq!(result, Some(Object::string("[true, false]".to_string())));
}

// ── The default respond_to_missing? ──────────────────────────────────────────

#[test]
fn every_object_answers_respond_to_missing_with_false() {
    let result = run("Object.new.respond_to_missing?(:anything, true)");
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn every_object_responds_to_respond_to_missing() {
    let result = run("Object.new.respond_to?(:respond_to_missing?, true)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn a_class_responds_to_respond_to_missing() {
    let result = run("Object.respond_to?(:respond_to_missing?, true)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn respond_to_missing_is_private_on_kernel() {
    let result = run("Kernel.private_instance_methods(false).include?(:respond_to_missing?)");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── Method#owner answers the module ──────────────────────────────────────────

#[test]
fn a_native_kernel_methods_owner_is_the_kernel_module() {
    let result = run("Kernel.method(:respond_to_missing?).owner == Kernel");
    assert_eq!(result, Some(Object::Bool(true)));
}

// ── respond_to? and native class-method visibility ───────────────────────────

const SEALED: &str = r#"
class Sealed
  class << self
    private :new
  end
end
"#;

#[test]
fn respond_to_is_false_for_a_private_native_class_method() {
    let result = run(&format!("{SEALED}\nSealed.respond_to?(:new)"));
    assert_eq!(result, Some(Object::Bool(false)));
}

#[test]
fn respond_to_with_private_allowed_finds_it() {
    let result = run(&format!("{SEALED}\nSealed.respond_to?(:new, true)"));
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn calling_a_private_native_class_method_raises() {
    let error = run_err(&format!("{SEALED}\nSealed.new"));
    assert!(error.contains("private method 'new' called for class Sealed"));
}

#[test]
fn a_public_class_still_answers_new() {
    let result = run(r#"
class Open
end
Open.respond_to?(:new)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn a_class_responds_to_modules_native_methods() {
    let result = run("Object.respond_to?(:instance_methods)");
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn respond_to_converts_its_argument_with_to_str() {
    let result = run(r#"
class Named
  def to_str
    "upcase"
  end
end
"text".respond_to?(Named.new)
"#);
    assert_eq!(result, Some(Object::Bool(true)));
}

#[test]
fn respond_to_reports_an_uncoercible_argument() {
    let error = run_err("Object.new.respond_to?(42)");
    assert!(error.contains("42 is not a symbol nor a string"));
}
