//! Working with classes and modules from C: their constants by any name,
//! aliases, method visibility, singleton and module functions, undefining
//! methods, names and ancestors.

use super::calls::{call, class_name_of, top_level_module};
use super::exports::{class_from, rb_define_method, text};
use super::handles::{Value, to_object, to_value};
use super::symbols::symbol_name;
use super::{called_from, interpreter, or_raise, raise};
use crate::object::{Method, Object};
use std::ffi::{CString, c_char};
use std::rc::Rc;

fn error(class: &str, message: String) -> ! {
    raise(crate::vm::errors::simple_exception(
        class,
        &message,
        called_from(),
    ))
}

fn is_constant_name(name: &str) -> bool {
    crate::vm::native_methods::is_valid_constant_name(name)
}

/// The constant `name` held by `module` itself, for a name Ruby code could
/// not spell, which only C reaches.
fn raw_constant(module: Value, name: &str) -> Option<Object> {
    class_from(module).get_class_var(name)
}

fn uninitialized(module: Value, name: &str) -> ! {
    let module_name = class_name_of(to_object(module));
    let shown = if module_name == "Object" {
        name.to_string()
    } else {
        format!("{}::{}", to_object(module), name)
    };
    error("NameError", format!("uninitialized constant {}", shown))
}

/// Whether `module`, its ancestors, or Object for a module, holds `name`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_const_defined(module: Value, name: Value) -> i32 {
    let name = symbol_name(name);
    if !is_constant_name(&name) {
        return raw_constant(module, &name).is_some() as i32;
    }
    let arguments = vec![Object::symbol(name), Object::Bool(true)];
    call(to_object(module), "const_defined?", arguments).is_truthy() as i32
}

/// Whether `module` itself holds `name`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_const_defined_at(module: Value, name: Value) -> i32 {
    let name = symbol_name(name);
    if !is_constant_name(&name) {
        return raw_constant(module, &name).is_some() as i32;
    }
    let arguments = vec![Object::symbol(name), Object::Bool(false)];
    call(to_object(module), "const_defined?", arguments).is_truthy() as i32
}

/// The constant `name` from `module`, its ancestors, or Object, running an
/// autoload for it and `const_missing` when there is none.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_const_get(module: Value, name: Value) -> Value {
    let named = symbol_name(name);
    if !is_constant_name(&named) {
        return raw_constant(module, &named)
            .map(|held| to_value(&held))
            .unwrap_or_else(|| uninitialized(module, &named));
    }
    to_value(&call(
        to_object(module),
        "const_get",
        vec![Object::symbol(named)],
    ))
}

/// The constant `name` from `module` itself, running an autoload for it and
/// `const_missing` when there is none.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_const_get_at(module: Value, name: Value) -> Value {
    let named = symbol_name(name);
    if !is_constant_name(&named) {
        return raw_constant(module, &named)
            .map(|held| to_value(&held))
            .unwrap_or_else(|| uninitialized(module, &named));
    }
    let arguments = vec![Object::symbol(named), Object::Bool(false)];
    to_value(&call(to_object(module), "const_get", arguments))
}

/// The constant `name` from `module` or its ancestors up to but not
/// including Object, running `const_missing` when none holds it.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_const_get_from(module: Value, name: Value) -> Value {
    let named = symbol_name(name);
    let module_object = to_object(module);
    let object_class = top_level_module("Object");
    let ancestors = match call(module_object.clone(), "ancestors", Vec::new()) {
        Object::Array(found) => found.borrow().clone(),
        _ => Vec::new(),
    };
    for ancestor in ancestors {
        if ancestor == object_class && module_object != object_class {
            break;
        }
        let symbol = Object::symbol(named.clone());
        let held_here = call(
            ancestor.clone(),
            "const_defined?",
            vec![symbol.clone(), Object::Bool(false)],
        );
        if held_here.is_truthy() {
            return to_value(&call(
                ancestor,
                "const_get",
                vec![symbol, Object::Bool(false)],
            ));
        }
    }
    to_value(&call(
        module_object,
        "const_missing",
        vec![Object::symbol(named)],
    ))
}

/// Binds the constant, warning when it was already bound, under any name,
/// including one Ruby code could not spell.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_const_set(module: Value, name: Value, value: Value) {
    let named = symbol_name(name);
    let class = class_from(module);
    if class.is_frozen() {
        raise(interpreter().frozen_modification_error(&to_object(module), called_from()));
    }
    or_raise(interpreter().assign_constant(&class, &named, to_object(value), called_from()));
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_global_const(name: *const c_char, value: Value) {
    let object_class = to_value(&top_level_module("Object"));
    super::definitions::rb_define_const(object_class, name, value);
}

fn alias(module: Value, new_name: String, old_name: String) {
    let arguments = vec![
        Object::symbol("alias_method"),
        Object::symbol(new_name),
        Object::symbol(old_name),
    ];
    call(to_object(module), "__send__", arguments);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_alias(
    module: Value,
    new_name: *const c_char,
    old_name: *const c_char,
) {
    alias(module, text(new_name), text(old_name));
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_alias(module: Value, new_name: Value, old_name: Value) {
    alias(module, symbol_name(new_name), symbol_name(old_name));
}

/// Gives the method `name` on `module` the visibility `which` names.
fn set_visibility(module: Value, name: &str, which: &str) {
    let arguments = vec![Object::symbol(which), Object::symbol(name)];
    call(to_object(module), "__send__", arguments);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_private_method(
    module: Value,
    name: *const c_char,
    function: *const (),
    arity: i32,
) {
    rb_define_method(module, name, function, arity);
    set_visibility(module, &text(name), "private");
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_protected_method(
    module: Value,
    name: *const c_char,
    function: *const (),
    arity: i32,
) {
    rb_define_method(module, name, function, arity);
    set_visibility(module, &text(name), "protected");
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_singleton_method(
    object: Value,
    name: *const c_char,
    function: *const (),
    arity: i32,
) {
    let singleton = call(to_object(object), "singleton_class", Vec::new());
    rb_define_method(to_value(&singleton), name, function, arity);
}

/// A private instance method of the module and a method of the module
/// itself, as `module_function` makes.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_module_function(
    module: Value,
    name: *const c_char,
    function: *const (),
    arity: i32,
) {
    rb_define_private_method(module, name, function, arity);
    rb_define_singleton_method(module, name, function, arity);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_global_function(
    name: *const c_char,
    function: *const (),
    arity: i32,
) {
    let kernel = to_value(&top_level_module("Kernel"));
    rb_define_module_function(kernel, name, function, arity);
}

/// Undefines `name` on the class or module, refusing a frozen one. A name
/// it does not answer is marked undefined all the same, which is what MRI's
/// `rb_undef_method` does.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_undef_method(module: Value, name: *const c_char) {
    let class = class_from(module);
    let named = text(name);
    if class.is_frozen() {
        let kind = if class.is_module() { "module" } else { "class" };
        let inspected = call(to_object(module), "inspect", Vec::new()).to_string();
        error(
            "FrozenError",
            format!("can't modify frozen {}: {}", kind, inspected),
        );
    }
    let arguments = vec![
        Object::symbol("undef_method"),
        Object::symbol(named.clone()),
    ];
    let machine = interpreter();
    let undone = machine.send_to_object(to_object(module), "__send__", arguments, called_from());
    if undone.is_err() {
        class.define_method(named.clone(), Rc::new(Method::undefined(named)));
    }
}

/// Undefines `name` as the `undef` keyword does, refusing a name the class
/// or module does not answer.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_undef(module: Value, name: Value) {
    let arguments = vec![
        Object::symbol("undef_method"),
        Object::symbol(symbol_name(name)),
    ];
    call(to_object(module), "__send__", arguments);
}

thread_local! {
    /// The class names `rb_class2name` has answered, kept for the life of
    /// the program since C may hold on to any of them.
    static CLASS_NAMES: std::cell::RefCell<Vec<CString>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_class2name(module: Value) -> *const c_char {
    let named = to_object(module).to_string();
    CLASS_NAMES.with(|held| {
        let mut held = held.borrow_mut();
        if let Some(found) = held.iter().find(|kept| kept.to_bytes() == named.as_bytes()) {
            return found.as_ptr();
        }
        held.push(CString::new(named).unwrap_or_default());
        held.last().map_or(std::ptr::null(), |kept| kept.as_ptr())
    })
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_mod_ancestors(module: Value) -> Value {
    to_value(&call(to_object(module), "ancestors", Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_mod_name(module: Value) -> Value {
    to_value(&call(to_object(module), "name", Vec::new()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_class_name(module: Value) -> Value {
    to_value(&Object::string(to_object(module).to_string()))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_class_path(module: Value) -> Value {
    rb_class_name(module)
}

/// The class or module `path` names, each segment held by the one before
/// it, starting from Object. An autoload registered for a segment runs.
pub(super) fn class_at_path(path: &str) -> Value {
    let undefined = || error("ArgumentError", format!("undefined class/module {}", path));
    let mut current = top_level_module("Object");
    let mut walked = String::new();
    for segment in path.split("::") {
        if !walked.is_empty() {
            walked.push_str("::");
        }
        walked.push_str(segment);
        if !matches!(current, Object::Class(_) | Object::Module(_)) {
            error(
                "TypeError",
                format!("{} does not refer to class/module", walked),
            );
        }
        let arguments = vec![Object::symbol(segment), Object::Bool(false)];
        if !is_constant_name(segment)
            || !call(current.clone(), "const_defined?", arguments.clone()).is_truthy()
        {
            undefined();
        }
        current = call(current, "const_get", arguments);
    }
    if !matches!(current, Object::Class(_) | Object::Module(_)) {
        error(
            "TypeError",
            format!("{} does not refer to class/module", path),
        );
    }
    to_value(&current)
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_path2class(path: *const c_char) -> Value {
    class_at_path(&text(path))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_path_to_class(path: Value) -> Value {
    class_at_path(&to_object(path).to_string())
}

/// Whether `klass` or an ancestor holds the class variable `name`. A name
/// without `@@` is looked up among the instance variables of the class
/// itself, which MRI keeps in the same table.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_cvar_defined(klass: Value, name: Value) -> Value {
    let named = symbol_name(name);
    let asked = if named.starts_with("@@") {
        "class_variable_defined?"
    } else {
        "instance_variable_defined?"
    };
    let defined = call(to_object(klass), asked, vec![Object::symbol(named)]);
    to_value(&defined)
}

fn class_variable_get(klass: Value, name: String) -> Value {
    let arguments = vec![Object::symbol(name)];
    to_value(&call(to_object(klass), "class_variable_get", arguments))
}

fn class_variable_set(klass: Value, name: String, value: Value) {
    let arguments = vec![Object::symbol(name), to_object(value)];
    call(to_object(klass), "class_variable_set", arguments);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_cvar_get(klass: Value, name: Value) -> Value {
    class_variable_get(klass, symbol_name(name))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_cvar_set(klass: Value, name: Value, value: Value) {
    class_variable_set(klass, symbol_name(name), value);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_cv_get(klass: Value, name: *const c_char) -> Value {
    class_variable_get(klass, text(name))
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_cv_set(klass: Value, name: *const c_char, value: Value) {
    class_variable_set(klass, text(name), value);
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_class_variable(klass: Value, name: *const c_char, value: Value) {
    class_variable_set(klass, text(name), value);
}

/// A public reader, writer, or both, for the instance variable `@name`.
#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_define_attr(klass: Value, name: *const c_char, read: i32, write: i32) {
    let named = text(name);
    for (wanted, definer) in [(read, "attr_reader"), (write, "attr_writer")] {
        if wanted != 0 {
            let arguments = vec![Object::symbol(definer), Object::symbol(named.clone())];
            call(to_object(klass), "__send__", arguments);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C-unwind" fn rb_include_module(klass: Value, module: Value) {
    let arguments = vec![Object::symbol("include"), to_object(module)];
    call(to_object(klass), "__send__", arguments);
}
