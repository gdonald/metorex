//! Global variables from C: ones whose value lives in C memory or comes
//! from C getter and setter functions, and reading and writing any global
//! by name.

use super::calls::call;
use super::exports::text;
use super::handles::{Value, to_object, to_value};
use super::{Caller, called_from, enter, interpreter, or_raise};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use std::ffi::c_char;

/// How a C-defined global is read: from the VALUE its data points at, as
/// the data itself, or through a C function given the ID and the data.
#[derive(Debug, Clone, Copy)]
pub(crate) enum GlobalGetter {
    Variable,
    Data,
    Function(usize),
}

/// How a C-defined global is written: into the VALUE its data points at,
/// not at all, or through a C function given the value, the ID and the data.
#[derive(Debug, Clone, Copy)]
pub(crate) enum GlobalSetter {
    Variable,
    ReadOnly,
    Function(usize),
}

/// A global a C extension defined.
#[derive(Debug, Clone, Copy)]
pub(crate) struct HookedGlobal {
    data: usize,
    getter: GlobalGetter,
    setter: GlobalSetter,
}

/// The ID a global's getter and setter are handed, which names it with `$`.
fn global_id(name: &str) -> Value {
    to_value(&Object::symbol(format!("${}", name)))
}

/// A global's name as metorex keeps it, without the `$` C may write.
fn stored_name(name: *const c_char) -> String {
    let written = text(name);
    written.strip_prefix('$').unwrap_or(&written).to_string()
}

fn caller_at(position: Position) -> Caller {
    Caller {
        position,
        block: None,
        keywords_given: false,
        method: None,
    }
}

pub(crate) fn read_hooked_global(
    machine: &mut VirtualMachine,
    name: &str,
    hooked: HookedGlobal,
    position: Position,
) -> Result<Object, MetorexError> {
    let value = match hooked.getter {
        GlobalGetter::Variable if hooked.data == 0 => super::handles::QNIL,
        // SAFETY: C handed over the address of a VALUE it keeps for the life
        // of the program.
        GlobalGetter::Variable => unsafe { *(hooked.data as *const Value) },
        GlobalGetter::Data => hooked.data,
        GlobalGetter::Function(address) => {
            let id = global_id(name);
            // SAFETY: `rb_define_hooked_variable` was handed a getter taking
            // the ID and the data.
            let getter: extern "C-unwind" fn(Value, usize) -> Value =
                unsafe { std::mem::transmute(address) };
            enter(machine, caller_at(position), || getter(id, hooked.data))?
        }
    };
    Ok(to_object(value))
}

pub(crate) fn write_hooked_global(
    machine: &mut VirtualMachine,
    name: &str,
    hooked: HookedGlobal,
    value: Object,
    position: Position,
) -> Result<(), MetorexError> {
    match hooked.setter {
        GlobalSetter::Variable if hooked.data == 0 => Ok(()),
        GlobalSetter::Variable => {
            // SAFETY: as for the getter.
            unsafe { *(hooked.data as *mut Value) = to_value(&value) };
            Ok(())
        }
        GlobalSetter::ReadOnly => Err(crate::vm::errors::simple_exception(
            "NameError",
            &format!("${} is a read-only variable", name),
            position,
        )),
        GlobalSetter::Function(address) => {
            let id = global_id(name);
            let value = to_value(&value);
            // SAFETY: `rb_define_hooked_variable` was handed a setter taking
            // the value, the ID and the data.
            let setter: extern "C-unwind" fn(Value, Value, usize) =
                unsafe { std::mem::transmute(address) };
            enter(machine, caller_at(position), || {
                setter(value, id, hooked.data)
            })
        }
    }
}

fn define(name: *const c_char, data: usize, getter: GlobalGetter, setter: GlobalSetter) {
    let name = stored_name(name);
    let machine = interpreter();
    // Holding the name in the global table is what lists it among the
    // program's globals. Reads and writes go through the hooks.
    machine
        .globals_mut()
        .set_variable(name.clone(), Object::Nil);
    machine.hooked_globals.insert(
        name,
        HookedGlobal {
            data,
            getter,
            setter,
        },
    );
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_hooked_variable(
    name: *const c_char,
    variable: *mut Value,
    getter: usize,
    setter: usize,
) {
    let getter = match getter {
        0 => GlobalGetter::Variable,
        address => GlobalGetter::Function(address),
    };
    let setter = match setter {
        0 => GlobalSetter::Variable,
        address => GlobalSetter::Function(address),
    };
    define(name, variable as usize, getter, setter);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_variable(name: *const c_char, variable: *mut Value) {
    define(
        name,
        variable as usize,
        GlobalGetter::Variable,
        GlobalSetter::Variable,
    );
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_readonly_variable(name: *const c_char, variable: *const Value) {
    define(
        name,
        variable as usize,
        GlobalGetter::Variable,
        GlobalSetter::ReadOnly,
    );
}

/// A global with no storage of its own. Without a getter it reads as the
/// data it has none of, which is false, and without a setter it is
/// read-only.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_virtual_variable(
    name: *const c_char,
    getter: usize,
    setter: usize,
) {
    let getter = match getter {
        0 => GlobalGetter::Data,
        address => GlobalGetter::Function(address),
    };
    let setter = match setter {
        0 => GlobalSetter::ReadOnly,
        address => GlobalSetter::Function(address),
    };
    define(name, 0, getter, setter);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gv_get(name: *const c_char) -> Value {
    let name = stored_name(name);
    to_value(&or_raise(
        interpreter().read_global_variable(&name, called_from()),
    ))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_gv_set(name: *const c_char, value: Value) -> Value {
    let name = stored_name(name);
    or_raise(interpreter().write_global_variable(&name, to_object(value), called_from()));
    value
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_f_global_variables() -> Value {
    to_value(&or_raise(interpreter().call_native_function(
        "global_variables",
        Vec::new(),
        called_from(),
    )))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_lastline_get() -> Value {
    to_value(&or_raise(
        interpreter().read_global_variable("_", called_from()),
    ))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_lastline_set(line: Value) {
    or_raise(interpreter().write_global_variable("_", to_object(line), called_from()));
}

thread_local! {
    static DEFAULT_SEPARATOR: std::cell::OnceCell<Value> = const { std::cell::OnceCell::new() };
}

/// `rb_default_rs`: the record separator Ruby starts with, a frozen "\n".
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_metorex_default_rs() -> Value {
    DEFAULT_SEPARATOR.with(|held| {
        *held.get_or_init(|| {
            let separator = Object::string("\n");
            call(separator.clone(), "freeze", Vec::new());
            to_value(&separator)
        })
    })
}
