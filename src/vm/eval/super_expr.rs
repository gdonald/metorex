// `super` expression: dispatch to the same method in the parent class.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::rc::Rc;

use crate::vm::core::VirtualMachine;
use crate::vm::utils::position_to_location;

impl VirtualMachine {
    /// Evaluate a `super` call. Walks the inheritance chain from the receiver's
    /// class to find the class defining the current method, then invokes the
    /// same-named method on that class's superclass.
    pub(super) fn eval_super(
        &mut self,
        arguments: &[Expression],
        forward_args: bool,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // Get the current method name from the call stack.
        // The call stack stores method names as "Class#method".
        let current_frame = self.get_current_method_name().ok_or_else(|| {
            MetorexError::runtime_error(
                "super called outside of a method context".to_string(),
                position_to_location(position),
            )
        })?;

        // Extract the class name and method name (format: "Class#method")
        let (class_name, method_name) = if let Some(pos) = current_frame.rfind('#') {
            (
                current_frame[..pos].to_string(),
                current_frame[pos + 1..].to_string(),
            )
        } else {
            return Err(MetorexError::runtime_error(
                "super called in invalid context (no class information)".to_string(),
                position_to_location(position),
            ));
        };

        // Class-method super: `def self.foo; super; end` — walk self's
        // ancestor chain looking for a matching *class* method (stored under
        // the `__class__` prefix or on the class's singleton class). When the
        // method was mixed in (e.g. `class << F; include M; end`), the defining
        // class may not be self's class itself; walking from self's immediate
        // superclass handles that case.
        if let Some(self_object @ (Object::Class(_) | Object::Module(_))) =
            self.environment().get("self")
        {
            let (Object::Class(self_class) | Object::Module(self_class)) = &self_object else {
                unreachable!("the match above admits only classes and modules")
            };
            let self_class = Rc::clone(self_class);
            let evaluated_args = if forward_args {
                self.method_arg_stack.last().cloned().unwrap_or_default()
            } else {
                let mut evaluated_args = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    evaluated_args.push(self.evaluate_expression(arg)?);
                }
                evaluated_args
            };
            // A module extended with another module reaches that module's
            // copy through `super`: the extended copy sits just above the
            // receiver's own module-level method in the singleton chain.
            if let Some(Object::Method(extended)) =
                self_class.get_class_var(&format!("__ext__{}", method_name))
            {
                return self.invoke_method(
                    Rc::clone(&self_class),
                    extended,
                    self_object,
                    evaluated_args,
                    position,
                );
            }
            let class_method_key = format!("__class__{}", method_name);
            let mut current = self_class.superclass();
            while let Some(cls) = current {
                let candidate = cls
                    .singleton_class_slot()
                    .clone()
                    .and_then(|sc| sc.find_method(&method_name))
                    .or_else(|| cls.find_method(&class_method_key));
                if let Some(method) = candidate {
                    return self.invoke_method(
                        Rc::clone(&cls),
                        method,
                        self_object,
                        evaluated_args,
                        position,
                    );
                }
                current = cls.superclass();
            }
            // The mixin hooks have real default implementations rather than
            // no-op ones: `super` from an override performs the default.
            match method_name.as_str() {
                "append_features" | "prepend_features" => {
                    if let Some(Object::Class(target) | Object::Module(target)) =
                        evaluated_args.first()
                    {
                        let target = Rc::clone(target);
                        self.default_append_features(&target, &self_class, position)?;
                        return Ok(Object::Module(self_class));
                    }
                }
                "extend_object" => {
                    if let Some(target) = evaluated_args.first() {
                        let target = target.clone();
                        self.default_extend_object(&target, &self_class, position)?;
                        return Ok(Object::Module(self_class));
                    }
                }
                _ => {}
            }
            // No superclass class method. The Module and Class hooks have
            // silent no-op default implementations, so `super` from an
            // override of one of them returns nil. Any other unresolved
            // class-method super is an error.
            if matches!(
                method_name.as_str(),
                "inherited" | "included" | "extended" | "prepended" | "const_added"
            ) {
                return Ok(Object::Nil);
            }
            return Err(MetorexError::runtime_error(
                format!(
                    "super: no superclass method '{}' for {}",
                    method_name,
                    self_class.ruby_name()
                ),
                position_to_location(position),
            ));
        }

        // Get the current self (must be an instance)
        let instance = match self.environment().get("self") {
            Some(Object::Instance(instance_rc)) => instance_rc,
            // An exception's built-in behaviour already ran, so `super` from
            // an override of one of these has nothing left to reach.
            Some(Object::Exception(_))
                if matches!(method_name.as_str(), "initialize" | "initialize_copy") =>
            {
                return Ok(Object::Nil);
            }
            Some(_) => {
                return Err(MetorexError::runtime_error(
                    "super can only be called from within an instance method".to_string(),
                    position_to_location(position),
                ));
            }
            None => {
                return Err(MetorexError::runtime_error(
                    "super can only be called from within a method".to_string(),
                    position_to_location(position),
                ));
            }
        };

        // Get the instance's class to walk the inheritance chain
        let instance_borrowed = instance.borrow();
        let instance_class = &instance_borrowed.class;

        // Find the class that matches the current frame's class name. We walk
        // the full ancestor chain — own class, its mixins, its superclass (and
        // that superclass's mixins, recursively) — so that methods mixed in
        // via `include` are reachable as the defining class.
        fn walk_ancestors(class: &Rc<Class>) -> Vec<Rc<Class>> {
            let mut out = Vec::new();
            // A prepended module sits ahead of the class, so `super` from it
            // reaches the class's own copy of the method.
            for prepended in class.prepend_chain() {
                for anc in walk_ancestors(&prepended) {
                    if !out.iter().any(|c| Rc::ptr_eq(c, &anc)) {
                        out.push(anc);
                    }
                }
            }
            if !out.iter().any(|c| Rc::ptr_eq(c, class)) {
                out.push(Rc::clone(class));
            }
            for mixin in class.mixin_chain() {
                // Include the mixin itself AND any modules it transitively
                // includes, so e.g. Target→Child→Parent is visited when only
                // `Target includes Child` and `Child includes Parent`.
                for anc in walk_ancestors(&mixin) {
                    if !out.iter().any(|c| Rc::ptr_eq(c, &anc)) {
                        out.push(anc);
                    }
                }
            }
            if let Some(sc) = class.superclass() {
                for anc in walk_ancestors(&sc) {
                    if !out.iter().any(|c| Rc::ptr_eq(c, &anc)) {
                        out.push(anc);
                    }
                }
            }
            out
        }
        use crate::class::Class;
        // A singleton method's `super` reaches the class's own copy, so the
        // receiver's singleton class comes ahead of its class in the chain.
        let mut chain = Vec::new();
        if let Some(singleton) = instance_borrowed.singleton_class.borrow().clone() {
            chain.push(singleton);
        }
        for ancestor in walk_ancestors(instance_class) {
            if !chain.iter().any(|c| Rc::ptr_eq(c, &ancestor)) {
                chain.push(ancestor);
            }
        }
        // The running method records the module it was defined in, which is
        // the only way to place a method from an anonymous module or from one
        // of two modules that share a name. Matching the frame's class name
        // covers the cases where no owner was recorded.
        let recorded_owner = self
            .method_owner_stack
            .last()
            .cloned()
            .flatten()
            .filter(|owner| chain.iter().any(|c| Rc::ptr_eq(c, owner)));
        let defining_class = recorded_owner
            .or_else(|| chain.iter().find(|c| c.name() == class_name).cloned())
            .ok_or_else(|| {
                MetorexError::runtime_error(
                    format!(
                        "Could not find defining class '{}' in inheritance chain",
                        class_name
                    ),
                    position_to_location(position),
                )
            })?;

        // Locate the method in the ancestor chain AFTER the defining class.
        // In Ruby, `super` looks up the next method in the chain — that may
        // live on the defining class's superclass, OR on a mixin that the
        // defining class brings in, OR on a class/module further up.
        let next_in_chain: Option<(Rc<Class>, Rc<crate::object::Method>)> = {
            let mut found = None;
            // Position in the overall chain where defining_class sits.
            let start_idx = chain.iter().position(|c| Rc::ptr_eq(c, &defining_class));
            if let Some(idx) = start_idx {
                // The chain is already linearized, so each ancestor is asked
                // only for its own method. Walking its mixins and prepends
                // again would hand back the method `super` was called from.
                for anc in chain.iter().skip(idx + 1) {
                    if let Some(method) = anc.find_own_method(&method_name) {
                        found = Some((Rc::clone(anc), method));
                        break;
                    }
                }
            }
            found
        };
        if let Some((owner, method)) = next_in_chain {
            let evaluated_args = if forward_args {
                self.method_arg_stack.last().cloned().unwrap_or_default()
            } else {
                let mut evaluated_args = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    evaluated_args.push(self.evaluate_expression(arg)?);
                }
                evaluated_args
            };
            drop(instance_borrowed);
            return self.invoke_method(
                owner,
                method,
                Object::Instance(Rc::clone(&instance)),
                evaluated_args,
                position,
            );
        }

        // Get the parent class of the defining class. If there's no superclass,
        // fall back to Object semantics for well-known methods.
        let parent_class = match defining_class.superclass() {
            Some(p) => p,
            None => {
                // Object#<=> returns 0 if self.equal?(other), else nil
                if method_name == "<=>" {
                    drop(instance_borrowed);
                    let mut evaluated_args = Vec::with_capacity(arguments.len());
                    for arg in arguments {
                        evaluated_args.push(self.evaluate_expression(arg)?);
                    }
                    let self_val = self.environment().get("self").unwrap_or(Object::Nil);
                    if let Some(other) = evaluated_args.first() {
                        let same = matches!(
                            (&self_val, other),
                            (Object::Instance(a), Object::Instance(b)) if Rc::ptr_eq(a, b)
                        );
                        return Ok(if same { Object::Int(0) } else { Object::Nil });
                    }
                    return Ok(Object::Nil);
                }
                // A prepended module whose class does not define the method
                // has nothing left above it, which Ruby reports as a
                // NoMethodError naming the method.
                let self_val = self.environment().get("self").unwrap_or(Object::Nil);
                let message = format!(
                    "super: no superclass method '{}' for an instance of {}",
                    method_name,
                    self.builtins().class_of(&self_val).name()
                );
                return Err(MetorexError::UncaughtException {
                    exception: crate::vm::errors::no_method_error(
                        &message,
                        &method_name,
                        &self_val,
                        &[],
                    ),
                    location: position_to_location(position),
                    message,
                });
            }
        };

        // Look up the method in the parent class. If not found and the method
        // is one of the well-known Object-level fallbacks (e.g. `<=>`), mirror
        // the None-superclass branch above so e.g. `super` in a user class's
        // `<=>` still degrades to `self.equal?(other) ? 0 : nil`.
        let method = match parent_class.find_method(&method_name) {
            Some(m) => m,
            None if method_name == "<=>" => {
                drop(instance_borrowed);
                let mut evaluated_args = Vec::with_capacity(arguments.len());
                for arg in arguments {
                    evaluated_args.push(self.evaluate_expression(arg)?);
                }
                let self_val = self.environment().get("self").unwrap_or(Object::Nil);
                if let Some(other) = evaluated_args.first() {
                    let same = matches!(
                        (&self_val, other),
                        (Object::Instance(a), Object::Instance(b)) if Rc::ptr_eq(a, b)
                    );
                    return Ok(if same { Object::Int(0) } else { Object::Nil });
                }
                return Ok(Object::Nil);
            }
            None => {
                // Kernel's methods live in the native dispatch tables rather
                // than in any class's method map, so `super` inside a class
                // that overrides one (a `def load` calling `super`) has to
                // reach them here.
                drop(instance_borrowed);
                let evaluated_args = if forward_args {
                    self.method_arg_stack.last().cloned().unwrap_or_default()
                } else {
                    let mut evaluated_args = Vec::with_capacity(arguments.len());
                    for arg in arguments {
                        evaluated_args.push(self.evaluate_expression(arg)?);
                    }
                    evaluated_args
                };
                let self_val = self.environment().get("self").unwrap_or(Object::Nil);
                if let Some(result) =
                    self.call_object_method(&self_val, &method_name, &evaluated_args, position)?
                {
                    return Ok(result);
                }
                if matches!(
                    self.globals().get(&method_name),
                    Some(Object::NativeFunction(_))
                ) {
                    return self.call_native_function(&method_name, evaluated_args, position);
                }
                // `super` from an override of `method_missing` reaches
                // BasicObject's, whose whole job is to raise NoMethodError
                // naming the method that was called.
                if method_name == "method_missing" {
                    let missing = match evaluated_args.first() {
                        Some(Object::Symbol(name) | Object::String(name)) => (**name).clone(),
                        Some(other) => other.to_string(),
                        None => "method_missing".to_string(),
                    };
                    let message = format!(
                        "undefined method '{}' for an instance of {}",
                        missing,
                        self.builtins().class_of(&self_val).name()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: crate::vm::errors::no_method_error(
                            &message,
                            &missing,
                            &self_val,
                            evaluated_args.get(1..).unwrap_or(&[]),
                        ),
                        location: position_to_location(position),
                        message,
                    });
                }
                // Nothing above the defining class answers the call, which
                // Ruby reports as a NoMethodError naming the method.
                let message = format!(
                    "super: no superclass method '{}' for an instance of {}",
                    method_name,
                    self.builtins().class_of(&self_val).name()
                );
                return Err(MetorexError::UncaughtException {
                    exception: crate::vm::errors::no_method_error(
                        &message,
                        &method_name,
                        &self_val,
                        &evaluated_args,
                    ),
                    location: position_to_location(position),
                    message,
                });
            }
        };

        // Evaluate the arguments (or forward the enclosing method's args for
        // bare `super`).
        let evaluated_args = if forward_args {
            self.method_arg_stack.last().cloned().unwrap_or_default()
        } else {
            let mut evaluated_args = Vec::with_capacity(arguments.len());
            for arg in arguments {
                evaluated_args.push(self.evaluate_expression(arg)?);
            }
            evaluated_args
        };

        // Drop the borrow before invoking the method
        drop(instance_borrowed);

        // Invoke the parent method with self as the receiver
        self.invoke_method(
            parent_class,
            method,
            Object::Instance(Rc::clone(&instance)),
            evaluated_args,
            position,
        )
    }
}
