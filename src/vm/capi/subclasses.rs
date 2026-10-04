//! Classes from C: making one, its superclass and real class, an instance of
//! it, its instance methods by visibility, the method its superclass holds,
//! and the type tag of any object.

use super::calls::{call, top_level_module};
use super::handles::{QFALSE, QNIL, Value, objects_from, to_object, to_value};
use super::{called_from, interpreter, or_raise, raise, running_method};
use crate::class::Class;
use crate::object::Object;
use std::rc::Rc;

fn type_error(message: &str) -> ! {
    raise(crate::vm::errors::simple_exception(
        "TypeError",
        message,
        called_from(),
    ))
}

/// An anonymous subclass of `superclass`, made without calling `inherited`,
/// as MRI's `rb_class_new` does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_class_new(superclass: Value) -> Value {
    let superclass = match to_object(superclass) {
        Object::Class(class) => class,
        other => type_error(&format!(
            "superclass must be an instance of Class (given an instance of {})",
            super::calls::class_name_of(other)
        )),
    };
    if superclass.is_singleton_class() {
        type_error("can't make subclass of singleton class");
    }
    if Object::Class(Rc::clone(&superclass)) == top_level_module("Class") {
        type_error("can't make subclass of Class");
    }
    let class = Class::new("", Some(Rc::clone(&superclass)));
    superclass.add_subclass(&class);
    to_value(&Object::Class(class))
}

/// The class `klass` stands for once singleton classes are skipped, or 0
/// for 0.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_class_real(klass: Value) -> Value {
    if klass == 0 {
        return 0;
    }
    let mut class = super::exports::class_from(klass);
    while class.is_singleton_class() {
        match class.superclass() {
            Some(above) => class = above,
            None => break,
        }
    }
    to_value(&Object::Class(class))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_class_superclass(klass: Value) -> Value {
    to_value(&call(to_object(klass), "superclass", Vec::new()))
}

/// The superclass of `klass`, or false when it has none.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_class_get_superclass(klass: Value) -> Value {
    match to_object(klass) {
        Object::Class(class) => class
            .superclass()
            .map_or(QFALSE, |above| to_value(&Object::Class(above))),
        _ => QFALSE,
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_class_new_instance_kw(
    count: i32,
    values: *const Value,
    klass: Value,
    keywords: i32,
) -> Value {
    let mut arguments = objects_from(i64::from(count), values);
    if keywords == super::procs::PASS_KEYWORDS {
        super::procs::pass_last_as_keywords(&mut arguments);
    }
    to_value(&call(to_object(klass), "new", arguments))
}

fn instance_methods(name: &str, count: i32, arguments: *const Value, module: Value) -> Value {
    let arguments = objects_from(i64::from(count), arguments);
    to_value(&call(to_object(module), name, arguments))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_class_instance_methods(
    count: i32,
    arguments: *const Value,
    module: Value,
) -> Value {
    instance_methods("instance_methods", count, arguments, module)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_class_public_instance_methods(
    count: i32,
    arguments: *const Value,
    module: Value,
) -> Value {
    instance_methods("public_instance_methods", count, arguments, module)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_class_protected_instance_methods(
    count: i32,
    arguments: *const Value,
    module: Value,
) -> Value {
    instance_methods("protected_instance_methods", count, arguments, module)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_class_private_instance_methods(
    count: i32,
    arguments: *const Value,
    module: Value,
) -> Value {
    instance_methods("private_instance_methods", count, arguments, module)
}

/// The ancestors of the receiver's class, singleton class first when it has
/// one.
fn ancestor_chain(receiver: &Object) -> Vec<Rc<Class>> {
    let machine = interpreter();
    let start = match machine.held_singleton_class(receiver) {
        Some(singleton) => Object::Class(singleton),
        None => call(receiver.clone(), "class", Vec::new()),
    };
    match call(start, "ancestors", Vec::new()) {
        Object::Array(found) => found
            .borrow()
            .iter()
            .filter_map(|held| match held {
                Object::Class(class) | Object::Module(class) => Some(Rc::clone(class)),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Calls the method the C method running now overrides, on the same
/// receiver, with `arguments`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_call_super(count: i32, arguments: *const Value) -> Value {
    let Some(running) = running_method() else {
        raise(crate::vm::errors::simple_exception(
            "RuntimeError",
            "super called outside of method",
            called_from(),
        ))
    };
    let arguments = objects_from(i64::from(count), arguments);
    let chain = ancestor_chain(&running.receiver);
    let after_owner = chain
        .iter()
        .position(|class| Rc::ptr_eq(class, &running.owner))
        .map_or(0, |at| at + 1);
    let next = chain[after_owner..].iter().find_map(|class| {
        class
            .find_own_method(&running.name)
            .map(|method| (Rc::clone(class), method))
    });
    let machine = interpreter();
    let position = called_from();
    let Some((owner, method)) = next.filter(|(_, method)| !method.is_undefined) else {
        let wording = machine.receiver_wording_for(&running.receiver, position);
        raise(crate::vm::errors::simple_exception(
            "NoMethodError",
            &format!(
                "super: no superclass method '{}' for {}",
                running.name, wording
            ),
            position,
        ))
    };
    to_value(&or_raise(machine.invoke_method(
        owner,
        method,
        running.receiver,
        arguments,
        position,
    )))
}

const T_OBJECT: i32 = 0x01;
const T_CLASS: i32 = 0x02;
const T_MODULE: i32 = 0x03;
const T_FLOAT: i32 = 0x04;
const T_STRING: i32 = 0x05;
const T_ARRAY: i32 = 0x07;
const T_HASH: i32 = 0x08;
const T_STRUCT: i32 = 0x09;
const T_BIGNUM: i32 = 0x0a;
const T_DATA: i32 = 0x0c;
const T_FILE: i32 = 0x0b;
const T_NIL: i32 = 0x11;
const T_TRUE: i32 = 0x12;
const T_FALSE: i32 = 0x13;
const T_SYMBOL: i32 = 0x14;
const T_FIXNUM: i32 = 0x15;

/// The type tag of `object`. A Range is a struct in MRI, an object wrapping a
/// C pointer is T_DATA, and an object of a kind with no tag of its own here
/// is T_OBJECT.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_type(object: Value) -> i32 {
    if object & 1 == 1 {
        return T_FIXNUM;
    }
    if object == QNIL {
        return T_NIL;
    }
    let held = to_object(object);
    if super::data::is_data(&held) {
        return T_DATA;
    }
    match held {
        Object::Bool(true) => T_TRUE,
        Object::Bool(false) => T_FALSE,
        Object::Int(_) | Object::BigInt(_) => T_BIGNUM,
        Object::Float(_) => T_FLOAT,
        Object::String(_) => T_STRING,
        Object::Symbol(_) => T_SYMBOL,
        Object::Array(_) => T_ARRAY,
        Object::Dict(_) => T_HASH,
        Object::Class(_) => T_CLASS,
        Object::Module(_) => T_MODULE,
        Object::Range { .. } => T_STRUCT,
        other => instance_type(other),
    }
}

/// The tag of an object of a class Ruby code can define: the tag of the
/// core class an Array, String or Hash subclass descends from, `T_FILE`
/// for an IO, `T_DATA` for a Time, whose state MRI keeps in C, and
/// `T_OBJECT` for the rest.
fn instance_type(object: Object) -> i32 {
    let is_a = |name: &str| {
        super::calls::call(
            object.clone(),
            "is_a?",
            vec![super::calls::top_level_module(name)],
        )
        .is_truthy()
    };
    if is_a("Array") {
        T_ARRAY
    } else if is_a("String") {
        T_STRING
    } else if is_a("Hash") {
        T_HASH
    } else if is_a("IO") {
        T_FILE
    } else if is_a("Time") {
        T_DATA
    } else {
        T_OBJECT
    }
}
