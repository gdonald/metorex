// The methods a class or module answers natively, as against those the
// prelude writes in Ruby.
//
// `dispatch` holds the order the groups below are tried in. Each group is a
// module of its own, and answers `None` for a name that is none of its own.

pub(crate) use crate::class::Class;
pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::Position;
pub(crate) use crate::object::{Method, Object};
pub(crate) use crate::vm::VirtualMachine;
pub(crate) use crate::vm::errors::*;
pub(crate) use crate::vm::native_methods::is_valid_constant_name;
pub(in crate::vm) use crate::vm::utils::{is_truthy, position_to_location};
pub(crate) use std::rc::Rc;

mod aliasing;
mod allocation;
mod arguments;
mod attributes;
mod autoload;
mod class_variables;
mod collections;
mod constant_lookup;
mod constants;
mod dispatch;
mod encoding_settings;
mod encoding_support;
mod fiber_storage;
mod files;
mod hooks;
mod instance_methods;
mod kernel_and_new;
mod method_definition;
mod method_tables;
mod mixins;
mod module_naming;
mod module_queries;
mod pattern_support;
mod primitives;
mod reflection;
mod regexps;
mod threads;

pub(crate) use aliasing::*;
pub(crate) use encoding_support::*;
pub(crate) use method_tables::*;
pub(crate) use pattern_support::*;

/// What one group of class methods made of a call.
pub(crate) enum ClassMethodAnswer {
    /// The group answered the call with this object.
    Answered(Object),
    /// The name is none of the group's own, so the next group is tried.
    Unclaimed,
    /// A method the program defines answers the name, so no group does.
    Deferred,
}

pub(crate) use ClassMethodAnswer::{Answered, Deferred, Unclaimed};

/// The answer a group gives when it hands the call to the whole chain again.
/// Nothing native answering there means the call is left to a method the
/// program defines, so no later group looks at it either.
pub(crate) fn nested_answer(found: Option<Object>) -> ClassMethodAnswer {
    match found {
        Some(result) => Answered(result),
        None => Deferred,
    }
}

/// Whether two objects are the same instance.
fn same_object(one: &Object, other: &Object) -> bool {
    matches!((one, other), (Object::Instance(a), Object::Instance(b)) if Rc::ptr_eq(a, b))
}

/// Whether a class is the named one or descends from it, which is what makes
/// a subclass answer the same native methods.
pub(crate) fn class_named_in_chain(class_rc: &Rc<Class>, wanted: &str) -> bool {
    let mut cursor = Some(Rc::clone(class_rc));
    while let Some(held) = cursor {
        if held.name() == wanted {
            return true;
        }
        cursor = held.superclass();
    }
    false
}

/// Whether a class has no uninitialized form, so `allocate` and `new` have
/// nothing to hand back.
pub(crate) fn lacks_an_allocator(class_rc: &Rc<Class>) -> bool {
    matches!(
        class_rc.name(),
        "TrueClass" | "FalseClass" | "NilClass" | "Symbol"
    )
}
