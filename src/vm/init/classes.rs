// Registering the core classes and the singleton values.

use super::*;

/// Initialize built-in methods for core classes.
pub(crate) fn initialize_builtin_methods(builtins: &BuiltinClasses) {
    builtin_classes::init_object_methods(builtins.object_class.as_ref());
    builtin_classes::init_proc_methods(builtins.proc_class.as_ref());
    builtin_classes::init_set_methods(builtins.set_class.as_ref());
    builtin_classes::init_regexp_methods(builtins.regexp_class.as_ref());
    builtin_classes::init_string_methods(builtins.string_class.as_ref());
    builtin_classes::init_symbol_methods(builtins.symbol_class.as_ref());
    builtin_classes::init_array_methods(builtins.array_class.as_ref());
    builtin_classes::init_integer_methods(builtins.integer_class.as_ref());
    builtin_classes::init_float_methods(builtins.float_class.as_ref());
    builtin_classes::init_hash_methods(builtins.hash_class.as_ref());
    builtin_classes::init_range_methods(builtins.range_class.as_ref());
    builtin_classes::init_exception_methods(builtins.exception_class.as_ref());
}

/// Register all built-in classes in the global registry.
pub(crate) fn register_builtin_classes(globals: &mut GlobalRegistry, builtins: &BuiltinClasses) {
    for (name, class) in builtins.all_classes() {
        globals.set(name, Object::Class(class));
    }
    // `initialize` answers natively rather than through a method entry. A
    // stub carrying the name is what the visibility checks read, and it hands
    // the call straight back to the native one.
    let mut stub =
        crate::object::Method::new("initialize".to_string(), vec!["args".to_string()], vec![]);
    stub.variadic_param = Some((0, "args".to_string()));
    stub.native_alias = Some("initialize".to_string());
    builtins
        .array_class
        .define_method("initialize", Rc::new(stub));
    builtins.array_class.set_method_private("initialize");
}

/// Register singleton values (nil, true, false) in the global registry.
pub(crate) fn register_singletons(globals: &mut GlobalRegistry) {
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
    // Metorex runs Ruby, so it names the engine the way the Ruby it follows
    // does. A test suite reads this to decide which of an implementation's
    // limits apply, and metorex carries Ruby's own.
    globals.set("RUBY_ENGINE", Object::string("ruby".to_string()));
    globals.set(
        "RUBY_PLATFORM",
        Object::string(crate::reported_ruby_platform()),
    );
    globals.set(
        "RUBY_DESCRIPTION",
        Object::string(crate::ruby_description()),
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
    globals.set(
        "RUBY_COPYRIGHT",
        Object::string(format!(
            "ruby - Copyright (C) 1993-2026 Yukihiro Matsumoto, metorex {}",
            env!("CARGO_PKG_VERSION")
        )),
    );
    globals.set(
        "RUBY_RELEASE_DATE",
        Object::string("2026-09-15".to_string()),
    );
    globals.set("RUBY_REVISION", Object::string("metorex".to_string()));
    // Every one of these names a string that does not change, and Ruby holds
    // them under a module of their own as well.
    let named_constants = [
        ("VERSION", "RUBY_VERSION"),
        ("PATCHLEVEL", "RUBY_PATCHLEVEL"),
        ("COPYRIGHT", "RUBY_COPYRIGHT"),
        ("DESCRIPTION", "RUBY_DESCRIPTION"),
        ("ENGINE", "RUBY_ENGINE"),
        ("ENGINE_VERSION", "RUBY_ENGINE_VERSION"),
        ("PLATFORM", "RUBY_PLATFORM"),
        ("RELEASE_DATE", "RUBY_RELEASE_DATE"),
        ("REVISION", "RUBY_REVISION"),
    ];
    let ruby_module = Class::new_module("Ruby");
    for (short, long) in named_constants {
        let Some(held) = globals.get(long) else {
            continue;
        };
        if let Object::String(text) = &held {
            text.mark_deduplicated();
        }
        ruby_module.set_class_var(short, held);
    }
    globals.set("Ruby", Object::Module(ruby_module));

    // BasicObject — Ruby's true root class. Object inherits from it.
    let basic_object = Class::new("BasicObject", None);
    globals.set("BasicObject", Object::Class(Rc::clone(&basic_object)));
    // Ruby's BasicObject holds a constant naming itself, which is what makes
    // `BasicObject::BasicObject` resolve.
    basic_object.set_class_var("BasicObject", Object::Class(Rc::clone(&basic_object)));

    // Object — root class that mspec reopens to inject describe/it/before/after
    let object = Class::new("Object", Some(Rc::clone(&basic_object)));
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
    let module_class = Class::new("Module", Some(Rc::clone(&object)));
    globals.set("Module", Object::Class(Rc::clone(&module_class)));
    // `include` and `prepend` answer natively, with no entry on Module to
    // find. A stub keeps them ahead of a method of the same name that a
    // program mixed into Object, which is where Ruby's own lookup stops.
    for name in ["include", "prepend"] {
        let held = crate::parser::ANONYMOUS_SPLAT.to_string();
        let mut stub = crate::object::Method::new(name.to_string(), vec![held.clone()], vec![]);
        stub.variadic_param = Some((0, held));
        stub.native_alias = Some(name.to_string());
        module_class.define_method(name, Rc::new(stub));
    }
    let class_class = Class::new("Class", Some(Rc::clone(&module_class)));
    // Class#initialize is private (Ruby semantics); the method itself is
    // implemented natively in call_class_methods, so the name only needs to
    // appear in Class's private_method_names for visibility checks.
    class_class.set_method_private("initialize");
    globals.set("Class", Object::Class(class_class));

    // The object a program runs against at the top level. Ruby calls it
    // `main`: an ordinary Object, whose singleton class is where a `def
    // self.name` written outside every class lands.
    let main = Object::Instance(crate::object::Instance::new(Rc::clone(&object)));
    globals.set("__main__", main.clone());
    // TOPLEVEL_BINDING — used by eval('code', TOPLEVEL_BINDING) at top level.
    globals.set(
        "TOPLEVEL_BINDING",
        Object::Binding(Rc::new(Binding::with_receiver(HashMap::new(), main))),
    );

    // TrueClass, FalseClass, NilClass — can't be instantiated
    let true_class = Class::new("TrueClass", Some(Rc::clone(&object)));
    globals.set("TrueClass", Object::Class(true_class));
    let false_class = Class::new("FalseClass", Some(Rc::clone(&object)));
    globals.set("FalseClass", Object::Class(false_class));
    let nil_class = Class::new("NilClass", Some(Rc::clone(&object)));
    globals.set("NilClass", Object::Class(nil_class));

    // Numeric types. Integer and Float already descend from a Numeric that
    // `register_builtin_classes` put here, and that is the one a program
    // reopens, so it is kept rather than replaced.
    let numeric = match globals.get("Numeric") {
        Some(Object::Class(existing)) => existing,
        _ => {
            let numeric = Class::new("Numeric", Some(Rc::clone(&object)));
            globals.set("Numeric", Object::Class(Rc::clone(&numeric)));
            numeric
        }
    };
    // A refinement is a module of its own kind, and Ruby gives it a class so
    // one can be told apart from an ordinary module.
    if let Some(Object::Class(module_class)) = globals.get("Module") {
        let refinement = Class::new("Refinement", Some(module_class));
        globals.set("Refinement", Object::Class(refinement));
    }
    let rational_class = Class::new("Rational", Some(Rc::clone(&numeric)));
    globals.set("Rational", Object::Class(rational_class));
    let complex_class = Class::new("Complex", Some(Rc::clone(&numeric)));
    globals.set("Complex", Object::Class(complex_class));
    // `Complex::I` is the imaginary unit, which the class carries as a
    // constant. It is built once the VM can make one, in the prelude.

    // Struct — the builder every `Struct.new(...)` class descends from.
    let object_class = match globals.get("Object") {
        Some(Object::Class(class)) => Some(class),
        _ => None,
    };
    let struct_class = Class::new("Struct", object_class);
    // Struct is created after the built-in modules are registered, so it takes
    // the Enumerable mixin here rather than in the loop over the others.
    if let Some(Object::Module(enumerable)) = globals.get("Enumerable") {
        struct_class.add_mixin(enumerable);
    }
    globals.set("Struct", Object::Class(struct_class));

    // Regexp — stub class so `case x; when Regexp; ...; end` and
    // `Regexp` and `Method` name the classes the interpreter already built,
    // which is what `obj.class == Method` compares against. A stub of the
    // same name here would stand in front of the real one and answer a
    // different object.
}
