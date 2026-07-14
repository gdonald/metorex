//! Native method implementations for the Object class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;

impl VirtualMachine {
    /// Execute native methods for the Object class.
    pub(crate) fn call_object_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // Kernel#autoload / #autoload? — a top-level (or any non-module)
        // receiver registers the autoload on Object, Ruby's home for
        // top-level constants.
        if matches!(method_name, "autoload" | "autoload?")
            && let Some(Object::Class(object_class)) = self.globals().get("Object")
        {
            return self.call_class_methods(&object_class, method_name, arguments, position);
        }

        // Nil-specific conversions: in Ruby `nil.to_i == 0`, `nil.to_s == ""`,
        // `nil.to_a == []`, `nil.to_f == 0.0`. The dispatch above checks the
        // class of the receiver first, so we have to intercept here for Nil
        // before falling through to the generic Object methods.
        if matches!(receiver, Object::Nil) {
            match method_name {
                "to_i" => return Ok(Some(Object::Int(0))),
                "to_f" => return Ok(Some(Object::Float(0.0))),
                "to_a" => {
                    return Ok(Some(Object::Array(std::rc::Rc::new(
                        std::cell::RefCell::new(Vec::new()),
                    ))));
                }
                "to_h" => {
                    return Ok(Some(Object::Dict(std::rc::Rc::new(
                        std::cell::RefCell::new(std::collections::HashMap::new()),
                    ))));
                }
                "to_s" => return Ok(Some(Object::string(""))),
                "inspect" => return Ok(Some(Object::string("nil"))),
                "to_r" | "rationalize" => {
                    if method_name == "rationalize" && arguments.len() > 1 {
                        let exc = Object::exception(
                            "ArgumentError",
                            format!(
                                "wrong number of arguments (given {}, expected 0..1)",
                                arguments.len()
                            ),
                        );
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: format!(
                                "wrong number of arguments (given {}, expected 0..1)",
                                arguments.len()
                            ),
                        });
                    }
                    // Return Rational(0, 1) — create an instance via global function
                    if let Some(Object::Class(rational_class)) = self.globals().get("Rational") {
                        let mut inst = crate::object::Instance::new(rational_class);
                        inst.set_var("numerator".to_string(), Object::Int(0));
                        inst.set_var("denominator".to_string(), Object::Int(1));
                        return Ok(Some(Object::Instance(std::rc::Rc::new(
                            std::cell::RefCell::new(inst),
                        ))));
                    }
                    return Ok(Some(Object::Int(0)));
                }
                "to_c" => {
                    if let Some(Object::Class(complex_class)) = self.globals().get("Complex") {
                        let mut inst = crate::object::Instance::new(complex_class);
                        inst.set_var("real".to_string(), Object::Int(0));
                        inst.set_var("imaginary".to_string(), Object::Int(0));
                        return Ok(Some(Object::Instance(std::rc::Rc::new(
                            std::cell::RefCell::new(inst),
                        ))));
                    }
                    return Ok(Some(Object::Int(0)));
                }
                _ => {}
            }
        }
        match method_name {
            "__send__" | "send" | "public_send" => {
                if arguments.is_empty() {
                    return Err(MetorexError::runtime_error(
                        "wrong number of arguments (given 0, expected 1+)".to_string(),
                        position_to_location(position),
                    ));
                }
                let method = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    Object::Symbol(s) => s.as_str().to_string(),
                    _ => {
                        return Err(MetorexError::runtime_error(
                            format!("{} requires a String or Symbol method name", method_name),
                            position_to_location(position),
                        ));
                    }
                };
                let rest_args: Vec<Object> = arguments[1..].to_vec();
                // Prefer full lookup (walks singleton class + mixins) so mocked
                // or per-instance overrides take precedence over the class's
                // own method table.
                if let Some((resolved_class, m)) = self.lookup_method(receiver, &method)
                    && !m.is_undefined
                {
                    return Ok(Some(self.invoke_method(
                        resolved_class,
                        m,
                        receiver.clone(),
                        rest_args,
                        position,
                    )?));
                }
                let class = self.builtins().class_of(receiver);
                if let Some(result) = self.call_native_method(
                    class.as_ref(),
                    receiver,
                    &method,
                    &rest_args,
                    position,
                )? {
                    return Ok(Some(result));
                }
                if let Some(result) =
                    self.call_object_method(receiver, &method, &rest_args, position)?
                {
                    return Ok(Some(result));
                }
                Err(undefined_method_error(&method, receiver, position))
            }
            "frozen?" => {
                // In Metorex, booleans, nil, integers, floats, and symbols are frozen
                let frozen = match receiver {
                    Object::Bool(_)
                    | Object::Nil
                    | Object::Int(_)
                    | Object::Float(_)
                    | Object::Symbol(_)
                    | Object::String(_) => true,
                    Object::Class(c) | Object::Module(c) => c.is_frozen(),
                    Object::Instance(inst) => inst.borrow().frozen,
                    _ => false,
                };
                Ok(Some(Object::Bool(frozen)))
            }
            "freeze" => {
                match receiver {
                    Object::Class(c) | Object::Module(c) => {
                        c.freeze();
                    }
                    Object::Instance(inst) => {
                        inst.borrow_mut().frozen = true;
                    }
                    _ => {}
                }
                Ok(Some(receiver.clone()))
            }
            "to_sym" => match receiver {
                Object::Symbol(_) => Ok(Some(receiver.clone())),
                Object::String(s) => Ok(Some(Object::Symbol(s.clone()))),
                _ => Ok(None),
            },
            "object_id" | "__id__" => {
                // Return a unique integer for the object (use pointer address for ref types)
                let id = match receiver {
                    Object::Instance(inst) => std::rc::Rc::as_ptr(inst) as i64,
                    Object::Array(arr) => std::rc::Rc::as_ptr(arr) as i64,
                    Object::Dict(dict) => std::rc::Rc::as_ptr(dict) as i64,
                    Object::Class(cls) => std::rc::Rc::as_ptr(cls) as i64,
                    Object::Module(m) => std::rc::Rc::as_ptr(m) as i64,
                    Object::Int(n) => 2 * n + 1, // Ruby's fixnum object_id
                    Object::Bool(true) => 2,
                    Object::Bool(false) => 0,
                    Object::Nil => 4,
                    _ => 0,
                };
                Ok(Some(Object::Int(id)))
            }
            "clamp" => {
                // Comparable#clamp — 1 or 2 args, Range, or beginless/endless ranges
                let (min, max) = if arguments.len() == 1 {
                    // Range argument
                    match &arguments[0] {
                        Object::Range {
                            start,
                            end,
                            exclusive,
                        } => {
                            // Exclusive range is an error — except when end is nil
                            // (endless exclusive range like `x...` is allowed).
                            if *exclusive && !matches!(**end, Object::Nil) {
                                let exc = Object::exception(
                                    "ArgumentError",
                                    "cannot clamp with an exclusive range".to_string(),
                                );
                                return Err(MetorexError::UncaughtException {
                                    exception: exc,
                                    location: position_to_location(position),
                                    message: "cannot clamp with an exclusive range".to_string(),
                                });
                            }
                            ((**start).clone(), (**end).clone())
                        }
                        _ => {
                            return Err(method_argument_error(
                                method_name,
                                1,
                                arguments.len(),
                                position,
                            ));
                        }
                    }
                } else if arguments.len() == 2 {
                    (arguments[0].clone(), arguments[1].clone())
                } else {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                };

                // Verify min <= max when both are non-nil (also raise if incomparable)
                if !matches!(min, Object::Nil) && !matches!(max, Object::Nil) {
                    let cmp = self.dispatch_spaceship(&min, &max, position)?;
                    match cmp {
                        Some(n) if n > 0 => {
                            let exc = Object::exception(
                                "ArgumentError",
                                "min argument must be smaller than max argument".to_string(),
                            );
                            return Err(MetorexError::UncaughtException {
                                exception: exc,
                                location: position_to_location(position),
                                message: "min argument must be smaller than max argument"
                                    .to_string(),
                            });
                        }
                        None => {
                            let exc =
                                Object::exception("ArgumentError", "comparison failed".to_string());
                            return Err(MetorexError::UncaughtException {
                                exception: exc,
                                location: position_to_location(position),
                                message: "comparison failed".to_string(),
                            });
                        }
                        _ => {}
                    }
                }

                // Clamp: return min if self < min, max if self > max, else self
                if !matches!(min, Object::Nil) {
                    let cmp = self.dispatch_spaceship(receiver, &min, position)?;
                    if matches!(cmp, Some(n) if n < 0) {
                        return Ok(Some(min));
                    }
                }
                if !matches!(max, Object::Nil) {
                    let cmp = self.dispatch_spaceship(receiver, &max, position)?;
                    if matches!(cmp, Some(n) if n > 0) {
                        return Ok(Some(max));
                    }
                }
                Ok(Some(receiver.clone()))
            }
            "between?" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                // between?(min, max) returns true if min <= self <= max
                // Try <=> dispatch
                let cmp_min = self.dispatch_spaceship(receiver, &arguments[0], position)?;
                let cmp_max = self.dispatch_spaceship(receiver, &arguments[1], position)?;
                let result = match (cmp_min, cmp_max) {
                    (Some(a), Some(b)) => a >= 0 && b <= 0,
                    _ => false,
                };
                Ok(Some(Object::Bool(result)))
            }
            "singleton_class" => {
                let sc = self.singleton_class_of(receiver);
                Ok(Some(Object::Class(sc)))
            }
            // Object#extend(mod) — add `mod` as a mixin on the receiver's
            // singleton class so the module's instance methods become callable
            // on this specific object. (Class/Module already have their own
            // dedicated `extend` in class_methods.rs that hits earlier.)
            "extend" => {
                if arguments.is_empty() {
                    return Err(method_argument_error(
                        "extend",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                for arg in arguments {
                    let module_rc = match arg {
                        Object::Module(m) => std::rc::Rc::clone(m),
                        Object::Class(c) => std::rc::Rc::clone(c),
                        other => {
                            return Err(method_argument_type_error(
                                "extend", "Module", other, position,
                            ));
                        }
                    };
                    let singleton = self.singleton_class_of(receiver);
                    singleton.add_mixin(module_rc);
                }
                Ok(Some(receiver.clone()))
            }
            "singleton_method" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let method = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    Object::Symbol(s) => s.as_str().to_string(),
                    _ => {
                        return Err(MetorexError::runtime_error(
                            "no implicit conversion into String".to_string(),
                            position_to_location(position),
                        ));
                    }
                };
                let msg = format!("undefined singleton method '{}' for {}", method, receiver);
                let exc = Object::exception("NameError", msg.clone());
                Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                })
            }
            "to_s" | "inspect" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                if let Object::Symbol(s) = receiver {
                    return Ok(Some(Object::string(if method_name == "to_s" {
                        (**s).clone()
                    } else {
                        format!(":{}", s)
                    })));
                }
                Ok(Some(Object::string(receiver.to_string())))
            }
            "class" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // For exceptions, return the specific exception class, not generic Exception
                if let Object::Exception(exc_ref) = receiver {
                    let exc_type = exc_ref.borrow().exception_type.clone();
                    if let Some(Object::Class(cls)) = self.globals().get(&exc_type) {
                        return Ok(Some(Object::Class(cls)));
                    }
                }
                // For booleans and nil, return the specific class
                match receiver {
                    Object::Bool(true) => {
                        if let Some(Object::Class(cls)) = self.globals().get("TrueClass") {
                            return Ok(Some(Object::Class(cls)));
                        }
                    }
                    Object::Bool(false) => {
                        if let Some(Object::Class(cls)) = self.globals().get("FalseClass") {
                            return Ok(Some(Object::Class(cls)));
                        }
                    }
                    Object::Nil => {
                        if let Some(Object::Class(cls)) = self.globals().get("NilClass") {
                            return Ok(Some(Object::Class(cls)));
                        }
                    }
                    _ => {}
                }
                Ok(Some(Object::Class(self.builtins().class_of(receiver))))
            }
            "method" => {
                // `obj.method(:name)` returns a Method object bound to obj.
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name_str = match &arguments[0] {
                    Object::String(s) => (**s).clone(),
                    Object::Symbol(s) => (**s).clone(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                let cls = self.builtins().class_of(receiver);
                if let Some(method) = cls.find_method(&name_str) {
                    let mut bound = method.as_ref().clone();
                    bound.receiver = Some(Box::new(receiver.clone()));
                    return Ok(Some(Object::Method(std::rc::Rc::new(bound))));
                }
                let msg = format!("undefined method '{}' for class '{}'", name_str, cls.name());
                let exc = Object::exception("NameError", msg.clone());
                Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                })
            }
            "respond_to?" => {
                // Accept String or Symbol method name; ignore optional second
                // `include_private` argument (Ruby's signature).
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let method_query = match &arguments[0] {
                    Object::String(name) => name.as_str().to_string(),
                    Object::Symbol(name) => name.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                Ok(Some(Object::Bool(
                    self.lookup_method(receiver, &method_query).is_some(),
                )))
            }
            "nil?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(matches!(receiver, Object::Nil))))
            }
            "get_source" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let query = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    Object::Symbol(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                match self.lookup_method(receiver, &query) {
                    Some((_class, method)) => Ok(Some(Object::Method(method))),
                    None => Ok(Some(Object::Nil)),
                }
            }
            "is_a?" | "kind_of?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let target_class = match &arguments[0] {
                    Object::Class(c) => c,
                    Object::Module(m) => m,
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Class",
                            other,
                            position,
                        ));
                    }
                };
                // A Class object is itself an instance of Class (and its
                // ancestors: Module, Object, BasicObject). Walk the global
                // Class's chain so anonymous Class.new instances answer
                // correctly without needing their own class pointer.
                if matches!(receiver, Object::Class(_) | Object::Module(_)) {
                    let target_name = target_class.name();
                    let meta_name = match receiver {
                        Object::Class(_) => "Class",
                        Object::Module(_) => "Module",
                        _ => unreachable!(),
                    };
                    if let Some(Object::Class(meta)) = self.globals().get(meta_name) {
                        if self.builtins().is_subclass_of(&meta, target_class) {
                            return Ok(Some(Object::Bool(true)));
                        }
                        let mut current = meta.superclass();
                        while let Some(anc) = current {
                            if anc.name() == target_name {
                                return Ok(Some(Object::Bool(true)));
                            }
                            current = anc.superclass();
                        }
                    }
                }
                Ok(Some(Object::Bool(
                    self.builtins().is_instance_of(receiver, target_class),
                )))
            }
            "instance_variables" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let vars = if let Object::Instance(inst_rc) = receiver {
                    let inst = inst_rc.borrow();
                    inst.instance_vars
                        .keys()
                        .map(|k| Object::string(format!("@{}", k)))
                        .collect()
                } else {
                    vec![]
                };
                Ok(Some(Object::Array(std::rc::Rc::new(
                    std::cell::RefCell::new(vars),
                ))))
            }
            "instance_variable_get" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let var_name = match &arguments[0] {
                    Object::String(s) => s.as_str().to_string(),
                    Object::Symbol(s) => s.as_str().to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                // Strip leading @ if present
                let clean_name = var_name.strip_prefix('@').unwrap_or(&var_name);
                match receiver {
                    Object::Instance(inst_rc) => {
                        let inst = inst_rc.borrow();
                        Ok(Some(
                            inst.instance_vars
                                .get(clean_name)
                                .cloned()
                                .unwrap_or(Object::Nil),
                        ))
                    }
                    Object::Class(class_rc) => Ok(Some(
                        class_rc
                            .get_class_var(&format!("@{}", clean_name))
                            .unwrap_or(Object::Nil),
                    )),
                    Object::Module(module_rc) => Ok(Some(
                        module_rc
                            .get_class_var(&format!("@{}", clean_name))
                            .unwrap_or(Object::Nil),
                    )),
                    _ => Ok(Some(Object::Nil)),
                }
            }
            "instance_variable_set" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let var_name = match &arguments[0] {
                    Object::String(s) => s.strip_prefix('@').unwrap_or(s).to_string(),
                    Object::Symbol(s) => s.strip_prefix('@').unwrap_or(s).to_string(),
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                let value = arguments[1].clone();
                match receiver {
                    Object::Instance(instance_rc) => {
                        let is_frozen = instance_rc.borrow().frozen;
                        if is_frozen {
                            let class_name = instance_rc.borrow().class.name().to_string();
                            let msg = format!("can't modify frozen {}", class_name);
                            let exc = Object::exception("FrozenError", msg.clone());
                            return Err(MetorexError::UncaughtException {
                                exception: exc,
                                location: position_to_location(position),
                                message: msg,
                            });
                        }
                        instance_rc.borrow_mut().set_var(var_name, value.clone());
                        Ok(Some(value))
                    }
                    Object::Class(class_rc) => {
                        class_rc.set_class_var(format!("@{}", var_name), value.clone());
                        Ok(Some(value))
                    }
                    Object::Module(module_rc) => {
                        module_rc.set_class_var(format!("@{}", var_name), value.clone());
                        Ok(Some(value))
                    }
                    // Immediates (true/false/nil/integers/symbols/floats) are
                    // always frozen — assigning an ivar raises FrozenError to
                    // match Ruby (FrozenError is a subclass of RuntimeError).
                    other => {
                        let class_name = match other {
                            Object::Bool(true) => "TrueClass".to_string(),
                            Object::Bool(false) => "FalseClass".to_string(),
                            Object::Nil => "NilClass".to_string(),
                            Object::Symbol(_) => "Symbol".to_string(),
                            _ => self.builtins().class_of(other).name().to_string(),
                        };
                        let msg = format!("can't modify frozen {}: {}", class_name, other);
                        let exc = Object::exception("FrozenError", msg.clone());
                        Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        })
                    }
                }
            }
            "instance_of?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let target_class = match &arguments[0] {
                    Object::Class(c) => c,
                    other => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Class",
                            other,
                            position,
                        ));
                    }
                };
                // Exceptions are stored as a single Object::Exception variant
                // tagged with the specific exception type (e.g. "NameError").
                // `instance_of?` should match against that type rather than
                // the broad `Exception` class so `e.instance_of?(NameError)`
                // works for a rescued NameError.
                if let Object::Exception(exc) = receiver {
                    let actual = exc.borrow().exception_type.clone();
                    return Ok(Some(Object::Bool(actual == target_class.name())));
                }
                let obj_class = self.builtins().class_of(receiver);
                Ok(Some(Object::Bool(obj_class.name() == target_class.name())))
            }
            "methods" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                // Optional `include_super` arg (default true). When false, the
                // walk over inherited methods is skipped — but the receiver's
                // own methods are still collected. Mirrors Ruby's
                // `obj.methods(false)`.
                let include_super = !matches!(arguments.first(), Some(Object::Bool(false)));
                let class = self.builtins().class_of(receiver);
                let mut names = class.method_names();
                if include_super {
                    // Walk the superclass chain to collect inherited methods
                    let mut current = class.superclass();
                    while let Some(parent) = current {
                        for name in parent.method_names() {
                            if !names.contains(&name) {
                                names.push(name);
                            }
                        }
                        current = parent.superclass();
                    }
                }
                // For instances, also include methods from the instance's class
                if let Object::Instance(inst_rc) = receiver {
                    let inst = inst_rc.borrow();
                    for name in inst.class.method_names() {
                        if !names.contains(&name) {
                            names.push(name);
                        }
                    }
                    if include_super {
                        let mut parent = inst.class.superclass();
                        while let Some(p) = parent {
                            for name in p.method_names() {
                                if !names.contains(&name) {
                                    names.push(name);
                                }
                            }
                            parent = p.superclass();
                        }
                    }
                }
                // For Class/Module receivers, also include the receiver's own
                // instance methods. Ruby's `Object.methods` exposes instance
                // methods of Object too (because Object's singleton class
                // inherits from Class → Module → Object). This also lets
                // `define_method` at TOPLEVEL_BINDING be observable via
                // `Object.methods.include?(...)`.
                if let Object::Class(c) | Object::Module(c) = receiver {
                    for name in c.method_names() {
                        if !names.contains(&name) {
                            names.push(name);
                        }
                    }
                    if include_super {
                        let mut parent = c.superclass();
                        while let Some(p) = parent {
                            for name in p.method_names() {
                                if !names.contains(&name) {
                                    names.push(name);
                                }
                            }
                            parent = p.superclass();
                        }
                    }
                }
                names.sort();
                names.dedup();
                let method_symbols: Vec<Object> = names
                    .into_iter()
                    .map(|n| Object::Symbol(std::rc::Rc::new(n)))
                    .collect();
                Ok(Some(Object::Array(std::rc::Rc::new(
                    std::cell::RefCell::new(method_symbols),
                ))))
            }
            "eql?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(receiver.equals(&arguments[0]))))
            }
            "equal?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = &arguments[0];
                let identity = match (receiver, other) {
                    (Object::Instance(a), Object::Instance(b)) => std::rc::Rc::ptr_eq(a, b),
                    (Object::Array(a), Object::Array(b)) => std::rc::Rc::ptr_eq(a, b),
                    (Object::Dict(a), Object::Dict(b)) => std::rc::Rc::ptr_eq(a, b),
                    (Object::Class(a), Object::Class(b)) => std::rc::Rc::ptr_eq(a, b),
                    (Object::Module(a), Object::Module(b)) => std::rc::Rc::ptr_eq(a, b),
                    // For value types, equal? is the same as ==
                    (a, b) => a == b,
                };
                Ok(Some(Object::Bool(identity)))
            }
            "dup" | "clone" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                match receiver {
                    Object::Instance(inst_rc) => {
                        let inst = inst_rc.borrow();
                        let mut new_inst =
                            crate::object::Instance::new(std::rc::Rc::clone(&inst.class));
                        for (k, v) in &inst.instance_vars {
                            new_inst.set_var(k.clone(), v.clone());
                        }
                        Ok(Some(Object::Instance(std::rc::Rc::new(
                            std::cell::RefCell::new(new_inst),
                        ))))
                    }
                    Object::Array(arr_rc) => {
                        let arr = arr_rc.borrow().clone();
                        Ok(Some(Object::Array(std::rc::Rc::new(
                            std::cell::RefCell::new(arr),
                        ))))
                    }
                    Object::Dict(dict_rc) => {
                        let dict = dict_rc.borrow().clone();
                        Ok(Some(Object::Dict(std::rc::Rc::new(
                            std::cell::RefCell::new(dict),
                        ))))
                    }
                    Object::Class(class_rc) => {
                        if class_rc.name() == "BasicObject" {
                            let msg = "can't copy the root class".to_string();
                            let exc = Object::exception("TypeError", msg.clone());
                            return Err(MetorexError::UncaughtException {
                                exception: exc,
                                location: position_to_location(position),
                                message: msg,
                            });
                        }
                        let copy = crate::class::Class::duplicate(class_rc);
                        Ok(Some(Object::Class(std::rc::Rc::new(copy))))
                    }
                    Object::Module(mod_rc) => {
                        let copy = crate::class::Class::duplicate(mod_rc);
                        Ok(Some(Object::Module(std::rc::Rc::new(copy))))
                    }
                    // Immutable types return themselves
                    _ => Ok(Some(receiver.clone())),
                }
            }
            "=~" => {
                // Regex match: string =~ regex or regex =~ string
                if arguments.len() != 1 {
                    return Err(method_argument_error("=~", 1, arguments.len(), position));
                }
                match (receiver, &arguments[0]) {
                    (Object::Regex(pattern, flags), Object::String(s))
                    | (Object::String(s), Object::Regex(pattern, flags)) => {
                        let case_insensitive = flags.contains('i');
                        let re_pattern = if case_insensitive {
                            format!("(?i){}", pattern)
                        } else {
                            pattern.as_ref().clone()
                        };
                        match regex::Regex::new(&re_pattern) {
                            Ok(re) => {
                                if let Some(m) = re.find(s.as_ref()) {
                                    Ok(Some(Object::Int(m.start() as i64)))
                                } else {
                                    Ok(Some(Object::Nil))
                                }
                            }
                            Err(_) => Ok(Some(Object::Nil)),
                        }
                    }
                    _ => Ok(Some(Object::Nil)),
                }
            }
            "!~" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error("!~", 1, arguments.len(), position));
                }
                match (receiver, &arguments[0]) {
                    (Object::Regex(pattern, flags), Object::String(s))
                    | (Object::String(s), Object::Regex(pattern, flags)) => {
                        let case_insensitive = flags.contains('i');
                        let re_pattern = if case_insensitive {
                            format!("(?i){}", pattern)
                        } else {
                            pattern.as_ref().clone()
                        };
                        match regex::Regex::new(&re_pattern) {
                            Ok(re) => Ok(Some(Object::Bool(!re.is_match(s.as_ref())))),
                            Err(_) => Ok(Some(Object::Bool(true))),
                        }
                    }
                    _ => Ok(Some(Object::Bool(true))),
                }
            }
            // An instance of a `Module` subclass (`class Sub < Module; end;
            // Sub.new`) is itself a module and answers the module-body methods.
            // Metorex models it as an `Instance`, so back it with a cached
            // anonymous class that hosts any methods the body defines.
            "class_exec" | "module_exec" | "class_eval" | "module_eval"
                if self.instance_acts_as_module(receiver) =>
            {
                let inst = match receiver {
                    Object::Instance(i) => std::rc::Rc::clone(i),
                    _ => unreachable!(),
                };
                let existing = match inst.borrow().get_var("__module_body_class__") {
                    Some(Object::Class(c)) => Some(std::rc::Rc::clone(c)),
                    _ => None,
                };
                let backing = match existing {
                    Some(c) => c,
                    None => {
                        let c = std::rc::Rc::new(crate::class::Class::new("", None));
                        inst.borrow_mut().set_var(
                            "__module_body_class__".to_string(),
                            Object::Class(std::rc::Rc::clone(&c)),
                        );
                        c
                    }
                };
                if method_name == "class_exec" || method_name == "module_exec" {
                    let block = match self.pending_block.take() {
                        Some(Object::Block(b)) => b,
                        _ => return Err(local_jump_error(method_name, position)),
                    };
                    let result = self.class_exec_block(
                        &backing,
                        receiver.clone(),
                        &block,
                        arguments.to_vec(),
                        position,
                    )?;
                    return Ok(Some(result));
                }
                let result =
                    self.class_eval_with_args(&backing, receiver.clone(), arguments, position)?;
                Ok(Some(result))
            }
            "instance_exec" | "instance_eval" => {
                let block = self.pending_block.take().or_else(|| {
                    if !arguments.is_empty()
                        && let Object::Block(_) = &arguments[0]
                    {
                        Some(arguments[0].clone())
                    } else {
                        None
                    }
                });
                match block {
                    Some(Object::Block(b)) => {
                        let args: Vec<Object> = arguments
                            .iter()
                            .filter(|a| !matches!(a, Object::Block(_)))
                            .cloned()
                            .collect();
                        let result =
                            self.execute_block_with_receiver(&b, receiver.clone(), args, position)?;
                        Ok(Some(result))
                    }
                    _ => Err(MetorexError::runtime_error(
                        format!("{} requires a block", method_name),
                        position_to_location(position),
                    )),
                }
            }
            _ => Ok(None),
        }
    }

    /// Whether `receiver` is an `Instance` whose class descends from `Module`
    /// (e.g. `class Sub < Module; end; Sub.new`). Such instances are modules
    /// and answer the module-body methods (`class_eval`/`class_exec`/...).
    fn instance_acts_as_module(&self, receiver: &Object) -> bool {
        let Object::Instance(inst) = receiver else {
            return false;
        };
        let class = std::rc::Rc::clone(&inst.borrow().class);
        match self.globals().get("Module") {
            Some(Object::Class(module_class)) => {
                self.builtins().is_subclass_of(&class, &module_class)
            }
            _ => false,
        }
    }

    /// Dispatch <=> on receiver with other, returning Some(i64) or None.
    fn dispatch_spaceship(
        &mut self,
        receiver: &Object,
        other: &Object,
        position: Position,
    ) -> Result<Option<i64>, MetorexError> {
        if let Some((class, method)) = self.lookup_method(receiver, "<=>") {
            let result = self.invoke_method(
                class,
                method,
                receiver.clone(),
                vec![other.clone()],
                position,
            )?;
            match result {
                Object::Int(n) => Ok(Some(n)),
                Object::Nil => Ok(None),
                _ => Ok(None),
            }
        } else {
            // Fallback for built-in types
            match (receiver, other) {
                (Object::Int(a), Object::Int(b)) => Ok(Some((*a).cmp(b) as i64)),
                (Object::Float(a), Object::Float(b)) => Ok(a.partial_cmp(b).map(|o| o as i64)),
                (Object::String(a), Object::String(b)) => Ok(Some((**a).cmp(b) as i64)),
                _ => Ok(None),
            }
        }
    }
}
