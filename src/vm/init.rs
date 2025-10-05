//! VM initialization functions.
//!
//! This module contains functions for initializing the virtual machine with built-in
//! classes, methods, and global values.

use super::GlobalRegistry;
use crate::builtin_classes::{self, BuiltinClasses};
use crate::environment::Environment;
use crate::object::Object;

/// Initialize built-in methods for core classes.
pub(super) fn initialize_builtin_methods(builtins: &BuiltinClasses) {
    builtin_classes::init_object_methods(builtins.object_class.as_ref());
    builtin_classes::init_string_methods(builtins.string_class.as_ref());
    builtin_classes::init_array_methods(builtins.array_class.as_ref());
    builtin_classes::init_hash_methods(builtins.hash_class.as_ref());
    builtin_classes::init_exception_methods(builtins.exception_class.as_ref());
}

/// Register all built-in classes in the global registry.
pub(super) fn register_builtin_classes(globals: &mut GlobalRegistry, builtins: &BuiltinClasses) {
    for (name, class) in builtins.all_classes() {
        globals.set(name, Object::Class(class));
    }
}

/// Register singleton values (nil, true, false) in the global registry.
pub(super) fn register_singletons(globals: &mut GlobalRegistry) {
    globals.set("nil", Object::Nil);
    globals.set("true", Object::Bool(true));
    globals.set("false", Object::Bool(false));
}

/// Register native functions in the global registry.
pub(super) fn register_native_functions(globals: &mut GlobalRegistry) {
    globals.set("puts", Object::NativeFunction("puts".to_string()));
    globals.set("method", Object::NativeFunction("method".to_string()));
    globals.set(
        "require_relative",
        Object::NativeFunction("require_relative".to_string()),
    );
}

/// Seed the environment with values from the global registry.
pub(super) fn seed_environment_with_globals(
    environment: &mut Environment,
    globals: &GlobalRegistry,
) {
    for (name, value) in globals.iter() {
        environment.define(name.clone(), value.clone());
    }
}
