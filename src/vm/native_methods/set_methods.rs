//! Native method implementations for the Set class.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::{Object, ObjectHash};
use crate::vm::VirtualMachine;
use crate::vm::errors::*;
use crate::vm::utils::position_to_location;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

impl VirtualMachine {
    /// Execute native methods for the Set class.
    pub(crate) fn call_set_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        let Object::Set(set_rc) = receiver else {
            return Ok(None);
        };
        match method_name {
            "add" | "insert" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let hash = ObjectHash::from_object(&arguments[0]).ok_or_else(|| {
                    MetorexError::runtime_error(
                        format!(
                            "Cannot add {} to set (not hashable)",
                            arguments[0].type_name()
                        ),
                        position_to_location(position),
                    )
                })?;
                set_rc.borrow_mut().insert(hash);
                Ok(Some(receiver.clone()))
            }
            "remove" | "delete" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let hash = ObjectHash::from_object(&arguments[0]).ok_or_else(|| {
                    MetorexError::runtime_error(
                        format!(
                            "Cannot remove {} from set (not hashable)",
                            arguments[0].type_name()
                        ),
                        position_to_location(position),
                    )
                })?;
                let removed = set_rc.borrow_mut().remove(&hash);
                Ok(Some(Object::Bool(removed)))
            }
            "contains?" | "include?" | "has?" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let hash = match ObjectHash::from_object(&arguments[0]) {
                    Some(h) => h,
                    None => return Ok(Some(Object::Bool(false))),
                };
                Ok(Some(Object::Bool(set_rc.borrow().contains(&hash))))
            }
            "size" | "length" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(set_rc.borrow().len() as i64)))
            }
            "empty?" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Bool(set_rc.borrow().is_empty())))
            }
            "to_a" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let elements: Vec<Object> = set_rc
                    .borrow()
                    .iter()
                    .map(|h| Object::string(h.hash_value.clone()))
                    .collect();
                Ok(Some(Object::Array(Rc::new(RefCell::new(elements)))))
            }
            "union" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = match &arguments[0] {
                    Object::Set(other_rc) => other_rc,
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Set",
                            &arguments[0],
                            position,
                        ));
                    }
                };
                let result: HashSet<ObjectHash> =
                    set_rc.borrow().union(&other.borrow()).cloned().collect();
                Ok(Some(Object::Set(Rc::new(RefCell::new(result)))))
            }
            "intersection" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = match &arguments[0] {
                    Object::Set(other_rc) => other_rc,
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Set",
                            &arguments[0],
                            position,
                        ));
                    }
                };
                let result: HashSet<ObjectHash> = set_rc
                    .borrow()
                    .intersection(&other.borrow())
                    .cloned()
                    .collect();
                Ok(Some(Object::Set(Rc::new(RefCell::new(result)))))
            }
            "difference" => {
                if arguments.len() != 1 {
                    return Err(method_argument_error(
                        method_name,
                        1,
                        arguments.len(),
                        position,
                    ));
                }
                let other = match &arguments[0] {
                    Object::Set(other_rc) => other_rc,
                    _ => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Set",
                            &arguments[0],
                            position,
                        ));
                    }
                };
                let result: HashSet<ObjectHash> = set_rc
                    .borrow()
                    .difference(&other.borrow())
                    .cloned()
                    .collect();
                Ok(Some(Object::Set(Rc::new(RefCell::new(result)))))
            }
            "each" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                let block = match self.pending_block.take() {
                    Some(Object::Block(b)) => b,
                    Some(other) => {
                        return Err(method_argument_type_error(
                            method_name,
                            "Block",
                            &other,
                            position,
                        ));
                    }
                    None => {
                        return Err(MetorexError::runtime_error(
                            "each requires a block",
                            position_to_location(position),
                        ));
                    }
                };
                let elements: Vec<ObjectHash> = set_rc.borrow().iter().cloned().collect();
                for elem in elements {
                    let args = vec![Object::string(elem.hash_value)];
                    match self.execute_block_with_control_flow(&block, args)? {
                        super::super::ControlFlow::Next
                        | super::super::ControlFlow::Value(_)
                        | super::super::ControlFlow::Redo { .. }
                        | super::super::ControlFlow::Continue { .. } => {
                            continue;
                        }
                        super::super::ControlFlow::Break { .. } => break,
                        super::super::ControlFlow::Return { value, position } => {
                            return Err(MetorexError::NonLocalReturn {
                                value,
                                location: super::super::utils::position_to_location(position),
                            });
                        }
                        super::super::ControlFlow::Exception {
                            exception,
                            position,
                        } => {
                            return Err(MetorexError::runtime_error(
                                format!(
                                    "Uncaught exception: {}",
                                    super::super::utils::format_exception(&exception)
                                ),
                                super::super::utils::position_to_location(position),
                            ));
                        }
                    }
                }
                Ok(Some(receiver.clone()))
            }
            _ => Ok(None),
        }
    }
}
