// Whether an object belongs to a class or one of its ancestors.

use super::*;

impl VirtualMachine {
    /// Whether an object belongs to a class or one of its ancestors.
    pub(crate) fn call_object_type_check_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
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
                // Metorex holds an encoding as a class of its own under
                // Encoding, so one is an instance of Encoding rather than of
                // Class, which is what `class` already reports.
                if self.names_an_encoding(receiver) {
                    return Ok(Some(Object::Bool(matches!(
                        target_class.name(),
                        "Encoding" | "Object" | "BasicObject"
                    ))));
                }
                // A class is a kind of the singleton class of itself and of
                // each class it inherits from, since those are the classes
                // its own singleton class inherits from.
                if let (Object::Class(named) | Object::Module(named), true) =
                    (receiver, target_class.is_singleton_class())
                    && let Some(Object::Class(attached) | Object::Module(attached)) =
                        target_class.get_class_var("__attached__")
                {
                    let mut current = Some(std::rc::Rc::clone(named));
                    while let Some(held) = current {
                        if std::rc::Rc::ptr_eq(&held, &attached) {
                            return Ok(Some(Object::Bool(true)));
                        }
                        current = held.superclass();
                    }
                    return Ok(Some(Object::Bool(false)));
                }
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
                // An exception is tagged with its type name rather than
                // carrying a class pointer, so its ancestry is walked from the
                // class that name resolves to.
                if let Object::Exception(details) = receiver {
                    let carried = details.borrow().class.clone().map(Object::Class);
                    let exception_type = details.borrow().exception_type.clone();
                    let resolved = match carried {
                        Some(class) => Some(class),
                        None => match self.globals().get(&exception_type) {
                            Some(class @ Object::Class(_)) => Some(class),
                            _ => self.resolve_qualified_constant(&exception_type),
                        },
                    };
                    if let Some(Object::Class(exception_class)) = resolved {
                        return Ok(Some(Object::Bool(
                            self.builtins()
                                .is_subclass_of(&exception_class, target_class),
                        )));
                    }
                }
                Ok(Some(Object::Bool(
                    self.builtins().is_instance_of(receiver, target_class),
                )))
            }
            _ => Ok(None),
        }
    }
}
