//! The debug inspector: `rb_debug_inspector_open` handing C the frames of
//! the call stack, innermost first, with the `self`, class and binding of
//! each and the backtrace location it stands at. The C method running now
//! is the first frame, and has no binding.

use super::calls::call;
use super::handles::{QNIL, Value, to_object, to_value};
use super::{called_from, interpreter, raise, running_method};
use crate::object::Object;

/// One frame as the inspector hands it to C.
struct InspectedFrame {
    receiver: Value,
    class: Value,
    binding: Value,
    location: Object,
    /// Whether the frame runs a method written in C, which has no
    /// instruction sequence.
    native: bool,
}

/// What `rb_debug_inspector_t` points to while the inspector is open.
pub(super) struct Inspector {
    frames: Vec<InspectedFrame>,
    locations: Value,
}

/// A Location like `location` naming `label` instead.
fn relabeled(location: &Object, label: &str) -> Object {
    let copy = call(location.clone(), "dup", Vec::new());
    let arguments = vec![Object::symbol("@label"), Object::string(label.to_string())];
    call(copy.clone(), "instance_variable_set", arguments);
    copy
}

/// The class the code `running` frames in from the innermost runs in, or
/// nil where no method is running.
fn frame_class(running: usize) -> Value {
    let machine = interpreter();
    let stack = machine.call_stack();
    let owner = stack
        .len()
        .checked_sub(running + 1)
        .and_then(|index| stack[index].owner_path())
        .map(str::to_string);
    owner.map_or(QNIL, |path| super::modules::class_at_path(&path))
}

fn inspected() -> Inspector {
    let position = called_from();
    let located = interpreter().caller_locations_with_frames(position);
    let mut frames = Vec::with_capacity(located.len() + 1);
    let mut locations = Vec::with_capacity(located.len() + 1);
    if let Some(method) = running_method() {
        let location = relabeled(&located[0].0, &method.name);
        frames.push(InspectedFrame {
            receiver: to_value(&method.receiver),
            class: to_value(&Object::Class(method.owner)),
            binding: QNIL,
            location: location.clone(),
            native: true,
        });
        locations.push(location);
    }
    for (location, running) in located {
        let path = call(location.clone(), "path", Vec::new()).to_string();
        let line = call(location.clone(), "lineno", Vec::new()).to_string();
        let binding = interpreter().frame_binding(running, path, line.parse().unwrap_or(0));
        let receiver = call(binding.clone(), "receiver", Vec::new());
        frames.push(InspectedFrame {
            receiver: to_value(&receiver),
            class: frame_class(running),
            binding: to_value(&binding),
            location: location.clone(),
            native: false,
        });
        locations.push(location);
    }
    Inspector {
        frames,
        locations: to_value(&Object::array(locations)),
    }
}

/// Calls `function` with an inspector over the call stack as it stands, and
/// answers what it answers.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_debug_inspector_open(
    function: extern "C-unwind" fn(*const Inspector, *mut std::ffi::c_void) -> Value,
    data: *mut std::ffi::c_void,
) -> Value {
    let inspector = inspected();
    function(&inspector, data)
}

/// The frame `index` frames in from the innermost, refusing one past the
/// outermost.
fn frame(inspector: *const Inspector, index: i64) -> &'static InspectedFrame {
    // SAFETY: C is handed the inspector only while `rb_debug_inspector_open`
    // keeps it alive, which is the only time it may read it.
    let inspector = unsafe { &*inspector };
    usize::try_from(index)
        .ok()
        .and_then(|index| inspector.frames.get(index))
        .unwrap_or_else(|| {
            raise(crate::vm::errors::simple_exception(
                "ArgumentError",
                "no such frame",
                called_from(),
            ))
        })
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_debug_inspector_frame_self_get(
    inspector: *const Inspector,
    index: i64,
) -> Value {
    frame(inspector, index).receiver
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_debug_inspector_frame_class_get(
    inspector: *const Inspector,
    index: i64,
) -> Value {
    frame(inspector, index).class
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_debug_inspector_frame_binding_get(
    inspector: *const Inspector,
    index: i64,
) -> Value {
    frame(inspector, index).binding
}

/// The instruction sequence the frame runs, or nil for a method written
/// in C.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_debug_inspector_frame_iseq_get(
    inspector: *const Inspector,
    index: i64,
) -> Value {
    let inspected = frame(inspector, index);
    if inspected.native {
        return QNIL;
    }
    let location = inspected.location.clone();
    let sequences = to_object(super::modules::class_at_path("RubyVM::InstructionSequence"));
    to_value(&call(sequences, "__frame__", vec![location]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_debug_inspector_backtrace_locations(
    inspector: *const Inspector,
) -> Value {
    // SAFETY: as for `frame`.
    unsafe { (*inspector).locations }
}
