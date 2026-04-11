//! VM initialization functions.
//!
//! This module contains functions for initializing the virtual machine with built-in
//! classes, methods, and global values.

use super::GlobalRegistry;
use crate::builtin_classes::{self, BuiltinClasses};
use crate::class::Class;
use crate::environment::Environment;
use crate::object::Object;
use std::cell::RefCell;
use std::rc::Rc;

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
    // block_given? defaults to false at global scope (no block context)
    globals.set("block_given?", Object::Bool(false));
    // Ruby constants that mspec and specs query
    globals.set("RUBY_VERSION", Object::String(Rc::new("4.0.2".to_string())));
    globals.set(
        "RUBY_ENGINE",
        Object::String(Rc::new("metorex".to_string())),
    );
    globals.set(
        "RUBY_PLATFORM",
        Object::String(Rc::new(std::env::consts::OS.to_string())),
    );
    globals.set(
        "RUBY_DESCRIPTION",
        Object::String(Rc::new("metorex (ruby-compatible)".to_string())),
    );
    // Standard IO stream placeholders (used as constants like STDOUT/STDERR/STDIN)
    globals.set("STDOUT", Object::String(Rc::new("STDOUT".to_string())));
    globals.set("STDERR", Object::String(Rc::new("STDERR".to_string())));
    globals.set("STDIN", Object::String(Rc::new("STDIN".to_string())));
}

/// Register built-in modules (Comparable, Enumerable, Kernel, etc.).
pub(super) fn register_builtin_modules(globals: &mut GlobalRegistry) {
    // Comparable — stub module, methods will be added later
    let comparable = Rc::new(Class::new("Comparable", None));
    globals.set("Comparable", Object::Module(comparable));

    // Enumerable — stub module
    let enumerable = Rc::new(Class::new("Enumerable", None));
    globals.set("Enumerable", Object::Module(enumerable));

    // Kernel — stub module
    let kernel = Rc::new(Class::new("Kernel", None));
    globals.set("Kernel", Object::Module(kernel));

    // Signal — stub module (trap is a no-op)
    let signal = Rc::new(Class::new("Signal", None));
    globals.set("Signal", Object::Module(signal));

    // ENV — use a Dict so ENV['KEY'] works. Keys are plain strings (no quotes)
    // because object_to_dict_key returns the raw String for Object::String.
    let mut env_map = std::collections::HashMap::new();
    for (k, v) in std::env::vars() {
        env_map.insert(k, Object::String(Rc::new(v)));
    }
    globals.set("ENV", Object::Dict(Rc::new(RefCell::new(env_map))));
}

/// Register Ruby special global variables.
pub(super) fn register_special_globals(globals: &mut GlobalRegistry) {
    // $LOAD_PATH / $: — shared array
    let load_path = Object::Array(Rc::new(RefCell::new(Vec::new())));
    globals.set(":", load_path.clone());
    globals.set("LOAD_PATH", load_path);

    // $LOADED_FEATURES / $" — shared array
    let loaded_features = Object::Array(Rc::new(RefCell::new(Vec::new())));
    globals.set("\"", loaded_features.clone());
    globals.set("LOADED_FEATURES", loaded_features);

    // $stdout / $stderr / $stdin — placeholders
    globals.set("stdout", Object::String(Rc::new("$stdout".to_string())));
    globals.set("stderr", Object::String(Rc::new("$stderr".to_string())));
    globals.set("stdin", Object::String(Rc::new("$stdin".to_string())));

    // $0 / $PROGRAM_NAME — set later by main when file is known
    globals.set("0", Object::String(Rc::new(String::new())));
    globals.set("PROGRAM_NAME", Object::String(Rc::new(String::new())));

    // $; $, $/ $\ — string separator globals
    globals.set(";", Object::Nil);
    globals.set(",", Object::Nil);
    globals.set("/", Object::String(Rc::new("\n".to_string())));
    globals.set("\\", Object::Nil);

    // $! $@ $~ $& — exception/regex globals
    globals.set("!", Object::Nil);
    globals.set("@", Object::Nil);
    globals.set("~", Object::Nil);
    globals.set("&", Object::Nil);

    // $? — process status
    globals.set("?", Object::Nil);

    // $_ — last input line
    globals.set("_", Object::Nil);

    // $. — line number
    globals.set(".", Object::Int(0));

    // $DEBUG / $VERBOSE
    globals.set("DEBUG", Object::Bool(false));
    globals.set("VERBOSE", Object::Bool(false));
}

/// Register native functions in the global registry.
pub(super) fn register_native_functions(globals: &mut GlobalRegistry) {
    globals.set("puts", Object::NativeFunction("puts".to_string()));
    globals.set("print", Object::NativeFunction("print".to_string()));
    globals.set("p", Object::NativeFunction("p".to_string()));
    globals.set("gets", Object::NativeFunction("gets".to_string()));
    globals.set("assert", Object::NativeFunction("assert".to_string()));
    globals.set(
        "assert_equal",
        Object::NativeFunction("assert_equal".to_string()),
    );
    globals.set(
        "assert_raises",
        Object::NativeFunction("assert_raises".to_string()),
    );
    globals.set("method", Object::NativeFunction("method".to_string()));
    globals.set("require", Object::NativeFunction("require".to_string()));
    globals.set(
        "require_relative",
        Object::NativeFunction("require_relative".to_string()),
    );
    globals.set("eval", Object::NativeFunction("eval".to_string()));
    globals.set("parse", Object::NativeFunction("parse".to_string()));
    globals.set("exit", Object::NativeFunction("exit".to_string()));
    globals.set("load", Object::NativeFunction("load".to_string()));
    // Visibility modifiers — stubs (no access control enforced).
    globals.set("private", Object::NativeFunction("private".to_string()));
    globals.set("public", Object::NativeFunction("public".to_string()));
    globals.set("protected", Object::NativeFunction("protected".to_string()));
    globals.set(
        "module_function",
        Object::NativeFunction("module_function".to_string()),
    );
    globals.set(
        "private_class_method",
        Object::NativeFunction("private_class_method".to_string()),
    );
    globals.set(
        "public_class_method",
        Object::NativeFunction("public_class_method".to_string()),
    );
    globals.set("freeze", Object::NativeFunction("freeze".to_string()));
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
