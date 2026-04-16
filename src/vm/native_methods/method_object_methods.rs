use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use std::rc::Rc;

impl VirtualMachine {
    pub(crate) fn call_method_object_methods(
        &mut self,
        receiver: &Object,
        method_name: &str,
        _arguments: &[Object],
        _position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        if let Object::Method(method_obj) = receiver {
            match method_name {
                "name" => {
                    return Ok(Some(Object::String(Rc::new(method_obj.name.clone()))));
                }
                "owner" => {
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
