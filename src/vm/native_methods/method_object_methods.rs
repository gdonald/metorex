use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::utils::position_to_location;
use std::rc::Rc;

impl VirtualMachine {
    pub(crate) fn call_method_object_methods(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if let Object::Method(method_obj) = receiver {
            match method_name {
                "bind" => {
                    let target = arguments.first().cloned().unwrap_or(Object::Nil);
                    let bound = method_obj.bind(target);
                    return Ok(Some(Object::Method(Rc::new(bound))));
                }
                "call" | "[]" | "===" => {
                    let bound = method_obj
                        .receiver
                        .as_ref()
                        .map(|b| (**b).clone())
                        .unwrap_or(Object::Nil);
                    // Module-private mixin hooks (`append_features` and
                    // friends) can't be invoked on a Class receiver; Ruby
                    // raises TypeError when the bound target is a Class.
                    if matches!(
                        method_obj.name.as_str(),
                        "append_features" | "prepend_features" | "extend_object"
                    ) && matches!(&bound, Object::Class(_))
                    {
                        let msg = format!(
                            "bind argument must be an instance of Module: {}",
                            method_obj.name
                        );
                        let exc = Object::exception("TypeError", msg.clone());
                        return Err(MetorexError::UncaughtException {
                            exception: exc,
                            location: position_to_location(position),
                            message: msg,
                        });
                    }
                    let owner = match &method_obj.owner_class {
                        Some(owner) => Rc::clone(owner),
                        None => self.builtins().class_of(&bound),
                    };
                    let result = self.invoke_method(
                        owner,
                        Rc::clone(method_obj),
                        bound,
                        arguments.to_vec(),
                        position,
                    )?;
                    return Ok(Some(result));
                }
                "name" => {
                    return Ok(Some(Object::String(Rc::new(method_obj.name.clone()))));
                }
                "unbind" => {
                    let mut unbound = (**method_obj).clone();
                    unbound.receiver = None;
                    return Ok(Some(Object::Method(Rc::new(unbound))));
                }
                // `Method#to_proc` stays attached to the receiver it was
                // extracted from, so the Proc keeps calling against that
                // object even after `define_method` installs it elsewhere.
                "to_proc" => {
                    let mut as_proc = (**method_obj).clone();
                    as_proc.bound_self = method_obj.receiver.clone();
                    return Ok(Some(Object::Method(Rc::new(as_proc))));
                }
                "owner" => {
                    if let Some(owner) = &method_obj.owner_class {
                        let owner = Rc::clone(owner);
                        return Ok(Some(if owner.is_module() {
                            Object::Module(owner)
                        } else {
                            Object::Class(owner)
                        }));
                    }
                    let owner_name = method_obj.owner.as_deref().unwrap_or("main");
                    return Ok(Some(Object::String(Rc::new(owner_name.to_string()))));
                }
                "source_location" => {
                    if let Some(loc) = &method_obj.source_location {
                        return Ok(Some(Object::String(Rc::new(loc.to_string()))));
                    } else {
                        return Ok(Some(Object::String(Rc::new("unknown".to_string()))));
                    }
                }
                "parameters" => {
                    let params: Vec<Object> = method_obj
                        .parameters
                        .iter()
                        .map(|p| Object::String(Rc::new(p.clone())))
                        .collect();
                    return Ok(Some(Object::Array(Rc::new(std::cell::RefCell::new(
                        params,
                    )))));
                }
                "body" => {
                    return Ok(Some(super::ast_methods::serialize_statements(
                        &method_obj.body,
                    )));
                }
                "arity" => {
                    return Ok(Some(Object::Int(method_obj.parameters.len() as i64)));
                }
                _ => {}
            }
        }

        if let Object::Block(block_obj) = receiver {
            match method_name {
                "statements" => {
                    return Ok(Some(super::ast_methods::serialize_statements(
                        &block_obj.body,
                    )));
                }
                "arity" => {
                    return Ok(Some(Object::Int(block_obj.parameters.len() as i64)));
                }
                _ => {}
            }
        }

        Ok(None)
    }
}
