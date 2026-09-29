// The registry of objects still alive, which ObjectSpace.each_object walks.

use metorex::object::live::{live_classes, live_instances};
use metorex::object::{Class, Instance};
use std::rc::Rc;

#[test]
fn test_live_instances_lists_an_instance_while_it_is_held() {
    let class = Class::new("Widget", None);
    let kept = Instance::new(Rc::clone(&class));

    assert!(
        live_instances()
            .iter()
            .any(|found| Rc::ptr_eq(found, &kept))
    );
}

#[test]
fn test_live_instances_leaves_out_an_instance_once_it_is_dropped() {
    let class = Class::new("Widget", None);
    let dropped = Rc::downgrade(&Instance::new(Rc::clone(&class)));

    assert!(
        !live_instances()
            .iter()
            .any(|found| std::ptr::eq(Rc::as_ptr(found), dropped.as_ptr()))
    );
}

#[test]
fn test_live_instances_keeps_the_held_ones_after_clearing_out_the_dead() {
    let class = Class::new("Widget", None);
    let kept = Instance::new(Rc::clone(&class));
    for _ in 0..5000 {
        Instance::new(Rc::clone(&class));
    }

    let found = live_instances();

    assert_eq!(found.len(), 1);
    assert!(Rc::ptr_eq(&found[0], &kept));
}

#[test]
fn test_live_classes_lists_a_module_while_it_is_held() {
    let module = Class::new_module("Helpers");

    assert!(
        live_classes()
            .iter()
            .any(|found| Rc::ptr_eq(found, &module))
    );
}
