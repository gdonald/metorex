// `yield` expression: invoke the block bound to the current method.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;

use crate::vm::core::VirtualMachine;
use crate::vm::utils::position_to_location;

impl VirtualMachine {
    /// Evaluate a `yield` expression by invoking the current method's block.
    pub(super) fn eval_yield(
        &mut self,
        arguments: &[Expression],
        position: Position,
    ) -> Result<Object, MetorexError> {
        let block = self.environment().get("__block__").or_else(|| {
            // Also check if there's a named block parameter
            self.environment().get("block_given?").and_then(|bg| {
                if bg == Object::Bool(true) {
                    // The block was bound to a named parameter — find it
                    None
                } else {
                    None
                }
            })
        });

        let block = match block {
            Some(Object::Block(b)) => b,
            _ => {
                // Ruby names this a jump with nowhere to land rather than a
                // plain runtime failure.
                let message = "no block given (yield)".to_string();
                return Err(MetorexError::UncaughtException {
                    exception: crate::object::Object::exception("LocalJumpError", message.clone()),
                    location: position_to_location(position),
                    message,
                });
            }
        };

        // A yield takes its arguments the way a call does: splats spread,
        // and `**hash` passes keywords, or nothing when the hash is empty.
        let evaluated_args = self.evaluate_arguments(arguments)?;

        block.call(self, evaluated_args, position)
    }
}
