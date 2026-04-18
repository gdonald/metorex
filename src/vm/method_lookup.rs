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

        // Try user-defined method lookup first
        match self.lookup_method(&receiver, method_name) {
            Some((class, method)) if !method.is_undefined => {
                return self.invoke_method(class, method, receiver, arguments, position);
            }
            _ => {}
        }

        // For Class/Module receivers, check for extended methods (__ext__name class var)
        let ext_class_rc = match &receiver {
            Object::Class(c) => Some(Rc::clone(c)),
            Object::Module(m) => Some(Rc::clone(m)),
            _ => None,
        };
        if let Some(class_rc) = ext_class_rc {
            let ext_key = format!("__ext__{}", method_name);
            if let Some(Object::Method(ext_method)) = class_rc.get_class_var(&ext_key) {
                return self.invoke_method(
                    class_rc,
                    ext_method,
                    receiver.clone(),
                    arguments,
                    position,
                );
            }
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
                // `class << C` installs a singleton class whose method table
                // holds the class's class-level methods — check it before
                // falling back to the `__class__` convention / instance table.
                if let Some(sc) = class_rc.singleton_class_slot().clone()
                    && let Some(method) = sc.find_method(method_name)
                {
                    return Some((sc, method));
                }
                let class_method_name = format!("__class__{}", method_name);
                class_rc
                    .find_method(&class_method_name)
                    .or_else(|| class_rc.find_method(method_name))
                    .map(|method| (Rc::clone(class_rc), method))
            }
            Object::Module(module_rc) => {
                if let Some(sc) = module_rc.singleton_class_slot().clone()
                    && let Some(method) = sc.find_method(method_name)
                {
                    return Some((sc, method));
                }
                let class_method_name = format!("__class__{}", method_name);
                module_rc
                    .find_method(&class_method_name)
                    .or_else(|| module_rc.find_method(method_name))
                    .map(|method| (Rc::clone(module_rc), method))
            }
            _ => {
                let class = self.builtins().class_of(receiver);
                class.find_method(method_name).map(|method| (class, method))
            }
        }
    }
}
