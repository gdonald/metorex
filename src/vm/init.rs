//! VM initialization functions.
//!
//! This module contains functions for initializing the virtual machine with built-in
//! classes, methods, and global values.

use super::GlobalRegistry;
use crate::builtin_classes::{self, BuiltinClasses};
use crate::class::Class;
use crate::environment::Environment;
use crate::object::{Binding, Object};
use std::cell::RefCell;
use std::collections::HashMap;
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

    // BasicObject — Ruby's true root class. Object inherits from it.
    let basic_object = Rc::new(Class::new("BasicObject", None));
    globals.set("BasicObject", Object::Class(Rc::clone(&basic_object)));

    // Object — root class that mspec reopens to inject describe/it/before/after
    let object = Rc::new(Class::new("Object", Some(Rc::clone(&basic_object))));
    basic_object.add_subclass(&object);
    // Mix Kernel in here so the chain Object → Kernel exists from the start;
    // Kernel itself was registered by `register_builtin_modules` already.
    if let Some(Object::Module(kernel)) = globals.get("Kernel") {
        object.add_mixin(kernel);
    }
    // Ruby's Object has `ruby2_keywords` as a private method; main inherits from Object.
    object.set_method_private("ruby2_keywords");
    globals.set("Object", Object::Class(Rc::clone(&object)));

    // Class and Module — used by `Class.new { ... }` and `Module.new { ... }`.
    let module_class = Rc::new(Class::new("Module", Some(Rc::clone(&object))));
    globals.set("Module", Object::Class(Rc::clone(&module_class)));
    let class_class = Rc::new(Class::new("Class", Some(Rc::clone(&module_class))));
    // Class#initialize is private (Ruby semantics); the method itself is
    // implemented natively in call_class_methods, so the name only needs to
    // appear in Class's private_method_names for visibility checks.
    class_class.set_method_private("initialize");
    globals.set("Class", Object::Class(class_class));

    // TOPLEVEL_BINDING — used by eval('code', TOPLEVEL_BINDING) at top level.
    // Its receiver is the top-level `main` object (an Object instance in Ruby; we
    // reuse the Object class here since private_methods dispatch works on Class).
    globals.set(
        "TOPLEVEL_BINDING",
        Object::Binding(Rc::new(Binding::with_receiver(
            HashMap::new(),
            Object::Class(Rc::clone(&object)),
        ))),
    );

    // TrueClass, FalseClass, NilClass — can't be instantiated
    let true_class = Rc::new(Class::new("TrueClass", Some(Rc::clone(&object))));
    globals.set("TrueClass", Object::Class(true_class));
    let false_class = Rc::new(Class::new("FalseClass", Some(Rc::clone(&object))));
    globals.set("FalseClass", Object::Class(false_class));
    let nil_class = Rc::new(Class::new("NilClass", Some(Rc::clone(&object))));
    globals.set("NilClass", Object::Class(nil_class));

    // Numeric types
    let numeric = Rc::new(Class::new("Numeric", Some(Rc::clone(&object))));
    globals.set("Numeric", Object::Class(Rc::clone(&numeric)));
    let rational_class = Rc::new(Class::new("Rational", Some(Rc::clone(&numeric))));
    globals.set("Rational", Object::Class(rational_class));
    let complex_class = Rc::new(Class::new("Complex", Some(Rc::clone(&numeric))));
    globals.set("Complex", Object::Class(complex_class));

    // Regexp — stub class so `case x; when Regexp; ...; end` and
    // `obj.kind_of?(Regexp)` resolve.
    let regexp_class = Rc::new(Class::new("Regexp", Some(Rc::clone(&object))));
    globals.set("Regexp", Object::Class(regexp_class));

    // Method — stub class so the constant resolves (`Module.constants`
    // includes :Method); Method objects themselves are a native type.
    let method_class = Rc::new(Class::new("Method", Some(Rc::clone(&object))));
    globals.set("Method", Object::Class(method_class));
}

/// Register standard Ruby exception class hierarchy as stubs.
pub(super) fn register_exception_classes(globals: &mut GlobalRegistry) {
    // Reuse Exception/StandardError/RuntimeError/TypeError already in BuiltinClasses;
    // add the remaining subclasses that specs and mspec reference.
    let exception = Rc::new(Class::new("Exception", None));
    let standard_error = Rc::new(Class::new("StandardError", Some(Rc::clone(&exception))));
    let runtime_error = Rc::new(Class::new("RuntimeError", Some(Rc::clone(&standard_error))));
    let name_error = Rc::new(Class::new("NameError", Some(Rc::clone(&standard_error))));
    let no_method_error = Rc::new(Class::new("NoMethodError", Some(Rc::clone(&name_error))));
    let argument_error = Rc::new(Class::new(
        "ArgumentError",
        Some(Rc::clone(&standard_error)),
    ));
    let type_error = Rc::new(Class::new("TypeError", Some(Rc::clone(&standard_error))));
    let range_error = Rc::new(Class::new("RangeError", Some(Rc::clone(&standard_error))));
    let io_error = Rc::new(Class::new("IOError", Some(Rc::clone(&standard_error))));
    let index_error = Rc::new(Class::new("IndexError", Some(Rc::clone(&standard_error))));
    let key_error = Rc::new(Class::new("KeyError", Some(Rc::clone(&index_error))));
    let stop_iteration = Rc::new(Class::new("StopIteration", Some(Rc::clone(&index_error))));
    let zero_division_error = Rc::new(Class::new(
        "ZeroDivisionError",
        Some(Rc::clone(&standard_error)),
    ));
    let float_domain_error = Rc::new(Class::new(
        "FloatDomainError",
        Some(Rc::clone(&range_error)),
    ));
    let script_error = Rc::new(Class::new("ScriptError", Some(Rc::clone(&exception))));
    let load_error = Rc::new(Class::new("LoadError", Some(Rc::clone(&script_error))));
    let syntax_error = Rc::new(Class::new("SyntaxError", Some(Rc::clone(&script_error))));
    let not_implemented_error = Rc::new(Class::new(
        "NotImplementedError",
        Some(Rc::clone(&script_error)),
    ));
    let system_exit = Rc::new(Class::new("SystemExit", Some(Rc::clone(&exception))));
    let interrupt = Rc::new(Class::new("Interrupt", Some(Rc::clone(&exception))));
    let signal_exception = Rc::new(Class::new("SignalException", Some(Rc::clone(&exception))));
    let system_call_error = Rc::new(Class::new(
        "SystemCallError",
        Some(Rc::clone(&standard_error)),
    ));
    let errno_module = Rc::new(Class::new("Errno", None));
    let encoding_error = Rc::new(Class::new(
        "EncodingError",
        Some(Rc::clone(&standard_error)),
    ));
    let frozen_error = Rc::new(Class::new("FrozenError", Some(Rc::clone(&runtime_error))));
    let local_jump_error = Rc::new(Class::new(
        "LocalJumpError",
        Some(Rc::clone(&standard_error)),
    ));
    let regexp_error = Rc::new(Class::new("RegexpError", Some(Rc::clone(&standard_error))));
    let math_domain_error = Rc::new(Class::new(
        "Math::DomainError",
        Some(Rc::clone(&argument_error)),
    ));

    globals.set("Exception", Object::Class(exception));
    globals.set("StandardError", Object::Class(standard_error));
    globals.set("RuntimeError", Object::Class(runtime_error));
    globals.set("NameError", Object::Class(name_error));
    globals.set("NoMethodError", Object::Class(no_method_error));
    globals.set("ArgumentError", Object::Class(argument_error));
    globals.set("TypeError", Object::Class(type_error));
    globals.set("RangeError", Object::Class(range_error));
    globals.set("IOError", Object::Class(io_error));
    globals.set("IndexError", Object::Class(index_error));
    globals.set("KeyError", Object::Class(key_error));
    globals.set("StopIteration", Object::Class(stop_iteration));
    globals.set("ZeroDivisionError", Object::Class(zero_division_error));
    globals.set("FloatDomainError", Object::Class(float_domain_error));
    globals.set("ScriptError", Object::Class(script_error));
    globals.set("LoadError", Object::Class(load_error));
    globals.set("SyntaxError", Object::Class(syntax_error));
    globals.set("NotImplementedError", Object::Class(not_implemented_error));
    globals.set("SystemExit", Object::Class(system_exit));
    globals.set("Interrupt", Object::Class(interrupt));
    globals.set("SignalException", Object::Class(signal_exception));
    globals.set("SystemCallError", Object::Class(system_call_error));
    globals.set("Errno", Object::Module(errno_module));
    globals.set("EncodingError", Object::Class(encoding_error));
    globals.set("FrozenError", Object::Class(frozen_error));
    globals.set("LocalJumpError", Object::Class(local_jump_error));
    globals.set("RegexpError", Object::Class(regexp_error));
    globals.set("Math::DomainError", Object::Class(math_domain_error));
}

/// Register built-in modules (Comparable, Enumerable, Kernel, etc.).
pub(super) fn register_builtin_modules(globals: &mut GlobalRegistry) {
    // Comparable — stub module, methods will be added later
    let comparable = Rc::new(Class::new("Comparable", None));
    globals.set("Comparable", Object::Module(comparable));

    // Enumerable — stub module
    let enumerable = Rc::new(Class::new("Enumerable", None));
    globals.set("Enumerable", Object::Module(enumerable));

    // Kernel — stub module mixed into Object so its instance methods are
    // available to every object (matches Ruby's standard ancestor chain).
    // The mixin link is established here against whatever Object class is
    // currently in globals at this point. Note that Object is replaced again
    // later in register_singletons, so `wire_kernel_into_object` is called
    // from VirtualMachine::new() after that step to re-establish the link.
    let kernel = Rc::new(Class::new("Kernel", None));
    globals.set("Kernel", Object::Module(kernel));

    // Encoding — metorex strings are UTF-8, so the named encodings exist as
    // distinct objects but every string reports UTF-8.
    let encoding = Rc::new(Class::new("Encoding", None));
    for name in ["UTF_8", "US_ASCII", "BINARY", "ASCII_8BIT"] {
        let display = name.replace('_', "-");
        let constant = Rc::new(Class::new(display, Some(Rc::clone(&encoding))));
        encoding.set_class_var(name, Object::Class(Rc::clone(&constant)));
        globals.set(format!("Encoding::{}", name), Object::Class(constant));
    }
    globals.set("Encoding", Object::Class(encoding));

    // Signal — stub module (trap is a no-op)
    let signal = Rc::new(Class::new("Signal", None));
    globals.set("Signal", Object::Module(signal));

    // Process — stub module (pid is a no-op)
    let process = Rc::new(Class::new("Process", None));
    globals.set("Process", Object::Module(process));

    // Math — stub module (constants will be added later if needed)
    let math = Rc::new(Class::new("Math", None));
    globals.set("Math", Object::Module(math));

    // GC — no-op stub
    let gc = Rc::new(Class::new("GC", None));
    globals.set("GC", Object::Module(gc));

    // ObjectSpace — no-op stub
    let object_space = Rc::new(Class::new("ObjectSpace", None));
    globals.set("ObjectSpace", Object::Module(object_space));

    // Warning — `Warning[:category]` reads and `Warning[:category] = bool`
    // writes the per-category warning switches. Categories are stored as
    // class variables on the module and start off, matching MRI's default
    // for `:deprecated`.
    let warning = Rc::new(Class::new("Warning", None));
    globals.set("Warning", Object::Module(warning));

    // Time / IO — placeholder stubs (used in mspec)
    let time = Rc::new(Class::new(
        "Time",
        Some(Rc::new(Class::new("Object", None))),
    ));
    globals.set("Time", Object::Class(time));
    let io = Rc::new(Class::new("IO", Some(Rc::new(Class::new("Object", None)))));
    globals.set("IO", Object::Class(io));

    // Thread — stub
    let thread = Rc::new(Class::new(
        "Thread",
        Some(Rc::new(Class::new("Object", None))),
    ));
    globals.set("Thread", Object::Class(thread));

    // Queue / SizedQueue — minimal FIFO stub. metorex runs Thread blocks
    // synchronously, so blocking-pop semantics aren't meaningful;
    // `pop` returns nil on an empty queue rather than blocking. Enough
    // for spec helpers and autoload coordination patterns to make
    // forward progress.
    let queue = Rc::new(Class::new(
        "Queue",
        Some(Rc::new(Class::new("Object", None))),
    ));
    globals.set("Queue", Object::Class(queue));
    let sized_queue = Rc::new(Class::new(
        "SizedQueue",
        Some(Rc::new(Class::new("Object", None))),
    ));
    globals.set("SizedQueue", Object::Class(sized_queue));

    // Mutex / ConditionVariable — single-threaded stubs. We don't have real
    // OS threads (Thread.new runs synchronously), so locks never contend and
    // condvars never need to actually wake anyone. Just enough surface for
    // fixtures (CyclicBarrier, ThreadSafeCounter, ...) to compile and run.
    let mutex = Rc::new(Class::new(
        "Mutex",
        Some(Rc::new(Class::new("Object", None))),
    ));
    globals.set("Mutex", Object::Class(mutex));
    let cv = Rc::new(Class::new(
        "ConditionVariable",
        Some(Rc::new(Class::new("Object", None))),
    ));
    globals.set("ConditionVariable", Object::Class(cv));

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
    globals.set("proc", Object::NativeFunction("proc".to_string()));
    globals.set("require", Object::NativeFunction("require".to_string()));
    globals.set(
        "require_relative",
        Object::NativeFunction("require_relative".to_string()),
    );
    globals.set("eval", Object::NativeFunction("eval".to_string()));
    globals.set("parse", Object::NativeFunction("parse".to_string()));
    globals.set("exit", Object::NativeFunction("exit".to_string()));
    globals.set("load", Object::NativeFunction("load".to_string()));
    // Refinements — `using` activates a refinement module (stub for now).
    globals.set("using", Object::NativeFunction("using".to_string()));

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
    globals.set(
        "private_constant",
        Object::NativeFunction("private_constant".to_string()),
    );
    globals.set(
        "public_constant",
        Object::NativeFunction("public_constant".to_string()),
    );
    globals.set(
        "deprecate_constant",
        Object::NativeFunction("deprecate_constant".to_string()),
    );
    globals.set("freeze", Object::NativeFunction("freeze".to_string()));
    // Lifecycle hooks — accept and discard the block, never run it.
    globals.set("at_exit", Object::NativeFunction("at_exit".to_string()));
    globals.set("END", Object::NativeFunction("at_exit".to_string()));
    globals.set(
        "trace_var",
        Object::NativeFunction("noop_with_block".to_string()),
    );
    globals.set(
        "untrace_var",
        Object::NativeFunction("noop_with_block".to_string()),
    );
    // Misc Kernel methods used by mspec
    globals.set("warn", Object::NativeFunction("warn".to_string()));
    globals.set("sprintf", Object::NativeFunction("sprintf".to_string()));
    globals.set("format", Object::NativeFunction("sprintf".to_string()));
    globals.set(
        "__method__",
        Object::NativeFunction("__method__".to_string()),
    );
    globals.set("caller", Object::NativeFunction("caller".to_string()));
    globals.set(
        "caller_locations",
        Object::NativeFunction("caller_locations".to_string()),
    );
    globals.set(
        "binding",
        Object::NativeFunction("binding_kernel".to_string()),
    );
    globals.set("rand", Object::NativeFunction("rand".to_string()));
    globals.set("srand", Object::NativeFunction("srand".to_string()));
    globals.set("sleep", Object::NativeFunction("sleep".to_string()));
    // Top-level `to_s` — Ruby's top-level self is "main", so bare to_s returns "main"
    globals.set("to_s", Object::NativeFunction("top_level_to_s".to_string()));
    // Top-level `define_method` — installs a method on Object (or current
    // class when invoked inside a class_eval / class body).
    globals.set(
        "define_method",
        Object::NativeFunction("define_method".to_string()),
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
