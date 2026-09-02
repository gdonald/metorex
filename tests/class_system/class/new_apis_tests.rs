// Coverage tests for the class.rs APIs added in the alias_method_spec work:
//   - freeze / is_frozen
//   - add_subclass / subclasses (with weak-ref GC)
//   - set_method_public + has_public_override
//   - current_visibility / set_current_visibility
//   - alias_method visibility preservation
//   - duplicate / inspect_name / set_assigned_name_if_anonymous

use metorex::class::Class;
use metorex::object::{Method, Object};
use std::rc::Rc;

// ── freeze / is_frozen ────────────────────────────────────────────────────

#[test]
fn class_is_not_frozen_by_default() {
    let c = Class::new("Foo", None);
    assert!(!c.is_frozen());
}

#[test]
fn class_freeze_sets_frozen_flag() {
    let c = Class::new("Foo", None);
    c.freeze();
    assert!(c.is_frozen());
}

#[test]
fn class_freeze_is_idempotent() {
    let c = Class::new("Foo", None);
    c.freeze();
    c.freeze();
    assert!(c.is_frozen());
}

// ── subclasses tracking ──────────────────────────────────────────────────

#[test]
fn new_class_has_no_subclasses() {
    let c = Class::new("Base", None);
    assert!(c.subclasses().is_empty());
}

#[test]
fn add_subclass_registers_child() {
    let base = Rc::new(Class::new("Base", None));
    let child = Rc::new(Class::new("Child", Some(Rc::clone(&base))));
    base.add_subclass(&child);
    let subs = base.subclasses();
    assert_eq!(subs.len(), 1);
    assert_eq!(subs[0].name(), "Child");
}

#[test]
fn add_subclass_multiple_children() {
    let base = Rc::new(Class::new("Base", None));
    // Keep the children alive — subclasses are stored as Weak refs.
    let _keepalive: Vec<Rc<Class>> = ["A", "B", "C"]
        .iter()
        .map(|n| {
            let child = Rc::new(Class::new(*n, Some(Rc::clone(&base))));
            base.add_subclass(&child);
            child
        })
        .collect();
    assert_eq!(base.subclasses().len(), 3);
}

#[test]
fn subclasses_drops_collected_children() {
    let base = Rc::new(Class::new("Base", None));
    {
        let child = Rc::new(Class::new("Transient", Some(Rc::clone(&base))));
        base.add_subclass(&child);
        assert_eq!(base.subclasses().len(), 1);
    }
    // child Rc dropped; weak ref should no longer upgrade.
    assert_eq!(base.subclasses().len(), 0);
}

// ── visibility ──────────────────────────────────────────────────────────

#[test]
fn set_method_private_marks_private() {
    let c = Class::new("Foo", None);
    c.set_method_private("secret");
    assert!(c.is_method_private("secret"));
}

#[test]
fn set_method_public_removes_private_flag_and_records_override() {
    let c = Class::new("Foo", None);
    c.set_method_private("secret");
    c.set_method_public("secret");
    assert!(!c.is_method_private("secret"));
    assert!(c.has_public_override("secret"));
}

#[test]
fn has_public_override_false_by_default() {
    let c = Class::new("Foo", None);
    assert!(!c.has_public_override("foo"));
}

#[test]
fn alias_preserves_private_visibility() {
    let c = Rc::new(Class::new("Foo", None));
    let m = Rc::new(Method::new("secret".to_string(), vec![], vec![]));
    c.define_method("secret", m);
    c.set_method_private("secret");
    assert!(c.alias_method("secret_alias", "secret"));
    // The alias should also be private on `c`.
    assert!(c.is_method_private("secret_alias"));
}

#[test]
fn alias_preserves_private_from_superclass_chain() {
    let base = Rc::new(Class::new("Base", None));
    let m = Rc::new(Method::new("secret".to_string(), vec![], vec![]));
    base.define_method("secret", Rc::clone(&m));
    base.set_method_private("secret");

    let child = Rc::new(Class::new("Child", Some(Rc::clone(&base))));
    assert!(child.alias_method("secret_alias", "secret"));
    assert!(child.is_method_private("secret_alias"));
}

#[test]
fn alias_does_not_copy_private_when_public_override_shadows() {
    let base = Rc::new(Class::new("Base", None));
    let m = Rc::new(Method::new("secret".to_string(), vec![], vec![]));
    base.define_method("secret", m);
    base.set_method_private("secret");

    let child = Rc::new(Class::new("Child", Some(Rc::clone(&base))));
    child.set_method_public("secret");
    assert!(child.alias_method("secret_alias", "secret"));
    assert!(!child.is_method_private("secret_alias"));
}

#[test]
fn alias_method_returns_false_when_source_missing() {
    let c = Rc::new(Class::new("Foo", None));
    assert!(!c.alias_method("new", "missing"));
}

// ── current_visibility ─────────────────────────────────────────────────

#[test]
fn current_visibility_defaults_to_public() {
    let c = Class::new("Foo", None);
    assert_eq!(c.current_visibility(), "public");
}

#[test]
fn set_current_visibility_updates_state() {
    let c = Class::new("Foo", None);
    c.set_current_visibility("private");
    assert_eq!(c.current_visibility(), "private");
    c.set_current_visibility("public");
    assert_eq!(c.current_visibility(), "public");
}

// ── inspect_name / set_assigned_name_if_anonymous ──────────────────────

#[test]
fn inspect_name_uses_ruby_name_when_present() {
    let c = Class::new("Foo", None);
    assert_eq!(c.inspect_name(), "Foo");
}

#[test]
fn inspect_name_falls_back_to_hex_label_for_anonymous() {
    let c = Class::new("", None);
    let label = c.inspect_name();
    assert!(label.starts_with("#<Class:0x"));
    assert!(label.ends_with('>'));
}

#[test]
fn set_assigned_name_if_anonymous_sets_assigned_only_when_blank() {
    let c = Class::new("", None);
    c.set_assigned_name_if_anonymous("Named");
    assert_eq!(c.ruby_name(), "Named");
}

#[test]
fn set_assigned_name_if_anonymous_is_noop_when_class_has_name() {
    let c = Class::new("Existing", None);
    c.set_assigned_name_if_anonymous("Other");
    assert_eq!(c.ruby_name(), "Existing");
}

#[test]
fn set_assigned_name_if_anonymous_is_noop_when_already_assigned() {
    let c = Class::new("", None);
    c.set_assigned_name_if_anonymous("First");
    c.set_assigned_name_if_anonymous("Second");
    assert_eq!(c.ruby_name(), "First");
}

// ── duplicate (also copies new fields) ─────────────────────────────────

#[test]
fn duplicate_clears_frozen_and_subclasses_but_keeps_methods() {
    let src = Rc::new(Class::new("Src", None));
    src.define_method("m", Rc::new(Method::new("m".to_string(), vec![], vec![])));
    src.freeze();
    let sc = Rc::new(Class::new("Sub", Some(Rc::clone(&src))));
    src.add_subclass(&sc);

    let dup = Class::duplicate(&src);
    assert!(!dup.is_frozen(), "dup should reset frozen flag");
    assert!(
        dup.subclasses().is_empty(),
        "dup should start with no subclasses"
    );
    assert!(dup.find_method("m").is_some(), "methods copy over");
    assert_eq!(dup.current_visibility(), "public");
}

// ── public_overrides carries through Clone ─────────────────────────────

#[test]
fn clone_preserves_public_overrides() {
    let c = Class::new("Foo", None);
    c.set_method_public("x");
    let cloned = c.clone();
    assert!(cloned.has_public_override("x"));
}

// ── set_class_var / remove_class_var interplay ─────────────────────────
// (covered elsewhere but included here for a sanity link with subclass bookkeeping)

#[test]
fn add_subclass_does_not_affect_class_vars() {
    let base = Rc::new(Class::new("Base", None));
    base.set_class_var("X", Object::Int(1));
    let child = Rc::new(Class::new("Child", Some(Rc::clone(&base))));
    base.add_subclass(&child);
    assert_eq!(base.get_class_var("X"), Some(Object::Int(1)));
}

// ── duplicate copies singleton class (lines 364-388 in class.rs) ────────

#[test]
fn duplicate_copies_singleton_class_methods() {
    let src = Rc::new(Class::new("SrcS", None));
    let sc = Rc::new(Class::new("#<Class:SrcS>", None));
    sc.define_method("sm", Rc::new(Method::new("sm".to_string(), vec![], vec![])));
    src.set_singleton_class(Rc::clone(&sc));

    let dup = Class::duplicate(&src);
    let dup_sc_slot = dup.singleton_class_slot();
    let dup_sc = dup_sc_slot.as_ref().expect("singleton class should copy");
    assert!(
        dup_sc.find_method("sm").is_some(),
        "singleton-class methods must be copied"
    );
    assert!(!dup_sc.is_frozen());
    assert_eq!(dup_sc.current_visibility(), "public");
}

#[test]
fn duplicate_filters_attached_class_var_from_singleton() {
    let src = Rc::new(Class::new("SrcA", None));
    let sc = Rc::new(Class::new("#<Class:SrcA>", None));
    sc.set_class_var("__attached__", Object::Int(42));
    sc.set_class_var("Regular", Object::Int(7));
    src.set_singleton_class(Rc::clone(&sc));

    let dup = Class::duplicate(&src);
    let dup_sc_slot = dup.singleton_class_slot();
    let dup_sc = dup_sc_slot.as_ref().expect("singleton class should copy");
    assert!(
        dup_sc.get_class_var("__attached__").is_none(),
        "__attached__ must be filtered"
    );
    assert_eq!(dup_sc.get_class_var("Regular"), Some(Object::Int(7)));
}

// ── is_method_private_in_chain via alias_method (lines 292-296 in class.rs) ─

#[test]
fn alias_method_honors_mixin_public_override() {
    // Mixin has a method `m` marked private AND explicitly re-publicized.
    // The public_override on the mixin must short-circuit the chain walk so
    // the alias is NOT marked private on the host.
    let mixin = Rc::new(Class::new("MixPub", None));
    mixin.define_method("m", Rc::new(Method::new("m".to_string(), vec![], vec![])));
    mixin.set_method_private("m");
    mixin.set_method_public("m");

    let host = Rc::new(Class::new("HostPub", None));
    host.add_mixin(Rc::clone(&mixin));
    assert!(host.alias_method("m_alias", "m"));
    assert!(
        !host.is_method_private("m_alias"),
        "public_override on mixin should prevent the alias from being private"
    );
}

#[test]
fn alias_method_inherits_mixin_private_flag() {
    // Mixin has `hidden` marked private without a public_override.
    // alias_method on the host should pick up the private flag via the mixin
    // branch of is_method_private_in_chain.
    let mixin = Rc::new(Class::new("MixPriv", None));
    mixin.define_method(
        "hidden",
        Rc::new(Method::new("hidden".to_string(), vec![], vec![])),
    );
    mixin.set_method_private("hidden");

    let host = Rc::new(Class::new("HostPriv", None));
    host.add_mixin(Rc::clone(&mixin));
    assert!(host.alias_method("hidden_alias", "hidden"));
    assert!(
        host.is_method_private("hidden_alias"),
        "private flag on mixin must propagate to alias via chain walk"
    );
}
