// What the core classes answer natively, and the ancestor walks that
// decide which of them a receiver reaches.

use super::*;

/// The visibility state a bare `module_function` sets in a module body. Every
/// method defined afterwards is copied to the module object and made private
/// as an instance method.
pub(crate) const MODULE_FUNCTION_VISIBILITY: &str = "module_function";

/// Kernel's process-control functions, which Ruby exposes as private instance
/// methods on Kernel and as public singleton methods on the module.
/// Whether a name is one of the mangled keys metorex stores alongside real
/// methods. Ruby's own `__`-prefixed methods, such as `__send__`, are not
/// among them and stay visible.
pub(crate) fn is_internal_method_key(name: &str) -> bool {
    const INTERNAL_PREFIXES: &[&str] = &[
        "__class__",
        "__ext__",
        "__refine__",
        "__singleton__",
        "__attached__",
        "__module_body_class__",
        "__struct_",
        "__refinement_label__",
    ];
    INTERNAL_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

/// BasicObject's public instance methods, which are native rather than
/// entries in its method table.
pub(crate) const NATIVE_BASIC_OBJECT_METHODS: &[&str] = &[
    "!",
    "!=",
    "==",
    "__id__",
    "__send__",
    "equal?",
    "instance_eval",
    "instance_exec",
];

/// BasicObject's private instance methods.
pub(crate) const BASIC_OBJECT_PRIVATE_METHODS: &[&str] = &[
    "initialize",
    "method_missing",
    "singleton_method_added",
    "singleton_method_removed",
    "singleton_method_undefined",
];

pub(crate) const KERNEL_PRIVATE_FUNCTIONS: &[&str] = &[
    "`",
    "abort",
    "caller",
    "caller_locations",
    "chomp",
    "chop",
    "eval",
    "exec",
    "exit",
    "exit!",
    "fork",
    "format",
    "load",
    "open",
    "spawn",
    "sprintf",
    "at_exit",
    "autoload",
    "autoload?",
    "binding",
    "block_given?",
    "catch",
    "fail",
    "gets",
    "global_variables",
    "initialize_clone",
    "initialize_copy",
    "initialize_dup",
    "lambda",
    "local_variables",
    "loop",
    "p",
    "pp",
    "print",
    "printf",
    "proc",
    "raise",
    "rand",
    "readline",
    "readlines",
    "require",
    "require_relative",
    "respond_to_missing?",
    "sleep",
    "srand",
    "system",
    "putc",
    "puts",
    "throw",
    "trace_var",
    "trap",
    "untrace_var",
    "warn",
];

/// The hooks Module defines as private instance methods with a no-op default
/// implementation. Each takes one argument and returns nil unless the module
/// overrides it.
pub(crate) const MODULE_PRIVATE_HOOKS: &[&str] = &[
    "append_features",
    "prepend_features",
    "extend_object",
    "extended",
    "included",
    "prepended",
    "const_added",
    "method_added",
    "method_removed",
    "method_undefined",
];

/// Module's private instance methods beyond the hooks: the declarations a
/// class or module body calls without a receiver. `alias_method` and
/// `define_method` are deliberately absent, being public in Ruby.
pub(crate) const MODULE_PRIVATE_DECLARATIONS: &[&str] = &[
    MODULE_FUNCTION_VISIBILITY,
    "private",
    "public",
    "protected",
    "remove_const",
];

/// The native Module and Class instance methods metorex implements, with the
/// parameters each takes and whether the last one is variadic.
/// `Module#instance_methods` advertises the names, and `Object#method` builds
/// a callable stub from the parameter list.
pub(crate) const NATIVE_MODULE_METHODS: &[(&str, &[&str], bool)] = &[
    ("alias_method", &["new_name", "old_name"], false),
    ("attr", &["names"], true),
    ("attr_accessor", &["names"], true),
    ("attr_reader", &["names"], true),
    ("attr_writer", &["names"], true),
    ("constants", &["inherit"], true),
    ("define_method", &["name", "body"], true),
    ("include", &["modules"], true),
    ("method_defined?", &["name", "inherit"], true),
    ("prepend", &["modules"], true),
    ("instance_method", &["name"], false),
    ("remove_method", &["names"], true),
    ("undef_method", &["names"], true),
    ("public_instance_method", &["name"], false),
    ("protected_instance_methods", &["include_super"], true),
    ("instance_methods", &["include_super"], true),
    ("public_instance_methods", &["include_super"], true),
    ("private_instance_methods", &["include_super"], true),
    ("module_function", &["names"], true),
    ("name", &[], false),
];

/// The Kernel methods `call_object_method` implements natively, with the
/// parameter list each one takes, so `obj.method(:name)` can hand out a stub
/// whose `arity` matches Ruby's. A trailing `true` marks the last parameter
/// variadic.
pub(crate) const NATIVE_KERNEL_METHODS: &[(&str, &[&str], bool)] = &[
    ("class", &[], false),
    ("clone", &["options"], true),
    ("dup", &[], false),
    ("eql?", &["other"], false),
    ("eval", &["arguments"], true),
    ("equal?", &["other"], false),
    ("extend", &["modules"], true),
    ("freeze", &[], false),
    ("frozen?", &[], false),
    ("hash", &[], false),
    ("inspect", &[], false),
    ("instance_of?", &["klass"], false),
    ("instance_variable_get", &["name"], false),
    ("instance_variable_set", &["name", "value"], false),
    ("instance_variables", &[], false),
    ("is_a?", &["klass"], false),
    ("itself", &[], false),
    ("kind_of?", &["klass"], false),
    ("lambda", &[], false),
    ("proc", &[], false),
    ("raise", &["arguments"], true),
    ("method", &["name"], false),
    ("public_method", &["name"], false),
    ("remove_instance_variable", &["name"], false),
    ("singleton_methods", &["all"], true),
    ("methods", &["regular"], true),
    ("nil?", &[], false),
    ("object_id", &[], false),
    ("public_send", &["arguments"], true),
    ("require", &["path"], false),
    ("require_relative", &["path"], false),
    ("respond_to?", &["arguments"], true),
    ("respond_to_missing?", &["name", "include_private"], false),
    ("send", &["arguments"], true),
    ("tap", &[], false),
    ("to_s", &[], false),
    ("__id__", &[], false),
    ("__send__", &["arguments"], true),
    ("instance_exec", &["arguments"], true),
    ("instance_eval", &["arguments"], true),
    ("warn", &["messages"], true),
];

/// A body-less stub for one of the natively implemented Kernel methods.
pub(crate) fn native_kernel_method_stub(name: &str) -> Option<Method> {
    let (_, parameters, variadic) = NATIVE_KERNEL_METHODS
        .iter()
        .find(|(entry, _, _)| *entry == name)?;
    let mut stub = Method::with_owner(
        name.to_string(),
        parameters.iter().map(|p| (*p).to_string()).collect(),
        vec![],
        "Kernel".to_string(),
    );
    if *variadic {
        let last = parameters.len().saturating_sub(1);
        stub.variadic_param = Some((last, parameters[last].to_string()));
    }
    Some(stub)
}

/// The NameError `Module#instance_method` raises for a name that is not
/// defined, or has been removed with `undef_method`. Ruby exposes the missing
/// name through `NameError#name`.
pub(crate) fn undefined_instance_method_error(
    name: &str,
    class_rc: &Rc<Class>,
    position: Position,
) -> MetorexError {
    let msg = format!("undefined method '{}' for {}", name, class_rc.name());
    let exc = Object::exception("NameError", msg.clone());
    if let Object::Exception(cell) = &exc {
        cell.borrow_mut().name = Some(name.to_string());
    }
    MetorexError::UncaughtException {
        exception: exc,
        location: position_to_location(position),
        message: msg,
    }
}

/// A body-less stub for one of the natively implemented Module methods,
/// carrying its parameter list so `arity` and `bind` behave. Invoking it
/// reaches the same native implementation.
pub(crate) fn native_module_method_stub(name: &str) -> Option<Method> {
    let (_, parameters, variadic) = NATIVE_MODULE_METHODS
        .iter()
        .find(|(entry, _, _)| *entry == name)?;
    let unnamed: Vec<String> = parameters
        .iter()
        .map(|_| crate::object::UNNAMED_PARAMETER.to_string())
        .collect();
    let mut stub = Method::with_owner(
        name.to_string(),
        unnamed.clone(),
        vec![],
        "Module".to_string(),
    );
    if *variadic {
        let last = unnamed.len().saturating_sub(1);
        stub.variadic_param = Some((last, unnamed[last].clone()));
    }
    Some(stub)
}

/// Whether a name is one of the methods an exception answers natively, on a
/// class that holds exceptions.
pub(crate) fn answers_exception_method(
    vm: &VirtualMachine,
    class_rc: &Rc<Class>,
    name: &str,
) -> bool {
    holds_exceptions(vm, class_rc)
        && crate::vm::native_methods::exception_methods::NATIVE_EXCEPTION_METHODS.contains(&name)
}

/// Whether a class stands for exceptions, either because it is one of their
/// classes or because it is the singleton class of an exception.
pub(crate) fn holds_exceptions(vm: &VirtualMachine, class_rc: &Rc<Class>) -> bool {
    if vm.is_exception_class(class_rc) {
        return true;
    }
    class_rc.is_singleton_class()
        && matches!(
            class_rc.get_class_var("__attached__"),
            Some(Object::Exception(_))
        )
}

/// How `undef_method` names its receiver. The metaclass of a class or module
/// is reported as that class, while any other singleton class is reported by
/// its own display.
pub(crate) fn undef_target_name(class_rc: &Rc<Class>) -> String {
    if class_rc.is_singleton_class()
        && let Some(Object::Class(attached) | Object::Module(attached)) =
            class_rc.get_class_var("__attached__")
    {
        return attached.inspect_name();
    }
    class_rc.inspect_name()
}

/// Whether `name` reads as a constant path, which `set_temporary_name`
/// rejects: a `::`-separated chain whose every segment is a constant name,
/// with an optional leading `::`.
pub(crate) fn looks_like_constant_path(name: &str) -> bool {
    let path = name.strip_prefix("::").unwrap_or(name);
    !path.is_empty() && path.split("::").all(is_valid_constant_name)
}

/// Whether the receiver answers `name` through a method of its own: an
/// instance method, a `def self.name` class method, or a singleton method.
/// Used to let a user-defined accessor win over a native handler.
pub(crate) fn has_user_defined_method(class_rc: &Rc<Class>, name: &str) -> bool {
    class_rc.find_method(name).is_some()
        || class_rc
            .find_method(&format!("__class__{}", name))
            .is_some()
        || class_rc
            .singleton_class_slot()
            .as_ref()
            .is_some_and(|sc| sc.find_method(name).is_some())
}

/// Append the transitive ancestor chain of a module (including itself and all
/// modules it mixes in, recursively) onto `chain`. Uses pointer identity in
/// `seen` to skip modules that have already been added, matching Ruby's
/// dedup-on-first-sighting semantics.
/// Append the modules prepended to `owner`, ahead of `owner` itself. Each one
/// gets a fresh visited set: Ruby lists a module once per place it was mixed
/// in, so a module prepended here still appears again where a superclass or an
/// include already carried it.
pub(crate) fn push_prepend_ancestors(owner: &Rc<Class>, chain: &mut Vec<Object>) {
    for prepended in owner.prepend_chain() {
        let mut prepend_seen: Vec<*const Class> = Vec::new();
        push_module_ancestors(&prepended, chain, &mut prepend_seen);
    }
}

pub(crate) fn push_module_ancestors(
    module: &Rc<Class>,
    chain: &mut Vec<Object>,
    seen: &mut Vec<*const Class>,
) {
    let ptr = Rc::as_ptr(module);
    if seen.contains(&ptr) {
        return;
    }
    seen.push(ptr);
    push_prepend_ancestors(module, chain);
    chain.push(Object::Module(Rc::clone(module)));
    for mixin in module.mixin_chain() {
        push_module_ancestors(&mixin, chain, seen);
    }
}

/// Append the full ancestor chain of a class (class itself, its mixins
/// recursively, then each superclass with its own mixins) onto `chain`.
pub(crate) fn push_class_ancestors(
    class: &Rc<Class>,
    chain: &mut Vec<Object>,
    seen: &mut Vec<*const Class>,
) {
    let ptr = Rc::as_ptr(class);
    if !seen.contains(&ptr) {
        seen.push(ptr);
        push_prepend_ancestors(class, chain);
        chain.push(Object::Class(Rc::clone(class)));
    }
    for mixin in class.mixin_chain() {
        push_module_ancestors(&mixin, chain, seen);
    }
    let mut current = class.superclass();
    while let Some(parent) = current {
        let pptr = Rc::as_ptr(&parent);
        if !seen.contains(&pptr) {
            seen.push(pptr);
            push_prepend_ancestors(&parent, chain);
            chain.push(Object::Class(Rc::clone(&parent)));
        }
        for mixin in parent.mixin_chain() {
            push_module_ancestors(&mixin, chain, seen);
        }
        current = parent.superclass();
    }
}

/// Kernel methods that `call_object_method` implements natively, so a
/// body-less stub can stand in for them in `Object.instance_method`.
/// The names an open IO handle answers to. Their bodies live in the native
/// dispatch tables rather than in the handle class's method map, so this is
/// what `respond_to?` has to consult for one.
pub(crate) fn is_native_io_method(name: &str) -> bool {
    matches!(
        name,
        "<<" | "each"
            | "each_line"
            | "eof"
            | "eof?"
            | "fileno"
            | "flush"
            | "getc"
            | "gets"
            | "lineno"
            | "lineno="
            | "path"
            | "pos"
            | "pos="
            | "print"
            | "putc"
            | "puts"
            | "read"
            | "readbyte"
            | "readchar"
            | "readline"
            | "readlines"
            | "rewind"
            | "seek"
            | "sync"
            | "sync="
            | "tell"
            | "to_io"
            | "write"
    )
}

pub(crate) fn is_native_kernel_method(name: &str) -> bool {
    // Kernel's private functions are native too, so an UnboundMethod for one
    // is available the same way.
    if KERNEL_PRIVATE_FUNCTIONS.contains(&name) {
        return true;
    }
    matches!(
        name,
        "class"
            | "clone"
            | "dup"
            | "eql?"
            | "equal?"
            | "extend"
            | "freeze"
            | "frozen?"
            | "hash"
            | "inspect"
            | "instance_of?"
            | "instance_variable_get"
            | "instance_variable_set"
            | "instance_variables"
            | "is_a?"
            | "itself"
            | "kind_of?"
            | "lambda"
            | "proc"
            | "method"
            | "methods"
            | "nil?"
            | "object_id"
            | "public_send"
            | "remove_instance_variable"
            | "singleton_methods"
            | "require"
            | "require_relative"
            | "respond_to?"
            | "respond_to_missing?"
            | "send"
            | "tap"
            | "to_s"
            | "to_enum"
            | "enum_for"
            | "display"
            | "then"
            | "yield_self"
            | "instance_variable_defined?"
            | "define_singleton_method"
            | "singleton_class"
            | "singleton_method"
            | "public_method"
            | "__id__"
            | "__send__"
    )
}

/// The hash `Hash[...]` answers: a plain one from Hash itself, and an
/// instance of the subclass when the call was made on one.
pub(crate) fn hash_of_class(
    class_rc: &Rc<crate::class::Class>,
    entries: indexmap::IndexMap<String, Object>,
) -> Object {
    let held = Object::Dict(Rc::new(std::cell::RefCell::new(entries)));
    if class_rc.name() == "Hash" {
        return held;
    }
    let instance = crate::object::Instance::new(Rc::clone(class_rc));
    instance.borrow_mut().set_var(
        crate::vm::native_methods::HASH_SUBCLASS_VAR.to_string(),
        held,
    );
    Object::Instance(instance)
}
