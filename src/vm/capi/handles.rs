//! The VALUE a C extension holds for each object: a fixnum, one of four
//! special constants, or a handle into a table that keeps the object alive.
//! An object handed to C twice gets the same handle both times, so C code
//! comparing two VALUEs compares the objects.

use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub(crate) type Value = usize;

pub(crate) const QFALSE: Value = 0x00;
pub(crate) const QNIL: Value = 0x08;
pub(crate) const QTRUE: Value = 0x14;
pub(crate) const QUNDEF: Value = 0x24;

/// The first handle, past every special constant.
const FIRST_HANDLE: Value = 0x40;
/// Handles are spaced so their low three bits stay clear, which keeps the
/// fixnum bit off.
const HANDLE_STRIDE: Value = 8;

/// The integers a VALUE carries directly, which are the ones that survive a
/// shift left by one.
const FIXNUM_MAX: i64 = i64::MAX >> 1;
const FIXNUM_MIN: i64 = i64::MIN >> 1;

#[derive(Default)]
struct HandleTable {
    objects: Vec<Object>,
    by_identity: HashMap<usize, Value>,
    by_symbol_name: HashMap<String, Value>,
}

thread_local! {
    static HANDLES: RefCell<HandleTable> = RefCell::new(HandleTable::default());
}

/// The address an object held by reference lives at, which is what tells it
/// apart from every other object of its kind.
fn identity(object: &Object) -> Option<usize> {
    let address = match object {
        Object::BigInt(held) => Rc::as_ptr(held) as *const () as usize,
        Object::String(held) => Rc::as_ptr(held) as *const () as usize,
        Object::Array(held) => Rc::as_ptr(held) as *const () as usize,
        Object::Dict(held) => Rc::as_ptr(held) as *const () as usize,
        Object::Instance(held) => Rc::as_ptr(held) as *const () as usize,
        Object::Class(held) | Object::Module(held) => Rc::as_ptr(held) as *const () as usize,
        Object::Method(held) => Rc::as_ptr(held) as *const () as usize,
        Object::Block(held) => Rc::as_ptr(held) as *const () as usize,
        Object::Exception(held) => Rc::as_ptr(held) as *const () as usize,
        Object::Set(held) => Rc::as_ptr(held) as *const () as usize,
        Object::Binding(held) => Rc::as_ptr(held) as *const () as usize,
        Object::Regex(pattern, _) => Rc::as_ptr(pattern) as *const () as usize,
        Object::Range { mark, .. } => Rc::as_ptr(mark) as usize,
        _ => return None,
    };
    Some(address)
}

/// The VALUE C code is handed for `object`.
pub(crate) fn to_value(object: &Object) -> Value {
    match object {
        Object::Bool(false) => QFALSE,
        Object::Nil => QNIL,
        Object::Bool(true) => QTRUE,
        Object::Int(number) if (FIXNUM_MIN..=FIXNUM_MAX).contains(number) => {
            ((*number as Value) << 1) | 1
        }
        _ => HANDLES.with(|table| {
            let mut table = table.borrow_mut();
            let symbol_name = match object {
                Object::Symbol(name) => Some(name.as_str().to_string()),
                _ => None,
            };
            let known = match (&symbol_name, identity(object)) {
                (Some(name), _) => table.by_symbol_name.get(name).copied(),
                (None, Some(address)) => table.by_identity.get(&address).copied(),
                (None, None) => None,
            };
            if let Some(value) = known {
                return value;
            }
            let value = FIRST_HANDLE + table.objects.len() * HANDLE_STRIDE;
            table.objects.push(object.clone());
            match (symbol_name, identity(object)) {
                (Some(name), _) => {
                    table.by_symbol_name.insert(name, value);
                }
                (None, Some(address)) => {
                    table.by_identity.insert(address, value);
                }
                (None, None) => {}
            }
            value
        }),
    }
}

/// The object a VALUE C code handed back stands for. A VALUE that is no
/// handle metorex gave out reads as nil.
pub(crate) fn to_object(value: Value) -> Object {
    match value {
        QFALSE => Object::Bool(false),
        QNIL | QUNDEF => Object::Nil,
        QTRUE => Object::Bool(true),
        _ if value & 1 == 1 => Object::Int((value as i64) >> 1),
        _ => HANDLES.with(|table| {
            let index = value.wrapping_sub(FIRST_HANDLE) / HANDLE_STRIDE;
            table
                .borrow()
                .objects
                .get(index)
                .cloned()
                .unwrap_or(Object::Nil)
        }),
    }
}

/// The objects the `count` VALUEs C handed over at `values` stand for.
pub(crate) fn objects_from(count: i64, values: *const Value) -> Vec<Object> {
    if count <= 0 {
        return Vec::new();
    }
    // SAFETY: C hands over `count` VALUEs at `values`.
    unsafe { std::slice::from_raw_parts(values, count as usize) }
        .iter()
        .map(|held| to_object(*held))
        .collect()
}

/// Let go of the object behind `value`, which C no longer reaches. The handle
/// is not given out again.
pub(crate) fn release(value: Value) {
    HANDLES.with(|table| {
        let mut table = table.borrow_mut();
        let index = value.wrapping_sub(FIRST_HANDLE) / HANDLE_STRIDE;
        // A handle C is let go of was given out, so its slot is there.
        let released = std::mem::replace(&mut table.objects[index], Object::Nil);
        if let Some(address) = identity(&released) {
            table.by_identity.remove(&address);
        }
    });
}
