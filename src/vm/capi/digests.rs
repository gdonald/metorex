//! Digest plugins: a subclass of Digest::Base that names, in its "metadata"
//! instance variable, the C functions computing its digest, which
//! Digest::Base runs on a context it keeps for each instance.

use super::calls::{call, kernel_function, top_level_module};
use super::handles::{Value, to_object, to_value};
use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;

/// The `rb_digest_metadata_t` a plugin hands over.
#[repr(C)]
struct Metadata {
    api_version: i32,
    digest_length: usize,
    block_length: usize,
    context_size: usize,
    init: extern "C-unwind" fn(*mut c_void) -> i32,
    update: extern "C-unwind" fn(*mut c_void, *const u8, usize),
    finish: extern "C-unwind" fn(*mut c_void, *mut u8) -> i32,
}

thread_local! {
    /// The context each digest instance computes in.
    static CONTEXTS: RefCell<HashMap<Value, Vec<u8>>> = RefCell::new(HashMap::new());
}

/// The metadata VALUE the class of `digest`, or a class above it, names,
/// or nil.
fn named_metadata(digest: Value) -> Value {
    let name = to_value(&Object::symbol("metadata"));
    let classes = super::arrays::array_of(call(
        call(to_object(digest), "class", Vec::new()),
        "ancestors",
        Vec::new(),
    ))
    .unwrap_or_default();
    classes
        .iter()
        .map(|class| super::objects::rb_ivar_get(to_value(class), name))
        .find(|held| *held != super::handles::QNIL)
        .unwrap_or(super::handles::QNIL)
}

/// The metadata the class of `digest`, or a class above it, names.
fn metadata_of(digest: Value) -> &'static Metadata {
    let found = named_metadata(digest);
    // SAFETY: the metadata a plugin names lives as long as the program, and
    // `__plugin__?` answered true before any of the methods reading it ran.
    unsafe { &*(super::data::wrapped_pointer(found) as *const Metadata) }
}

/// Whether the class of `digest` names a plugin.
extern "C-unwind" fn plugin_named(digest: Value) -> Value {
    if named_metadata(digest) == super::handles::QNIL {
        super::handles::QFALSE
    } else {
        super::handles::QTRUE
    }
}

/// Runs `with` on the context of `digest`, starting one when it has none.
fn with_context<T>(digest: Value, with: impl FnOnce(&Metadata, *mut c_void) -> T) -> T {
    let metadata = metadata_of(digest);
    if CONTEXTS.with(|held| !held.borrow().contains_key(&digest)) {
        plugin_reset(digest);
    }
    let context = CONTEXTS.with(|held| {
        held.borrow_mut()
            .get_mut(&digest)
            .map_or(std::ptr::null_mut(), |bytes| {
                bytes.as_mut_ptr() as *mut c_void
            })
    });
    with(metadata, context)
}

/// Starts a fresh context for `digest` and runs the plugin's init on it.
extern "C-unwind" fn plugin_reset(digest: Value) -> Value {
    let metadata = metadata_of(digest);
    let mut context = vec![0_u8; metadata.context_size.max(1)];
    (metadata.init)(context.as_mut_ptr() as *mut c_void);
    CONTEXTS.with(|held| held.borrow_mut().insert(digest, context));
    digest
}

extern "C-unwind" fn plugin_update(digest: Value, string: Value) -> Value {
    let bytes = super::arrays::array_of(call(to_object(string), "bytes", Vec::new()))
        .unwrap_or_default()
        .into_iter()
        .map(|byte| super::numbers::rb_num2long(to_value(&byte)) as u8)
        .collect::<Vec<u8>>();
    with_context(digest, |metadata, context| {
        (metadata.update)(context, bytes.as_ptr(), bytes.len())
    });
    digest
}

extern "C-unwind" fn plugin_finish(digest: Value) -> Value {
    let written = with_context(digest, |metadata, context| {
        let mut written = vec![0_u8; metadata.digest_length];
        (metadata.finish)(context, written.as_mut_ptr());
        written
    });
    super::strings::rb_str_new(written.as_ptr(), written.len() as i64)
}

extern "C-unwind" fn plugin_lengths(digest: Value) -> Value {
    let metadata = metadata_of(digest);
    let lengths = vec![
        Object::Int(metadata.digest_length as i64),
        Object::Int(metadata.block_length as i64),
    ];
    to_value(&Object::array(lengths))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_digest_namespace() -> Value {
    kernel_function("require", vec![Object::string("digest")]);
    let digest = top_level_module("Digest");
    let base = call(digest.clone(), "const_get", vec![Object::symbol("Base")]);
    let base = to_value(&base);
    let methods: [(&std::ffi::CStr, *const (), i32); 5] = [
        (c"__plugin__?", plugin_named as *const (), 0),
        (c"__plugin_reset__", plugin_reset as *const (), 0),
        (c"__plugin_update__", plugin_update as *const (), 1),
        (c"__plugin_finish__", plugin_finish as *const (), 0),
        (c"__plugin_lengths__", plugin_lengths as *const (), 0),
    ];
    for (name, function, arity) in methods {
        super::exports::rb_define_method(base, name.as_ptr(), function, arity);
    }
    to_value(&digest)
}
