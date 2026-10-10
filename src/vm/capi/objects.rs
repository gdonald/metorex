//! Objects from C: allocating and copying them, asking what they are and
//! what they answer to, converting them, and their instance variables,
//! including the ones C names without an `@`, which Ruby code cannot see.

use super::calls::{answers, call, class_name_of, kernel_function};
use super::handles::{QFALSE, QNIL, QTRUE, Value, objects_from, to_object, to_value};
use super::symbols::symbol_name;
use super::{called_from, called_with_block, interpreter, raise, running_method};
use crate::object::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char};

thread_local! {
    /// The instance variables C set under a name without an `@`, by object.
    static HIDDEN_IVARS: RefCell<HashMap<Value, HashMap<String, Value>>> = RefCell::new(HashMap::new());
    /// The class names `rb_obj_classname` handed out, kept so the pointers
    /// stay valid.
    static CLASS_NAMES: RefCell<HashMap<String, CString>> = RefCell::new(HashMap::new());
    /// How many `rb_get_alloc_func` default allocators are running now,
    /// which allocate without any allocator C defined.
    static ALLOCATING_WITHOUT_C: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

fn text(pointer: *const c_char) -> String {
    // SAFETY: C hands over a NUL-terminated string.
    unsafe { CStr::from_ptr(pointer) }
        .to_string_lossy()
        .into_owned()
}

fn truth(answered: bool) -> Value {
    if answered { QTRUE } else { QFALSE }
}

fn type_error(message: String) -> ! {
    raise(crate::vm::errors::simple_exception(
        "TypeError",
        &message,
        called_from(),
    ))
}

fn send(object: Value, name: &str, arguments: Vec<Object>) -> Value {
    to_value(&call(to_object(object), name, arguments))
}

/// Whether default allocators are running, so `allocate` skips the
/// allocators C defined.
pub(crate) fn allocating_without_c() -> bool {
    ALLOCATING_WITHOUT_C.with(|held| held.get()) > 0
}

/// The allocator `rb_get_alloc_func` answers for a class with none C
/// defined: what `allocate` makes when no C allocator is in the way.
extern "C-unwind" fn default_allocator(klass: Value) -> Value {
    struct Done;
    impl Drop for Done {
        fn drop(&mut self) {
            ALLOCATING_WITHOUT_C.with(|held| held.set(held.get() - 1));
        }
    }
    ALLOCATING_WITHOUT_C.with(|held| held.set(held.get() + 1));
    let _done = Done;
    send(klass, "allocate", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_get_alloc_func(klass: Value) -> *const () {
    match super::data::allocator_of(&super::exports::class_from(klass)) {
        Some(0) => std::ptr::null(),
        Some(address) => address as *const (),
        None => default_allocator as *const (),
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_undef_alloc_func(klass: Value) {
    super::data::rb_define_alloc_func(klass, std::ptr::null());
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_alloc(klass: Value) -> Value {
    send(klass, "allocate", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_dup(object: Value) -> Value {
    send(object, "dup", Vec::new())
}

/// Calls `initialize` on `object`, handing it the block of the C method
/// running now.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_call_init(object: Value, count: i32, values: *const Value) {
    interpreter().pending_block = called_with_block();
    send(object, "initialize", objects_from(i64::from(count), values));
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_class(object: Value) -> Value {
    send(object, "class", Vec::new())
}

/// The singleton class of `object`, made if it has none yet.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_singleton_class(object: Value) -> Value {
    send(object, "singleton_class", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_classname(object: Value) -> *const c_char {
    let name = class_name_of(to_object(object));
    CLASS_NAMES.with(|held| {
        held.borrow_mut()
            .entry(name.clone())
            .or_insert_with(|| CString::new(name).unwrap_or_default())
            .as_ptr()
    })
}

/// Freezes `object` without calling a `freeze` it defines.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_freeze(object: Value) -> Value {
    interpreter().freeze_object(&to_object(object));
    object
}

/// Whether `object` is frozen, without calling a `frozen?` it defines.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_frozen_p(object: Value) -> Value {
    truth(interpreter().object_is_frozen(&to_object(object)))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_check_frozen(object: Value) {
    if rb_obj_frozen_p(object) == QTRUE {
        super::exceptions::rb_error_frozen_object(object);
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_id(object: Value) -> Value {
    send(object, "object_id", Vec::new())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_is_instance_of(object: Value, klass: Value) -> Value {
    send(object, "instance_of?", vec![to_object(klass)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_is_kind_of(object: Value, klass: Value) -> Value {
    send(object, "kind_of?", vec![to_object(klass)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_method(object: Value, name: Value) -> Value {
    send(object, "method", vec![to_object(name)])
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_method_arity(object: Value, name: Value) -> i32 {
    let method = call(to_object(object), "method", vec![to_object(name)]);
    super::numbers::rb_num2long(to_value(&call(method, "arity", Vec::new()))) as i32
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_respond_to(
    object: Value,
    name: Value,
    include_private: i32,
) -> i32 {
    let arguments = vec![to_object(name), Object::Bool(include_private != 0)];
    call(to_object(object), "respond_to?", arguments).is_truthy() as i32
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_respond_to(object: Value, name: Value) -> i32 {
    rb_obj_respond_to(object, name, 0)
}

/// Whether instances of `klass` have the method `name`, leaving out a
/// private one when `exclude_private` says so.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_method_boundp(klass: Value, name: Value, exclude_private: i32) -> i32 {
    let named = vec![to_object(name)];
    let klass = to_object(klass);
    let bound = call(klass.clone(), "method_defined?", named.clone()).is_truthy()
        || (exclude_private == 0 && call(klass, "private_method_defined?", named).is_truthy());
    bound as i32
}

/// Whether `object` is one of the values C holds directly rather than
/// through a handle: nil, true, false, a Fixnum or a Symbol.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_special_const_p(object: Value) -> Value {
    let special = object & 1 == 1
        || matches!(
            to_object(object),
            Object::Nil | Object::Bool(_) | Object::Symbol(_)
        );
    truth(special)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_to_id(name: Value) -> Value {
    match to_object(name) {
        Object::Symbol(_) => name,
        Object::String(text) => to_value(&Object::symbol(text.as_str().to_string())),
        other => {
            let shown = call(other, "inspect", Vec::new());
            type_error(format!("{} is not a symbol nor a string", shown))
        }
    }
}

/// The class name a conversion error names `object` by: `nil`, `true`
/// and `false` by themselves.
fn named_in_error(object: &Object) -> String {
    match object {
        Object::Nil => "nil".to_string(),
        Object::Bool(held) => held.to_string(),
        other => class_name_of(other.clone()),
    }
}

/// Whether `object` is of the type tag `type_tag` and so needs no converting.
fn of_type(object: Value, type_tag: i32) -> bool {
    super::subclasses::rb_type(object) == type_tag
}

/// `object` itself when it is of `type_tag`, what `method` converts it to
/// when that is, nil when it has no such method or `method` answers nil,
/// and a TypeError for anything else, as `rb_check_convert_type` does.
fn converted(object: Value, type_tag: i32, type_name: &str, method: &str) -> Option<Value> {
    if of_type(object, type_tag) {
        return Some(object);
    }
    let held = to_object(object);
    if !answers(&held, method) {
        return None;
    }
    let answered = to_value(&call(held.clone(), method, Vec::new()));
    if answered == QNIL {
        return None;
    }
    if !of_type(answered, type_tag) {
        let class_name = class_name_of(held);
        type_error(format!(
            "can't convert {} to {} ({}#{} gives {})",
            class_name,
            type_name,
            class_name,
            method,
            class_name_of(to_object(answered))
        ));
    }
    Some(answered)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_check_convert_type(
    object: Value,
    type_tag: i32,
    type_name: *const c_char,
    method: *const c_char,
) -> Value {
    converted(object, type_tag, &text(type_name), &text(method)).unwrap_or(QNIL)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_convert_type(
    object: Value,
    type_tag: i32,
    type_name: *const c_char,
    method: *const c_char,
) -> Value {
    let type_name = text(type_name);
    converted(object, type_tag, &type_name, &text(method)).unwrap_or_else(|| {
        type_error(format!(
            "no implicit conversion of {} into {}",
            named_in_error(&to_object(object)),
            type_name
        ))
    })
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_check_array_type(object: Value) -> Value {
    converted(object, 0x07, "Array", "to_ary").unwrap_or(QNIL)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_check_string_type(object: Value) -> Value {
    converted(object, 0x05, "String", "to_str").unwrap_or(QNIL)
}

fn is_integer(object: &Object) -> bool {
    matches!(object, Object::Int(_) | Object::BigInt(_))
}

/// `object` when it is an Integer, or what `method` converts it to when
/// that is one, or nil.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_check_to_integer(object: Value, method: *const c_char) -> Value {
    let held = to_object(object);
    if is_integer(&held) {
        return object;
    }
    let method = text(method);
    if !answers(&held, &method) {
        return QNIL;
    }
    let answered = call(held, &method, Vec::new());
    if is_integer(&answered) {
        to_value(&answered)
    } else {
        QNIL
    }
}

/// `object` as an Integer through `to_int`, refusing what has none or
/// converts to something else.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_to_int(object: Value) -> Value {
    let held = to_object(object);
    if is_integer(&held) {
        return object;
    }
    if !answers(&held, "to_int") {
        type_error(format!(
            "no implicit conversion of {} into Integer",
            named_in_error(&held)
        ));
    }
    let answered = call(held.clone(), "to_int", Vec::new());
    if !is_integer(&answered) {
        let class_name = class_name_of(held);
        type_error(format!(
            "can't convert {} to Integer ({}#to_int gives {})",
            class_name,
            class_name,
            class_name_of(answered)
        ));
    }
    to_value(&answered)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_extend_object(object: Value, module: Value) {
    send(object, "extend", vec![to_object(module)]);
}

/// Runs the block of the C method running now with `object` as self.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_instance_eval(
    count: i32,
    values: *const Value,
    object: Value,
) -> Value {
    interpreter().pending_block = called_with_block();
    send(
        object,
        "instance_eval",
        objects_from(i64::from(count), values),
    )
}

/// What `Kernel#to_s` answers for any object: its class and where it is.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_any_to_s(object: Value) -> Value {
    let class_name = class_name_of(to_object(object));
    to_value(&Object::string(format!(
        "#<{}:0x{:016x}>",
        class_name, object
    )))
}

/// Whether `first` and `second` are the same object or `==` says they are
/// equal.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_equal(first: Value, second: Value) -> Value {
    if first == second {
        return QTRUE;
    }
    truth(call(to_object(first), "==", vec![to_object(second)]).is_truthy())
}

fn is_module(object: &Object) -> bool {
    matches!(object, Object::Class(_) | Object::Module(_))
}

/// Whether `module` is `other` or descends from it, or nil when the two
/// are unrelated, as `Module#<=` answers.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_class_inherited_p(module: Value, other: Value) -> Value {
    let (module, other) = (to_object(module), to_object(other));
    if !is_module(&module) || !is_module(&other) {
        type_error("compared with non class/module".to_string());
    }
    to_value(&call(module, "<=", vec![other]))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_require(feature: *const c_char) -> Value {
    to_value(&kernel_function(
        "require",
        vec![Object::string(text(feature))],
    ))
}

/// The function behind a method a platform does not have. Such a method
/// answers false to `respond_to?`, and raises NotImplementedError when
/// called anyway.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_f_notimplement(
    _count: i32,
    _values: *const Value,
    _object: Value,
    _marker: Value,
) -> Value {
    let name = running_method()
        .map(|method| method.name)
        .unwrap_or_default();
    raise(crate::vm::errors::simple_exception(
        "NotImplementedError",
        &format!("{}() function is unimplemented on this machine", name),
        called_from(),
    ))
}

/// Whether `function` is `rb_f_notimplement`, which `respond_to?` answers
/// false for.
pub(crate) fn is_not_implemented(function: &super::CFunction) -> bool {
    function.address == rb_f_notimplement as *const () as usize
}

/// Whether `name` is one Ruby code reads as an instance variable.
fn is_visible_name(name: &str) -> bool {
    name.starts_with('@') && !name.starts_with("@@") && name.len() > 1
}

fn hidden(object: Value, name: &str) -> Option<Value> {
    HIDDEN_IVARS.with(|held| {
        held.borrow()
            .get(&object)
            .and_then(|names| names.get(name).copied())
    })
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ivar_get(object: Value, name: Value) -> Value {
    let name = symbol_name(name);
    if is_visible_name(&name) {
        return send(object, "instance_variable_get", vec![Object::symbol(&name)]);
    }
    hidden(object, &name).unwrap_or(QNIL)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ivar_set(object: Value, name: Value, value: Value) -> Value {
    let name = symbol_name(name);
    if is_visible_name(&name) {
        let arguments = vec![Object::symbol(&name), to_object(value)];
        return send(object, "instance_variable_set", arguments);
    }
    HIDDEN_IVARS.with(|held| {
        held.borrow_mut()
            .entry(object)
            .or_default()
            .insert(name, value)
    });
    value
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ivar_defined(object: Value, name: Value) -> Value {
    let name = symbol_name(name);
    if is_visible_name(&name) {
        return send(
            object,
            "instance_variable_defined?",
            vec![Object::symbol(&name)],
        );
    }
    truth(hidden(object, &name).is_some())
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_attr_get(object: Value, name: Value) -> Value {
    rb_ivar_get(object, name)
}

fn interned(name: *const c_char) -> Value {
    to_value(&Object::symbol(text(name)))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_iv_get(object: Value, name: *const c_char) -> Value {
    rb_ivar_get(object, interned(name))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_iv_set(object: Value, name: *const c_char, value: Value) -> Value {
    rb_ivar_set(object, interned(name), value)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_obj_instance_variables(object: Value) -> Value {
    send(object, "instance_variables", Vec::new())
}

/// The names an Array `name` called on `object` answers.
fn names_from(object: &Object, name: &str) -> Vec<Object> {
    super::arrays::array_of(call(object.clone(), name, Vec::new())).unwrap_or_default()
}

fn visible_names(object: Value) -> Vec<Object> {
    names_from(&to_object(object), "instance_variables")
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ivar_count(object: Value) -> usize {
    let hidden_count = HIDDEN_IVARS.with(|held| held.borrow().get(&object).map_or(0, HashMap::len));
    visible_names(object).len() + hidden_count
}

/// Calls `function` with the name and value of each class variable of a
/// class or module and then each instance variable, until it answers
/// `ST_STOP`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_ivar_foreach(
    object: Value,
    function: extern "C-unwind" fn(Value, Value, Value) -> i32,
    data: Value,
) {
    const ST_STOP: i32 = 1;
    let held = to_object(object);
    let mut pairs = Vec::new();
    if is_module(&held) {
        for name in names_from(&held, "class_variables") {
            let value = call(held.clone(), "class_variable_get", vec![name.clone()]);
            pairs.push((name, value));
        }
    }
    for name in visible_names(object) {
        let value = call(held.clone(), "instance_variable_get", vec![name.clone()]);
        pairs.push((name, value));
    }
    for (name, value) in pairs {
        if function(to_value(&name), to_value(&value), data) == ST_STOP {
            break;
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_copy_generic_ivar(clone: Value, object: Value) {
    for name in visible_names(object) {
        let value = call(
            to_object(object),
            "instance_variable_get",
            vec![name.clone()],
        );
        call(to_object(clone), "instance_variable_set", vec![name, value]);
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_free_generic_ivar(object: Value) {
    for name in visible_names(object) {
        call(to_object(object), "remove_instance_variable", vec![name]);
    }
    HIDDEN_IVARS.with(|held| held.borrow_mut().remove(&object));
}
