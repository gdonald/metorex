// `super` expression: dispatch to the same method in the parent class.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::rc::Rc;

use crate::class::Class;
use crate::vm::core::VirtualMachine;
use crate::vm::utils::position_to_location;

/// What a `super` call was written as: the arguments it named, whether it
/// forwards the enclosing method's own, and the block it carries.
struct SuperCall<'a> {
    arguments: &'a [Expression],
    forward_args: bool,
    block: Option<Object>,
}

impl VirtualMachine {
    /// Evaluate a `super` call. Walks the inheritance chain from the receiver's
    /// class to find the class defining the current method, then invokes the
    /// same-named method on that class's superclass.
    pub(super) fn eval_super(
        &mut self,
        arguments: &[Expression],
        forward_args: bool,
        trailing_block: Option<&Expression>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // A block written on the super call stands in for the one the caller
        // supplied, and it is evaluated once no matter which branch below
        // ends up reaching the parent method.
        let super_block = match trailing_block {
            Some(block) => Some(self.attach_trailing_block(block)?),
            // With no block of its own, `super` hands the parent the block
            // this method was called with.
            None => self
                .environment()
                .get("__block__")
                .filter(|held| matches!(held, Object::Block(_))),
        };
        // Get the current method name from the call stack.
        // The call stack stores method names as "Class#method".
        // A block names the method it was written in, whichever method is
        // running it, and `super` there reaches that method's parent.
        let current_frame = match self.call_stack.last() {
            Some(top) if top.block_depth() > 0 => top.written_in().map(str::to_string),
            Some(top) => Some(top.name().to_string()),
            None => None,
        }
        .ok_or_else(|| {
            MetorexError::runtime_error(
                "super called outside of a method context".to_string(),
                position_to_location(position),
            )
        })?;

        // Extract the class name and method name. An instance method is
        // named `Class#method` and one on a class object `Class.method`,
        // which is how a backtrace writes each.
        let separator = current_frame
            .rfind('#')
            .into_iter()
            .chain(current_frame.rfind('.'))
            .max();
        let (class_name, method_name) = if let Some(pos) = separator {
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
            let evaluated_args =
                self.super_arguments(arguments, forward_args, &super_block, position)?;
            // A module extended with another module reaches that module's
            // copy through `super`: the extended copy sits just above the
            // receiver's own module-level method in the singleton chain. The
            // copy's own `super` goes on past it, below.
            let running_owner = self.method_owner_stack.last().cloned().flatten();
            if let Some(Object::Method(extended)) =
                self_class.get_class_var(&format!("__ext__{}", method_name))
                && !self
                    .method_running_stack
                    .last()
                    .is_some_and(|running| Rc::ptr_eq(running, &extended))
            {
                return self.invoke_method(
                    Rc::clone(&self_class),
                    extended,
                    self_object,
                    evaluated_args,
                    position,
                );
            }
            // A method written in a module a class extended sits among the
            // class's singleton ancestors, and `super` there reaches the next
            // one of them that defines the method.
            let mut holder = Some(Rc::clone(&self_class));
            while let (Some(owner), Some(class)) = (&running_owner, holder.clone()) {
                if let Some(singleton) = class.singleton_class_slot().clone() {
                    let chain = walk_ancestors(&singleton);
                    if let Some(at) = chain.iter().position(|held| Rc::ptr_eq(held, owner)) {
                        if let Some((next, method)) = chain.iter().skip(at + 1).find_map(|held| {
                            held.find_own_method(&method_name)
                                .map(|method| (Rc::clone(held), method))
                        }) {
                            return self.invoke_method(
                                next,
                                method,
                                self_object,
                                evaluated_args,
                                position,
                            );
                        }
                        break;
                    }
                }
                holder = class.superclass();
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
            // `super` from an exception subclass's own `initialize` is what
            // gives the exception its message, the way Exception#initialize
            // does. `initialize_copy` has nothing left to reach.
            Some(held @ Object::Exception(_))
                if matches!(method_name.as_str(), "initialize" | "initialize_copy") =>
            {
                if method_name == "initialize_copy" {
                    return Ok(Object::Nil);
                }
                let evaluated_args = if forward_args {
                    self.implicit_super_arguments(position)?
                } else {
                    self.evaluate_arguments(arguments)?
                };
                return self.exception_initialize(&held, &evaluated_args, position);
            }
            // `self` is a built-in value, so the running method was written
            // on a reopened core class or on a module prepended to one.
            Some(other) => {
                return self.eval_super_on_builtin(
                    other,
                    &class_name,
                    &method_name,
                    SuperCall {
                        arguments,
                        forward_args,
                        block: super_block,
                    },
                    position,
                );
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
        let running_owner = self.method_owner_stack.last().cloned().flatten();
        // A method a refinement holds stands in front of the class it
        // refines, so `super` from it starts at that class's own method.
        let refined_at = running_owner
            .as_ref()
            .and_then(|owner| owner.get_class_var(crate::vm::native_methods::REFINEMENT_TARGET_KEY))
            .and_then(|target| match target {
                Object::Class(refined) | Object::Module(refined) => {
                    chain.iter().position(|c| Rc::ptr_eq(c, &refined))
                }
                _ => None,
            });
        let recorded_owner =
            running_owner.filter(|owner| chain.iter().any(|c| Rc::ptr_eq(c, owner)));
        // A method bound into a hierarchy its own module is not part of has
        // no place in the chain. `super` from it reaches whatever the
        // receiver's own ancestors define, so the search starts at the top.
        let placed =
            recorded_owner.or_else(|| chain.iter().find(|c| c.name() == class_name).cloned());
        let defining_class = match &placed {
            Some(found) => Rc::clone(found),
            None => Rc::clone(&chain[0]),
        };

        // Locate the method in the ancestor chain AFTER the defining class.
        // In Ruby, `super` looks up the next method in the chain — that may
        // live on the defining class's superclass, OR on a mixin that the
        // defining class brings in, OR on a class/module further up.
        let next_in_chain: Option<(Rc<Class>, Rc<crate::object::Method>)> = {
            let mut found = None;
            // How much of the chain the defining class stands above, which
            // is the whole of it when the method belongs nowhere in it.
            let start_at = match &placed {
                _ if refined_at.is_some() => refined_at,
                Some(found) => chain
                    .iter()
                    .position(|c| Rc::ptr_eq(c, found))
                    .map(|at| at + 1),
                None => Some(0),
            };
            if let Some(idx) = start_at {
                // The chain is already linearized, so each ancestor is asked
                // only for its own method. Walking its mixins and prepends
                // again would hand back the method `super` was called from.
                for anc in chain.iter().skip(idx) {
                    if let Some(method) = anc.find_own_method(&method_name) {
                        found = Some((Rc::clone(anc), method));
                        break;
                    }
                }
            }
            found
        };
        if let Some((owner, method)) = next_in_chain {
            let evaluated_args =
                self.super_arguments(arguments, forward_args, &super_block, position)?;
            drop(instance_borrowed);
            // An ancestor that undefined the method ends the search there.
            if method.is_undefined {
                let self_val = Object::Instance(Rc::clone(&instance));
                return self.super_finds_nothing(&self_val, &method_name, evaluated_args, position);
            }
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
                    let evaluated_args = self.evaluate_arguments(arguments)?;
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
                if method_name == "initialize" {
                    drop(instance_borrowed);
                    let evaluated_args = self.evaluate_arguments(arguments)?;
                    return self.super_initialize(&evaluated_args, position);
                }
                // A module has no superclass of its own, so what answers
                // next is what every object answers.
                drop(instance_borrowed);
                let evaluated_args =
                    self.super_arguments(arguments, forward_args, &super_block, position)?;
                return self.super_past_every_ancestor(&method_name, evaluated_args, position);
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
                let evaluated_args = self.evaluate_arguments(arguments)?;
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
                let evaluated_args =
                    self.super_arguments(arguments, forward_args, &super_block, position)?;
                return self.super_past_every_ancestor(&method_name, evaluated_args, position);
            }
        };

        // Evaluate the arguments (or forward the enclosing method's args for
        // bare `super`).
        let evaluated_args =
            self.super_arguments(arguments, forward_args, &super_block, position)?;

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

    /// `super` from a method whose `self` is a built-in value rather than an
    /// instance: `class Integer; def +(o); super; end; end`, or the same
    /// method written on a module prepended to Integer. Walks the receiver's
    /// ancestors past the defining class, then falls through to the native
    /// implementation the core class carries.
    /// The arguments a `super` call hands on, with the block it carries made
    /// pending. A written `&arg` names the block itself, so the one the
    /// enclosing method was called with stands in only when there is none.
    fn super_arguments(
        &mut self,
        arguments: &[Expression],
        forward_args: bool,
        super_block: &Option<Object>,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        // The block `super` hands on was given to the method making the call,
        // which is the one that answers for whether it is used.
        self.pending_block_from_ampersand = super_block.is_some();
        if forward_args {
            self.pending_block = super_block.clone();
            return self.implicit_super_arguments(position);
        }
        if arguments
            .iter()
            .any(|argument| matches!(argument, Expression::BlockArg { .. }))
        {
            self.pending_block = None;
            return self.evaluate_arguments(arguments);
        }
        let evaluated = self.evaluate_arguments(arguments)?;
        self.pending_block = super_block.clone();
        Ok(evaluated)
    }

    /// What `super` reaches once no ancestor defines the method: the
    /// methods every object answers, which live in the native tables.
    fn super_past_every_ancestor(
        &mut self,
        method_name: &str,
        evaluated_args: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let self_val = self.environment().get("self").unwrap_or(Object::Nil);
        if let Some(result) =
            self.call_object_method(&self_val, method_name, &evaluated_args, position)?
        {
            return Ok(result);
        }
        if matches!(
            self.globals().get(method_name),
            Some(Object::NativeFunction(_))
        ) {
            return self.call_native_function(method_name, evaluated_args, position);
        }
        // `super` from an override of `method_missing` reaches
        // BasicObject's, whose whole job is to raise NoMethodError
        // naming the method that was called.
        if method_name == "method_missing" {
            let missing = match evaluated_args.first() {
                Some(Object::Symbol(name)) => name.as_str().to_string(),
                Some(Object::String(name)) => name.as_str().to_string(),
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
        // `super` from an `initialize` that nothing above defines
        // reaches Object's, which takes no arguments and does
        // nothing at all.
        if method_name == "initialize" {
            return self.super_initialize(&evaluated_args, position);
        }
        // A collection the program subclassed answers the methods of
        // the collection it is backed by, and those live in the
        // native table rather than in any method map above. Only an
        // instance is backed that way, and a class reaching its own
        // native method through here would find this same method
        // again.
        let backing = self.builtins().class_of(&self_val);
        if matches!(self_val, Object::Instance(_))
            && let Some(result) = self.call_native_method(
                backing.as_ref(),
                &self_val,
                method_name,
                &evaluated_args,
                position,
            )?
        {
            return Ok(result);
        }
        self.super_finds_nothing(&self_val, method_name, evaluated_args, position)
    }

    /// Nothing answers a `super`: a `method_missing` the program defined is
    /// asked, and without one Ruby reports a NoMethodError naming the method.
    fn super_finds_nothing(
        &mut self,
        self_val: &Object,
        method_name: &str,
        evaluated_args: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if let Some((owner, missing)) = self.lookup_method(self_val, "method_missing")
            && !missing.is_undefined
        {
            let mut handed = vec![Object::symbol(method_name.to_string())];
            handed.extend(evaluated_args);
            return self.invoke_method(owner, missing, self_val.clone(), handed, position);
        }
        let message = format!(
            "super: no superclass method '{}' for an instance of {}",
            method_name,
            self.builtins().class_of(self_val).name()
        );
        Err(MetorexError::UncaughtException {
            exception: crate::vm::errors::no_method_error(
                &message,
                method_name,
                self_val,
                &evaluated_args,
            ),
            location: position_to_location(position),
            message,
        })
    }

    /// What a `super` written without arguments hands on: the running
    /// method's parameters as they stand now, so a value the body changed
    /// goes along changed and a default the call left out goes along too.
    pub(crate) fn implicit_super_arguments(
        &mut self,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let Some(method) = self.method_running_stack.last().cloned() else {
            return Ok(self.method_arg_stack.last().cloned().unwrap_or_default());
        };
        // A method built from a block has a block's parameters, which Ruby
        // does not hand on for it.
        if method.lambda_body {
            return Err(crate::vm::errors::simple_exception(
                "RuntimeError",
                "implicit argument passing of super from method defined by define_method() is not supported. Specify all arguments explicitly.",
                position,
            ));
        }
        // A parameter that takes its argument apart has no one local holding
        // it, so the arguments go along as they were given.
        if method
            .parameters
            .iter()
            .any(|name| name.starts_with(crate::object::DESTRUCTURED_GROUP_PREFIX))
        {
            return Ok(self.method_arg_stack.last().cloned().unwrap_or_default());
        }
        let mut handed = Vec::new();
        let mut tail_from_splat = false;
        for index in 0..method.parameters.len() {
            let local = crate::vm::param_binding::parameter_local(&method.parameters, index);
            let value = self.environment().get(&local);
            match (&method.variadic_param, value) {
                (Some((at, _)), value) if *at == index => match value {
                    Some(Object::Array(items)) => {
                        tail_from_splat =
                            index + 1 == method.parameters.len() && !items.borrow().is_empty();
                        handed.extend(items.borrow().iter().cloned())
                    }
                    Some(other) => handed.push(other),
                    None => {}
                },
                (_, value) => handed.push(value.unwrap_or(Object::Nil)),
            }
        }
        // The keyword rest goes first and the named keywords after it, the
        // order Ruby hands them on in.
        let mut keywords = indexmap::IndexMap::new();
        if let Some(rest) = &method.keyword_rest_parameter
            && let Some(Object::Dict(held)) = self.environment().get(rest)
        {
            for (key, value) in held.borrow().iter() {
                if key != crate::vm::param_binding::KWARGS_MARKER {
                    keywords.insert(key.clone(), value.clone());
                }
            }
        }
        for (name, _) in &method.keyword_parameters {
            let value = self.environment().get(name).unwrap_or(Object::Nil);
            keywords.insert(format!(":{name}"), value);
        }
        // A hash `ruby2_keywords` marked, last among what the rest parameter
        // gathered, goes on as the keywords it was gathered from.
        if keywords.is_empty()
            && tail_from_splat
            && let Some(Object::Dict(entries)) = handed.last()
            && entries
                .borrow()
                .contains_key(crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY)
        {
            keywords = entries
                .borrow()
                .iter()
                .filter(|(key, _)| {
                    key.as_str() != crate::vm::native_methods::hash_methods::RUBY2_KEYWORDS_KEY
                })
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect();
            handed.pop();
        }
        if !keywords.is_empty() {
            keywords.shift_insert(
                0,
                crate::vm::param_binding::KWARGS_MARKER.to_string(),
                Object::Bool(true),
            );
            handed.push(Object::Dict(Rc::new(std::cell::RefCell::new(keywords))));
        }
        Ok(handed)
    }

    fn eval_super_on_builtin(
        &mut self,
        receiver: Object,
        class_name: &str,
        method_name: &str,
        call: SuperCall<'_>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let SuperCall {
            arguments,
            forward_args,
            block: super_block,
        } = call;
        let receiver_class = self.builtins().class_of(&receiver);
        let mut chain = Vec::new();
        if let Some(singleton) = self.existing_singleton_class(&receiver) {
            chain.push(singleton);
        }
        for ancestor in walk_ancestors(&receiver_class) {
            if !chain.iter().any(|c| Rc::ptr_eq(c, &ancestor)) {
                chain.push(ancestor);
            }
        }
        let defining_class = self
            .method_owner_stack
            .last()
            .cloned()
            .flatten()
            .filter(|owner| chain.iter().any(|c| Rc::ptr_eq(c, owner)))
            .or_else(|| chain.iter().find(|c| c.name() == class_name).cloned());

        let evaluated_args =
            self.super_arguments(arguments, forward_args, &super_block, position)?;

        if let Some(defining_class) = &defining_class
            && let Some(index) = chain.iter().position(|c| Rc::ptr_eq(c, defining_class))
        {
            for ancestor in chain.iter().skip(index + 1) {
                if let Some(method) = ancestor.find_own_method(method_name)
                    && !method.body.is_empty()
                {
                    return self.invoke_method(
                        Rc::clone(ancestor),
                        method,
                        receiver.clone(),
                        evaluated_args,
                        position,
                    );
                }
            }
        }

        if let Some(result) = self.call_native_method(
            &receiver_class,
            &receiver,
            method_name,
            &evaluated_args,
            position,
        )? {
            return Ok(result);
        }

        // An operator written in syntax rather than in a method table: the
        // core arithmetic lives in the binary-operator evaluator, so `super`
        // from an override of `+` reaches it there.
        if evaluated_args.len() == 1
            && let Some(op) = crate::vm::native_methods::binary_op_for_method_name(method_name)
        {
            return self.evaluate_binary_operation(
                &op,
                receiver.clone(),
                evaluated_args[0].clone(),
                position,
            );
        }

        if method_name == "initialize" {
            return self.super_initialize(&evaluated_args, position);
        }

        let message = format!(
            "super: no superclass method '{}' for an instance of {}",
            method_name,
            receiver_class.name()
        );
        Err(MetorexError::UncaughtException {
            exception: crate::vm::errors::no_method_error(
                &message,
                method_name,
                &receiver,
                &evaluated_args,
            ),
            location: position_to_location(position),
            message,
        })
    }

    /// What `super` from a constructor with nothing above it reaches. A
    /// String subclass takes the characters it was given, and everything else
    /// reaches Object's own, which takes no arguments at all.
    fn super_initialize(
        &mut self,
        arguments: &[Object],
        position: crate::lexer::Position,
    ) -> Result<Object, MetorexError> {
        let receiver = self.environment().get("self").unwrap_or(Object::Nil);
        if let Object::Instance(instance) = &receiver
            && instance
                .borrow()
                .instance_vars
                .contains_key(crate::vm::native_methods::STRING_SUBCLASS_VAR)
        {
            let spelled = self.string_from_new_arguments(arguments, position)?;
            instance.borrow_mut().set_var(
                crate::vm::native_methods::STRING_SUBCLASS_VAR.to_string(),
                spelled,
            );
            return Ok(Object::Nil);
        }
        if matches!(receiver, Object::Exception(_)) {
            return self.exception_initialize(&receiver, arguments, position);
        }
        Self::object_initialize(arguments, position)
    }

    /// Exception#initialize, which a subclass's own `initialize` reaches
    /// through `super`. The argument it is handed is the message.
    fn exception_initialize(
        &mut self,
        receiver: &Object,
        arguments: &[Object],
        position: crate::lexer::Position,
    ) -> Result<Object, MetorexError> {
        let Object::Exception(details) = receiver else {
            return Ok(Object::Nil);
        };
        match arguments.first() {
            None => {}
            Some(Object::Nil) => {
                let mut held = details.borrow_mut();
                held.message = String::new();
                held.message_given = false;
            }
            Some(given) => {
                let spelled = self.coerce_name_argument(given, position)?;
                let mut held = details.borrow_mut();
                held.message = spelled;
                held.message_given = true;
                if matches!(given, Object::String(_)) {
                    held.instance_vars
                        .insert(crate::vm::MESSAGE_STRING_KEY.to_string(), given.clone());
                }
            }
        }
        Ok(Object::Nil)
    }

    /// Object#initialize, which every `super` from a constructor with nothing
    /// above it reaches. It takes no arguments and answers nil.
    fn object_initialize(
        arguments: &[Object],
        position: crate::lexer::Position,
    ) -> Result<Object, MetorexError> {
        if arguments.is_empty() {
            return Ok(Object::Nil);
        }
        let message = format!(
            "wrong number of arguments (given {}, expected 0)",
            arguments.len()
        );
        Err(MetorexError::UncaughtException {
            exception: Object::exception("ArgumentError", message.clone()),
            location: position_to_location(position),
            message,
        })
    }
}

/// The ancestors of `class` in lookup order: its prepended modules, the class
/// itself, its included modules, then the same for each superclass.
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

impl VirtualMachine {
    /// Whether `super` written in the running method would reach a method,
    /// which is what `defined?(super)` reports. A method an ancestor undefined
    /// is not reached.
    pub(crate) fn super_method_defined(&self) -> bool {
        // A block names the method it was written in, whichever method is
        // running it.
        let (frame, in_a_block) = match self.call_stack.last() {
            Some(top) if top.block_depth() > 0 => match top.written_in() {
                Some(written) => (written.to_string(), true),
                None => return false,
            },
            Some(top) => (top.name().to_string(), false),
            None => return false,
        };
        let Some(separator) = frame.rfind('#').into_iter().chain(frame.rfind('.')).max() else {
            return false;
        };
        let (class_name, method_name) = (&frame[..separator], &frame[separator + 1..]);
        match self.environment().get("self") {
            Some(Object::Class(self_class) | Object::Module(self_class)) => {
                let class_method_key = format!("__class__{}", method_name);
                let mut current = self_class.superclass();
                while let Some(class) = current {
                    let candidate = class
                        .singleton_class_slot()
                        .clone()
                        .and_then(|singleton| singleton.find_method(method_name))
                        .or_else(|| class.find_method(&class_method_key));
                    if let Some(method) = candidate {
                        return !method.is_undefined;
                    }
                    current = class.superclass();
                }
                false
            }
            Some(Object::Instance(instance)) => {
                let borrowed = instance.borrow();
                let mut chain = Vec::new();
                if let Some(singleton) = borrowed.singleton_class.borrow().clone() {
                    chain.push(singleton);
                }
                for ancestor in walk_ancestors(&borrowed.class) {
                    if !chain.iter().any(|held| Rc::ptr_eq(held, &ancestor)) {
                        chain.push(ancestor);
                    }
                }
                let running_owner = self
                    .method_owner_stack
                    .last()
                    .cloned()
                    .flatten()
                    .filter(|_| !in_a_block)
                    .filter(|owner| chain.iter().any(|held| Rc::ptr_eq(held, owner)));
                let placed = running_owner
                    .or_else(|| chain.iter().find(|held| held.name() == class_name).cloned());
                let start = match &placed {
                    Some(found) => chain
                        .iter()
                        .position(|held| Rc::ptr_eq(held, found))
                        .map_or(0, |at| at + 1),
                    None => 0,
                };
                chain
                    .iter()
                    .skip(start)
                    .find_map(|ancestor| ancestor.find_own_method(method_name))
                    .is_some_and(|method| !method.is_undefined)
            }
            _ => false,
        }
    }
}
