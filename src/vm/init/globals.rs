// The special global variables, and the native functions a program
// reaches without a receiver.

use super::*;

/// Register Ruby special global variables.
pub(crate) fn register_special_globals(globals: &mut GlobalRegistry) {
    // $LOAD_PATH / $: — shared array
    let load_path = Object::Array(Rc::new(RefCell::new(Vec::new())));
    globals.set_variable(":", load_path.clone());
    globals.set_variable("LOAD_PATH", load_path);

    // $LOADED_FEATURES / $" — shared array. What the interpreter carries
    // itself is listed from the start, so a program that asks for one of
    // them is told it has it already.
    let carried: Vec<Object> = [
        "complex.so",
        "enumerator.so",
        "fiber.so",
        "pathname.so",
        "rational.so",
        "ruby2_keywords.rb",
        "set.rb",
        "thread.rb",
    ]
    .iter()
    .map(|named| Object::string((*named).to_string()))
    .collect();
    let loaded_features = Object::Array(Rc::new(RefCell::new(carried)));
    globals.set_variable("\"", loaded_features.clone());
    globals.set_variable("LOADED_FEATURES", loaded_features);

    // $stdout / $stderr / $stdin — placeholders
    globals.set_variable("stdout", Object::string("$stdout".to_string()));
    globals.set_variable("stderr", Object::string("$stderr".to_string()));
    globals.set_variable("stdin", Object::string("$stdin".to_string()));

    // $. — how many lines have been read, and $FILENAME — the file they came
    // from. Both follow ARGF as it walks the files it was handed.
    globals.set_variable(".", Object::Int(0));
    // With no file named, ARGF reads standard input, which Ruby names `-`.
    globals.set_variable("FILENAME", Object::string("-"));

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
pub(crate) fn register_native_functions(globals: &mut GlobalRegistry) {
    globals.set("puts", Object::NativeFunction("puts".to_string()));
    globals.set("print", Object::NativeFunction("print".to_string()));
    globals.set("p", Object::NativeFunction("p".to_string()));
    globals.set("gets", Object::NativeFunction("gets".to_string()));
    // ARGF — the stream `gets` reads from. An instance rather than a module,
    // so a singleton method can stand in for `gets` during a test.
    let argf_class = Class::new("ARGF.class", None);
    globals.set("ARGF.class", Object::Class(Rc::clone(&argf_class)));
    let argf = Object::instance(argf_class);
    globals.set("ARGF", argf.clone());
    // `$<` is the stream a program reads without naming one, which is ARGF
    // under the name Ruby's punctuation gives it.
    globals.set_variable("<", argf);
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
    globals.set("spawn", Object::NativeFunction("spawn".to_string()));
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
    globals.set(
        "__begin_once__",
        Object::NativeFunction("__begin_once__".to_string()),
    );
    globals.set(
        "__end_once__",
        Object::NativeFunction("__end_once__".to_string()),
    );
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
    // The pair behind `Timeout.timeout`, which name when the block it runs
    // has to be over and what to raise when it is.
    globals.set(
        "__timeout_open__",
        Object::NativeFunction("__timeout_open__".to_string()),
    );
    // What `callcc` reads to run its statement again for a continuation
    // called after the block returned.
    for name in [
        "__continuation_site__",
        "__continuation_resumed__",
        "__continuation_resume__",
    ] {
        globals.set(name, Object::NativeFunction(name.to_string()));
    }
    // Whether a value was given a singleton class, which decides the class
    // `Numeric#coerce` compares.
    globals.set(
        "__singleton_given__",
        Object::NativeFunction("__singleton_given__".to_string()),
    );
    for name in ["iterator?", "instance_variables_to_inspect"] {
        globals.set(name, Object::NativeFunction(name.to_string()));
    }
    for name in [
        "__console_mode_get__",
        "__console_mode_set__",
        "__console_mode_change__",
        "__console_mode_query__",
        "__console_winsize__",
        "__console_set_winsize__",
        "__console_flush__",
        "__console_beep__",
        "__console_ttyname__",
        "__prism_version__",
        "__prism_serialize__",
        "__prism_serialize_stream__",
        "__prism_parse_success__",
        "__prism_string_query__",
    ] {
        globals.set(name, Object::NativeFunction(name.to_string()));
    }
    globals.set(
        "__embedded_library_names__",
        Object::NativeFunction("__embedded_library_names__".to_string()),
    );
    globals.set(
        "__timeout_close__",
        Object::NativeFunction("__timeout_close__".to_string()),
    );
    globals.set(
        "__undefine_allocator__",
        Object::NativeFunction("__undefine_allocator__".to_string()),
    );
    globals.set(
        "__load_abstract_syntax_tree__",
        Object::NativeFunction("__load_abstract_syntax_tree__".to_string()),
    );
    globals.set(
        "__load_instruction_sequence__",
        Object::NativeFunction("__load_instruction_sequence__".to_string()),
    );
    globals.set(
        "__allocate_instance__",
        Object::NativeFunction("__allocate_instance__".to_string()),
    );
    globals.set(
        "__block_position__",
        Object::NativeFunction("__block_position__".to_string()),
    );
    globals.set(
        "__ractor_move__",
        Object::NativeFunction("__ractor_move__".to_string()),
    );
    // The one primitive behind the Math module, which the prelude wraps in a
    // method per function.
    globals.set(
        "__math_function__",
        Object::NativeFunction("__math_function__".to_string()),
    );
    globals.set(
        "__interpreter_path__",
        Object::NativeFunction("__interpreter_path__".to_string()),
    );
    globals.set(
        "__header_directory__",
        Object::NativeFunction("__header_directory__".to_string()),
    );
    for primitive in ["__weak_reference__", "__weak_target__", "__wrapped_size__"] {
        globals.set(primitive, Object::NativeFunction(primitive.to_string()));
    }
    globals.set(
        "__resolve_feature_path__",
        Object::NativeFunction("__resolve_feature_path__".to_string()),
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
pub(crate) fn seed_environment_with_globals(
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
