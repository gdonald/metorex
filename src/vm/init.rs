//! VM initialization functions.
//!
//! This module contains functions for initializing the virtual machine with built-in
//! classes, methods, and global values.

use super::GlobalRegistry;
use crate::builtin_classes::{self, BuiltinClasses};
use crate::class::Class;
use crate::environment::Environment;
use crate::object::{Binding, Object};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The encodings metorex names, as (constant, the name the encoding reports,
/// whether it is a dummy). Ruby spells most of the names by turning the
/// underscores into dashes, and the ones that break that rule carry their own
/// spelling here. A dummy encoding is one Ruby names and tags strings with but
/// cannot convert through.
/// The waiting, scheduling, and resource-limit settings the operating system
/// names by number, which Ruby carries as constants on Process.
pub(crate) const PROCESS_CONSTANTS: [(&str, i64); 22] = [
    ("WNOHANG", libc::WNOHANG as i64),
    ("WUNTRACED", libc::WUNTRACED as i64),
    ("PRIO_PROCESS", libc::PRIO_PROCESS as i64),
    ("PRIO_PGRP", libc::PRIO_PGRP as i64),
    ("PRIO_USER", libc::PRIO_USER as i64),
    ("RLIM_INFINITY", libc::RLIM_INFINITY as i64),
    // Both names stand for "no limit at all" on the systems that
    // carry them, which is the value RLIM_INFINITY holds.
    ("RLIM_SAVED_MAX", libc::RLIM_INFINITY as i64),
    ("RLIM_SAVED_CUR", libc::RLIM_INFINITY as i64),
    ("RLIMIT_CPU", libc::RLIMIT_CPU as i64),
    ("RLIMIT_FSIZE", libc::RLIMIT_FSIZE as i64),
    ("RLIMIT_DATA", libc::RLIMIT_DATA as i64),
    ("RLIMIT_STACK", libc::RLIMIT_STACK as i64),
    ("RLIMIT_CORE", libc::RLIMIT_CORE as i64),
    ("RLIMIT_AS", libc::RLIMIT_AS as i64),
    ("RLIMIT_MEMLOCK", libc::RLIMIT_MEMLOCK as i64),
    ("RLIMIT_NPROC", libc::RLIMIT_NPROC as i64),
    ("RLIMIT_NOFILE", libc::RLIMIT_NOFILE as i64),
    ("RLIMIT_RSS", libc::RLIMIT_RSS as i64),
    ("CLOCK_REALTIME", libc::CLOCK_REALTIME as i64),
    ("CLOCK_MONOTONIC", libc::CLOCK_MONOTONIC as i64),
    (
        "CLOCK_PROCESS_CPUTIME_ID",
        libc::CLOCK_PROCESS_CPUTIME_ID as i64,
    ),
    (
        "CLOCK_THREAD_CPUTIME_ID",
        libc::CLOCK_THREAD_CPUTIME_ID as i64,
    ),
];

pub(crate) const ENCODING_NAMES: [(&str, &str, bool); 48] = [
    ("UTF_8", "UTF-8", false),
    ("US_ASCII", "US-ASCII", false),
    ("ASCII_8BIT", "ASCII-8BIT", false),
    ("BINARY", "ASCII-8BIT", false),
    ("UTF_16", "UTF-16", false),
    ("UTF_16BE", "UTF-16BE", false),
    ("UTF_16LE", "UTF-16LE", false),
    ("UTF_32", "UTF-32", false),
    ("UTF_32BE", "UTF-32BE", false),
    ("UTF_32LE", "UTF-32LE", false),
    ("ISO_8859_1", "ISO-8859-1", false),
    ("ISO_8859_2", "ISO-8859-2", false),
    ("ISO_8859_3", "ISO-8859-3", false),
    ("ISO_8859_4", "ISO-8859-4", false),
    ("ISO_8859_5", "ISO-8859-5", false),
    ("ISO_8859_6", "ISO-8859-6", false),
    ("ISO_8859_7", "ISO-8859-7", false),
    ("ISO_8859_8", "ISO-8859-8", false),
    ("ISO_8859_9", "ISO-8859-9", false),
    ("ISO_8859_10", "ISO-8859-10", false),
    ("ISO_8859_11", "ISO-8859-11", false),
    ("ISO_8859_13", "ISO-8859-13", false),
    ("ISO_8859_14", "ISO-8859-14", false),
    ("ISO_8859_15", "ISO-8859-15", false),
    ("ISO_8859_16", "ISO-8859-16", false),
    ("EUC_JP", "EUC-JP", false),
    ("EUC_KR", "EUC-KR", false),
    ("EUC_TW", "EUC-TW", false),
    ("EUC_CN", "EUC-CN", false),
    ("Shift_JIS", "Shift_JIS", false),
    ("SHIFT_JIS", "Shift_JIS", false),
    ("Windows_31J", "Windows-31J", false),
    ("KOI8_R", "KOI8-R", false),
    ("KOI8_U", "KOI8-U", false),
    ("Big5", "Big5", false),
    ("BIG5", "Big5", false),
    ("Emacs_Mule", "Emacs-Mule", false),
    ("EMACS_MULE", "Emacs-Mule", false),
    ("GB18030", "GB18030", false),
    ("GBK", "GBK", false),
    ("IBM437", "IBM437", false),
    ("IBM866", "IBM866", false),
    ("MacJapanese", "MacJapanese", false),
    // The dummy encodings: Ruby names them and tags strings with them, but
    // converts nothing through them.
    ("ISO_2022_JP", "ISO-2022-JP", true),
    ("ISO_2022_JP_2", "ISO-2022-JP-2", true),
    ("UTF_7", "UTF-7", true),
    ("Stateless_ISO_2022_JP", "stateless-ISO-2022-JP", true),
    ("STATELESS_ISO_2022_JP", "stateless-ISO-2022-JP", true),
];

/// The flags `File.open` accepts in `flags:`, and the ones a glob or fnmatch
/// is narrowed with. File, IO, and File::Constants all carry them.
const FILE_OPEN_FLAGS: [(&str, i64); 15] = [
    ("RDONLY", 0),
    ("WRONLY", 1),
    ("RDWR", 2),
    ("CREAT", 0o100),
    ("EXCL", 0o200),
    ("TRUNC", 0o1000),
    ("APPEND", 0o2000),
    ("NONBLOCK", 0o4000),
    ("FNM_NOESCAPE", 0x01),
    ("FNM_PATHNAME", 0x02),
    ("FNM_DOTMATCH", 0x04),
    ("FNM_CASEFOLD", 0x08),
    ("FNM_EXTGLOB", 0x10),
    ("FNM_SYSCASE", 0),
    ("FNM_SHORTNAME", 0),
];

/// Initialize built-in methods for core classes.
pub(super) fn initialize_builtin_methods(builtins: &BuiltinClasses) {
    builtin_classes::init_object_methods(builtins.object_class.as_ref());
    builtin_classes::init_proc_methods(builtins.proc_class.as_ref());
    builtin_classes::init_set_methods(builtins.set_class.as_ref());
    builtin_classes::init_regexp_methods(builtins.regexp_class.as_ref());
    builtin_classes::init_string_methods(builtins.string_class.as_ref());
    builtin_classes::init_array_methods(builtins.array_class.as_ref());
    builtin_classes::init_integer_methods(builtins.integer_class.as_ref());
    builtin_classes::init_float_methods(builtins.float_class.as_ref());
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
    globals.set(
        "RUBY_VERSION",
        Object::string(crate::reported_ruby_version()),
    );
    globals.set("RUBY_ENGINE", Object::string("metorex".to_string()));
    globals.set(
        "RUBY_PLATFORM",
        Object::string(crate::reported_ruby_platform()),
    );
    globals.set(
        "RUBY_DESCRIPTION",
        Object::string(format!(
            "metorex {} (ruby-compatible) [{}]",
            crate::reported_ruby_version(),
            crate::reported_ruby_platform()
        )),
    );
    // The patch number the version carries, which Ruby reports apart from
    // the version itself.
    globals.set(
        "RUBY_PATCHLEVEL",
        Object::Int(
            crate::reported_ruby_version()
                .split('.')
                .nth(2)
                .and_then(|held| held.parse::<i64>().ok())
                .unwrap_or(0),
        ),
    );
    globals.set(
        "RUBY_ENGINE_VERSION",
        Object::string(crate::reported_ruby_version()),
    );
    // Standard IO stream placeholders (used as constants like STDOUT/STDERR/STDIN)
    globals.set("STDOUT", Object::string("STDOUT".to_string()));
    globals.set("STDERR", Object::string("STDERR".to_string()));
    globals.set("STDIN", Object::string("STDIN".to_string()));

    // BasicObject — Ruby's true root class. Object inherits from it.
    let basic_object = Rc::new(Class::new("BasicObject", None));
    globals.set("BasicObject", Object::Class(Rc::clone(&basic_object)));
    // Ruby's BasicObject holds a constant naming itself, which is what makes
    // `BasicObject::BasicObject` resolve.
    basic_object.set_class_var("BasicObject", Object::Class(Rc::clone(&basic_object)));

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

    // Numeric types. Integer and Float already descend from a Numeric that
    // `register_builtin_classes` put here, and that is the one a program
    // reopens, so it is kept rather than replaced.
    let numeric = match globals.get("Numeric") {
        Some(Object::Class(existing)) => existing,
        _ => {
            let numeric = Rc::new(Class::new("Numeric", Some(Rc::clone(&object))));
            globals.set("Numeric", Object::Class(Rc::clone(&numeric)));
            numeric
        }
    };
    // A refinement is a module of its own kind, and Ruby gives it a class so
    // one can be told apart from an ordinary module.
    if let Some(Object::Class(module_class)) = globals.get("Module") {
        let refinement = Rc::new(Class::new("Refinement", Some(module_class)));
        globals.set("Refinement", Object::Class(refinement));
    }
    let rational_class = Rc::new(Class::new("Rational", Some(Rc::clone(&numeric))));
    globals.set("Rational", Object::Class(rational_class));
    let complex_class = Rc::new(Class::new("Complex", Some(Rc::clone(&numeric))));
    globals.set("Complex", Object::Class(complex_class));
    // `Complex::I` is the imaginary unit, which the class carries as a
    // constant. It is built once the VM can make one, in the prelude.

    // Struct — the builder every `Struct.new(...)` class descends from.
    let object_class = match globals.get("Object") {
        Some(Object::Class(class)) => Some(class),
        _ => None,
    };
    let struct_class = Rc::new(Class::new("Struct", object_class));
    // Struct is created after the built-in modules are registered, so it takes
    // the Enumerable mixin here rather than in the loop over the others.
    if let Some(Object::Module(enumerable)) = globals.get("Enumerable") {
        struct_class.add_mixin(enumerable);
    }
    globals.set("Struct", Object::Class(struct_class));

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
    // Exception descends from Object, the way every other class does, and it
    // has to be the same Object the rest of the world sees.
    let object_class = match globals.get("Object") {
        Some(Object::Class(object_class)) => object_class,
        _ => Rc::new(Class::new("Object", None)),
    };
    let exception = Rc::new(Class::new("Exception", Some(object_class)));
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
    let eof_error = Rc::new(Class::new("EOFError", Some(Rc::clone(&io_error))));
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
    let signal_exception = Rc::new(Class::new("SignalException", Some(Rc::clone(&exception))));
    // Ruby puts Interrupt under SignalException, not directly under Exception.
    let interrupt = Rc::new(Class::new("Interrupt", Some(Rc::clone(&signal_exception))));
    // The remaining built-in exception classes, so the hierarchy is complete.
    let no_memory_error = Rc::new(Class::new("NoMemoryError", Some(Rc::clone(&exception))));
    let security_error = Rc::new(Class::new("SecurityError", Some(Rc::clone(&exception))));
    let system_stack_error = Rc::new(Class::new("SystemStackError", Some(Rc::clone(&exception))));
    let fiber_error = Rc::new(Class::new("FiberError", Some(Rc::clone(&standard_error))));
    let thread_error = Rc::new(Class::new("ThreadError", Some(Rc::clone(&standard_error))));
    let closed_queue_error = Rc::new(Class::new(
        "ClosedQueueError",
        Some(Rc::clone(&stop_iteration)),
    ));
    let system_call_error = Rc::new(Class::new(
        "SystemCallError",
        Some(Rc::clone(&standard_error)),
    ));
    let errno_module = Rc::new(Class::new_module("Errno"));
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
    let uncaught_throw_error = Rc::new(Class::new(
        "UncaughtThrowError",
        Some(Rc::clone(&argument_error)),
    ));
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
    globals.set("EOFError", Object::Class(eof_error));
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
    let system_call_error_for_errno = Rc::clone(&system_call_error);
    globals.set("SystemCallError", Object::Class(system_call_error));
    register_errno_classes(&errno_module, &system_call_error_for_errno);
    globals.set("NoMemoryError", Object::Class(no_memory_error));
    globals.set("SecurityError", Object::Class(security_error));
    globals.set("SystemStackError", Object::Class(system_stack_error));
    globals.set("FiberError", Object::Class(fiber_error));
    globals.set("ThreadError", Object::Class(thread_error));
    globals.set("ClosedQueueError", Object::Class(closed_queue_error));
    globals.set("Errno", Object::Module(errno_module));
    globals.set("EncodingError", Object::Class(encoding_error));
    globals.set("FrozenError", Object::Class(frozen_error));
    globals.set("LocalJumpError", Object::Class(local_jump_error));
    globals.set("RegexpError", Object::Class(regexp_error));
    globals.set("UncaughtThrowError", Object::Class(uncaught_throw_error));
    globals.set(
        "Math::DomainError",
        Object::Class(Rc::clone(&math_domain_error)),
    );
    // The Math module holds it as a constant too, which is how the qualified
    // name resolves. Math itself was registered with the other modules first.
    if let Some(Object::Module(math) | Object::Class(math)) = globals.get("Math") {
        math.set_class_var("DomainError", Object::Class(math_domain_error));
    }
}

/// Register built-in modules (Comparable, Enumerable, Kernel, etc.).
pub(super) fn register_builtin_modules(globals: &mut GlobalRegistry, builtins: &BuiltinClasses) {
    // Comparable — stub module, methods will be added later. Ruby mixes it
    // into the classes whose values have an order, which is what
    // `Integer.include?(Comparable)` reports.
    let comparable = Rc::new(Class::new_module("Comparable"));
    for name in ["Numeric", "Integer", "Float", "String", "Symbol"] {
        if let Some(Object::Class(class)) = globals.get(name) {
            class.add_mixin(Rc::clone(&comparable));
        }
    }
    globals.set("Comparable", Object::Module(comparable));

    // Enumerable — stub module, mixed into the classes whose values can be
    // walked, which is what `Array.include?(Enumerable)` reports.
    let enumerable = Rc::new(Class::new_module("Enumerable"));
    for name in ["Array", "Hash", "Range", "Set", "Struct", "Enumerator"] {
        if let Some(Object::Class(class)) = globals.get(name) {
            class.add_mixin(Rc::clone(&enumerable));
        }
    }
    globals.set("Enumerable", Object::Module(enumerable));

    // Kernel — stub module mixed into Object so its instance methods are
    // available to every object (matches Ruby's standard ancestor chain).
    // The mixin link is established here against whatever Object class is
    // currently in globals at this point. Note that Object is replaced again
    // later in register_singletons, so `wire_kernel_into_object` is called
    // from VirtualMachine::new() after that step to re-establish the link.
    let kernel = Rc::new(Class::new_module("Kernel"));
    globals.set("Kernel", Object::Module(kernel));

    // Encoding — metorex strings are UTF-8, so the named encodings exist as
    // distinct objects but every string reports UTF-8.
    let encoding = Rc::new(Class::new("Encoding", None));
    // Two constants that name the same encoding, such as BINARY and
    // ASCII_8BIT, reach one object, so `Encoding.find` on the name it reports
    // answers the same encoding whichever constant it came from.
    let mut built: HashMap<&str, Rc<Class>> = HashMap::new();
    for (name, display, _) in ENCODING_NAMES {
        let constant = Rc::clone(
            built
                .entry(display)
                .or_insert_with(|| Rc::new(Class::new(display, Some(Rc::clone(&encoding))))),
        );
        encoding.set_class_var(name, Object::Class(Rc::clone(&constant)));
        globals.set(format!("Encoding::{}", name), Object::Class(constant));
    }
    // `Encoding.list` reports each encoding once, however many names reach it.
    let mut listed: Vec<Object> = Vec::new();
    for (_, display, _) in ENCODING_NAMES {
        if let Some(found) = built.get(display)
            && !listed
                .iter()
                .any(|held| matches!(held, Object::Class(class) if Rc::ptr_eq(class, found)))
        {
            listed.push(Object::Class(Rc::clone(found)));
        }
    }
    globals.set(
        "__Encoding_list",
        Object::Array(Rc::new(RefCell::new(listed))),
    );
    // The errors Encoding raises are constants on it, and descend from
    // StandardError the way every other one does.
    if let Some(Object::Class(standard_error)) = globals.get("StandardError") {
        for name in [
            "CompatibilityError",
            "ConverterNotFoundError",
            "UndefinedConversionError",
            "InvalidByteSequenceError",
        ] {
            let error = Rc::new(Class::new(
                format!("Encoding::{}", name),
                Some(Rc::clone(&standard_error)),
            ));
            encoding.set_class_var(name, Object::Class(Rc::clone(&error)));
            globals.set(format!("Encoding::{}", name), Object::Class(error));
        }
    }
    globals.set("Encoding", Object::Class(encoding));

    // The open flags `File.open` and `Kernel#open` accept in `flags:`.
    if let Some(Object::Class(file_class)) = globals.get("File") {
        for (name, value) in FILE_OPEN_FLAGS {
            file_class.set_class_var(name, Object::Int(value));
            globals.set(format!("File::{}", name), Object::Int(value));
        }
    }

    // File::Constants carries the open and match flags, and File includes it,
    // which is where `File.include?(File::Constants)` reads them from.
    if let Some(Object::Class(file_class)) = globals.get("File") {
        let constants = Rc::new(Class::new_module("File::Constants"));
        for (name, value) in FILE_OPEN_FLAGS {
            constants.set_class_var(name, Object::Int(value));
        }
        file_class.set_class_var("Constants", Object::Module(Rc::clone(&constants)));
        globals.set("File::Constants", Object::Module(Rc::clone(&constants)));
        file_class.add_mixin(constants);
    }

    // A file reads as a sequence of lines, so File carries Enumerable the way
    // IO does.
    if let (Some(Object::Class(file_class)), Some(Object::Module(enumerable))) =
        (globals.get("File"), globals.get("Enumerable"))
    {
        file_class.add_mixin(enumerable);
    }

    // The path that discards everything written to it.
    if let Some(Object::Class(file_class)) = globals.get("File") {
        let null = Object::string("/dev/null");
        file_class.set_class_var("NULL", null.clone());
        globals.set("File::NULL", null);
    }

    // File::Separator and its aliases, which a path built by hand uses.
    if let Some(Object::Class(file_class)) = globals.get("File") {
        for name in ["Separator", "SEPARATOR"] {
            let separator = Object::string("/");
            file_class.set_class_var(name, separator.clone());
            globals.set(format!("File::{}", name), separator);
        }
        let alt = Object::Nil;
        file_class.set_class_var("ALT_SEPARATOR", alt.clone());
        globals.set("File::ALT_SEPARATOR", alt);
        let path_separator = Object::string(":");
        file_class.set_class_var("PATH_SEPARATOR", path_separator.clone());
        globals.set("File::PATH_SEPARATOR", path_separator);
    }

    // Signal — stub module (trap is a no-op)
    let signal = Rc::new(Class::new_module("Signal"));
    globals.set("Signal", Object::Module(signal));

    // Process — stub module (pid is a no-op)
    let process = Rc::new(Class::new_module("Process"));
    // `Process::Status` describes how a child ended. The instances come from
    // whatever waits for one, and this is the class they share.
    let process_status = Rc::new(Class::new("Process::Status", None));
    process.set_class_var("Status", Object::Class(Rc::clone(&process_status)));
    globals.set("Process::Status", Object::Class(Rc::clone(&process_status)));
    globals.set("__Process_Status_class", Object::Class(process_status));
    // The numbers the operating system names its waiting, scheduling, and
    // resource settings by, which Ruby carries as constants on Process.
    for (name, value) in PROCESS_CONSTANTS {
        process.set_class_var(name, Object::Int(value));
        globals.set(format!("Process::{}", name), Object::Int(value));
    }
    // `Process::GID`, `Process::UID`, and `Process::Sys` name the same ids
    // Process itself does, gathered under the words Ruby gathers them under.
    for named in ["GID", "UID", "Sys"] {
        let holder = Rc::new(Class::new_module(format!("Process::{}", named)));
        process.set_class_var(named, Object::Module(Rc::clone(&holder)));
        globals.set(format!("Process::{}", named), Object::Module(holder));
    }
    globals.set("Process", Object::Module(process));

    // Math — stub module (constants will be added later if needed)
    let math = Rc::new(Class::new_module("Math"));
    // The two constants Math carries, which every trigonometric answer is
    // measured against.
    math.set_class_var("PI", Object::Float(std::f64::consts::PI));
    math.set_class_var("E", Object::Float(std::f64::consts::E));
    globals.set("Math", Object::Module(math));

    // GC — no-op stub
    let gc = Rc::new(Class::new_module("GC"));
    globals.set("GC", Object::Module(gc));

    // ObjectSpace — no-op stub
    let object_space = Rc::new(Class::new_module("ObjectSpace"));
    globals.set("ObjectSpace", Object::Module(object_space));

    // Warning — `Warning[:category]` reads and `Warning[:category] = bool`
    // writes the per-category warning switches. Categories are stored as
    // class variables on the module and start off, matching MRI's default
    // for `:deprecated`.
    let warning = Rc::new(Class::new_module("Warning"));
    globals.set("Warning", Object::Module(warning));

    // Time / IO — placeholder stubs (used in mspec)
    let time = Rc::new(Class::new(
        "Time",
        Some(Rc::new(Class::new("Object", None))),
    ));
    globals.set("Time", Object::Class(time));
    // File already stands under IO, so the global name has to reach that same
    // class rather than a second one wearing the name.
    let io = Rc::clone(&builtins.io_class);
    // An IO reads as a sequence of lines, and it answers the open flags under
    // its own name, both of which Ruby arranges by including these two.
    if let Some(Object::Module(constants)) = globals.get("File::Constants") {
        io.add_mixin(constants);
    }
    if let Some(Object::Module(enumerable)) = globals.get("Enumerable") {
        io.add_mixin(enumerable);
    }
    globals.set("IO", Object::Class(io));

    // Thread — stub
    let thread = Rc::new(Class::new(
        "Thread",
        Some(Rc::new(Class::new("Object", None))),
    ));
    globals.set("Thread", Object::Class(Rc::clone(&thread)));

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
    // Ruby names both of these under Thread as well as at the top level, and
    // the two names reach the same class.
    if let (Some(queue_class), Some(sized_class)) =
        (globals.get("Queue"), globals.get("SizedQueue"))
    {
        thread.set_class_var("Queue", queue_class.clone());
        thread.set_class_var("SizedQueue", sized_class.clone());
        globals.set("Thread::Queue", queue_class);
        globals.set("Thread::SizedQueue", sized_class);
    }

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
    let mut env_map = IndexMap::new();
    for (k, v) in std::env::vars() {
        env_map.insert(k, Object::string(v));
    }
    globals.set("ENV", Object::Dict(Rc::new(RefCell::new(env_map))));
}

/// Register Ruby special global variables.
pub(super) fn register_special_globals(globals: &mut GlobalRegistry) {
    // $LOAD_PATH / $: — shared array
    let load_path = Object::Array(Rc::new(RefCell::new(Vec::new())));
    globals.set_variable(":", load_path.clone());
    globals.set_variable("LOAD_PATH", load_path);

    // $LOADED_FEATURES / $" — shared array
    let loaded_features = Object::Array(Rc::new(RefCell::new(Vec::new())));
    globals.set_variable("\"", loaded_features.clone());
    globals.set_variable("LOADED_FEATURES", loaded_features);

    // $stdout / $stderr / $stdin — placeholders
    globals.set_variable("stdout", Object::string("$stdout".to_string()));
    globals.set_variable("stderr", Object::string("$stderr".to_string()));
    globals.set_variable("stdin", Object::string("$stdin".to_string()));

    // $. — how many lines have been read, and $FILENAME — the file they came
    // from. Both follow ARGF as it walks the files it was handed.
    globals.set_variable(".", Object::Int(0));
    globals.set_variable("FILENAME", Object::Nil);

    // $$ — this process's own id, which a script prints to say which one it
    // is running as.
    globals.set_variable("$", Object::Int(std::process::id() as i64));

    // $0 / $PROGRAM_NAME — set later by main when file is known
    globals.set_variable("0", Object::string(String::new()));
    globals.set_variable("PROGRAM_NAME", Object::string(String::new()));

    // $; $, $/ $\ — string separator globals
    globals.set_variable(";", Object::Nil);
    globals.set_variable(",", Object::Nil);
    globals.set_variable("/", Object::string("\n".to_string()));
    globals.set_variable("\\", Object::Nil);

    // $! $@ $~ $& — exception/regex globals
    globals.set_variable("!", Object::Nil);
    globals.set_variable("@", Object::Nil);
    globals.set_variable("~", Object::Nil);
    globals.set_variable("&", Object::Nil);

    // $? — process status
    globals.set_variable("?", Object::Nil);

    // $_ — last input line
    globals.set_variable("_", Object::Nil);

    // $. — line number
    globals.set_variable(".", Object::Int(0));

    // $DEBUG / $VERBOSE
    globals.set_variable("DEBUG", Object::Bool(false));
    globals.set_variable("VERBOSE", Object::Bool(false));
}

/// Register native functions in the global registry.
pub(super) fn register_native_functions(globals: &mut GlobalRegistry) {
    globals.set("puts", Object::NativeFunction("puts".to_string()));
    globals.set("print", Object::NativeFunction("print".to_string()));
    globals.set("p", Object::NativeFunction("p".to_string()));
    globals.set("gets", Object::NativeFunction("gets".to_string()));
    // ARGF — the stream `gets` reads from. An instance rather than a module,
    // so a singleton method can stand in for `gets` during a test.
    let argf_class = Rc::new(Class::new("ARGF.class", None));
    globals.set("ARGF.class", Object::Class(Rc::clone(&argf_class)));
    globals.set("ARGF", Object::instance(argf_class));
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
    globals.set("lambda", Object::NativeFunction("lambda".to_string()));
    globals.set("loop", Object::NativeFunction("loop".to_string()));
    globals.set("raise", Object::NativeFunction("raise".to_string()));
    globals.set("readline", Object::NativeFunction("readline".to_string()));
    globals.set("readlines", Object::NativeFunction("readlines".to_string()));
    globals.set(
        "local_variables",
        Object::NativeFunction("local_variables".to_string()),
    );
    globals.set("proc", Object::NativeFunction("proc".to_string()));
    globals.set("require", Object::NativeFunction("require".to_string()));
    // `autoload` and `autoload?` named bare register on Object, the home of
    // top-level constants.
    globals.set("`", Object::NativeFunction("`".to_string()));
    globals.set("exec", Object::NativeFunction("exec".to_string()));
    globals.set("exit!", Object::NativeFunction("exit!".to_string()));
    globals.set("fork", Object::NativeFunction("fork".to_string()));
    globals.set("open", Object::NativeFunction("open".to_string()));
    globals.set("pp", Object::NativeFunction("pp".to_string()));
    globals.set("printf", Object::NativeFunction("printf".to_string()));
    // `chomp` and `chop` without a receiver work on `$_`, the line `-n` read.
    globals.set("chomp", Object::NativeFunction("chomp".to_string()));
    globals.set("chop", Object::NativeFunction("chop".to_string()));
    globals.set("autoload", Object::NativeFunction("autoload".to_string()));
    globals.set("autoload?", Object::NativeFunction("autoload?".to_string()));
    globals.set(
        "require_relative",
        Object::NativeFunction("require_relative".to_string()),
    );
    globals.set("eval", Object::NativeFunction("eval".to_string()));
    globals.set("parse", Object::NativeFunction("parse".to_string()));
    globals.set("exit", Object::NativeFunction("exit".to_string()));
    globals.set("exit!", Object::NativeFunction("exit!".to_string()));
    globals.set("abort", Object::NativeFunction("abort".to_string()));
    globals.set("system", Object::NativeFunction("system".to_string()));
    globals.set("fork", Object::NativeFunction("fork".to_string()));
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
    globals.set("trace_var", Object::NativeFunction("trace_var".to_string()));
    globals.set(
        "untrace_var",
        Object::NativeFunction("untrace_var".to_string()),
    );
    // Misc Kernel methods used by mspec
    globals.set("warn", Object::NativeFunction("warn".to_string()));
    // `trap` is Kernel's name for `Signal.trap`.
    globals.set("trap", Object::NativeFunction("trap".to_string()));
    globals.set("sprintf", Object::NativeFunction("sprintf".to_string()));
    globals.set("format", Object::NativeFunction("sprintf".to_string()));
    globals.set(
        "__method__",
        Object::NativeFunction("__method__".to_string()),
    );
    globals.set(
        "__callee__",
        Object::NativeFunction("__callee__".to_string()),
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
    globals.set("fail", Object::NativeFunction("fail".to_string()));
    globals.set(
        "global_variables",
        Object::NativeFunction("global_variables".to_string()),
    );
    globals.set("catch", Object::NativeFunction("catch".to_string()));
    globals.set("throw", Object::NativeFunction("throw".to_string()));
    globals.set("rand", Object::NativeFunction("rand".to_string()));
    globals.set("srand", Object::NativeFunction("srand".to_string()));
    globals.set("sleep", Object::NativeFunction("sleep".to_string()));
    // The one primitive behind the Math module, which the prelude wraps in a
    // method per function.
    globals.set(
        "__math_function__",
        Object::NativeFunction("__math_function__".to_string()),
    );
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
        // A global variable is held under its name with the `$` dropped, so
        // seeding it here would put `$DEBUG` in front of a program's own
        // `DEBUG` constant. The two are different names in Ruby.
        if globals.constant(name).is_none() {
            continue;
        }
        environment.define(name.clone(), value.clone());
    }
}

/// Where an Errno class keeps the message its number stands for.
pub(crate) const ERRNO_MESSAGE_KEY: &str = "__errno_message__";

/// Every `Errno::EXXX` class the platform names, each a subclass of
/// SystemCallError carrying its own number in an `Errno` constant. The numbers
/// and the names come from libc rather than a table, since they differ between
/// Linux and macOS.
fn register_errno_classes(errno_module: &Rc<Class>, system_call_error: &Rc<Class>) {
    #[cfg(target_os = "macos")]
    const ERRNO_NUMBERS: &[(&str, i32)] = &[
        ("EPERM", libc::EPERM),
        ("ENOENT", libc::ENOENT),
        ("ESRCH", libc::ESRCH),
        ("EINTR", libc::EINTR),
        ("EIO", libc::EIO),
        ("ENXIO", libc::ENXIO),
        ("E2BIG", libc::E2BIG),
        ("ENOEXEC", libc::ENOEXEC),
        ("EBADF", libc::EBADF),
        ("ECHILD", libc::ECHILD),
        ("EDEADLK", libc::EDEADLK),
        ("ENOMEM", libc::ENOMEM),
        ("EACCES", libc::EACCES),
        ("EFAULT", libc::EFAULT),
        ("ENOTBLK", libc::ENOTBLK),
        ("EBUSY", libc::EBUSY),
        ("EEXIST", libc::EEXIST),
        ("EXDEV", libc::EXDEV),
        ("ENODEV", libc::ENODEV),
        ("ENOTDIR", libc::ENOTDIR),
        ("EISDIR", libc::EISDIR),
        ("EINVAL", libc::EINVAL),
        ("ENFILE", libc::ENFILE),
        ("EMFILE", libc::EMFILE),
        ("ENOTTY", libc::ENOTTY),
        ("ETXTBSY", libc::ETXTBSY),
        ("EFBIG", libc::EFBIG),
        ("ENOSPC", libc::ENOSPC),
        ("ESPIPE", libc::ESPIPE),
        ("EROFS", libc::EROFS),
        ("EMLINK", libc::EMLINK),
        ("EPIPE", libc::EPIPE),
        ("EDOM", libc::EDOM),
        ("ERANGE", libc::ERANGE),
        ("EAGAIN", libc::EAGAIN),
        ("EINPROGRESS", libc::EINPROGRESS),
        ("EALREADY", libc::EALREADY),
        ("ENOTSOCK", libc::ENOTSOCK),
        ("EDESTADDRREQ", libc::EDESTADDRREQ),
        ("EMSGSIZE", libc::EMSGSIZE),
        ("EPROTOTYPE", libc::EPROTOTYPE),
        ("ENOPROTOOPT", libc::ENOPROTOOPT),
        ("EPROTONOSUPPORT", libc::EPROTONOSUPPORT),
        ("ESOCKTNOSUPPORT", libc::ESOCKTNOSUPPORT),
        ("ENOTSUP", libc::ENOTSUP),
        ("EPFNOSUPPORT", libc::EPFNOSUPPORT),
        ("EAFNOSUPPORT", libc::EAFNOSUPPORT),
        ("EADDRINUSE", libc::EADDRINUSE),
        ("EADDRNOTAVAIL", libc::EADDRNOTAVAIL),
        ("ENETDOWN", libc::ENETDOWN),
        ("ENETUNREACH", libc::ENETUNREACH),
        ("ENETRESET", libc::ENETRESET),
        ("ECONNABORTED", libc::ECONNABORTED),
        ("ECONNRESET", libc::ECONNRESET),
        ("ENOBUFS", libc::ENOBUFS),
        ("EISCONN", libc::EISCONN),
        ("ENOTCONN", libc::ENOTCONN),
        ("ESHUTDOWN", libc::ESHUTDOWN),
        ("ETOOMANYREFS", libc::ETOOMANYREFS),
        ("ETIMEDOUT", libc::ETIMEDOUT),
        ("ECONNREFUSED", libc::ECONNREFUSED),
        ("ELOOP", libc::ELOOP),
        ("ENAMETOOLONG", libc::ENAMETOOLONG),
        ("EHOSTDOWN", libc::EHOSTDOWN),
        ("EHOSTUNREACH", libc::EHOSTUNREACH),
        ("ENOTEMPTY", libc::ENOTEMPTY),
        ("EPROCLIM", libc::EPROCLIM),
        ("EUSERS", libc::EUSERS),
        ("EDQUOT", libc::EDQUOT),
        ("ESTALE", libc::ESTALE),
        ("EREMOTE", libc::EREMOTE),
        ("EBADRPC", libc::EBADRPC),
        ("ERPCMISMATCH", libc::ERPCMISMATCH),
        ("EPROGUNAVAIL", libc::EPROGUNAVAIL),
        ("EPROGMISMATCH", libc::EPROGMISMATCH),
        ("EPROCUNAVAIL", libc::EPROCUNAVAIL),
        ("ENOLCK", libc::ENOLCK),
        ("ENOSYS", libc::ENOSYS),
        ("EFTYPE", libc::EFTYPE),
        ("EAUTH", libc::EAUTH),
        ("ENEEDAUTH", libc::ENEEDAUTH),
        ("EPWROFF", libc::EPWROFF),
        ("EDEVERR", libc::EDEVERR),
        ("EOVERFLOW", libc::EOVERFLOW),
        ("EBADEXEC", libc::EBADEXEC),
        ("EBADARCH", libc::EBADARCH),
        ("ESHLIBVERS", libc::ESHLIBVERS),
        ("EBADMACHO", libc::EBADMACHO),
        ("ECANCELED", libc::ECANCELED),
        ("EIDRM", libc::EIDRM),
        ("ENOMSG", libc::ENOMSG),
        ("EILSEQ", libc::EILSEQ),
        ("ENOATTR", libc::ENOATTR),
        ("EBADMSG", libc::EBADMSG),
        ("EMULTIHOP", libc::EMULTIHOP),
        ("ENODATA", libc::ENODATA),
        ("ENOLINK", libc::ENOLINK),
        ("ENOSR", libc::ENOSR),
        ("ENOSTR", libc::ENOSTR),
        ("EPROTO", libc::EPROTO),
        ("ETIME", libc::ETIME),
        ("EOPNOTSUPP", libc::EOPNOTSUPP),
        ("ENOPOLICY", libc::ENOPOLICY),
        ("ENOTRECOVERABLE", libc::ENOTRECOVERABLE),
        ("EOWNERDEAD", libc::EOWNERDEAD),
        ("EQFULL", libc::EQFULL),
    ];
    #[cfg(not(target_os = "macos"))]
    const ERRNO_NUMBERS: &[(&str, i32)] = &[
        ("EPERM", libc::EPERM),
        ("ENOENT", libc::ENOENT),
        ("ESRCH", libc::ESRCH),
        ("EINTR", libc::EINTR),
        ("EIO", libc::EIO),
        ("ENXIO", libc::ENXIO),
        ("E2BIG", libc::E2BIG),
        ("ENOEXEC", libc::ENOEXEC),
        ("EBADF", libc::EBADF),
        ("ECHILD", libc::ECHILD),
        ("EAGAIN", libc::EAGAIN),
        ("ENOMEM", libc::ENOMEM),
        ("EACCES", libc::EACCES),
        ("EFAULT", libc::EFAULT),
        ("ENOTBLK", libc::ENOTBLK),
        ("EBUSY", libc::EBUSY),
        ("EEXIST", libc::EEXIST),
        ("EXDEV", libc::EXDEV),
        ("ENODEV", libc::ENODEV),
        ("ENOTDIR", libc::ENOTDIR),
        ("EISDIR", libc::EISDIR),
        ("EINVAL", libc::EINVAL),
        ("ENFILE", libc::ENFILE),
        ("EMFILE", libc::EMFILE),
        ("ENOTTY", libc::ENOTTY),
        ("ETXTBSY", libc::ETXTBSY),
        ("EFBIG", libc::EFBIG),
        ("ENOSPC", libc::ENOSPC),
        ("ESPIPE", libc::ESPIPE),
        ("EROFS", libc::EROFS),
        ("EMLINK", libc::EMLINK),
        ("EPIPE", libc::EPIPE),
        ("EDOM", libc::EDOM),
        ("ERANGE", libc::ERANGE),
        ("EDEADLK", libc::EDEADLK),
        ("ENAMETOOLONG", libc::ENAMETOOLONG),
        ("ENOLCK", libc::ENOLCK),
        ("ENOSYS", libc::ENOSYS),
        ("ENOTEMPTY", libc::ENOTEMPTY),
        ("ELOOP", libc::ELOOP),
        ("ENOMSG", libc::ENOMSG),
        ("EIDRM", libc::EIDRM),
        ("ECHRNG", libc::ECHRNG),
        ("EL2NSYNC", libc::EL2NSYNC),
        ("EL3HLT", libc::EL3HLT),
        ("EL3RST", libc::EL3RST),
        ("ELNRNG", libc::ELNRNG),
        ("EUNATCH", libc::EUNATCH),
        ("ENOCSI", libc::ENOCSI),
        ("EL2HLT", libc::EL2HLT),
        ("EBADE", libc::EBADE),
        ("EBADR", libc::EBADR),
        ("EXFULL", libc::EXFULL),
        ("ENOANO", libc::ENOANO),
        ("EBADRQC", libc::EBADRQC),
        ("EBADSLT", libc::EBADSLT),
        ("EBFONT", libc::EBFONT),
        ("ENOSTR", libc::ENOSTR),
        ("ENODATA", libc::ENODATA),
        ("ETIME", libc::ETIME),
        ("ENOSR", libc::ENOSR),
        ("ENONET", libc::ENONET),
        ("ENOPKG", libc::ENOPKG),
        ("EREMOTE", libc::EREMOTE),
        ("ENOLINK", libc::ENOLINK),
        ("EADV", libc::EADV),
        ("ESRMNT", libc::ESRMNT),
        ("ECOMM", libc::ECOMM),
        ("EPROTO", libc::EPROTO),
        ("EMULTIHOP", libc::EMULTIHOP),
        ("EDOTDOT", libc::EDOTDOT),
        ("EBADMSG", libc::EBADMSG),
        ("EOVERFLOW", libc::EOVERFLOW),
        ("ENOTUNIQ", libc::ENOTUNIQ),
        ("EBADFD", libc::EBADFD),
        ("EREMCHG", libc::EREMCHG),
        ("ELIBACC", libc::ELIBACC),
        ("ELIBBAD", libc::ELIBBAD),
        ("ELIBSCN", libc::ELIBSCN),
        ("ELIBMAX", libc::ELIBMAX),
        ("ELIBEXEC", libc::ELIBEXEC),
        ("EILSEQ", libc::EILSEQ),
        ("ERESTART", libc::ERESTART),
        ("ESTRPIPE", libc::ESTRPIPE),
        ("EUSERS", libc::EUSERS),
        ("ENOTSOCK", libc::ENOTSOCK),
        ("EDESTADDRREQ", libc::EDESTADDRREQ),
        ("EMSGSIZE", libc::EMSGSIZE),
        ("EPROTOTYPE", libc::EPROTOTYPE),
        ("ENOPROTOOPT", libc::ENOPROTOOPT),
        ("EPROTONOSUPPORT", libc::EPROTONOSUPPORT),
        ("ESOCKTNOSUPPORT", libc::ESOCKTNOSUPPORT),
        ("EOPNOTSUPP", libc::EOPNOTSUPP),
        ("EPFNOSUPPORT", libc::EPFNOSUPPORT),
        ("EAFNOSUPPORT", libc::EAFNOSUPPORT),
        ("EADDRINUSE", libc::EADDRINUSE),
        ("EADDRNOTAVAIL", libc::EADDRNOTAVAIL),
        ("ENETDOWN", libc::ENETDOWN),
        ("ENETUNREACH", libc::ENETUNREACH),
        ("ENETRESET", libc::ENETRESET),
        ("ECONNABORTED", libc::ECONNABORTED),
        ("ECONNRESET", libc::ECONNRESET),
        ("ENOBUFS", libc::ENOBUFS),
        ("EISCONN", libc::EISCONN),
        ("ENOTCONN", libc::ENOTCONN),
        ("ESHUTDOWN", libc::ESHUTDOWN),
        ("ETOOMANYREFS", libc::ETOOMANYREFS),
        ("ETIMEDOUT", libc::ETIMEDOUT),
        ("ECONNREFUSED", libc::ECONNREFUSED),
        ("EHOSTDOWN", libc::EHOSTDOWN),
        ("EHOSTUNREACH", libc::EHOSTUNREACH),
        ("EALREADY", libc::EALREADY),
        ("EINPROGRESS", libc::EINPROGRESS),
        ("ESTALE", libc::ESTALE),
        ("EUCLEAN", libc::EUCLEAN),
        ("ENOTNAM", libc::ENOTNAM),
        ("ENAVAIL", libc::ENAVAIL),
        ("EISNAM", libc::EISNAM),
        ("EREMOTEIO", libc::EREMOTEIO),
        ("EDQUOT", libc::EDQUOT),
        ("ENOMEDIUM", libc::ENOMEDIUM),
        ("EMEDIUMTYPE", libc::EMEDIUMTYPE),
        ("ECANCELED", libc::ECANCELED),
        ("ENOKEY", libc::ENOKEY),
        ("EKEYEXPIRED", libc::EKEYEXPIRED),
        ("EKEYREVOKED", libc::EKEYREVOKED),
        ("EKEYREJECTED", libc::EKEYREJECTED),
        ("EOWNERDEAD", libc::EOWNERDEAD),
        ("ENOTRECOVERABLE", libc::ENOTRECOVERABLE),
        ("ERFKILL", libc::ERFKILL),
        ("EHWPOISON", libc::EHWPOISON),
    ];
    for (name, number) in ERRNO_NUMBERS {
        let class = Rc::new(Class::new(
            format!("Errno::{}", name),
            Some(Rc::clone(system_call_error)),
        ));
        class.set_class_var("Errno", Object::Int(*number as i64));
        // The message the errno stands for, which an instance reports when
        // no custom one is given. Kept under a mangled key so it does not
        // show up as a Ruby-visible constant.
        class.set_class_var(
            ERRNO_MESSAGE_KEY,
            Object::string(errno_description(*number)),
        );
        errno_module.set_class_var(*name, Object::Class(class));
    }
    // `Errno::NOERROR` stands for a successful call, which Ruby defines
    // alongside the numbered errors.
    let no_error = Rc::new(Class::new(
        "Errno::NOERROR",
        Some(Rc::clone(system_call_error)),
    ));
    no_error.set_class_var("Errno", Object::Int(0));
    no_error.set_class_var(ERRNO_MESSAGE_KEY, Object::string("Success".to_string()));
    errno_module.set_class_var("NOERROR", Object::Class(no_error));
    // Ruby aliases these where the platform gives them the same number.
    for (alias, canonical) in [
        ("EWOULDBLOCK", "EAGAIN"),
        ("ENOTSUP", "EOPNOTSUPP"),
        ("EDEADLOCK", "EDEADLK"),
    ] {
        if errno_module.get_class_var(alias).is_none()
            && let Some(existing) = errno_module.get_class_var(canonical)
        {
            errno_module.set_class_var(alias, existing);
        }
    }
}

/// What the C library calls the number, which is the message Ruby reports for
/// it. Rust renders an os error as "Invalid argument (os error 22)", and Ruby
/// reports only the first half.
pub(crate) fn errno_description(number: i32) -> String {
    let rendered = std::io::Error::from_raw_os_error(number).to_string();
    match rendered.rfind(" (os error ") {
        Some(cut) => rendered[..cut].to_string(),
        None => rendered,
    }
}
