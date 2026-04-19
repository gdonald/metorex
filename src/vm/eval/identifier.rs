// Identifier resolution: local variables, methods on `self`, bare `new`.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::rc::Rc;

use crate::vm::core::VirtualMachine;
use crate::vm::errors::undefined_variable_error;

impl VirtualMachine {
    /// Evaluate a bare identifier expression.
    ///
    /// Resolution order:
    ///   1. Local variable / parameter from the environment.
    ///   2. If `self` is in scope: a method on the receiver (zero-arg auto-call,
    ///      bound `Method` for non-zero-arg methods).
    ///   3. Bare `new` inside a class method instantiates the class.
    ///   4. Otherwise raise an undefined-variable error.
    pub(super) fn eval_identifier(
        &mut self,
        name: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if let Some(val) = self.environment().get(name) {
            // Bare `to_s` at top-level must return "main" (Ruby's top-level self).
            if let Object::NativeFunction(fn_name) = &val
                && fn_name == "top_level_to_s"
            {
                return self.call_native_function(fn_name, vec![], position);
            }
            // Bare `using` always invokes (0-arg form raises ArgumentError).
            if let Object::NativeFunction(fn_name) = &val
                && fn_name == "using"
            {
                return self.call_native_function(fn_name, vec![], position);
            }
            return Ok(val);
        }

        // Constants (uppercase) resolve from globals regardless of scope.
        if name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && let Some(val) = self.globals().get(name)
        {
            return Ok(val);
        }

        let receiver = if let Some(r) = self.environment().get("self") {
            r
        } else {
            // Check global Object class for injected methods (mspec describe/it)
            if let Some(Object::Class(object_class)) = self.globals().get("Object")
                && let Some(method) = object_class.find_method(name)
                && !method.is_undefined
            {
                let required = method.parameters.len()
                    - method.default_parameters.len()
                    - if method.variadic_param.is_some() {
                        1
                    } else {
                        0
                    };
                if required == 0 {
                    return self.invoke_method(object_class, method, Object::Nil, vec![], position);
                } else {
                    let mut bound = method.as_ref().clone();
                    bound.receiver = Some(Box::new(Object::Nil));
                    return Ok(Object::Method(Rc::new(bound)));
                }
            }
            return Err(undefined_variable_error(name, position));
        };

        // Constant lookup: bare `NAME` inside a class/method resolves to the
        // class variable of the enclosing class (or the receiver's class),
        // walking the superclass chain — Ruby makes constants inherited from
        // superclasses visible at the child level too.
        if name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
            let class_opt = match &receiver {
                Object::Class(c) | Object::Module(c) => Some(Rc::clone(c)),
                Object::Instance(inst) => Some(Rc::clone(&inst.borrow().class)),
                _ => None,
            };
            if let Some(class) = class_opt {
                let mut current = Some(class);
                while let Some(cls) = current {
                    if let Some(val) = cls.get_class_var(name) {
                        return Ok(val);
                    }
                    current = cls.superclass();
                }
            }
            // Walk the lexical def-scope stack (outer class/module bodies) so a
            // nested class/module can reference sibling constants defined in an
            // enclosing module without qualifying them, and so a module can
            // reference itself by name before it has been bound in globals.
            for enclosing in self.def_scope_stack.iter().rev() {
                if let Some(val) = enclosing.get_class_var(name) {
                    return Ok(val);
                }
                if enclosing.name() == name {
                    return Ok(Object::Module(Rc::clone(enclosing)));
                }
            }
            // Constants defined in `class Object` are globally accessible (Ruby semantics).
            if let Some(Object::Class(object_class)) = self.globals().get("Object")
                && let Some(val) = object_class.get_class_var(name)
            {
                return Ok(val);
            }
        }

        // In class/module body, bare identifiers resolve methods on self.
        // Guard: don't re-invoke the current method (prevents infinite recursion)
        let in_same_method = self
            .get_current_method_name()
            .is_some_and(|frame| frame.ends_with(&format!("#{}", name)));
        if !in_same_method
            && let Some((class, method)) = self.lookup_method(&receiver, name)
            && !method.is_undefined
        {
            let required = method.parameters.len()
                - method.default_parameters.len()
                - if method.variadic_param.is_some() {
                    1
                } else {
                    0
                };
            if required == 0 {
                return self.invoke_method(class, method, receiver, vec![], position);
            } else {
                let mut bound = method.as_ref().clone();
                bound.receiver = Some(Box::new(receiver));
                return Ok(Object::Method(Rc::new(bound)));
            }
        }

        // Bare `new` inside a class method should instantiate the class.
        if name == "new"
            && let Object::Class(_) = &receiver
        {
            return self.invoke_callable(receiver, vec![], position);
        }

        // Fall back to native-method dispatch on `self` so identifiers that
        // map to native-only methods (e.g. `constants`, `const_get`) work
        // bare, the same way `self.constants` does.
        if !in_same_method {
            let class_for_native = self.builtins().class_of(&receiver);
            if let Ok(Some(result)) =
                self.call_native_method(&class_for_native, &receiver, name, &[], position)
            {
                return Ok(result);
            }
        }

        // Fallback: check global Object class (mspec injects describe/it/before/after)
        if let Some(Object::Class(object_class)) = self.globals().get("Object")
            && let Some(method) = object_class.find_method(name)
            && !method.is_undefined
        {
            let required = method.parameters.len()
                - method.default_parameters.len()
                - if method.variadic_param.is_some() {
                    1
                } else {
                    0
                };
            if required == 0 {
                return self.invoke_method(object_class, method, receiver, vec![], position);
            } else {
                let mut bound = method.as_ref().clone();
                bound.receiver = Some(Box::new(receiver));
                return Ok(Object::Method(Rc::new(bound)));
            }
        }

        Err(undefined_variable_error(name, position))
    }
}
