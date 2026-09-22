// The method names an object answers, by visibility.

use super::*;

impl VirtualMachine {
    /// The method names an object answers, by visibility.
    pub(crate) fn call_object_method_list_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
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
                    // An object is never an instance *of* a module, so a
                    // module argument is simply false rather than an error.
                    Object::Module(_) => return Ok(Some(Object::Bool(false))),
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
                // An encoding stands for an Encoding rather than for the
                // class it is held as.
                if self.names_an_encoding(receiver) {
                    return Ok(Some(Object::Bool(target_class.name() == "Encoding")));
                }
                // A Class is an instance of Class and a Module of Module,
                // which `class_of` does not report: it answers Object for
                // both so `is_a?` can walk the inheritance chain.
                let actual_name = match receiver {
                    Object::Class(_) => "Class".to_string(),
                    Object::Module(_) => "Module".to_string(),
                    other => self.builtins().class_of(other).name().to_string(),
                };
                Ok(Some(Object::Bool(actual_name == target_class.name())))
            }
            // `public_methods` is `methods` minus the restricted ones.
            // Kernel#public_methods / #private_methods / #protected_methods —
            // the names of that visibility reachable on the object, including
            // those a `class << obj` or `extend` supplied.
            "public_methods" | "private_methods" | "protected_methods" => {
                if arguments.len() > 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let include_super = !matches!(
                    arguments.first(),
                    Some(Object::Bool(false)) | Some(Object::Nil)
                );
                let mut names = match method_name {
                    "public_methods" => self.public_method_names_for(receiver, include_super),
                    "private_methods" => self.private_method_names_for(receiver, include_super),
                    _ => self.protected_method_names_for(receiver, include_super),
                };
                names.sort();
                names.dedup();
                let symbols: Vec<Object> = names.into_iter().map(Object::symbol).collect();
                Ok(Some(Object::Array(std::rc::Rc::new(
                    std::cell::RefCell::new(symbols),
                ))))
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
                // `obj.methods(false)` is the singleton methods alone: those
                // defined with `def obj.name` and any on the singleton class.
                if !include_super {
                    return Ok(Some(self.singleton_method_names(receiver)));
                }
                // The methods an object carries of its own come first, which
                // is where a module's `module_function` names live.
                let mut names: Vec<String> = match self.singleton_method_names(receiver) {
                    Object::Array(held) => held
                        .borrow()
                        .iter()
                        .map(|name| name.to_string().trim_start_matches(':').to_string())
                        .collect(),
                    _ => Vec::new(),
                };
                // A name the object carries of its own answers whatever an
                // instance method of the same name says about itself, which
                // is what `module_function` leaves behind.
                let carried = names.clone();
                let class = self.builtins().class_of(receiver);
                for name in class.method_names() {
                    if !names.contains(&name) {
                        names.push(name);
                    }
                }
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
                // For instances, also include methods from the instance's
                // class. The ancestor walk covers the modules it includes and
                // prepends as well as its superclasses, which is the same
                // chain a call travels.
                if let Object::Instance(inst_rc) = receiver {
                    let own_class = std::rc::Rc::clone(&inst_rc.borrow().class);
                    let mut chain = Vec::new();
                    let mut seen = Vec::new();
                    if include_super {
                        crate::vm::native_methods::class_methods::push_class_ancestors(
                            &own_class, &mut chain, &mut seen,
                        );
                    } else {
                        chain.push(Object::Class(own_class));
                    }
                    for ancestor in &chain {
                        let (Object::Class(carrier) | Object::Module(carrier)) = ancestor else {
                            continue;
                        };
                        for name in carrier.method_names() {
                            if !names.contains(&name) {
                                names.push(name);
                            }
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
                        // `def self.name` lands in the method table under the
                        // `__class__` convention; report it under the name
                        // Ruby shows.
                        let name = match name.strip_prefix("__class__") {
                            Some(bare) => bare.to_string(),
                            None => name,
                        };
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
                for name in self.singleton_layer_names(receiver) {
                    if !names.contains(&name) {
                        names.push(name);
                    }
                }
                // `def self.name` is stored under the `__class__` convention;
                // it belongs to the class object, not to its instances.
                if matches!(receiver, Object::Instance(_)) {
                    names.retain(|name| !name.starts_with("__class__"));
                }
                // A tombstone left by `undef_method` is not a method any more.
                let lookup_class = match receiver {
                    Object::Class(c) | Object::Module(c) => Some(std::rc::Rc::clone(c)),
                    Object::Instance(inst) => Some(std::rc::Rc::clone(&inst.borrow().class)),
                    _ => None,
                };
                if let Some(class) = lookup_class {
                    names.retain(|n| class.find_method(n).is_none_or(|m| !m.is_undefined));
                }
                let singleton = match receiver {
                    Object::Class(c) | Object::Module(c) => c.singleton_class_slot().clone(),
                    Object::Instance(inst) => inst.borrow().singleton_class.borrow().clone(),
                    _ => None,
                };
                if let Some(singleton_class) = singleton {
                    names.retain(|n| {
                        singleton_class
                            .find_own_method(n)
                            .is_none_or(|m| !m.is_undefined)
                    });
                }
                // A private method is not among the ones an object answers
                // to from outside, so `methods` leaves it out.
                let holder = self.builtins().class_of(receiver);
                names.retain(|name| {
                    carried.contains(name) || !self.method_is_private_anywhere(&holder, name)
                });
                names.sort();
                names.dedup();
                let method_symbols: Vec<Object> = names.into_iter().map(Object::symbol).collect();
                Ok(Some(Object::Array(std::rc::Rc::new(
                    std::cell::RefCell::new(method_symbols),
                ))))
            }
            _ => Ok(None),
        }
    }
}
