//! Method lookup and dispatch for the virtual machine.
//!
//! This module handles resolving method calls on receiver objects and dispatching
//! to the appropriate method implementation.

use super::VirtualMachine;
use super::errors::*;
use crate::ast::Expression;
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Method, Object};
use std::cell::RefCell;
use std::rc::Rc;

/// Build the refinement lookup key for a receiver. For user-defined class
/// instances we key by pointer (to avoid anonymous-class name collisions);
/// for builtin types we use the well-known class name alone.
pub(crate) fn refinement_target_name(
    receiver: &Object,
    vm: &crate::vm::VirtualMachine,
) -> Option<String> {
    match receiver {
        Object::String(_) => Some(builtin_key(vm, "String")),
        Object::Int(_) => Some(builtin_key(vm, "Integer")),
        Object::Float(_) => Some(builtin_key(vm, "Float")),
        Object::Array(_) => Some(builtin_key(vm, "Array")),
        Object::Dict(_) => Some(builtin_key(vm, "Hash")),
        Object::Symbol(_) => Some(builtin_key(vm, "Symbol")),
        Object::Instance(inst) => {
            let cls = &inst.borrow().class;
            Some(format!("__refine__{}@{:p}", cls.name(), Rc::as_ptr(cls)))
        }
        _ => None,
    }
}

fn builtin_key(vm: &crate::vm::VirtualMachine, name: &str) -> String {
    if let Some(Object::Class(c)) = vm.globals().get(name) {
        format!("__refine__{}@{:p}", name, Rc::as_ptr(&c))
    } else {
        format!("__refine__{}", name)
    }
}

impl VirtualMachine {
    /// Evaluate a method call expression on a receiver object.
    pub(crate) fn evaluate_method_call(
        &mut self,
        receiver_expr: &Expression,
        method_name: &str,
        argument_exprs: &[Expression],
        trailing_block: Option<&Expression>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let has_block = trailing_block.is_some();
        let calling_frame = self.current_method_frame;
        let result = self.evaluate_method_call_inner(
            receiver_expr,
            method_name,
            argument_exprs,
            trailing_block,
            position,
        );
        // Ruby: `break <value>` inside the block passed to this call unwinds
        // to *this* method call and makes the call return `value`. Only catch
        // when a block was attached here and the break came from a block
        // written in this frame, so a nested call does not absorb a break
        // meant for an outer one.
        match result {
            Err(MetorexError::BlockBreak {
                value, home_frame, ..
            }) if has_block && (home_frame.is_none() || home_frame == calling_frame) => Ok(value),
            other => other,
        }
    }

    fn evaluate_method_call_inner(
        &mut self,
        receiver_expr: &Expression,
        method_name: &str,
        argument_exprs: &[Expression],
        trailing_block: Option<&Expression>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // `native_fn[args]` — a call with the bracketed args wrapped as an
        // Array, matching Ruby's `private [:foo, :bar]` where `[...]` is the
        // sole argument. Natives that auto-invoke when their name is evaluated
        // (`private` inside a class body) make this check precede evaluating
        // the receiver expression.
        if method_name == "[]"
            && let Expression::Identifier { name, .. } = receiver_expr
            && let Some(Object::NativeFunction(native_name)) = self.environment().get(name)
        {
            let arguments = self.evaluate_arguments(argument_exprs)?;
            let array_arg = Object::Array(Rc::new(RefCell::new(arguments)));
            return self.call_native_function(&native_name, vec![array_arg], position);
        }

        let receiver = self.evaluate_expression(receiver_expr)?;
        let arguments = self.evaluate_arguments(argument_exprs)?;

        // If there's a trailing block, evaluate it and store as pending_block.
        // Native methods (each, map, etc.) will take it from self.pending_block.
        if let Some(block_expr) = trailing_block {
            self.pending_block = Some(self.evaluate_expression(block_expr)?);
            self.pending_block_from_ampersand = false;
        }

        // Refinement dispatch: if an active refinement covers this receiver's
        // class and defines this method, use it.
        if let Some(target_key) = refinement_target_name(&receiver, self)
            && let Some(method) = self.find_refined_method(&target_key, method_name)
        {
            let class = self.builtins().class_of(&receiver);
            return self.invoke_method(class, method, receiver, arguments, position);
        }

        // For Class/Module receivers, a module-level copy of a method wins
        // over the instance method of the same name: `module_function :foo`
        // and `extend` both leave the instance method private, and `Mod.foo`
        // must reach the copy rather than trip visibility enforcement.
        if let Object::Class(class_rc) | Object::Module(class_rc) = &receiver {
            let class_rc = Rc::clone(class_rc);
            if let Some(method) = module_level_method(&class_rc, method_name) {
                let is_explicit_receiver = !names_self(receiver_expr);
                if is_explicit_receiver && self.method_is_restricted(&receiver, method_name) {
                    if let Some(handled) = self.restricted_call_via_method_missing(
                        &receiver,
                        method_name,
                        &arguments,
                        position,
                    ) {
                        return handled;
                    }
                    let msg = format!(
                        "private method '{}' called for {}",
                        method_name,
                        class_rc.ruby_name()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: crate::vm::errors::no_method_error(
                            &msg,
                            method_name,
                            &receiver,
                            &arguments,
                        ),
                        location: crate::vm::utils::position_to_location(position),
                        message: msg,
                    });
                }
                return self.invoke_method(class_rc, method, receiver.clone(), arguments, position);
            }
        }

        // Enumerable is written over `each`, so its methods stand in only
        // where the receiver's class has none of its own. A native method
        // counts as the class's own, which is what puts Struct#to_h ahead of
        // Enumerable#to_h the way Ruby's ancestor order does.
        if self.enumerable_stands_in(&receiver, method_name) {
            let class = self.builtins().class_of(&receiver);
            if let Some(result) =
                self.call_native_method(&class, &receiver, method_name, &arguments, position)?
            {
                return Ok(result);
            }
        }

        // Try user-defined method lookup first
        match self.lookup_method(&receiver, method_name) {
            Some((class, method)) if !method.is_undefined => {
                // Visibility check: an *explicit-receiver* call (e.g.
                // `obj.foo`, anything other than `self.foo` or a bare ident)
                // can only invoke public methods. Private/protected methods
                // raise NoMethodError when called externally. We treat
                // `Expression::SelfExpr` as implicit self even though it's
                // syntactically present, matching Ruby — `self.foo` is
                // allowed regardless of visibility.
                let is_explicit_receiver = !names_self(receiver_expr);
                let mut is_private = self.method_is_restricted(&receiver, method_name);
                // A prepended module sits ahead of the class, so when one of
                // them supplies the method its visibility is the one in force
                // and the class's own marking does not apply.
                let prepended_owner = class
                    .transitive_prepends()
                    .into_iter()
                    .find(|prepended| prepended.find_own_method(method_name).is_some());
                if class.has_public_override(method_name) {
                    is_private = false;
                } else if let Some(owner) = prepended_owner {
                    is_private = self.refuses_explicit_receiver(&owner, method_name);
                } else if self.refuses_explicit_receiver(&class, method_name) {
                    is_private = true;
                } else if !is_private {
                    let mut current = class.superclass();
                    while let Some(sc) = current {
                        if sc.has_public_override(method_name) {
                            is_private = false;
                            break;
                        }
                        if sc.find_method(method_name).is_some() {
                            is_private = self.refuses_explicit_receiver(&sc, method_name);
                            break;
                        }
                        current = sc.superclass();
                    }
                }
                if is_explicit_receiver && is_private {
                    if let Some(handled) = self.restricted_call_via_method_missing(
                        &receiver,
                        method_name,
                        &arguments,
                        position,
                    ) {
                        return handled;
                    }
                    let msg = format!(
                        "{} method '{}' called for an instance of {}",
                        self.visibility_word(&receiver, method_name),
                        method_name,
                        class.name()
                    );
                    let exc = crate::vm::errors::no_method_error(
                        &msg,
                        method_name,
                        &receiver,
                        &arguments,
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: exc,
                        location: crate::vm::utils::position_to_location(position),
                        message: msg,
                    });
                }
                return self.invoke_method(class, method, receiver, arguments, position);
            }
            _ => {}
        }

        // A natively-implemented class method carries visibility too, so
        // `private_class_method :new` refuses an explicit-receiver call the
        // way a table-backed one does.
        if let Object::Class(class_rc) | Object::Module(class_rc) = &receiver
            && !matches!(receiver_expr, Expression::SelfExpr { .. })
            && self.method_is_restricted(&receiver, method_name)
        {
            let class_rc = Rc::clone(class_rc);
            if let Some(handled) = self.restricted_call_via_method_missing(
                &receiver,
                method_name,
                &arguments,
                position,
            ) {
                return handled;
            }
            let msg = format!(
                "private method '{}' called for {}",
                method_name,
                class_rc.ruby_name()
            );
            return Err(MetorexError::UncaughtException {
                exception: crate::vm::errors::no_method_error(
                    &msg,
                    method_name,
                    &receiver,
                    &arguments,
                ),
                location: crate::vm::utils::position_to_location(position),
                message: msg,
            });
        }

        // Try native method as fallback
        let class = self.builtins().class_of(&receiver);
        let native_result =
            self.call_native_method(&class, &receiver, method_name, &arguments, position)?;

        if let Some(result) = native_result {
            return Ok(result);
        }

        // For user-defined class instances, fall back to base Object methods
        let object_result =
            self.call_object_method(&receiver, method_name, &arguments, position)?;

        if let Some(result) = object_result {
            return Ok(result);
        }

        // Fallback: methods defined on the global Object class (mspec injects
        // describe/it/before/after there).
        if let Some(Object::Class(object_class)) = self.globals().get("Object")
            && let Some(method) = object_class.find_method(method_name)
        {
            return self.invoke_method(object_class, method, receiver, arguments, position);
        }

        // Kernel carries its functions as module functions as well as
        // private instance methods, so `Kernel.rand` reaches the same
        // implementation a bare `rand` does.
        if let Object::Module(module_rc) = &receiver
            && module_rc.ruby_name() == "Kernel"
            && crate::vm::native_methods::is_kernel_private_function(method_name)
        {
            return self.call_native_function(method_name, arguments, position);
        }

        // Try method_missing as a final fallback
        if let Some((method_missing_class, method_missing_method)) =
            self.lookup_method(&receiver, "method_missing")
        {
            // Ruby hands `method_missing` the name as a Symbol followed by
            // the call's own arguments, one by one, so a handler written as
            // `def method_missing(name, path)` reads the path as itself.
            let mut method_missing_args = Vec::with_capacity(arguments.len() + 1);
            method_missing_args.push(Object::symbol(method_name.to_string()));
            method_missing_args.extend(arguments.iter().cloned());
            self.invoke_method(
                method_missing_class,
                method_missing_method,
                receiver,
                method_missing_args,
                position,
            )
        } else {
            Err(undefined_method_error(
                method_name,
                &receiver,
                &arguments,
                position,
            ))
        }
    }

    /// A call that visibility refuses still reaches a user-defined
    /// `method_missing`, which is where Ruby's default implementation turns
    /// it into the NoMethodError. Answers None when nothing defines one, so
    /// the caller raises the error itself.
    fn restricted_call_via_method_missing(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Option<Result<Object, MetorexError>> {
        let (owner, handler) = self.lookup_method(receiver, "method_missing")?;
        if handler.is_undefined {
            return None;
        }
        let mut handler_arguments = vec![Object::symbol(method_name.to_string())];
        handler_arguments.extend(arguments.iter().cloned());
        Some(self.invoke_method(
            owner,
            handler,
            receiver.clone(),
            handler_arguments,
            position,
        ))
    }

    /// Look up a method on the receiver and return its class and method definition.
    /// Whether the current `self` is a class or module object, i.e. execution
    /// is inside a class or module body rather than at the top level.
    pub(crate) fn self_is_class_or_module(&self) -> bool {
        matches!(
            self.environment().get("self"),
            Some(Object::Class(_) | Object::Module(_))
        ) || self.def_scope_stack.last().is_some()
    }

    /// The class or module a receiverless declaration applies to: the current
    /// `self` when it is one, otherwise the innermost lexical class or module
    /// body, which is where an `eval`'d declaration lands.
    pub(crate) fn current_definee(&self) -> Option<Rc<Class>> {
        if let Some(definee) = self.def_scope_stack.last() {
            return Some(Rc::clone(definee));
        }
        match self.environment().get("self") {
            Some(Object::Class(class) | Object::Module(class)) => Some(class),
            _ => None,
        }
    }

    /// Whether `receiver` answers to `name`. A class or module object answers
    /// to its module-level and singleton methods, never to its own instance
    /// methods, which belong to the objects it describes rather than to it.
    pub(crate) fn responds_to(&self, receiver: &Object, name: &str) -> bool {
        let (Object::Class(class_rc) | Object::Module(class_rc)) = receiver else {
            // A tombstone left by `undef_method` is not something the object
            // responds to.
            // Kernel methods live in the native dispatch tables rather than
            // in any class's method map, so they are checked separately.
            return match self.lookup_method(receiver, name) {
                Some((_, method)) => !method.is_undefined,
                None => {
                    crate::vm::native_methods::is_native_kernel_method(name)
                        || (self.builtins().class_of(receiver).name() == "File"
                            && crate::vm::native_methods::class_methods::is_native_io_method(name))
                }
            };
        };
        if let Some(method) = module_level_method(class_rc, name) {
            return !method.is_undefined;
        }
        let mut cursor = Some(Rc::clone(class_rc));
        while let Some(current) = cursor {
            if let Some(sc) = current.singleton_class_slot().clone()
                && sc.find_method(name).is_some()
            {
                return true;
            }
            cursor = current.superclass();
        }
        if self
            .builtins()
            .class_of(receiver)
            .find_method(name)
            .is_some()
        {
            return true;
        }
        // A class answers to `new` and to Module's own natives, which live in
        // the dispatch tables rather than in a method map. It also carries
        // Kernel's default `respond_to_missing?` the way any object does. The
        // rest of Kernel's surface is not reported here yet: turning it all on
        // changes which names mspec's mocks alias and breaks Module#autoload.
        if name == "respond_to_missing?" || (name == "new" && matches!(receiver, Object::Class(_)))
        {
            return true;
        }
        // Reopening `Class` or `Module` adds a method every class and module
        // answers to, the same chain `lookup_method` walks for the call.
        for global_name in ["Class", "Module"] {
            if let Some(Object::Class(global_class)) = self.globals().get(global_name)
                && let Some(method) = global_class.find_method(name)
            {
                return !method.is_undefined;
            }
        }
        crate::vm::native_methods::native_module_method_stub(name).is_some()
    }

    /// Whether `name` resolves to a private or protected method on
    /// `receiver`, so an explicit-receiver call would be refused.
    /// Whether `name` already carries `visibility` through `class_rc`'s
    /// ancestors, so declaring it again records nothing.
    pub(crate) fn inherits_visibility(
        &self,
        class_rc: &Rc<Class>,
        name: &str,
        visibility: &str,
    ) -> bool {
        match visibility {
            "private" => self.inherits_private(class_rc, name),
            "protected" => self.inherits_protected(class_rc, name),
            _ => {
                class_rc.find_method(name).is_some()
                    && self.inherited_visibility(class_rc, name).is_none()
            }
        }
    }

    /// Whether `name` is already private through `class_rc`'s ancestors.
    pub(crate) fn inherits_private(&self, class_rc: &Rc<Class>, name: &str) -> bool {
        matches!(self.inherited_visibility(class_rc, name), Some(true))
    }

    /// Whether `name` is already protected through `class_rc`'s ancestors.
    pub(crate) fn inherits_protected(&self, class_rc: &Rc<Class>, name: &str) -> bool {
        matches!(self.inherited_visibility(class_rc, name), Some(false))
    }

    /// The visibility `name` carries from the nearest ancestor that marks or
    /// defines it: `Some(true)` for private, `Some(false)` for protected,
    /// `None` for public or undefined.
    fn inherited_visibility(&self, class_rc: &Rc<Class>, name: &str) -> Option<bool> {
        for ancestor in class_rc
            .mixin_chain()
            .into_iter()
            .chain(std::iter::successors(class_rc.superclass(), |current| {
                current.superclass()
            }))
        {
            if ancestor.has_public_override(name) {
                return None;
            }
            if ancestor.is_method_private(name) {
                return Some(true);
            }
            if ancestor.is_method_protected(name) {
                return Some(false);
            }
            if ancestor.find_own_method(name).is_some() {
                return None;
            }
        }
        None
    }

    pub(crate) fn method_is_restricted(&self, receiver: &Object, name: &str) -> bool {
        let owner = self.visibility_owner(receiver, name);
        // A singleton class is where class-method visibility is recorded, so
        // its answer settles the question.
        if let Some(owner) = &owner
            && owner.is_singleton_class()
        {
            return !owner.has_public_override(name) && owner.is_method_restricted(name);
        }
        // Otherwise a module-level copy is public even when the instance
        // method it was copied from is private, as `module_function` leaves it.
        if let Object::Class(class_rc) | Object::Module(class_rc) = receiver
            && module_level_method(class_rc, name).is_some()
        {
            return false;
        }
        match owner {
            Some(owner) => self.refuses_explicit_receiver(&owner, name),
            None => false,
        }
    }

    /// How a refused call names the marking that refused it, which Ruby
    /// spells out as either private or protected.
    fn visibility_word(&self, receiver: &Object, name: &str) -> &'static str {
        match self.visibility_owner(receiver, name) {
            Some(owner) if owner.is_method_protected(name) && !owner.is_method_private(name) => {
                "protected"
            }
            _ => "private",
        }
    }

    /// Whether `owner`'s marking on `name` refuses an explicit-receiver call
    /// from where this call is being made. A private method always refuses;
    /// a protected one refuses only from outside the class.
    pub(crate) fn refuses_explicit_receiver(&self, owner: &Rc<Class>, name: &str) -> bool {
        if owner.has_public_override(name) {
            return false;
        }
        if owner.is_method_private(name) {
            return true;
        }
        owner.is_method_protected(name) && !self.calling_from_within(owner)
    }

    /// Whether the code making this call is running as an object the given
    /// class or module answers for, which is what a protected method asks.
    fn calling_from_within(&self, owner: &Rc<Class>) -> bool {
        let Some(caller) = self.environment().get("self") else {
            return false;
        };
        let mut cursor = match &caller {
            Object::Class(class_rc) | Object::Module(class_rc) => Some(Rc::clone(class_rc)),
            Object::Instance(instance) => Some(Rc::clone(&instance.borrow().class)),
            other => Some(self.builtins().class_of(other)),
        };
        while let Some(current) = cursor {
            if Rc::ptr_eq(&current, owner) || current.name() == owner.name() {
                return true;
            }
            if current
                .transitive_mixins()
                .iter()
                .any(|mixin| Rc::ptr_eq(mixin, owner))
            {
                return true;
            }
            cursor = current.superclass();
        }
        false
    }

    /// The class or module that actually defines `name` for `receiver`, which
    /// is where its visibility is recorded. `lookup_method` reports the class
    /// it dispatched through, and for a mixed-in method that is the including
    /// class rather than the module carrying the private marking.
    fn visibility_owner(&self, receiver: &Object, name: &str) -> Option<Rc<Class>> {
        // A class method's visibility is recorded on the singleton class,
        // even though the method itself may live in the class's own table
        // under the `__class__` convention.
        if let Object::Class(class_rc) | Object::Module(class_rc) = receiver {
            let mut defining_singleton = None;
            let mut cursor = Some(Rc::clone(class_rc));
            while let Some(current) = cursor {
                if let Some(sc) = current.singleton_class_slot().clone() {
                    // The nearest singleton with its own marking for this
                    // name settles it; a subclass's `private_class_method`
                    // marks the name without redefining the method. A
                    // natively-implemented name such as `new` is marked the
                    // same way with no method map entry to find.
                    if sc.has_public_override(name) || sc.is_method_restricted(name) {
                        return Some(sc);
                    }
                    if let Some((owner, _)) = sc.find_method_with_owner(name) {
                        defining_singleton.get_or_insert(owner);
                    }
                }
                cursor = current.superclass();
            }
            if defining_singleton.is_some() {
                return defining_singleton;
            }
        }
        let Object::Instance(instance_rc) = receiver else {
            return self.lookup_method(receiver, name).map(|(class, _)| class);
        };
        let instance_ref = instance_rc.borrow();
        let singleton = instance_ref.singleton_class.borrow().clone();
        let class = Rc::clone(&instance_ref.class);
        drop(instance_ref);
        singleton
            .and_then(|sc| sc.find_method_with_owner(name))
            .or_else(|| class.find_method_with_owner(name))
            .map(|(owner, _)| owner)
    }

    /// Whether the method the receiver would answer comes from the Enumerable
    /// module rather than from its own class.
    pub(crate) fn enumerable_stands_in(&self, receiver: &Object, method_name: &str) -> bool {
        let class = match receiver {
            Object::Instance(instance_rc) => Rc::clone(&instance_rc.borrow().class),
            Object::Class(_) | Object::Module(_) => return false,
            other => self.builtins().class_of(other),
        };
        class
            .find_method_with_owner(method_name)
            .is_some_and(|(owner, _)| owner.ruby_name() == "Enumerable")
    }

    /// An operator method written on a built-in value's own class, on one of
    /// the modules prepended to it, or on the value's singleton class. Used
    /// when `1 + 2` has to reach a reopened `Integer#+` rather than the native
    /// arithmetic. An ancestor's copy does not count: the core library writes
    /// several operators in Ruby on `Numeric` and on `Comparable`, and those
    /// stand for the native implementation rather than replacing it. A
    /// body-less stub does not count either, for the same reason.
    pub(crate) fn lookup_own_operator_method(
        &self,
        receiver: &Object,
        method_name: &str,
    ) -> Option<(Rc<Class>, Rc<Method>)> {
        let mut candidates = Vec::new();
        if let Some(singleton) = self.existing_singleton_class(receiver) {
            candidates.push(singleton);
        }
        let class = self.builtins().class_of(receiver);
        candidates.extend(class.prepend_chain());
        candidates.push(class);
        for owner in candidates {
            if let Some(method) = owner.find_own_method(method_name)
                && !method.is_undefined
                && !method.body.is_empty()
            {
                return Some((owner, method));
            }
        }
        None
    }

    pub(crate) fn lookup_method(
        &self,
        receiver: &Object,
        method_name: &str,
    ) -> Option<(Rc<Class>, Rc<Method>)> {
        match receiver {
            Object::Instance(instance_rc) => {
                let instance_ref = instance_rc.borrow();
                let class = Rc::clone(&instance_ref.class);
                if let Some(sing) = instance_ref.find_singleton_method(method_name) {
                    return Some((class, sing));
                }
                if let Some(sc) = instance_ref.singleton_class.borrow().clone()
                    && let Some(method) = sc.find_method(method_name)
                {
                    return Some((sc, method));
                }
                drop(instance_ref);
                class.find_method(method_name).map(|method| (class, method))
            }
            Object::Class(class_rc) => {
                // A module-level method (`def self.name`, or one copied in by
                // `extend`) is checked before walking the singleton class's
                // superclass chain, so a method from the singleton's ancestors
                // (e.g. Object#describe, when Object has been reopened) does
                // not shadow the class's own class-level method.
                if let Some(method) = module_level_method(class_rc, method_name) {
                    return Some((Rc::clone(class_rc), method));
                }
                // Walk the receiver's superclass chain, checking each
                // ancestor's singleton class. Mirrors Ruby's metaclass
                // chain so `class << Parent; attr_accessor :x; end` is
                // visible on `Child.x` when `Child < Parent`.
                let mut cursor = Some(Rc::clone(class_rc));
                while let Some(current) = cursor {
                    if let Some(sc) = current.singleton_class_slot().clone()
                        && let Some(method) = sc.find_method(method_name)
                    {
                        return Some((sc, method));
                    }
                    cursor = current.superclass();
                }
                // A class renders as its own name, so an `inspect` or `to_s`
                // written for its instances does not answer for the class
                // object itself. The same holds for every name Kernel gives
                // an object: `UNIXSocket.send(:open, path)` is Object#send,
                // not the `send` its instances answer.
                if !matches!(method_name, "inspect" | "to_s")
                    && !crate::vm::native_methods::is_native_kernel_method(method_name)
                    && let Some(method) = class_rc.find_method(method_name)
                {
                    return Some((Rc::clone(class_rc), method));
                }
                // Every class is an instance of `Class`, which descends from
                // `Module`, so reopening either (`class Module; def
                // const_added(name); ...`) adds a method every class answers.
                for global_name in ["Class", "Module"] {
                    if let Some(Object::Class(global_class)) = self.globals().get(global_name)
                        && let Some(method) = global_class.find_method(method_name)
                    {
                        return Some((global_class, method));
                    }
                }
                None
            }
            Object::Module(module_rc) => {
                if let Some(method) = module_level_method(module_rc, method_name) {
                    return Some((Rc::clone(module_rc), method));
                }
                let mut cursor = Some(Rc::clone(module_rc));
                while let Some(current) = cursor {
                    if let Some(sc) = current.singleton_class_slot().clone()
                        && let Some(method) = sc.find_method(method_name)
                    {
                        return Some((sc, method));
                    }
                    cursor = current.superclass();
                }
                if let Some(method) = module_rc.find_method(method_name) {
                    return Some((Rc::clone(module_rc), method));
                }
                // User-defined modules are instances of the global `Module`
                // class. Methods mixed into Module via `class Module; include
                // X; end` (mspec does this for matchers like `be_nil`) live
                // on that global class, not on the user module itself, so
                // fall back to `Module`'s method table when the receiver's
                // own chain comes up empty.
                if let Some(Object::Class(global_module)) = self.globals().get("Module")
                    && let Some(method) = global_module.find_method(method_name)
                {
                    return Some((global_module, method));
                }
                None
            }
            _ => {
                // A value kind with its own singleton class, such as an
                // exception given a `def obj.name`, answers from there first.
                if let Some(singleton) = self.existing_singleton_class(receiver)
                    && let Some(method) = singleton.find_method(method_name)
                {
                    return Some((singleton, method));
                }
                // A program that reopens `NilClass` or one of the other
                // immediate classes makes a class of its own in globals,
                // which is where the method it wrote lives.
                if let Some(named) = reopened_class_name(receiver)
                    && let Some(Object::Class(reopened)) = self.globals().get(named)
                    && let Some(method) = reopened.find_method(method_name)
                    && !method.body.is_empty()
                {
                    return Some((reopened, method));
                }
                // An exception built from a user-defined subclass looks the
                // method up on that class, which `class_of` cannot report.
                if let Object::Exception(details) = receiver
                    && let Some(class) = details.borrow().class.clone()
                    && let Some(method) = class.find_method(method_name)
                    // A body-less stub stands in for a native method, so it
                    // must not shadow the implementation.
                    && !method.body.is_empty()
                {
                    return Some((class, method));
                }
                let class = self.builtins().class_of(receiver);
                class.find_method(method_name).map(|method| (class, method))
            }
        }
    }
}

/// The class name a program writes to reopen one of the immediate values,
/// whose methods live in a class of their own rather than on the builtin.
fn reopened_class_name(receiver: &Object) -> Option<&'static str> {
    match receiver {
        Object::Nil => Some("NilClass"),
        Object::Bool(true) => Some("TrueClass"),
        Object::Bool(false) => Some("FalseClass"),
        Object::Symbol(_) => Some("Symbol"),
        _ => None,
    }
}

/// A module-level method on a class or module object: one stored under the
/// `__class__` convention by `def self.name` or `module_function`, or copied
/// in by `extend` under the `__ext__` convention.
pub(crate) fn module_level_method(class_rc: &Rc<Class>, method_name: &str) -> Option<Rc<Method>> {
    module_own_method(class_rc, method_name)
        .or_else(|| module_extended_method(class_rc, method_name))
}

/// A method written on the module itself with `def self.name` or `def Mod.name`.
pub(crate) fn module_own_method(class_rc: &Rc<Class>, method_name: &str) -> Option<Rc<Method>> {
    class_rc.find_method(&format!("__class__{}", method_name))
}

/// A method the module answers to because it extended a module, its own
/// instance methods included when it extended itself.
pub(crate) fn module_extended_method(
    class_rc: &Rc<Class>,
    method_name: &str,
) -> Option<Rc<Method>> {
    match class_rc.get_class_var(&format!("__ext__{}", method_name)) {
        Some(Object::Method(method)) => Some(method),
        _ => None,
    }
}

/// Whether a receiver expression names `self`, which Ruby treats as an
/// implicit receiver: `self.name` reaches a private method the way a bare
/// `name` does. The lexer reads `self` as an identifier, so the keyword form
/// and the parser's own `SelfExpr` both have to be recognized here.
pub(crate) fn names_self(receiver_expr: &Expression) -> bool {
    match receiver_expr {
        Expression::SelfExpr { .. } => true,
        Expression::Identifier { name, .. } => name == "self",
        _ => false,
    }
}
