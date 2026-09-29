// Registering the exception classes.

use super::*;

/// Register standard Ruby exception class hierarchy as stubs.
pub(crate) fn register_exception_classes(globals: &mut GlobalRegistry) {
    // Reuse Exception/StandardError/RuntimeError/TypeError already in BuiltinClasses;
    // add the remaining subclasses that specs and mspec reference.
    // Exception descends from Object, the way every other class does, and it
    // has to be the same Object the rest of the world sees.
    let object_class = match globals.get("Object") {
        Some(Object::Class(object_class)) => object_class,
        _ => Class::new("Object", None),
    };
    let exception = Class::new("Exception", Some(object_class));
    let standard_error = Class::new("StandardError", Some(Rc::clone(&exception)));
    let runtime_error = Class::new("RuntimeError", Some(Rc::clone(&standard_error)));
    let name_error = Class::new("NameError", Some(Rc::clone(&standard_error)));
    let no_method_error = Class::new("NoMethodError", Some(Rc::clone(&name_error)));
    let argument_error = Class::new("ArgumentError", Some(Rc::clone(&standard_error)));
    let type_error = Class::new("TypeError", Some(Rc::clone(&standard_error)));
    let range_error = Class::new("RangeError", Some(Rc::clone(&standard_error)));
    let io_error = Class::new("IOError", Some(Rc::clone(&standard_error)));
    let eof_error = Class::new("EOFError", Some(Rc::clone(&io_error)));
    let index_error = Class::new("IndexError", Some(Rc::clone(&standard_error)));
    let key_error = Class::new("KeyError", Some(Rc::clone(&index_error)));
    let stop_iteration = Class::new("StopIteration", Some(Rc::clone(&index_error)));
    let zero_division_error = Class::new("ZeroDivisionError", Some(Rc::clone(&standard_error)));
    let float_domain_error = Class::new("FloatDomainError", Some(Rc::clone(&range_error)));
    let script_error = Class::new("ScriptError", Some(Rc::clone(&exception)));
    let load_error = Class::new("LoadError", Some(Rc::clone(&script_error)));
    let syntax_error = Class::new("SyntaxError", Some(Rc::clone(&script_error)));
    let not_implemented_error = Class::new("NotImplementedError", Some(Rc::clone(&script_error)));
    let system_exit = Class::new("SystemExit", Some(Rc::clone(&exception)));
    let signal_exception = Class::new("SignalException", Some(Rc::clone(&exception)));
    // Ruby puts Interrupt under SignalException, not directly under Exception.
    let interrupt = Class::new("Interrupt", Some(Rc::clone(&signal_exception)));
    // The remaining built-in exception classes, so the hierarchy is complete.
    let no_memory_error = Class::new("NoMemoryError", Some(Rc::clone(&exception)));
    let security_error = Class::new("SecurityError", Some(Rc::clone(&exception)));
    let system_stack_error = Class::new("SystemStackError", Some(Rc::clone(&exception)));
    let fiber_error = Class::new("FiberError", Some(Rc::clone(&standard_error)));
    let thread_error = Class::new("ThreadError", Some(Rc::clone(&standard_error)));
    let closed_queue_error = Class::new("ClosedQueueError", Some(Rc::clone(&stop_iteration)));
    let system_call_error = Class::new("SystemCallError", Some(Rc::clone(&standard_error)));
    let errno_module = Class::new_module("Errno");
    let encoding_error = Class::new("EncodingError", Some(Rc::clone(&standard_error)));
    let frozen_error = Class::new("FrozenError", Some(Rc::clone(&runtime_error)));
    // A pattern that covers no value raises this, and a hash pattern missing
    // a key it named raises the one below it.
    let no_matching_pattern_error =
        Class::new("NoMatchingPatternError", Some(Rc::clone(&standard_error)));
    let no_matching_pattern_key_error = Class::new(
        "NoMatchingPatternKeyError",
        Some(Rc::clone(&no_matching_pattern_error)),
    );
    let local_jump_error = Class::new("LocalJumpError", Some(Rc::clone(&standard_error)));
    let regexp_error = Class::new("RegexpError", Some(Rc::clone(&standard_error)));
    let uncaught_throw_error = Class::new("UncaughtThrowError", Some(Rc::clone(&argument_error)));
    let math_domain_error = Class::new("Math::DomainError", Some(Rc::clone(&argument_error)));

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
    // The errors Encoding raises are constants on it, and each one is an
    // EncodingError, which is what lets a rescue name them all at once.
    if let Some(Object::Class(encoding)) = globals.get("Encoding") {
        for name in [
            "CompatibilityError",
            "ConverterNotFoundError",
            "UndefinedConversionError",
            "InvalidByteSequenceError",
        ] {
            let error = Class::new(
                format!("Encoding::{}", name),
                Some(Rc::clone(&encoding_error)),
            );
            encoding.set_class_var(name, Object::Class(Rc::clone(&error)));
            globals.set(format!("Encoding::{}", name), Object::Class(error));
        }
    }
    globals.set("EncodingError", Object::Class(encoding_error));
    globals.set("FrozenError", Object::Class(frozen_error));
    globals.set(
        "NoMatchingPatternError",
        Object::Class(no_matching_pattern_error),
    );
    globals.set(
        "NoMatchingPatternKeyError",
        Object::Class(no_matching_pattern_key_error),
    );
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
