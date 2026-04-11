// Call expression evaluation: bare-id self-method dispatch and callable invocation.

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;

use crate::vm::core::VirtualMachine;

impl VirtualMachine {
    /// Evaluate a `Call` expression. Bare-identifier callees prefer
    /// `self.method(args)` dispatch over auto-invoking the identifier; this
    /// avoids the bug where the identifier path would call the method with
    /// zero args and discard the user's arguments.
    pub(super) fn eval_call(
        &mut self,
        callee: &Expression,
        arguments: &[Expression],
        trailing_block: Option<&Expression>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // If callee is a bare identifier and it's not a local variable,
        // dispatch as a method call with the supplied arguments.
        if let Expression::Identifier { name, .. } = callee
            && self.environment().get(name).is_none()
            && self.environment().get("self").is_some()
        {
            return self.evaluate_method_call(
                &Expression::SelfExpr { position },
                name,
                arguments,
                trailing_block,
                position,
            );
        }

        let callable = self.evaluate_expression(callee);
        let evaluated_args = self.evaluate_arguments(arguments)?;
        if let Some(block_expr) = trailing_block {
            self.pending_block = Some(self.evaluate_expression(block_expr)?);
        }
        match callable {
            Ok(func) => self.invoke_callable(func, evaluated_args, position),
            Err(_) => {
                if let Expression::Identifier { name, .. } = callee
                    && self.environment().get("self").is_some()
                {
                    return self.evaluate_method_call(
                        &Expression::SelfExpr { position },
                        name,
                        arguments,
                        trailing_block,
                        position,
                    );
                }
                callable.and_then(|f| self.invoke_callable(f, evaluated_args, position))
            }
        }
    }
}
