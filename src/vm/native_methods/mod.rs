//! Native (built-in) method implementations for the virtual machine.
//!
//! This module contains the implementations of all built-in methods for
//! standard classes like Object, String, and Array.

mod array_methods;
pub(crate) mod ast_methods;
mod class_methods;
mod exception_methods;
mod file_methods;
mod float_methods;
mod hash_methods;
mod int_methods;
mod method_object_methods;
mod module_methods;
mod object_methods;
mod range_methods;
mod set_methods;
mod string_methods;
mod visibility;

use super::VirtualMachine;
use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use std::rc::Rc;

impl VirtualMachine {
    /// Attempt to execute a native (built-in) method implementation.
    ///
    /// Returns `Ok(Some(result))` if a native method was found and executed successfully,
    /// `Ok(None)` if no native method exists (allowing fallback to user-defined methods),
    /// or `Err` if the method call failed.
    pub(crate) fn call_native_method(
        &mut self,
        class: &Class,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // Binding receiver
        if let Object::Binding(binding) = receiver
            && method_name == "receiver"
        {
            return Ok(Some(binding.receiver.clone().unwrap_or(Object::Nil)));
        }

        // Block/Lambda methods
        if let Object::Block(block) = receiver {
            match method_name {
                "call" | "[]" => {
                    return Ok(Some(block.call(self, arguments.to_vec(), position)?));
                }
                "binding" => {
                    use crate::object::Binding;
                    let binding = Binding::new(block.captured_vars().clone());
                    return Ok(Some(Object::Binding(Rc::new(binding))));
                }
                _ => {}
            }
        }

        // Module-specific methods (refine, module_eval, stdlib stubs). Falls
        // through to call_class_methods afterwards so Class/Module share the
        // same table for things like `name`, `extend`, `remove_const`, etc.
        if let Object::Module(module_rc) = receiver {
            if let Some(result) =
                self.call_module_methods(module_rc, receiver, method_name, arguments, position)?
            {
                return Ok(Some(result));
            }
            if let Some(result) =
                self.call_class_methods(module_rc, method_name, arguments, position)?
            {
                return Ok(Some(result));
            }
        }

        // Class-specific methods (File/Dir dispatch first, then general class methods)
        if let Object::Class(class_rc) = receiver {
            if let Some(result) =
                self.call_file_dir_methods(class_rc, method_name, arguments, position)?
            {
                return Ok(Some(result));
            }
            if let Some(result) =
                self.call_class_methods(class_rc, method_name, arguments, position)?
            {
                return Ok(Some(result));
            }
        }

        // Method/Block object introspection
        if let Some(result) =
            self.call_method_object_methods(receiver, method_name, arguments, position)?
        {
            return Ok(Some(result));
        }

        // Dispatch to the appropriate class-specific method implementation
        match class.name() {
            "Object" => self.call_object_method(receiver, method_name, arguments, position),
            "String" => self.call_string_method(receiver, method_name, arguments, position),
            "Integer" => self.call_int_method(receiver, method_name, arguments, position),
            "Array" => self.call_array_method(receiver, method_name, arguments, position),
            "Hash" => self.call_hash_method(receiver, method_name, arguments, position),
            "Float" => self.call_float_method(receiver, method_name, arguments, position),
            "Range" => self.call_range_method(receiver, method_name, arguments, position),
            "Set" => self.call_set_method(receiver, method_name, arguments, position),
            "Exception" => self.call_exception_method(receiver, method_name, arguments, position),
            "Thread" => self.call_thread_method(receiver, method_name, arguments, position),
            _ => Ok(None),
        }
    }

    /// Instance-level Thread methods. The "thread" runs synchronously when
    /// `value`/`join` is called for the first time.
    pub(crate) fn call_thread_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let inst = match receiver {
            Object::Instance(i) => Rc::clone(i),
            _ => return Ok(None),
        };
        match method_name {
            "value" | "join" => {
                if !arguments.is_empty() && method_name == "value" {
                    return Err(crate::vm::errors::method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let cached = inst.borrow().get_var("__thread_value").cloned();
                if let Some(val) = cached {
                    return Ok(Some(if method_name == "join" {
                        receiver.clone()
                    } else {
                        val
                    }));
                }
                let block_obj = inst
                    .borrow()
                    .get_var("__thread_block")
                    .cloned()
                    .unwrap_or(Object::Nil);
                let value = if let Object::Block(b) = block_obj {
                    self.execute_block_body(&b, vec![])?
                } else {
                    Object::Nil
                };
                inst.borrow_mut()
                    .set_var("__thread_value".to_string(), value.clone());
                Ok(Some(if method_name == "join" {
                    receiver.clone()
                } else {
                    value
                }))
            }
            "alive?" | "stop?" => Ok(Some(Object::Bool(false))),
            "status" => Ok(Some(Object::Bool(false))),
            _ => Ok(None),
        }
    }
}
