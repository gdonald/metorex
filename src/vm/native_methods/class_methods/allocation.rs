// Building an instance without running `initialize`.

use super::*;

impl VirtualMachine {
    /// `allocate` and the classes that refuse it, whose instances are
    /// primitives with no uninitialized form.
    pub(crate) fn call_allocation_class_methods(
        &mut self,
        class_rc: &Rc<Class>,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<ClassMethodAnswer, MetorexError> {
        // The top-level `self` is Object, so a `using` sent to it is
        // `main.using`. Ruby permits that only at the top level, which a
        // class or module body is not.
        if method_name == "using" && class_rc.name() == "Object" {
            let inside_body = matches!(
                self.environment().get("self"),
                Some(Object::Class(current) | Object::Module(current)) if current.name() != "Object"
            );
            if inside_body {
                let message = "main.using is permitted only at toplevel".to_string();
                return Err(MetorexError::UncaughtException {
                    exception: Object::exception("RuntimeError", message.clone()),
                    location: position_to_location(position),
                    message,
                });
            }
            return self
                .call_native_function("using", arguments.to_vec(), position)
                .map(Answered);
        }
        // A singleton class belongs to the one object it was made for.
        if class_rc.is_singleton_class() && matches!(method_name, "new" | "allocate") {
            return Err(crate::vm::errors::simple_exception(
                "TypeError",
                "can't create instance of singleton class",
                position,
            ));
        }
        // A number is not built by hand: `Float.new`, `Rational.new` and
        // `Complex.new` do not exist, and `allocate` has nothing to allocate.
        if matches!(
            class_rc.name(),
            "Float" | "Rational" | "Complex" | "Integer"
        ) && matches!(method_name, "new" | "allocate")
        {
            let message = format!(
                "undefined method '{}' for class '{}'",
                method_name,
                class_rc.name()
            );
            let exception = if method_name == "allocate" {
                Object::exception(
                    "TypeError",
                    format!("allocator undefined for {}", class_rc.name()),
                )
            } else {
                crate::vm::errors::no_method_error(
                    &message,
                    method_name,
                    &Object::Class(Rc::clone(class_rc)),
                    arguments,
                )
            };
            return Err(MetorexError::UncaughtException {
                exception,
                location: position_to_location(position),
                message,
            });
        }
        // A Thread has nothing to be without the block that gives it something
        // to run, so Ruby refuses to hand back an uninitialized one.
        // An allocated String holds nothing and is read as bytes until it is
        // told otherwise.
        if method_name == "allocate" && class_rc.name() == "String" {
            return Ok(Answered(Object::String(Rc::new(
                crate::object::StringValue::from_bytes(String::new()),
            ))));
        }
        if method_name == "allocate" && class_rc.name() == "Thread" {
            return Err(crate::vm::errors::simple_exception(
                "TypeError",
                "allocator undefined for Thread",
                position,
            ));
        }
        // `allocate` on a class whose instances are primitives answers an
        // empty one of them, since there is no separate uninitialized form.
        if method_name == "allocate" && matches!(class_rc.name(), "Array" | "Hash" | "Set") {
            if !arguments.is_empty() {
                return Err(method_argument_error(
                    "allocate",
                    0,
                    arguments.len(),
                    position,
                ));
            }
            return Ok(Answered(match class_rc.name() {
                "Array" => Object::empty_array(),
                "Hash" => Object::empty_dict(),
                _ => Object::empty_set(),
            }));
        }
        // A Proc has no allocator, and MatchData has no `allocate` at all.
        if method_name == "allocate" && class_rc.name() == "MatchData" {
            let message = "undefined method 'allocate' for class 'MatchData'".to_string();
            return Err(MetorexError::UncaughtException {
                exception: crate::vm::errors::no_method_error(
                    &message,
                    method_name,
                    &Object::Class(Rc::clone(class_rc)),
                    arguments,
                ),
                location: position_to_location(position),
                message,
            });
        }
        // `Proc.new` builds one from a block, but there is no uninitialized
        // Proc for `allocate` to hand back.
        if (lacks_an_allocator(class_rc) || class_rc.name() == "Proc") && method_name == "allocate"
        {
            let exc = Object::exception(
                "TypeError",
                format!("allocator undefined for {}", class_rc.name()),
            );
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message: format!("allocator undefined for {}", class_rc.name()),
            });
        }
        // Class.allocate and subclasses: uninitialized class instance. `new` and
        // `superclass` on it must raise TypeError (Ruby semantics).
        if class_rc.get_class_var("__uninitialized__").is_some()
            && matches!(method_name, "new" | "superclass")
        {
            let message = "uninitialized class".to_string();
            let exc = Object::exception("TypeError", message.clone());
            return Err(MetorexError::UncaughtException {
                exception: exc,
                location: position_to_location(position),
                message,
            });
        }
        if method_name == "allocate"
            && let Some(made) = self.allocate_through_c(class_rc, position)?
        {
            return Ok(Answered(made));
        }
        // A class that defines its own `allocate` answers with that, so the
        // uninitialized instance below is not what it hands back.
        if method_name == "allocate" && self.class_method_of(class_rc, "allocate").is_some() {
            return Ok(Deferred);
        }
        if method_name == "allocate" {
            if class_rc.name() == "Class" {
                let anon = Class::new("", None);
                anon.set_class_var("__uninitialized__", Object::Bool(true));
                return Ok(Answered(Object::Class(anon)));
            }
            // An exception class allocates an exception, with no message
            // given yet, which is what Marshal builds one back from.
            if self.is_exception_class(class_rc) {
                let made = Object::exception(class_rc.name(), String::new());
                if let Object::Exception(details) = &made {
                    let mut details = details.borrow_mut();
                    details.class = Some(Rc::clone(class_rc));
                    details.message_given = false;
                }
                return Ok(Answered(made));
            }
            let inst = crate::object::Instance::new(Rc::clone(class_rc));
            // A subclass of String, Array or Hash starts out holding an empty
            // one, which is what its methods then work on.
            let descends = |name: &str| {
                class_rc.name() != name
                    && crate::vm::method_invocation::descends_from(class_rc, name)
            };
            let backing = if descends("String") {
                Some((
                    crate::vm::native_methods::STRING_SUBCLASS_VAR,
                    self.string_from_new_arguments(&[], position)?,
                ))
            } else if descends("Array") {
                Some((
                    crate::vm::native_methods::ARRAY_SUBCLASS_VAR,
                    Object::array(Vec::new()),
                ))
            } else if descends("Hash") {
                Some((
                    crate::vm::native_methods::HASH_SUBCLASS_VAR,
                    Object::Dict(Rc::new(std::cell::RefCell::new(indexmap::IndexMap::new()))),
                ))
            } else {
                None
            };
            if let Some((slot, value)) = backing {
                inst.borrow_mut().set_var(slot.to_string(), value);
            }
            return Ok(Answered(Object::Instance(inst)));
        }
        Ok(Unclaimed)
    }
}
