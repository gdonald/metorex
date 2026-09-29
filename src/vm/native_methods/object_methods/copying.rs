// The copy `dup` and `clone` hand back, and what it carries over.

use super::*;

impl VirtualMachine {
    /// The copy `dup` and `clone` hand back, before any frozen state or
    /// singleton class is carried over.
    /// Whether the receiver is an instance whose class chain reaches
    /// BasicObject without passing through Object. Only such an object is
    /// limited to BasicObject's own methods.
    pub(crate) fn rooted_at_basic_object(&self, receiver: &Object) -> bool {
        let Object::Instance(instance) = receiver else {
            return false;
        };
        let mut walked = Some(std::rc::Rc::clone(&instance.borrow().class));
        let mut reaches_basic_object = false;
        while let Some(class) = walked {
            if class.name() == "Object" {
                return false;
            }
            // A class rooted at BasicObject may still take Kernel on, and
            // then it answers everything Kernel defines.
            if class
                .mixin_chain()
                .iter()
                .any(|mixed| mixed.name() == "Kernel")
            {
                return false;
            }
            reaches_basic_object |= class.name() == "BasicObject";
            walked = class.superclass();
        }
        reaches_basic_object
    }

    pub(crate) fn copy_for(
        &mut self,
        receiver: &Object,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match receiver {
            // Complex and Rational are value objects: there is nothing
            // to copy, so Ruby answers the receiver itself.
            Object::Instance(inst_rc)
                if matches!(inst_rc.borrow().class.name(), "Complex" | "Rational") =>
            {
                Ok(Some(receiver.clone()))
            }
            Object::Instance(inst_rc) => {
                let copy = {
                    let inst = inst_rc.borrow();
                    let new_inst = crate::object::Instance::new(std::rc::Rc::clone(&inst.class));
                    for (k, v) in &inst.instance_vars {
                        // The characters or elements behind a subclass of a
                        // primitive belong to the instance, so the copy gets
                        // its own rather than sharing them.
                        let held = match (k.as_str(), v) {
                            (
                                crate::vm::native_methods::STRING_SUBCLASS_VAR,
                                Object::String(text),
                            ) => {
                                let made = crate::object::StringValue::with_encoding(
                                    text.to_text(),
                                    text.encoding_name(),
                                );
                                if text.holds_bytes() {
                                    made.mark_bytes();
                                }
                                Object::String(std::rc::Rc::new(made))
                            }
                            (
                                crate::vm::native_methods::ARRAY_SUBCLASS_VAR,
                                Object::Array(elements),
                            ) => Object::array(elements.borrow().clone()),
                            (crate::vm::native_methods::HASH_SUBCLASS_VAR, Object::Dict(pairs)) => {
                                Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(
                                    pairs.borrow().clone(),
                                )))
                            }
                            (crate::vm::native_methods::SET_SUBCLASS_VAR, Object::Set(held)) => {
                                Object::Set(std::rc::Rc::new(std::cell::RefCell::new(
                                    held.borrow().clone(),
                                )))
                            }
                            _ => v.clone(),
                        };
                        new_inst.borrow_mut().set_var(k.clone(), held);
                    }
                    Object::Instance(new_inst)
                };
                // The copy gets `initialize_copy` with the original, so
                // a class can deep-copy what the shallow copy shared.
                if let Some((class, method)) = self.lookup_method(&copy, "initialize_copy")
                    && !method.is_undefined
                {
                    self.invoke_method(
                        class,
                        method,
                        copy.clone(),
                        vec![receiver.clone()],
                        position,
                    )?;
                }
                Ok(Some(copy))
            }
            // An exception copies its message, backtrace, cause, and
            // instance variables. `dup` leaves the singleton class
            // behind, so a method defined on the original is not on
            // the copy.
            Object::Exception(details) => {
                let copy = {
                    // The backtrace Array is shared with the original,
                    // the way Ruby's copy shares it.
                    let copied = details.borrow().clone();
                    Object::Exception(std::rc::Rc::new(std::cell::RefCell::new(copied)))
                };
                if let Some((class, method)) = self.lookup_method(&copy, "initialize_copy")
                    && !method.is_undefined
                    && !method.body.is_empty()
                {
                    self.invoke_method(
                        class,
                        method,
                        copy.clone(),
                        vec![receiver.clone()],
                        position,
                    )?;
                }
                Ok(Some(copy))
            }
            // A copy of a string holds its own characters, so changing one
            // of them leaves the other alone.
            Object::String(text) => {
                let made =
                    crate::object::StringValue::with_encoding(text.to_text(), text.encoding_name());
                if text.holds_bytes() {
                    made.mark_bytes();
                }
                Ok(Some(Object::String(std::rc::Rc::new(made))))
            }
            Object::Array(arr_rc) => {
                let arr = arr_rc.borrow().clone();
                let copy = Object::Array(std::rc::Rc::new(std::cell::RefCell::new(arr)));
                self.carry_collection_variables(receiver, &copy);
                Ok(Some(copy))
            }
            Object::Dict(dict_rc) => {
                let dict = dict_rc.borrow().clone();
                let copy = Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(dict)));
                self.carry_collection_variables(receiver, &copy);
                Ok(Some(copy))
            }
            Object::Set(set_rc) => {
                let elements = set_rc.borrow().clone();
                let copy = Object::Set(std::rc::Rc::new(std::cell::RefCell::new(elements)));
                self.carry_collection_variables(receiver, &copy);
                // A set that places its elements by identity hands that on to
                // the copy, which holds the same elements.
                if self.set_by_identity(set_rc)
                    && let Object::Set(made) = &copy
                {
                    self.identity_sets
                        .insert(std::rc::Rc::as_ptr(made) as usize, copy.clone());
                }
                Ok(Some(copy))
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
            // A callable and a Binding copy as a new reference to the same
            // code and scope, so the copy is a separate object with the
            // instance variables the original carried.
            // A copy of a range is a range of its own, which `equal?` tells
            // apart from the one it was cut from.
            Object::Range {
                start,
                end,
                exclusive,
                ..
            } => Ok(Some(Object::Range {
                start: start.clone(),
                end: end.clone(),
                exclusive: *exclusive,
                mark: std::rc::Rc::new(()),
            })),
            Object::Method(method) => {
                let copy = Object::Method(std::rc::Rc::new((**method).clone()));
                self.carry_collection_variables(receiver, &copy);
                Ok(Some(copy))
            }
            Object::Block(block) => {
                let copy = Object::Block(std::rc::Rc::new((**block).clone()));
                self.carry_collection_variables(receiver, &copy);
                Ok(Some(copy))
            }
            Object::Binding(binding) => {
                let copy = Object::Binding(std::rc::Rc::new((**binding).clone()));
                self.carry_collection_variables(receiver, &copy);
                Ok(Some(copy))
            }
            // Immutable types return themselves
            _ => Ok(Some(receiver.clone())),
        }
    }

    /// Whether any finalizer has been registered at all, which is what
    /// makes a clone worth asking ObjectSpace about.
    fn has_finalizers(&self) -> bool {
        let Some(Object::Module(space) | Object::Class(space)) = self.globals().get("ObjectSpace")
        else {
            return false;
        };
        matches!(space.get_class_var("@finalizers"), Some(Object::Dict(held)) if !held.borrow().is_empty())
    }

    /// Give the copy the instance variables the original was carrying, for
    /// the objects that keep them against their address.
    fn carry_collection_variables(&mut self, receiver: &Object, copy: &Object) {
        let Some(from) = Self::collection_address(receiver) else {
            return;
        };
        let Some(held) = self.collection_variables.get(&from).cloned() else {
            return;
        };
        if let Some(to) = Self::collection_address(copy) {
            self.collection_variables.insert(to, held);
            self.collection_variable_owners.insert(to, copy.clone());
        }
    }

    /// The `freeze:` keyword `clone` was given: Some(true), Some(false), or
    /// None for `freeze: nil` and for no keyword at all.
    pub(crate) fn clone_freeze_argument(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<bool>, MetorexError> {
        let Some(Object::Dict(entries)) = arguments.first() else {
            if arguments.is_empty() {
                return Ok(None);
            }
            return Err(method_argument_error("clone", 0, arguments.len(), position));
        };
        let value = entries.borrow().get(":freeze").cloned();
        match value {
            None | Some(Object::Nil) => Ok(None),
            Some(Object::Bool(freeze)) => Ok(Some(freeze)),
            Some(other) => {
                let message = format!(
                    "unexpected value for freeze: {}",
                    self.builtins().class_of(&other).name()
                );
                Err(MetorexError::UncaughtException {
                    exception: Object::exception("ArgumentError", message.clone()),
                    location: position_to_location(position),
                    message,
                })
            }
        }
    }

    /// Carry over what `clone` copies beyond the instance variables: the
    /// singleton class, the frozen state, and the `initialize_clone` call.
    pub(crate) fn finish_clone(
        &mut self,
        receiver: &Object,
        copy: &Object,
        freeze: Option<bool>,
        arguments: &[Object],
        position: Position,
    ) -> Result<(), MetorexError> {
        // A string handed back with notice that it will be frozen carries that
        // notice into a clone, the way Ruby's does.
        if let (Object::String(original), Object::String(copied)) = (receiver, copy)
            && original.is_chilled()
        {
            let notice = original.take_chill();
            if let Some(notice) = notice {
                original.chill(notice.clone());
                copied.chill(notice);
            }
        }
        // A singleton class travels with a clone, so a method defined on the
        // original answers on the copy too.
        if let (Object::Instance(original), Object::Instance(copied)) = (receiver, copy) {
            let singleton = original.borrow().singleton_class.borrow().clone();
            if let Some(singleton) = singleton {
                let copied_singleton = crate::class::Class::duplicate(&singleton);
                *copied.borrow().singleton_class.borrow_mut() =
                    Some(std::rc::Rc::new(copied_singleton));
            }
            let methods = original.borrow().singleton_methods.borrow().clone();
            for (name, method) in methods {
                copied
                    .borrow_mut()
                    .singleton_methods
                    .borrow_mut()
                    .insert(name, method);
            }
        }
        // A collection keeps its singleton methods against its address, so
        // they are carried across the same way a class's are.
        if !matches!(receiver, Object::Instance(_)) {
            self.copy_singleton_methods(receiver, copy);
        }
        if let Some((class, method)) = self.lookup_method(copy, "initialize_clone")
            && !method.is_undefined
            && !method.body.is_empty()
        {
            let mut call_arguments = vec![receiver.clone()];
            call_arguments.extend(arguments.iter().cloned());
            self.invoke_method(class, method, copy.clone(), call_arguments, position)?;
        }
        // Ruby's `clone` copies the finalizers the original was given, so
        // both the original and the copy run theirs.
        if self.has_finalizers()
            && let Some(space) = self.globals().get("ObjectSpace")
        {
            self.send_to_object(
                space,
                "__carry_finalizers__",
                vec![receiver.clone(), copy.clone()],
                position,
            )?;
        }
        let frozen = match freeze {
            Some(freeze) => freeze,
            None => self.object_is_frozen(receiver),
        };
        if frozen {
            match copy {
                Object::Class(class) | Object::Module(class) => class.freeze(),
                Object::Instance(instance) => instance.borrow_mut().frozen = true,
                Object::String(text) => text.freeze(),
                Object::Array(_)
                | Object::Dict(_)
                | Object::Set(_)
                | Object::Method(_)
                | Object::Block(_)
                | Object::Binding(_)
                | Object::Regex(_, _) => {
                    if let Some(address) = Self::collection_address(copy) {
                        self.frozen_collections.insert(address, copy.clone());
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
