use crate::class::Class;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Method, Object};
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

impl VirtualMachine {
    pub(crate) fn call_module_methods(
        &mut self,
        module_rc: &Rc<Class>,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if (method_name == "module_eval" || method_name == "class_eval")
            && let Some(Object::Block(block)) = self.pending_block.take()
        {
            self.apply_block_as_class_body_with_self(
                module_rc,
                &block,
                position,
                Object::Module(Rc::clone(module_rc)),
            )?;
            return Ok(Some(Object::Module(Rc::clone(module_rc))));
        }

        if method_name == "refine" {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    "refine",
                    1,
                    arguments.len(),
                    position,
                ));
            }
            let target = match &arguments[0] {
                Object::Class(c) => Rc::clone(c),
                other => {
                    return Err(method_argument_type_error(
                        "refine", "Class", other, position,
                    ));
                }
            };
            let refinement_key = format!("__refine__{}@{:p}", target.name(), Rc::as_ptr(&target));
            let holder = match module_rc.get_class_var(&refinement_key) {
                Some(Object::Class(existing)) => existing,
                _ => Rc::new(Class::new(format!("<refinement:{}>", target.name()), None)),
            };
            if let Some(Object::Block(block)) = self.pending_block.take() {
                self.apply_block_as_class_body(&holder, &block, position)?;
            }
            module_rc.set_class_var(&refinement_key, Object::Class(Rc::clone(&holder)));
            return Ok(Some(Object::Module(holder)));
        }

        if module_rc.name() == "Kernel" && method_name == "load" {
            if arguments.len() != 1 {
                return Err(method_argument_error("load", 1, arguments.len(), position));
            }
            let path = match &arguments[0] {
                Object::String(s) => s.as_str().to_string(),
                other => {
                    return Err(method_argument_type_error(
                        "load", "String", other, position,
                    ));
                }
            };
            self.execute_file(std::path::Path::new(&path))?;
            return Ok(Some(Object::Bool(true)));
        }

        if module_rc.name() == "Signal" && method_name == "trap" {
            self.pending_block.take();
            return Ok(Some(Object::Nil));
        }

        if module_rc.name() == "Process" {
            match method_name {
                "pid" => return Ok(Some(Object::Int(std::process::id() as i64))),
                "ppid" => return Ok(Some(Object::Int(0))),
                "kill" | "wait" | "waitall" | "exit" | "exit!" | "abort" => {
                    return Ok(Some(Object::Nil));
                }
                _ => {}
            }
        }

        if (module_rc.name() == "GC" || module_rc.name() == "ObjectSpace") && method_name != "name"
        {
            return Ok(Some(Object::Nil));
        }

        match method_name {
            "name" => {
                return Ok(Some(Object::String(Rc::new(module_rc.name().to_string()))));
            }
            "remove_method" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "remove_method",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name_str = match &arguments[0] {
                    Object::String(s) => s.as_ref().clone(),
                    Object::Symbol(s) => s.as_ref().clone(),
                    other => {
                        return Err(method_argument_type_error(
                            "remove_method",
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                if !module_rc.remove_method(&name_str) {
                    return Err(MetorexError::runtime_error(
                        format!("method '{}' not defined in {}", name_str, module_rc.name()),
                        position_to_location(position),
                    ));
                }
                return Ok(Some(Object::Nil));
            }
            "undef_method" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "undef_method",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name_str = match &arguments[0] {
                    Object::String(s) => s.as_ref().clone(),
                    Object::Symbol(s) => s.as_ref().clone(),
                    other => {
                        return Err(method_argument_type_error(
                            "undef_method",
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                let sentinel = Method::undefined(name_str.clone());
                module_rc.define_method(&name_str, Rc::new(sentinel));
                return Ok(Some(Object::Nil));
            }
            "alias_method" => {
                if arguments.len() != 2 {
                    return Err(method_argument_error(
                        "alias_method",
                        2,
                        arguments.len(),
                        position,
                    ));
                }
                let new_name = match &arguments[0] {
                    Object::String(s) => s.as_ref().clone(),
                    Object::Symbol(s) => s.as_ref().clone(),
                    other => {
                        return Err(method_argument_type_error(
                            "alias_method",
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                let old_name = match &arguments[1] {
                    Object::String(s) => s.as_ref().clone(),
                    Object::Symbol(s) => s.as_ref().clone(),
                    other => {
                        return Err(method_argument_type_error(
                            "alias_method",
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                if !module_rc.alias_method(&new_name, &old_name) {
                    return Err(MetorexError::runtime_error(
                        format!(
                            "undefined method '{}' for module '{}'",
                            old_name,
                            module_rc.name()
                        ),
                        position_to_location(position),
                    ));
                }
                return Ok(Some(Object::Nil));
            }
            "module_function" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        "module_function",
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let name_str = match &arguments[0] {
                    Object::String(s) => s.as_ref().clone(),
                    Object::Symbol(s) => s.as_ref().clone(),
                    other => {
                        return Err(method_argument_type_error(
                            "module_function",
                            "String or Symbol",
                            other,
                            position,
                        ));
                    }
                };
                if let Some(method) = module_rc.find_method(&name_str) {
                    module_rc.set_class_var(format!("__ext__{}", name_str), Object::Method(method));
                } else {
                    return Err(MetorexError::runtime_error(
                        format!(
                            "undefined method '{}' for module '{}'",
                            name_str,
                            module_rc.name()
                        ),
                        position_to_location(position),
                    ));
                }
                return Ok(Some(Object::Nil));
            }
            _ => {}
        }

        // Fall through to receiver-agnostic dispatch
        let _ = receiver;
        Ok(None)
    }
}
