//! TracePoints C makes with `rb_tracepoint_new`, whose hook is a C function,
//! and enabling, disabling and asking about any TracePoint.

use super::calls::{call, top_level_module};
use super::handles::{QNIL, Value, to_object, to_value};
use super::methods::{CFunction, TRACEPOINT_HOOK_ARITY};
use crate::object::{Method, Object};
use std::rc::Rc;

/// The `RUBY_EVENT_*` bit for each event a TracePoint can be made for.
const EVENTS: [(u32, &str); 15] = [
    (0x0001, "line"),
    (0x0002, "class"),
    (0x0004, "end"),
    (0x0008, "call"),
    (0x0010, "return"),
    (0x0020, "c_call"),
    (0x0040, "c_return"),
    (0x0080, "raise"),
    (0x0100, "b_call"),
    (0x0200, "b_return"),
    (0x0400, "thread_begin"),
    (0x0800, "thread_end"),
    (0x1000, "fiber_switch"),
    (0x2000, "script_compiled"),
    (0x4000, "rescue"),
];

/// Makes a TracePoint for the events `flags` names whose hook calls
/// `function` with the TracePoint and `data`. MRI ignores the target
/// thread, and so does this.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_tracepoint_new(
    _target_thread: Value,
    flags: u32,
    function: *const (),
    data: usize,
) -> Value {
    let events: Vec<Object> = EVENTS
        .iter()
        .filter(|(bit, _)| flags & bit != 0)
        .map(|(_, name)| Object::symbol(*name))
        .collect();
    let trace = call(top_level_module("TracePoint"), "allocate", Vec::new());
    let mut hook = Method::new("call".to_string(), Vec::new(), Vec::new());
    hook.bound_self = Some(Box::new(trace.clone()));
    hook.c_function = Some(CFunction {
        address: function as usize,
        arity: TRACEPOINT_HOOK_ARITY,
        data,
    });
    let hook = Object::Method(Rc::new(hook));
    let events = Object::array(events);
    call(
        trace.clone(),
        "__send__",
        vec![Object::symbol("__set_up__"), events.clone(), hook],
    );
    // A TracePoint made for no events traces none, where `TracePoint.new`
    // with none traces every one.
    call(
        trace.clone(),
        "instance_variable_set",
        vec![Object::symbol("@events"), events],
    );
    to_value(&trace)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_tracepoint_enable(trace: Value) -> Value {
    call(to_object(trace), "enable", Vec::new());
    QNIL
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_tracepoint_disable(trace: Value) -> Value {
    call(to_object(trace), "disable", Vec::new());
    QNIL
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_tracepoint_enabled_p(trace: Value) -> Value {
    to_value(&call(to_object(trace), "enabled?", Vec::new()))
}
