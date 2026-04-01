// Coverage tests for class.rs Clone and PartialEq implementations

use metorex::class::Class;

// ── Clone impl (lines 109-118) ────────────────────────────────────────────────

#[test]
fn class_clone_produces_equal_name() {
    let class = Class::new("MyClass", None);
    let cloned = class.clone();
    assert_eq!(class.name(), cloned.name());
}

#[test]
fn class_clone_with_superclass() {
    use std::rc::Rc;
    let base = Rc::new(Class::new("Base", None));
    let child = Class::new("Child", Some(Rc::clone(&base)));
    let cloned = child.clone();
    assert_eq!(cloned.name(), "Child");
    assert!(cloned.superclass().is_some());
    assert_eq!(cloned.superclass().unwrap().name(), "Base");
}

#[test]
fn class_clone_with_instance_var() {
    let class = Class::new("WithVar", None);
    class.declare_instance_var("@name");
    let cloned = class.clone();
    assert!(cloned.has_instance_var("@name"));
}

#[test]
fn class_clone_with_class_var() {
    let class = Class::new("WithClassVar", None);
    class.set_class_var("count".to_string(), metorex::object::Object::Int(42));
    let cloned = class.clone();
    assert_eq!(
        cloned.get_class_var("count"),
        Some(metorex::object::Object::Int(42))
    );
}

// ── PartialEq impl (lines 120-160) ────────────────────────────────────────────

#[test]
fn class_eq_same_name_no_vars_equal() {
    let a = Class::new("Foo", None);
    let b = Class::new("Foo", None);
    assert_eq!(a, b);
}

#[test]
fn class_eq_different_names_not_equal() {
    let a = Class::new("Foo", None);
    let b = Class::new("Bar", None);
    assert_ne!(a, b);
}

#[test]
fn class_eq_different_instance_vars_not_equal() {
    let a = Class::new("Foo", None);
    let b = Class::new("Foo", None);
    a.declare_instance_var("@x");
    assert_ne!(a, b);
}

#[test]
fn class_eq_same_instance_vars_equal() {
    let a = Class::new("Foo", None);
    let b = Class::new("Foo", None);
    a.declare_instance_var("@x");
    b.declare_instance_var("@x");
    assert_eq!(a, b);
}

#[test]
fn class_eq_different_class_vars_not_equal() {
    let a = Class::new("Foo", None);
    let b = Class::new("Foo", None);
    a.set_class_var("count".to_string(), metorex::object::Object::Int(1));
    b.set_class_var("count".to_string(), metorex::object::Object::Int(2));
    // Different values → not equal (different class var values)
    assert_ne!(a, b);
}

#[test]
fn class_eq_same_class_vars_equal() {
    let a = Class::new("Foo", None);
    let b = Class::new("Foo", None);
    a.set_class_var("count".to_string(), metorex::object::Object::Int(5));
    b.set_class_var("count".to_string(), metorex::object::Object::Int(5));
    assert_eq!(a, b);
}

#[test]
fn class_eq_different_class_var_count_not_equal() {
    let a = Class::new("Foo", None);
    let b = Class::new("Foo", None);
    a.set_class_var("count".to_string(), metorex::object::Object::Int(1));
    // b has no class vars → different count → not equal
    assert_ne!(a, b);
}

#[test]
fn class_eq_different_superclass_not_equal() {
    use std::rc::Rc;
    let base1 = Rc::new(Class::new("Base1", None));
    let base2 = Rc::new(Class::new("Base2", None));
    let a = Class::new("Child", Some(Rc::clone(&base1)));
    let b = Class::new("Child", Some(Rc::clone(&base2)));
    // Different superclass pointers → not equal
    assert_ne!(a, b);
}

// ── class PartialEq with methods (class.rs lines 143-153) ──────────────────

#[test]
fn class_eq_different_method_count() {
    use metorex::object::Method;
    use std::rc::Rc;
    let a = Class::new("Foo", None);
    let b = Class::new("Foo", None);
    let method = Rc::new(Method::new("bar".to_string(), vec![], vec![]));
    a.define_method("bar", method);
    // a has 1 method, b has 0 → not equal
    assert_ne!(a, b);
}

#[test]
fn class_eq_different_methods() {
    use metorex::object::Method;
    use std::rc::Rc;
    let a = Class::new("Foo", None);
    let b = Class::new("Foo", None);
    let method_a = Rc::new(Method::new("bar".to_string(), vec![], vec![]));
    let method_b = Rc::new(Method::new("baz".to_string(), vec![], vec![]));
    a.define_method("bar", method_a);
    b.define_method("baz", method_b);
    assert_ne!(a, b);
}

#[test]
fn class_eq_same_methods_equal() {
    use metorex::object::Method;
    use std::rc::Rc;
    let a = Class::new("Foo", None);
    let b = Class::new("Foo", None);
    let method_a = Rc::new(Method::new("bar".to_string(), vec![], vec![]));
    let method_b = Rc::new(Method::new("bar".to_string(), vec![], vec![]));
    a.define_method("bar", Rc::clone(&method_a));
    b.define_method("bar", method_b);
    // Same method name + same structure → equal
    assert_eq!(a, b);
}
