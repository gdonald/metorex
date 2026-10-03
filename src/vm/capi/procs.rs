//! Procs from C: making one around a C block function, asking its arity,
//! calling one with arguments, keywords and a block, and telling one apart.

use super::calls::{call, class_name_of, top_level_module};
use super::fibers::block_function_method;
use super::handles::{QFALSE, QNIL, QTRUE, Value, objects_from, to_object, to_value};
use super::{called_from, interpreter, raise};
use crate::object::Object;
use crate::vm::program::proc_for_c_function;

/// What `RB_PASS_KEYWORDS` asks for: the last argument is the keyword Hash.
pub(super) const PASS_KEYWORDS: i32 = 1;

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_proc_new(function: *const (), data: Value) -> Value {
    to_value(&proc_for_c_function(block_function_method(function, data)))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_proc_arity(procedure: Value) -> i32 {
    match call(to_object(procedure), "arity", Vec::new()) {
        Object::Int(arity) => arity as i32,
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_is_proc(value: Value) -> Value {
    let object = to_object(value);
    let is_proc = matches!(object, Object::Block(_))
        || call(object, "is_a?", vec![top_level_module("Proc")]).is_truthy();
    if is_proc { QTRUE } else { QFALSE }
}

/// Marks the last argument as the keyword Hash, refusing one that is not a
/// Hash.
pub(super) fn pass_last_as_keywords(arguments: &mut [Object]) {
    let Some(last) = arguments.last_mut() else {
        return;
    };
    let Object::Dict(pairs) = &*last else {
        raise(crate::vm::errors::simple_exception(
            "TypeError",
            &format!(
                "no implicit conversion of {} into Hash",
                interpreter().conversion_name(last)
            ),
            called_from(),
        ))
    };
    let mut marked = pairs.borrow().clone();
    marked.insert("__MX_KWARGS__".to_string(), Object::Bool(true));
    *last = Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(marked)));
}

/// Calls the Proc with `arguments`, the last as keywords when `keywords`
/// says so, and with `block` unless it is nil.
fn call_proc(procedure: Value, mut arguments: Vec<Object>, keywords: i32, block: Value) -> Value {
    if keywords == PASS_KEYWORDS {
        pass_last_as_keywords(&mut arguments);
    }
    if block != QNIL {
        interpreter().pending_block = Some(to_object(block));
    }
    to_value(&call(to_object(procedure), "call", arguments))
}

/// The elements of the Array `values` as arguments.
fn array_arguments(values: Value) -> Vec<Object> {
    match to_object(values) {
        Object::Array(elements) => elements.borrow().clone(),
        other => raise(crate::vm::errors::simple_exception(
            "TypeError",
            &format!(
                "wrong argument type {} (expected Array)",
                class_name_of(other)
            ),
            called_from(),
        )),
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_proc_call(procedure: Value, values: Value) -> Value {
    call_proc(procedure, array_arguments(values), 0, QNIL)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_proc_call_kw(procedure: Value, values: Value, keywords: i32) -> Value {
    call_proc(procedure, array_arguments(values), keywords, QNIL)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_proc_call_with_block(
    procedure: Value,
    count: i32,
    values: *const Value,
    block: Value,
) -> Value {
    call_proc(procedure, objects_from(i64::from(count), values), 0, block)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_proc_call_with_block_kw(
    procedure: Value,
    count: i32,
    values: *const Value,
    block: Value,
    keywords: i32,
) -> Value {
    call_proc(
        procedure,
        objects_from(i64::from(count), values),
        keywords,
        block,
    )
}
