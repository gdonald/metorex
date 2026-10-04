//! The `struct RBasic` C reads an object's flags and class through with
//! `RBASIC`. Metorex keeps neither in a struct of that shape, so each object
//! C asks about is given one, filled from the object when C asks for it.
//! What C writes into its flags is kept, and the FREEZE bit of it is carried
//! back into the object whenever control leaves C.

use super::exports::rb_class_of;
use super::handles::{Value, to_object};
use super::{interpreter, running_interpreter};
use std::cell::RefCell;
use std::collections::HashMap;

/// `RUBY_FL_FREEZE` in the headers.
const FREEZE_FLAG: Value = 1 << 11;

#[repr(C)]
pub(super) struct RBasic {
    flags: Value,
    klass: Value,
}

/// The struct handed to C for one object, and the flags it held when C was
/// last handed it, which tell a write of C's apart.
struct Handed {
    basic: Box<RBasic>,
    flags: Value,
}

thread_local! {
    static HANDED: RefCell<HashMap<Value, Handed>> = RefCell::new(HashMap::new());
}

/// Freezes or thaws each object whose FREEZE flag C changed.
pub(super) fn carry_flag_writes_back() {
    let changed: Vec<(Value, bool)> = HANDED.with(|handed| {
        handed
            .borrow_mut()
            .iter_mut()
            .filter(|(_, held)| held.basic.flags != held.flags)
            .map(|(value, held)| {
                held.flags = held.basic.flags;
                (*value, held.flags & FREEZE_FLAG != 0)
            })
            .collect()
    });
    for (value, frozen) in changed {
        let object = to_object(value);
        let machine = running_interpreter();
        if frozen == machine.object_is_frozen(&object) {
            continue;
        }
        if frozen {
            machine.freeze_object(&object);
        } else {
            machine.thaw_object(&object);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_rbasic(object: Value) -> *mut RBasic {
    let machine = interpreter();
    let frozen = machine.object_is_frozen(&to_object(object));
    let klass = rb_class_of(object);
    HANDED.with(|handed| {
        let mut handed = handed.borrow_mut();
        let held = handed.entry(object).or_insert_with(|| Handed {
            basic: Box::new(RBasic { flags: 0, klass }),
            flags: 0,
        });
        let kept = held.basic.flags & !FREEZE_FLAG;
        held.basic.flags = kept | if frozen { FREEZE_FLAG } else { 0 };
        held.basic.klass = klass;
        held.flags = held.basic.flags;
        &mut *held.basic as *mut RBasic
    })
}
