// A method held as an object, and whether the receiver answers a name
// at all.

use super::*;

impl VirtualMachine {
    /// A method held as an object, and whether the receiver answers a name
    /// at all.
    pub(crate) fn call_object_method_lookup_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            // `obj.method(:name)` returns a Method object bound to obj.
            // `public_method` is the same lookup with private and protected
            // names refused.
            "method" | "public_method" => {
                // A class that defines `def self.method` or `def self.
                // public_method` of its own owns the name; the class-method
                // table records it under the `__class__` convention, which
                // the caller checks after this returns None.
                if let Object::Class(class_rc) | Object::Module(class_rc) = receiver
                    && class_rc
                        .find_method(&format!("__class__{}", method_name))
                        .is_some()
                {
                    return Ok(None);
                }
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name_str = self.coerce_name_argument(&arguments[0], position)?;
                // Full lookup so singleton methods (`class << obj`,
                // `def self.foo`) and mixins resolve, not just the class's
                // own method table.
                let public_only = method_name == "public_method";
                // A method the receiver's class supplies natively is that
                // class's own, even when Enumerable declares the same name.
                let shadowed_by_enumerable = self.enumerable_stands_in(receiver, &name_str);
                // A refinement in force here is what the name stands for
                // while it lasts, so the method object answers to it.
                let refined = crate::vm::method_lookup::refinement_target_name(receiver, self)
                    .and_then(|target| self.find_refined_method(&target, &name_str));
                if let Some(refined) = refined {
                    let owner = self.builtins().class_of(receiver);
                    let mut bound = (*refined).clone();
                    bound.receiver = Some(Box::new(receiver.clone()));
                    bound.owner = Some(owner.ruby_name());
                    bound.owner_class = Some(owner);
                    return Ok(Some(Object::Method(std::rc::Rc::new(bound))));
                }
                if let Some((resolved_class, method)) = self
                    .lookup_method(receiver, &name_str)
                    .filter(|_| !shadowed_by_enumerable)
                {
                    if public_only && self.method_is_restricted(receiver, &name_str) {
                        let msg = format!(
                            "undefined method '{}' for class '{}'",
                            name_str,
                            self.builtins().class_of(receiver).name()
                        );
                        let exc = Object::exception("NameError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                    let mut bound = (*method).clone();
                    bound.receiver = Some(Box::new(receiver.clone()));
                    // Two names that are the same native method answer the
                    // same Method object, which is what makes
                    // `method(:to_s) == method(:inspect)` hold for an alias.
                    if let Some(target) =
                        native_alias_target(self.builtins().class_of(receiver).name(), &name_str)
                    {
                        if bound.original_name.is_none() {
                            bound.original_name = Some(target.to_string());
                        }
                        // The receiver's own class answers it, whatever the
                        // lookup walked past to get here.
                        let owner = self.builtins().class_of(receiver);
                        bound.owner = Some(owner.ruby_name());
                        bound.owner_class = Some(owner);
                    }
                    // `public :name` inside a class makes that class the
                    // owner of the method it opened up, whichever ancestor
                    // wrote it. The ancestor that wrote it is kept, since
                    // that is where `super` carries on from.
                    if resolved_class.has_public_override(&name_str) {
                        if bound.origin_class.is_none() {
                            bound.origin_class = bound.owner_class.clone();
                        }
                        bound.owner = Some(resolved_class.ruby_name());
                        bound.owner_class = Some(std::rc::Rc::clone(&resolved_class));
                    }
                    if bound.owner_class.is_none() {
                        // The lookup answers the class it resolved through,
                        // which for a singleton class is where the walk began
                        // rather than where the method is written. The one
                        // that holds it is the one that owns it.
                        let holder = resolved_class
                            .find_method_with_owner(&name_str)
                            .map(|(held, _)| held)
                            .unwrap_or(resolved_class);
                        bound.owner = Some(holder.ruby_name());
                        bound.owner_class = Some(holder);
                    }
                    return Ok(Some(Object::Method(std::rc::Rc::new(bound))));
                }
                // Module and Class instance methods are implemented natively
                // rather than living in a method table, so hand out a stub
                // carrying the right parameter list.
                if matches!(receiver, Object::Class(_) | Object::Module(_))
                    && let Some(mut stub) =
                        crate::vm::native_methods::class_methods::native_module_method_stub(
                            &name_str,
                        )
                {
                    stub.receiver = Some(Box::new(receiver.clone()));
                    return Ok(Some(Object::Method(std::rc::Rc::new(stub))));
                }
                // The Kernel methods every object carries are native too.
                if let Some(mut stub) =
                    crate::vm::native_methods::class_methods::native_kernel_method_stub(&name_str)
                {
                    stub.receiver = Some(Box::new(receiver.clone()));
                    return Ok(Some(Object::Method(std::rc::Rc::new(stub))));
                }
                // A method the receiver's own class implements natively, such
                // as an operator on a number, is handed out as a stub bound to
                // the receiver. `invoke_method` runs the native one.
                let is_operator = matches!(
                    name_str.as_str(),
                    "+" | "-"
                        | "*"
                        | "/"
                        | "%"
                        | "**"
                        | "=="
                        | "!="
                        | "<"
                        | ">"
                        | "<="
                        | ">="
                        | "<=>"
                        | "==="
                        | "<<"
                        | ">>"
                        | "&"
                        | "|"
                        | "^"
                        | "[]"
                        | "[]="
                        | "-@"
                        | "+@"
                        | "~"
                );
                if is_operator || self.responds_to(receiver, &name_str) {
                    let owner = self.builtins().class_of(receiver);
                    let mut stub = crate::object::Method::new(
                        name_str.clone(),
                        vec!["args".to_string()],
                        vec![],
                    );
                    stub.receiver = Some(Box::new(receiver.clone()));
                    stub.owner = Some(owner.ruby_name());
                    // Two names that are the same native method answer the
                    // same Method object, which is what makes `method(:to_s)
                    // == method(:inspect)` hold for an alias.
                    stub.original_name = Some(
                        native_alias_target(owner.name(), &name_str)
                            .unwrap_or(&name_str)
                            .to_string(),
                    );
                    stub.owner_class = Some(owner);
                    return Ok(Some(Object::Method(std::rc::Rc::new(stub))));
                }
                // An object that answers `respond_to_missing?` for the name
                // has a method as far as Ruby is concerned, so hand out one
                // that routes through `method_missing`.
                if self.responds_via_missing(receiver, &name_str, !public_only, position)? {
                    let mut stub = method_missing_dispatcher(&name_str, receiver, position);
                    // The method belongs to the class that claimed the name,
                    // which is the receiver's own.
                    let owner = self.builtins().class_of(receiver);
                    stub.owner = Some(owner.ruby_name());
                    stub.owner_class = Some(owner);
                    return Ok(Some(Object::Method(std::rc::Rc::new(stub))));
                }
                let cls = self.builtins().class_of(receiver);
                let msg = format!("undefined method '{}' for class '{}'", name_str, cls.name());
                let exc = Object::exception("NameError", msg.clone());
                Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: position_to_location(position),
                    message: msg,
                })
            }
            // Kernel#respond_to_missing? — the default answer is false. A
            // class overrides it to claim names it handles through
            // `method_missing`.
            "respond_to_missing?" => {
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(false)))
            }
            "respond_to?" => {
                // Accept String or Symbol method name, plus the optional
                // `include_private` flag.
                if arguments.is_empty() || arguments.len() > 2 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let method_query = self.coerce_method_name(&arguments[0], method_name, position)?;
                let include_private = matches!(arguments.get(1), Some(value) if value.is_truthy());
                // A method a C extension defined as unimplemented on this
                // platform is there, and still answers false.
                let unimplemented =
                    self.lookup_method(receiver, &method_query)
                        .is_some_and(|(_, method)| {
                            method
                                .c_function
                                .as_ref()
                                .is_some_and(crate::vm::capi::is_not_implemented)
                        });
                if unimplemented {
                    return Ok(Some(Object::Bool(false)));
                }
                if self.responds_to(receiver, &method_query)
                    && (include_private || !self.method_is_restricted(receiver, &method_query))
                {
                    return Ok(Some(Object::Bool(true)));
                }
                // Ruby asks `respond_to_missing?` for anything the lookup
                // missed, passing the same private flag it was given.
                Ok(Some(Object::Bool(self.responds_via_missing(
                    receiver,
                    &method_query,
                    include_private,
                    position,
                )?)))
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
            _ => Ok(None),
        }
    }
}
