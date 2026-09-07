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
                return Err(MetorexError::runtime_error(
                    "no block given (yield)".to_string(),
                    position_to_location(position),
                ));
            }
        };

        // `yield(*values)` hands the block the elements one by one, the way a
        // splat does in any other argument list.
        let mut evaluated_args = Vec::with_capacity(arguments.len());
        for argument in arguments {
            if let Expression::Splat { expression, .. } = argument {
                match self.evaluate_expression(expression)? {
                    Object::Array(elements) => {
                        evaluated_args.extend(elements.borrow().iter().cloned());
                    }
                    other => evaluated_args.push(other),
                }
                continue;
            }
            evaluated_args.push(self.evaluate_expression(argument)?);
        }

        block.call(self, evaluated_args, position)
    }
}
