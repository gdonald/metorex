//! Yielding from a C method to the block it was called with.

use super::handles::{QUNDEF, Value, objects_from, to_object, to_value};
use super::{called_from, called_with_block, interpreter, or_raise, raise};
use crate::object::Object;

fn yield_objects(arguments: Vec<Object>) -> Value {
    let position = called_from();
    let Some(Object::Block(block)) = called_with_block() else {
        raise(crate::vm::errors::simple_exception(
            "LocalJumpError",
            "no block given (yield)",
            position,
        ))
    };
    let answered = or_raise(interpreter().execute_block_callable(&block, arguments, position));
    super::strings::carry_changes_in();
    to_value(&answered)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_block_given_p() -> i32 {
    matches!(called_with_block(), Some(Object::Block(_))) as i32
}

/// Yields `value`, or nothing when it is `Qundef`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_yield(value: Value) -> Value {
    let arguments = if value == QUNDEF {
        Vec::new()
    } else {
        vec![to_object(value)]
    };
    yield_objects(arguments)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_yield_values2(count: i32, values: *const Value) -> Value {
    yield_objects(objects_from(i64::from(count), values))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_yield_splat(values: Value) -> Value {
    match super::arrays::array_of(to_object(values)) {
        Some(elements) => yield_objects(elements),
        None => raise(crate::vm::errors::simple_exception(
            "ArgumentError",
            "not an array",
            called_from(),
        )),
    }
}
