// Identifier resolution: local variables, methods on `self`, bare `new`.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::rc::Rc;

use crate::vm::core::VirtualMachine;
use crate::vm::errors::undefined_variable_error;

impl VirtualMachine {
    /// Whether `name` is bound to the very method a `def` installed on the
    /// default definee, rather than to a local variable holding a Method.
    pub(crate) fn name_is_a_definition(&self, name: &str, value: &Object) -> bool {
        let Object::Method(method) = value else {
            return false;
        };
        if method.name != name || method.receiver.is_some() {
            return false;
        }
        let same = |owner: &Rc<crate::class::Class>| {
            owner
                .find_own_method(name)
                .is_some_and(|installed| Rc::ptr_eq(&installed, method))
        };
        if self.def_scope_stack.iter().any(same) {
            return true;
        }
        matches!(self.globals().get("Object"), Some(Object::Class(object_class)) if same(&object_class))
    }

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
        // A class that does not descend from Object never reaches top-level
        // constants: Ruby finds them among Object's own, which such a class
        // does not inherit. Only the lexical chain answers there.
        if name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && !self.lexical_scope_reaches_top_level()
        {
            for enclosing in self.def_scope_stack.iter().rev() {
                if let Some(val) = enclosing.get_class_var(name) {
                    return Ok(val);
                }
                if enclosing.name() == name {
                    return Ok(Object::Class(Rc::clone(enclosing)));
                }
            }
            let message = format!("uninitialized constant {}", name);
            return Err(MetorexError::UncaughtException {
                exception: Object::exception("NameError", message.clone()),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        }
        if let Some(val) = self.environment().get(name) {
            // A method on `self` wins over a same-named Kernel function, so a
            // bare `to_s` inside a class reaches that class's `to_s` rather
            // than the top-level one.
            if matches!(val, Object::NativeFunction(_))
                && let Some(current_self) = self.environment().get("self")
                && let Some((class, method)) = self.lookup_method(&current_self, name)
                && !method.is_undefined
                && method.parameters.is_empty()
                && method.variadic_param.is_none()
            {
                return self.invoke_method(class, method, current_self, vec![], position);
            }
            // A few natives are always a call rather than a reference when
            // named bare: top-level `to_s` (Ruby's "main"), `using` (whose
            // 0-arg form raises ArgumentError), `abort` (whose 0-arg form
            // raises SystemExit), and the visibility modifiers, whose 0-arg
            // form is a toggle on the enclosing class or module.
            if let Object::NativeFunction(fn_name) = &val
                && (matches!(
                    fn_name.as_str(),
                    "top_level_to_s"
                        | "using"
                        | "__method__"
                        | "__callee__"
                        | "abort"
                        | "fail"
                        | "gets"
                        | "global_variables"
                        | "local_variables"
                        | "p"
                        | "pp"
                        | "proc"
                        | "print"
                        | "putc"
                        | "puts"
                        | "rand"
                        | "readline"
                        | "readlines"
                        | "srand"
                        | "throw"
                        | "binding_kernel"
                ) || (matches!(
                    fn_name.as_str(),
                    "module_function" | "private" | "public" | "protected"
                ) && self.self_is_class_or_module()))
            {
                return self.call_native_function(fn_name, vec![], position);
            }
            // A `def` registers its name in the environment as a Method so the
            // function is reachable, but Ruby's bare `foo` is a call, not a
            // reference to the method. Invoke it when the environment entry is
            // the very method the definee holds under that name and it needs
            // no arguments. A Method held in a local (from `method(:x)` or
            // `instance_method(:x)`) is a different object, so it stays a value.
            if let Object::Method(method) = &val
                && method.parameters.is_empty()
                && method.variadic_param.is_none()
                && self.name_is_a_definition(name, &val)
            {
                let receiver = self.environment().get("self").unwrap_or(Object::Nil);
                let class = self.builtins().class_of(&receiver);
                let method = Rc::clone(method);
                return self.invoke_method(class, method, receiver, vec![], position);
            }
            return Ok(val);
        }

        // Constants (uppercase) resolve from globals regardless of scope.
        if name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && let Some(val) = self.globals().get(name)
        {
            return Ok(val);
        }
        // Constant + def-scope chain has no class_var hit yet — try the
        // lexical chain for an autoload registration before falling through
        // to method dispatch. Walk the def-scope stack innermost-first;
        // `try_autoload_constant` itself walks the ancestor chain on each
        // module so the spec's `autoload :X, ...` on the parent module fires
        // even when we're nested deeper. The loaded file may define the
        // constant at top level (i.e. on globals/Object) rather than on the
        // module that owned the autoload entry, so re-check globals after
        // firing.
        if name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
            let scopes: Vec<_> = self.def_scope_stack.iter().rev().cloned().collect();
            for enclosing in scopes {
                if let Some(val) = self.try_autoload_constant(&enclosing, name)? {
                    return Ok(val);
                }
                // The file may have defined the constant globally rather than
                // on the autoload owner — fall back to globals before moving on.
                if let Some(val) = self.globals().get(name) {
                    return Ok(val);
                }
            }
            if let Some(Object::Class(object_class)) = self.globals().get("Object")
                && let Some(val) = self.try_autoload_constant(&object_class, name)?
            {
                return Ok(val);
            }
            if let Some(val) = self.globals().get(name) {
                return Ok(val);
            }
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
                    // Constants defined in included/prepended modules are
                    // visible at the including class — walk the mixin
                    // chain at each ancestor level too.
                    for mixin in cls.transitive_mixins() {
                        if let Some(val) = mixin.get_class_var(name) {
                            return Ok(val);
                        }
                    }
                    // Fire any autoload registered for this name on this
                    // ancestor (or its mixins/superclasses) so a method
                    // body referencing `MetaScope` triggers the load even
                    // when def_scope_stack doesn't include this class.
                    if let Some(val) = self.try_autoload_constant(&cls, name)? {
                        return Ok(val);
                    }
                    current = cls.superclass();
                }
            }
            // Inside a method body the lexical scope is the one open where the
            // method was defined, which is what `Module.nesting` reports. A
            // method on a nested class reaches its own name and its enclosing
            // module's constants through it.
            if let Some(nesting) = self.method_nesting_stack.last() {
                for enclosing in nesting.clone() {
                    if enclosing.name() == name {
                        return Ok(if enclosing.is_module() {
                            Object::Module(enclosing)
                        } else {
                            Object::Class(enclosing)
                        });
                    }
                    if let Some(val) = enclosing.get_class_var(name) {
                        return Ok(val);
                    }
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
            // The Object/Kernel natives (`object_id`, `frozen?`, `inspect`)
            // are reachable bare too, the same way `self.object_id` is.
            if let Ok(Some(result)) = self.call_object_method(&receiver, name, &[], position) {
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
