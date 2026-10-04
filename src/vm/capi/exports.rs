//! The `rb_*` functions the binary exports for a C extension to call.

use super::handles::{Value, to_object, to_value};
use super::methods::{CFunction, MAX_FIXED_ARITY};
use super::{interpreter, or_raise, raise};
use crate::class::Class;
use crate::object::{Method, Object};
use crate::vm::VirtualMachine;
use std::ffi::{CStr, c_char};
use std::rc::Rc;

/// Text C code handed over as a NUL-terminated string.
pub(super) fn text(pointer: *const c_char) -> String {
    // SAFETY: the headers declare these parameters as C strings.
    unsafe { CStr::from_ptr(pointer) }
        .to_string_lossy()
        .into_owned()
}

/// The class or module a VALUE C code handed over stands for.
pub(super) fn class_from(value: Value) -> Rc<Class> {
    match to_object(value) {
        Object::Class(class) | Object::Module(class) => class,
        other => raise(crate::vm::errors::simple_exception(
            "TypeError",
            &format!("{} is not a class/module", other),
            super::called_from(),
        )),
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_class_of(object: Value) -> Value {
    if super::string_functions::is_hidden(object) {
        return super::handles::QFALSE;
    }
    let machine = interpreter();
    let object = to_object(object);
    // Every class and module has a singleton class from the start in Ruby,
    // where other objects get one only when something asks for it.
    let singleton = match &object {
        Object::Class(_) | Object::Module(_) => Some(machine.singleton_class_of(&object)),
        _ => machine.held_singleton_class(&object),
    };
    let class = match singleton {
        Some(singleton) => Object::Class(singleton),
        None => or_raise(machine.send_to_object(object, "class", Vec::new(), super::called_from())),
    };
    to_value(&class)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_method(
    class: Value,
    name: *const c_char,
    function: *const (),
    arity: i32,
) {
    let class = class_from(class);
    let name = text(name);
    if !(-2..=MAX_FIXED_ARITY).contains(&arity) {
        raise(crate::vm::errors::simple_exception(
            "ArgumentError",
            &format!("arity out of range: {} for -2..{}", arity, MAX_FIXED_ARITY),
            super::called_from(),
        ));
    }
    let mut method = Method::new(name.clone(), Vec::new(), Vec::new());
    method.owner = Some(class.name().to_string());
    method.owner_class = Some(Rc::clone(&class));
    method.c_function = Some(CFunction {
        address: function as usize,
        arity,
        data: 0,
    });
    class.define_method(name, Rc::new(method));
}

impl VirtualMachine {
    /// The singleton class `object` already has, without making one.
    pub(super) fn held_singleton_class(&self, object: &Object) -> Option<Rc<Class>> {
        match object {
            Object::Instance(instance) => instance.borrow().singleton_class.borrow().clone(),
            other => self.existing_singleton_class(other),
        }
    }
}
