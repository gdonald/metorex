// The assertions the test helpers are written against.

use super::*;

impl VirtualMachine {
    pub(crate) fn assert_true(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.is_empty() || arguments.len() > 2 {
            return Err(MetorexError::runtime_error(
                format!("assert() expects 1-2 arguments, got {}", arguments.len()),
                crate::vm::utils::position_to_location(position),
            ));
        }
        if arguments[0].is_truthy() {
            Ok(Object::Bool(true))
        } else {
            let msg = if arguments.len() == 2 {
                self.get_string_representation(&arguments[1], position)?
            } else {
                "Assertion failed".to_string()
            };
            Err(MetorexError::runtime_error(
                msg,
                crate::vm::utils::position_to_location(position),
            ))
        }
    }

    pub(crate) fn assert_equality(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if arguments.len() < 2 || arguments.len() > 3 {
            return Err(MetorexError::runtime_error(
                format!(
                    "assert_equal() expects 2-3 arguments, got {}",
                    arguments.len()
                ),
                crate::vm::utils::position_to_location(position),
            ));
        }
        if arguments[0].equals(&arguments[1]) {
            Ok(Object::Bool(true))
        } else {
            let msg = if arguments.len() == 3 {
                self.get_string_representation(&arguments[2], position)?
            } else {
                format!(
                    "Expected {}, got {}",
                    self.get_string_representation(&arguments[0], position)?,
                    self.get_string_representation(&arguments[1], position)?
                )
            };
            Err(MetorexError::runtime_error(
                msg,
                crate::vm::utils::position_to_location(position),
            ))
        }
    }

    pub(crate) fn assert_raises_exception(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // assert_raises expects a block that should raise an error
        if !arguments.is_empty() {
            return Err(MetorexError::runtime_error(
                format!(
                    "assert_raises() expects 0 arguments (with a block), got {}",
                    arguments.len()
                ),
                crate::vm::utils::position_to_location(position),
            ));
        }
        let block = match self.pending_block.take() {
            Some(Object::Block(b)) => b,
            _ => {
                return Err(MetorexError::runtime_error(
                    "assert_raises requires a block",
                    crate::vm::utils::position_to_location(position),
                ));
            }
        };
        match self.execute_block_body(&block, vec![]) {
            Err(_) => Ok(Object::Bool(true)),
            Ok(_) => Err(MetorexError::runtime_error(
                "Expected block to raise an error, but it did not",
                crate::vm::utils::position_to_location(position),
            )),
        }
    }
}
