//! Enumerators C builds with `rb_enumeratorize`, and the size function
//! `rb_enumeratorize_with_size` names, which answers `Enumerator#size`.

use super::handles::{Value, objects_from, to_object, to_value};
use super::methods::{CFunction, ENUMERATOR_SIZE_ARITY};
use super::symbols::symbol_name;
use super::{interpreter, or_raise};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Method, Object};
use crate::vm::VirtualMachine;
use std::rc::Rc;

fn enumeratorize(
    receiver: Value,
    method_name: Value,
    count: i32,
    arguments: *const Value,
    size_function: *const (),
) -> Value {
    let arguments = objects_from(i64::from(count), arguments);
    let machine = interpreter();
    let enumerator = or_raise(machine.build_enumerator(
        to_object(receiver),
        &symbol_name(method_name),
        arguments,
        None,
        super::called_from(),
    ));
    if !size_function.is_null() {
        let mut sizing = Method::new("size".to_string(), Vec::new(), Vec::new());
        sizing.bound_self = Some(Box::new(enumerator.clone()));
        sizing.c_function = Some(CFunction {
            address: size_function as usize,
            arity: ENUMERATOR_SIZE_ARITY,
            data: 0,
        });
        let arguments = vec![Object::symbol("@size"), Object::Method(Rc::new(sizing))];
        or_raise(machine.send_to_object(
            enumerator.clone(),
            "instance_variable_set",
            arguments,
            super::called_from(),
        ));
    }
    to_value(&enumerator)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enumeratorize(
    receiver: Value,
    method_name: Value,
    count: i32,
    arguments: *const Value,
) -> Value {
    enumeratorize(receiver, method_name, count, arguments, std::ptr::null())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_enumeratorize_with_size(
    receiver: Value,
    method_name: Value,
    count: i32,
    arguments: *const Value,
    size_function: *const (),
) -> Value {
    enumeratorize(receiver, method_name, count, arguments, size_function)
}

impl VirtualMachine {
    /// What a size function is called with: the object the enumerator walks,
    /// the arguments it walks it with, and the enumerator itself.
    pub(super) fn size_function_arguments(
        &mut self,
        enumerator: Object,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let mut read = |name: &str| {
            self.send_to_object(
                enumerator.clone(),
                "instance_variable_get",
                vec![Object::symbol(name)],
                position,
            )
        };
        let receiver = read("@receiver")?;
        let arguments = read("@arguments")?;
        Ok(vec![receiver, arguments, enumerator])
    }
}
