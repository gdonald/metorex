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
        let result = self.evaluate_method_call_inner(
            receiver_expr,
            method_name,
            argument_exprs,
            trailing_block,
            position,
        );
        // Ruby: `break <value>` inside the block passed to this call unwinds
        // to *this* method call and makes the call return `value`. Only catch
        // when a block was attached here, so nested invocations don't absorb
        // breaks meant for an outer call.
        match result {
            Err(MetorexError::BlockBreak { value, .. }) if has_block => Ok(value),
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
        let receiver = self.evaluate_expression(receiver_expr)?;
        let arguments = self.evaluate_arguments(argument_exprs)?;

        // `native_fn[args]` — treat as a call with the bracketed args wrapped as an Array.
        // This matches Ruby's `private [:foo, :bar]` where `[...]` is the sole argument.
        if method_name == "[]"
            && let Object::NativeFunction(name) = &receiver
        {
            let array_arg = Object::Array(Rc::new(RefCell::new(arguments)));
            return self.call_native_function(&name.clone(), vec![array_arg], position);
        }

        // If there's a trailing block, evaluate it and store as pending_block.
        // Native methods (each, map, etc.) will take it from self.pending_block.
        if let Some(block_expr) = trailing_block {
            self.pending_block = Some(self.evaluate_expression(block_expr)?);
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
                let is_explicit_receiver = !matches!(receiver_expr, Expression::SelfExpr { .. });
                if is_explicit_receiver && self.method_is_restricted(&receiver, method_name) {
                    let msg = format!(
                        "private method '{}' called for {}",
                        method_name,
                        class_rc.ruby_name()
                    );
                    return Err(MetorexError::UncaughtException {
                        exception: Object::exception("NoMethodError", msg.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message: msg,
                    });
                }
                return self.invoke_method(class_rc, method, receiver.clone(), arguments, position);
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
                let is_explicit_receiver = !matches!(receiver_expr, Expression::SelfExpr { .. });
                let mut is_private = self.method_is_restricted(&receiver, method_name);
                if class.has_public_override(method_name) {
                    is_private = false;
                } else if class.is_method_restricted(method_name) {
                    is_private = true;
                } else if !is_private {
                    let mut current = class.superclass();
                    while let Some(sc) = current {
                        if sc.has_public_override(method_name) {
                            is_private = false;
                            break;
                        }
                        if sc.find_method(method_name).is_some() {
                            is_private = sc.is_method_restricted(method_name);
                            break;
                        }
                        current = sc.superclass();
                    }
                }
                if is_explicit_receiver && is_private {
                    let msg = format!(
                        "private method '{}' called for an instance of {}",
                        method_name,
                        class.name()
                    );
                    let exc = Object::exception("NoMethodError", msg.clone());
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

        // Try method_missing as a final fallback
        if let Some((method_missing_class, method_missing_method)) =
            self.lookup_method(&receiver, "method_missing")
        {
            let method_name_obj = Object::String(Rc::new(method_name.to_string()));
            let arity = method_missing_method.parameters.len();
            let has_variadic = method_missing_method.variadic_param.is_some();
            let method_missing_args = if arity <= 1 {
                vec![method_name_obj]
            } else if has_variadic {
                // `def method_missing(name, *args, &block)` — spread original
                // args so the splat collects them as individual positionals.
                let mut v = Vec::with_capacity(arguments.len() + 1);
                v.push(method_name_obj);
                v.extend(arguments);
                v
            } else {
                let args_array = Object::Array(Rc::new(RefCell::new(arguments)));
                vec![method_name_obj, args_array]
            };
            self.invoke_method(
                method_missing_class,
                method_missing_method,
                receiver,
                method_missing_args,
                position,
            )
        } else {
            Err(undefined_method_error(method_name, &receiver, position))
        }
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
            return self.lookup_method(receiver, name).is_some();
        };
        if module_level_method(class_rc, name).is_some() {
            return true;
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
        self.builtins()
            .class_of(receiver)
            .find_method(name)
            .is_some()
    }

    /// Whether `name` resolves to a private or protected method on
    /// `receiver`, so an explicit-receiver call would be refused.
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
            Some(owner) => !owner.has_public_override(name) && owner.is_method_restricted(name),
            None => false,
        }
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
                if let Some(sc) = current.singleton_class_slot().clone()
                    && let Some((owner, _)) = sc.find_method_with_owner(name)
                {
                    // The nearest singleton with its own marking for this
                    // name settles it; a subclass's `private_class_method`
                    // marks the name without redefining the method.
                    if sc.has_public_override(name) || sc.is_method_restricted(name) {
                        return Some(sc);
                    }
                    defining_singleton.get_or_insert(owner);
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
                class_rc
                    .find_method(method_name)
                    .map(|method| (Rc::clone(class_rc), method))
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
                let class = self.builtins().class_of(receiver);
                class.find_method(method_name).map(|method| (class, method))
            }
        }
    }
}

/// A module-level method on a class or module object: one stored under the
/// `__class__` convention by `def self.name` or `module_function`, or copied
/// in by `extend` under the `__ext__` convention.
fn module_level_method(class_rc: &Rc<Class>, method_name: &str) -> Option<Rc<Method>> {
    if let Some(method) = class_rc.find_method(&format!("__class__{}", method_name)) {
        return Some(method);
    }
    match class_rc.get_class_var(&format!("__ext__{}", method_name)) {
        Some(Object::Method(method)) => Some(method),
        _ => None,
    }
}
