// Identifier resolution: local variables, methods on `self`, bare `new`.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::rc::Rc;

use crate::vm::core::VirtualMachine;
use crate::vm::errors::undefined_variable_error;

impl VirtualMachine {
    /// The object a NameError for `name` was raised on. A constant belongs to
    /// `Object` when nothing namespaces it, and any other name to `self`.
    pub(crate) fn name_error_receiver(&self, name: &str) -> Option<Object> {
        if name.starts_with(char::is_uppercase) {
            return self.globals().get("Object");
        }
        // The object the program runs against is named `main` however the
        // code reached it, a binding included.
        match self.environment().get("self") {
            Some(held) if self.is_the_main_object(&held) => None,
            other => other,
        }
    }

    /// A NameError for a constant nothing defines, raised on `Object`.
    pub(crate) fn constant_name_error(&self, message: &str, name: &str) -> Object {
        let exception = Object::exception("NameError", message);
        if let Object::Exception(details) = &exception {
            let mut details = details.borrow_mut();
            details.name = Some(name.to_string());
            details.receiver = self.globals().get("Object").map(Box::new);
        }
        exception
    }

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
        // The class the `def` installed it in, such as the singleton class
        // of an object `instance_exec` ran a block against, holds it too.
        if self.def_scope_stack.iter().any(same) || method.definee.as_ref().is_some_and(same) {
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
            for enclosing in self.def_scope_stack.clone().iter().rev() {
                if let Some(val) = enclosing.get_class_var(name) {
                    return Ok(val);
                }
                // A name this scope registered an autoload for is loaded
                // here rather than looked for further out.
                if enclosing.lookup_autoload(name).is_some()
                    && let Some(val) = self.try_autoload_constant(enclosing, name)?
                {
                    return Ok(val);
                }
                if enclosing.name() == name {
                    return Ok(Object::Class(Rc::clone(enclosing)));
                }
            }
            let message = format!("uninitialized constant {}", name);
            return Err(MetorexError::UncaughtException {
                exception: self.constant_name_error(&message, name),
                location: crate::vm::utils::position_to_location(position),
                message,
            });
        }
        // TOPLEVEL_BINDING stands over the main script's own scope, so the
        // locals it names are whatever that scope holds right now.
        if name == "TOPLEVEL_BINDING" {
            self.refresh_toplevel_binding();
        }
        // A constant of a scope open where the code was written, or of an
        // ancestor of the innermost one, comes ahead of the top level's,
        // which is how a class's own NameError hides Ruby's.
        // A scope with an autoload registered for the name holds it there,
        // and loading it is left to the lookup further on.
        if name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && !self.autoload_pending_in_lexical_scope(name)
            && let Some(val) = self.constant_before_object(name)
        {
            return Ok(val);
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
            // 0-arg form raises ArgumentError), `abort` and `exit` (whose
            // 0-arg forms raise SystemExit), and the visibility modifiers,
            // whose 0-arg form is a toggle on the enclosing class or module.
            // `to_s` answers "main" only at the top level. Inside a class
            // or module body, and on any other receiver, it is a call on
            // whatever `self` is there.
            if let Object::NativeFunction(fn_name) = &val
                && fn_name == "top_level_to_s"
                && let Some(current_self) = self.environment().get("self")
                && !matches!(current_self, Object::Nil)
                && (!self.def_scope_stack.is_empty()
                    || !matches!(&current_self, Object::Class(held) if held.name() == "Object"))
            {
                return self.send_to_object(current_self, "to_s", vec![], position);
            }
            if let Object::NativeFunction(fn_name) = &val
                && (runs_when_named_bare(fn_name)
                    || (matches!(
                        fn_name.as_str(),
                        "module_function" | "private" | "public" | "protected"
                    ) && (self.self_is_class_or_module()
                        || (self.def_scope_stack.is_empty()
                            && self.current_method_frame == self.toplevel_frame))))
            {
                return self.call_native_function(fn_name, vec![], position);
            }
            // A `def` registers its name in the environment as a Method so the
            // function is reachable, but Ruby's bare `foo` is a call, not a
            // reference to the method, and one missing its arguments raises.
            // Invoke it when the environment entry is the very method the
            // definee holds under that name. A Method held in a local (from
            // `method(:x)` or `instance_method(:x)`) is a different object, so
            // it stays a value.
            if let Object::Method(method) = &val
                && self.name_is_a_definition(name, &val)
            {
                let receiver = self.environment().get("self").unwrap_or(Object::Nil);
                let class = self.builtins().class_of(&receiver);
                let method = Rc::clone(method);
                return self.invoke_method(class, method, receiver, vec![], position);
            }
            return Ok(val);
        }

        // Constants (uppercase) resolve from globals, unless a scope open
        // where the code was written, or an ancestor of the innermost one,
        // holds one of the same name ahead of Object.
        if name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && let Some(val) = self.globals().constant(name)
        {
            return Ok(self.constant_before_object(name).unwrap_or(val));
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
                if let Some(val) = self.globals().constant(name) {
                    return Ok(val);
                }
            }
            if let Some(Object::Class(object_class)) = self.globals().get("Object")
                && let Some(val) = self.try_autoload_constant(&object_class, name)?
            {
                return Ok(val);
            }
            if let Some(val) = self.globals().constant(name) {
                return Ok(val);
            }
        }

        let receiver = if let Some(r) = self.environment().get("self") {
            r
        } else {
            // A constant written in a `class Object` body, or in a module
            // included at the top level, is one the top level reads.
            if name.starts_with(char::is_uppercase)
                && let Some(value) = self.object_constant(name)
            {
                return Ok(value);
            }
            // At the top level `self` is `main`, the object TOPLEVEL_BINDING
            // was built around. Nothing binds it there, so it is answered
            // here rather than reported as an undefined name.
            if name == "self"
                && let Some(Object::Binding(binding)) = self.globals().get("TOPLEVEL_BINDING")
                && let Some(main) = binding.receiver.clone()
            {
                return Ok(main);
            }
            // Check global Object class for injected methods (mspec describe/it)
            if let Some(Object::Class(object_class)) = self.globals().get("Object")
                && let Some(method) = object_class.find_method(name)
                && !method.is_undefined
            {
                // A bare name is a call, and one missing arguments raises.
                return self.invoke_method(object_class, method, Object::Nil, vec![], position);
            }
            // At the top level a bare name may name a method written on
            // `main` alone, which nothing else answers to.
            if let Ok(main) = self.eval_self(position)
                && let Some((owner, method)) = self.lookup_method(&main, name)
                && !method.is_undefined
            {
                return self.invoke_method(owner, method, main, vec![], position);
            }
            return Err(undefined_variable_error(
                name,
                self.name_error_receiver(name),
                position,
            ));
        };

        // Constant lookup: bare `NAME` inside a class/method resolves to the
        // class variable of the enclosing class (or the receiver's class),
        // walking the superclass chain — Ruby makes constants inherited from
        // superclasses visible at the child level too.
        if name.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
            // Walk the lexical def-scope stack (outer class/module bodies) so a
            // nested class/module can reference sibling constants defined in an
            // enclosing module without qualifying them, and so a module can
            // reference itself by name before it has been bound in globals.
            for enclosing in self.def_scope_stack.iter().rev() {
                if let Some(val) = enclosing.get_class_var(name) {
                    return Ok(val);
                }
                if enclosing.name() == name {
                    return Ok(crate::vm::class_execution::constants::scope_object(
                        enclosing,
                    ));
                }
            }
            // Inside a method body the lexical scope is the one open where the
            // method was defined, which is what `Module.nesting` reports. A
            // method on a nested class reaches its own name and its enclosing
            // module's constants through it.
            if let Some(nesting) = self.method_nesting_stack.last().cloned() {
                for enclosing in nesting {
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
                    if enclosing.lookup_autoload(name).is_some()
                        && let Some(val) = self.try_autoload_constant(&enclosing, name)?
                    {
                        return Ok(val);
                    }
                }
            }
            // What the name is looked up through after the open scopes: the
            // ancestors of the innermost scope the code was written in. Code
            // written in no scope at all reads through whatever it is
            // running against.
            let class_opt = self
                .method_nesting_stack
                .last()
                .and_then(|nesting| nesting.first().cloned())
                .or_else(|| self.def_scope_stack.last().cloned())
                .or_else(|| match &receiver {
                    Object::Class(c) | Object::Module(c) => Some(Rc::clone(c)),
                    Object::Instance(inst) => Some(Rc::clone(&inst.borrow().class)),
                    _ => None,
                })
                // A singleton class stands for whatever it is attached to,
                // whose ancestors are the ones the name is looked up through.
                .map(|held| match held.get_class_var("__attached__") {
                    Some(Object::Class(attached) | Object::Module(attached))
                        if held.is_singleton_class() =>
                    {
                        attached
                    }
                    _ => match (held.is_singleton_class(), &receiver) {
                        (true, Object::Instance(inst)) => Rc::clone(&inst.borrow().class),
                        _ => held,
                    },
                });
            if let Some(class) = class_opt {
                let mut current = Some(class);
                while let Some(cls) = current {
                    // A prepended module sits ahead of the class, so its
                    // constants shadow the class's own.
                    for prepended in cls.transitive_prepends() {
                        if let Some(val) = prepended.get_class_var(name) {
                            return Ok(val);
                        }
                    }
                    if let Some(val) = cls.get_class_var(name) {
                        return Ok(val);
                    }
                    // Constants defined in included modules are visible at
                    // the including class — walk the mixin chain at each
                    // ancestor level too.
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
            // Constants defined in `class Object`, or in a module included
            // there, are reachable from anywhere.
            if let Some(val) = self.object_constant(name) {
                return Ok(val);
            }
        }

        // In class/module body, bare identifiers resolve methods on self.
        if let Some((class, method)) = self.lookup_method(&receiver, name)
            && !method.is_undefined
        {
            // A bare name is a call, and one missing arguments raises.
            return self.invoke_method(class, method, receiver, vec![], position);
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
        let class_for_native = self.builtins().class_of(&receiver);
        if let Ok(Some(result)) =
            self.call_native_method(&class_for_native, &receiver, name, &[], position)
        {
            return Ok(result);
        }
        // The Object/Kernel natives (`object_id`, `frozen?`, `inspect`) are
        // reachable bare too, the same way `self.object_id` is.
        if let Ok(Some(result)) = self.call_object_method(&receiver, name, &[], position) {
            return Ok(result);
        }

        // Fallback: check global Object class (mspec injects describe/it/before/after)
        if let Some(Object::Class(object_class)) = self.globals().get("Object")
            && let Some(method) = object_class.find_method(name)
            && !method.is_undefined
        {
            // A bare name is a call, and one missing arguments raises.
            return self.invoke_method(object_class, method, receiver, vec![], position);
        }

        // Bare `include` / `prepend` in a class or module body is a call with
        // no arguments, so Ruby reports the missing module rather than a
        // missing name.
        if matches!(name, "include" | "prepend")
            && let Some(receiver @ (Object::Class(_) | Object::Module(_))) =
                self.environment().get("self")
        {
            let (Object::Class(class_rc) | Object::Module(class_rc)) = &receiver else {
                unreachable!("the match above admits only classes and modules")
            };
            let class_rc = Rc::clone(class_rc);
            return self
                .call_native_method(&class_rc, &receiver, name, &[], position)
                .map(|result| result.unwrap_or(Object::Nil));
        }

        // A fiber body runs with a scope of its own, which does not reach
        // the one the interpreter's own functions were registered in. The
        // bare name still names one of them.
        if let Some(Object::NativeFunction(fn_name)) = self.globals().get(name)
            && runs_when_named_bare(&fn_name)
        {
            return self.call_native_function(&fn_name, vec![], position);
        }

        // A constant no scope holds is asked of `const_missing` on the class
        // or module the code was written in.
        if name.starts_with(char::is_uppercase)
            && let Some(written_in) = self
                .method_nesting_stack
                .last()
                .and_then(|nesting| nesting.first().cloned())
                .or_else(|| self.def_scope_stack.last().cloned())
        {
            return self.dispatch_const_missing(&written_in, name, position);
        }
        Err(undefined_variable_error(
            name,
            self.name_error_receiver(name),
            position,
        ))
    }
}

/// Whether a Kernel function runs when its bare name is evaluated rather than
/// answering the function itself: top-level `to_s` (Ruby's "main"), `using`
/// (whose 0-arg form raises ArgumentError), `abort` and `exit` (whose 0-arg
/// forms raise SystemExit), and the rest whose 0-arg form does the work.
pub(crate) fn runs_when_named_bare(name: &str) -> bool {
    matches!(
        name,
        "top_level_to_s"
            | "using"
            | "__method__"
            | "__callee__"
            | "abort"
            | "at_exit"
            | "caller"
            | "caller_locations"
            | "chomp"
            | "chop"
            | "fork"
            | "loop"
            | "open"
            | "exit"
            | "exit!"
            | "fail"
            | "gets"
            | "global_variables"
            | "local_variables"
            | "p"
            | "pp"
            | "proc"
            | "print"
            | "puts"
            | "rand"
            | "sleep"
            | "readline"
            | "readlines"
            | "srand"
            | "throw"
            | "binding_kernel"
    )
}

impl VirtualMachine {
    /// The constant `name` names in the scopes open where the running code
    /// was written, or in the ancestors of the innermost of them short of
    /// Object. Top-level code has no such scope. Fires no autoload.
    pub(crate) fn constant_before_object(&self, name: &str) -> Option<Object> {
        let nesting: Vec<Rc<crate::class::Class>> = match self.method_nesting_stack.last() {
            Some(nesting) if !nesting.is_empty() => nesting.clone(),
            _ => self.def_scope_stack.iter().rev().cloned().collect(),
        };
        let innermost = nesting.first()?;
        if let Some(value) = nesting.iter().find_map(|scope| scope.get_class_var(name)) {
            return Some(value);
        }
        let mut cursor = Some(Rc::clone(innermost));
        while let Some(class) = cursor {
            if class.name() == "Object" {
                break;
            }
            let found = class
                .transitive_prepends()
                .iter()
                .find_map(|prepended| prepended.get_class_var(name))
                .or_else(|| class.get_class_var(name))
                .or_else(|| {
                    class
                        .transitive_mixins()
                        .iter()
                        .find_map(|mixin| mixin.get_class_var(name))
                });
            if found.is_some() {
                return found;
            }
            cursor = class.superclass();
        }
        None
    }

    /// Whether a scope open where the code was written has an autoload
    /// registered for `name` and no constant of that name yet.
    fn autoload_pending_in_lexical_scope(&self, name: &str) -> bool {
        let nesting: Vec<Rc<crate::class::Class>> = match self.method_nesting_stack.last() {
            Some(nesting) if !nesting.is_empty() => nesting.clone(),
            _ => self.def_scope_stack.iter().rev().cloned().collect(),
        };
        nesting.iter().any(|scope| {
            scope.get_class_var(name).is_none() && scope.lookup_autoload(name).is_some()
        })
    }

    /// The constant `name` names on Object, or in a module included there.
    pub(crate) fn object_constant(&self, name: &str) -> Option<Object> {
        let Some(Object::Class(object_class)) = self.globals().get("Object") else {
            return None;
        };
        object_class.get_class_var(name).or_else(|| {
            object_class
                .transitive_mixins()
                .iter()
                .find_map(|mixin| mixin.get_class_var(name))
        })
    }
}
