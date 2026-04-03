use metorex::class::Class;
use metorex::object::{Method, Object};
use std::rc::Rc;

fn make_empty_method(name: &str) -> Rc<Method> {
    Rc::new(Method::new(name.to_string(), Vec::new(), Vec::new()))
}

#[test]
fn class_tracks_name_and_superclass() {
    let base = Rc::new(Class::new("Base", None));
    let child = Class::new("Child", Some(Rc::clone(&base)));

    assert_eq!(child.name(), "Child");
    let child_super = child.superclass().expect("superclass missing");
    assert!(Rc::ptr_eq(&child_super, &base));
}

#[test]
fn class_defines_and_finds_methods() {
    let class = Class::new("Calculator", None);
    let add_method = make_empty_method("add");
    let sub_method = make_empty_method("subtract");

    class.define_method("add", Rc::clone(&add_method));
    class.define_method("subtract", Rc::clone(&sub_method));

    let mut names = class.method_names();
    names.sort();
    assert_eq!(names, vec!["add".to_string(), "subtract".to_string()]);
    assert!(class.has_own_method("add"));
    assert!(class.find_method("add").is_some());
    assert!(class.find_method("multiply").is_none());
}

#[test]
fn class_inheritance_resolves_methods() {
    let base = Rc::new(Class::new("Base", None));
    base.define_method("greet", make_empty_method("greet"));

    let derived = Class::new("Derived", Some(Rc::clone(&base)));

    assert!(derived.find_method("greet").is_some());
    assert!(!derived.has_own_method("greet"));
}

#[test]
fn class_tracks_instance_variables() {
    let class = Class::new("Point", None);
    class.declare_instance_var("x");
    class.declare_instance_var("y");

    let mut vars = class.instance_variables();
    vars.sort();
    assert_eq!(vars, vec!["x".to_string(), "y".to_string()]);
    assert!(class.has_instance_var("x"));
    assert!(!class.has_instance_var("z"));
}

#[test]
fn class_handles_class_variables() {
    let class = Class::new("Counter", None);
    class.set_class_var("count", Object::Int(0));

    assert_eq!(class.get_class_var("count"), Some(Object::Int(0)));
    assert_eq!(class.get_class_var("missing"), None);
}
