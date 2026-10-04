// Registering the core modules.

use super::*;

/// Register built-in modules (Comparable, Enumerable, Kernel, etc.).
pub(crate) fn register_builtin_modules(globals: &mut GlobalRegistry, builtins: &BuiltinClasses) {
    // Comparable — stub module, methods will be added later. Ruby mixes it
    // into the classes whose values have an order, which is what
    // `Integer.include?(Comparable)` reports.
    let comparable = Class::new_module("Comparable");
    // Integer and Float reach Comparable through Numeric, so only the classes
    // Ruby mixes it into directly carry it.
    for name in ["Numeric", "String", "Symbol"] {
        if let Some(Object::Class(class)) = globals.get(name) {
            class.add_mixin(Rc::clone(&comparable));
        }
    }
    globals.set("Comparable", Object::Module(comparable));

    // Enumerable — stub module, mixed into the classes whose values can be
    // walked, which is what `Array.include?(Enumerable)` reports.
    let enumerable = Class::new_module("Enumerable");
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
    let kernel = Class::new_module("Kernel");
    // The Object every primitive answers is the one the builtins hold, so
    // Kernel is mixed into it as well as into the one registered later.
    builtins.object_class.add_mixin(Rc::clone(&kernel));
    globals.set("Kernel", Object::Module(kernel));

    // Encoding — metorex strings are UTF-8, so the named encodings exist as
    // distinct objects but every string reports UTF-8.
    let encoding = Class::new("Encoding", None);
    // Two constants that name the same encoding, such as BINARY and
    // ASCII_8BIT, reach one object, so `Encoding.find` on the name it reports
    // answers the same encoding whichever constant it came from.
    let mut built: HashMap<&str, Rc<Class>> = HashMap::new();
    for (name, display, _) in ENCODING_NAMES {
        let constant = Rc::clone(
            built
                .entry(display)
                .or_insert_with(|| Class::new(display, Some(Rc::clone(&encoding)))),
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
        let constants = Class::new_module("File::Constants");
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
    let signal = Class::new_module("Signal");
    globals.set("Signal", Object::Module(signal));

    // Process — stub module (pid is a no-op)
    let process = Class::new_module("Process");
    // `Process::Status` describes how a child ended. The instances come from
    // whatever waits for one, and this is the class they share.
    let process_status = Class::new("Process::Status", None);
    process.set_class_var("Status", Object::Class(Rc::clone(&process_status)));
    globals.set("Process::Status", Object::Class(Rc::clone(&process_status)));
    globals.set("__Process_Status_class", Object::Class(process_status));
    // The numbers the operating system names its waiting, scheduling, and
    // resource settings by, which Ruby carries as constants on Process.
    for &(name, value) in PROCESS_CONSTANTS {
        process.set_class_var(name, Object::Int(value));
        globals.set(format!("Process::{}", name), Object::Int(value));
    }
    // `Process::GID`, `Process::UID`, and `Process::Sys` name the same ids
    // Process itself does, gathered under the words Ruby gathers them under.
    for named in ["GID", "UID", "Sys"] {
        let holder = Class::new_module(format!("Process::{}", named));
        process.set_class_var(named, Object::Module(Rc::clone(&holder)));
        globals.set(format!("Process::{}", named), Object::Module(holder));
    }
    globals.set("Process", Object::Module(process));

    // Math — stub module (constants will be added later if needed)
    let math = Class::new_module("Math");
    // The two constants Math carries, which every trigonometric answer is
    // measured against.
    math.set_class_var("PI", Object::Float(std::f64::consts::PI));
    math.set_class_var("E", Object::Float(std::f64::consts::E));
    globals.set("Math", Object::Module(math));

    // GC — no-op stub
    let gc = Class::new_module("GC");
    globals.set("GC", Object::Module(gc));

    // ObjectSpace — no-op stub
    let object_space = Class::new_module("ObjectSpace");
    globals.set("ObjectSpace", Object::Module(object_space));

    // Warning — `Warning[:category]` reads and `Warning[:category] = bool`
    // writes the per-category warning switches. Categories are stored as
    // class variables on the module and start off, matching MRI's default
    // for `:deprecated`.
    let warning = Class::new_module("Warning");
    globals.set("Warning", Object::Module(warning));

    // Time / IO — placeholder stubs (used in mspec)
    let time = Class::new("Time", Some(Class::new("Object", None)));
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
    let thread = Class::new("Thread", Some(Class::new("Object", None)));
    globals.set("Thread", Object::Class(Rc::clone(&thread)));

    // Fiber — a block that runs on a stack of its own and suspends part-way
    // through. Its methods are answered natively, since the coroutine behind
    // one lives in the interpreter rather than in the object.
    let fiber = Class::new("Fiber", Some(Class::new("Object", None)));
    globals.set("Fiber", Object::Class(Rc::clone(&fiber)));
    let fiber_error = Class::new("FiberError", Some(Class::new("StandardError", None)));
    globals.set("FiberError", Object::Class(fiber_error));

    // Queue, SizedQueue, Mutex, and ConditionVariable live under Thread, and
    // Ruby names each at the top level too. The two names reach the same
    // class.
    // A SizedQueue is a Queue with a limit.
    let queue = Class::new("Thread::Queue", Some(Class::new("Object", None)));
    for (short, full, made) in [
        ("Queue", "Thread::Queue", Rc::clone(&queue)),
        (
            "SizedQueue",
            "Thread::SizedQueue",
            Class::new("Thread::SizedQueue", Some(queue)),
        ),
        (
            "Mutex",
            "Thread::Mutex",
            Class::new("Thread::Mutex", Some(Class::new("Object", None))),
        ),
        (
            "ConditionVariable",
            "Thread::ConditionVariable",
            Class::new(
                "Thread::ConditionVariable",
                Some(Class::new("Object", None)),
            ),
        ),
    ] {
        let made = Object::Class(made);
        thread.set_class_var(short, made.clone());
        globals.set(full, made.clone());
        globals.set(short, made);
    }

    // ENV — use a Dict so ENV['KEY'] works. Keys are plain strings (no quotes)
    // because object_to_dict_key returns the raw String for Object::String.
    // A program reads its environment in the encoding the locale names, and
    // bytes that do not fit that encoding stand for themselves.
    let charmap = crate::vm::locale_charmap_name();
    let mut env_map = IndexMap::new();
    for (k, v) in std::env::vars() {
        let named = match charmap.as_str() {
            "" => "UTF-8".to_string(),
            "ANSI_X3.4-1968" | "US-ASCII" if !v.is_ascii() => "ASCII-8BIT".to_string(),
            "ANSI_X3.4-1968" => "US-ASCII".to_string(),
            other => other.to_string(),
        };
        env_map.insert(
            k,
            Object::String(Rc::new(crate::object::StringValue::with_encoding(v, named))),
        );
    }
    globals.set("ENV", Object::Dict(Rc::new(RefCell::new(env_map))));
}
