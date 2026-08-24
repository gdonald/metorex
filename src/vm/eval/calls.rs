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
        // Also prefer self-method dispatch when the env binding is a global
        // NativeFunction (e.g. `define_method`) but the receiver actually has
        // a method by that name — this lets `class_eval { define_method ... }`
        // hit the class's own define_method instead of the global stub.
        if let Expression::Identifier { name, .. } = callee
            && self.environment().get("self").is_some()
        {
            let env_val = self.environment().get(name);
            let dispatch_to_self = match &env_val {
                None => true,
                // `define_method`, `private`, `public` live in globals as
                // top-level conveniences, but inside a class/module body or
                // class_eval they should resolve to the receiver's method so
                // `private :foo` marks Tally#foo (not Object#foo) as private.
                Some(Object::NativeFunction(fn_name))
                    if matches!(
                        fn_name.as_str(),
                        "define_method" | "private" | "public" | "protected" | "module_function"
                    ) =>
                {
                    matches!(
                        self.environment().get("self"),
                        Some(Object::Class(_)) | Some(Object::Module(_))
                    )
                }
                _ => false,
            };
            if dispatch_to_self {
                return self.evaluate_method_call(
                    &Expression::SelfExpr { position },
                    name,
                    arguments,
                    trailing_block,
                    position,
                );
            }
        }

        // `using` is a special native whose identifier resolution auto-invokes
        // with zero args; when args are present we must bypass that and call
        // directly with the evaluated args.
        if let Expression::Identifier { name, .. } = callee
            && name == "using"
            && let Some(Object::NativeFunction(fn_name)) = self.environment().get("using")
            && fn_name == "using"
        {
            let evaluated_args = self.evaluate_arguments(arguments)?;
            if let Some(block_expr) = trailing_block {
                self.pending_block = Some(self.evaluate_expression(block_expr)?);
            }
            return self.call_native_function("using", evaluated_args, position);
        }

        // `__method__()`, `__callee__()`, `abort` and `binding` are
        // auto-invoked when named bare, so a call form has to reach the native
        // function rather than calling whatever the bare name evaluated to.
        if let Expression::Identifier { name, .. } = callee
            && matches!(
                name.as_str(),
                "__method__" | "__callee__" | "abort" | "binding"
            )
            && let Some(Object::NativeFunction(native_name)) = self.environment().get(name)
        {
            let evaluated_args = self.evaluate_arguments(arguments)?;
            return self.call_native_function(&native_name, evaluated_args, position);
        }

        // `Hash(x)` and the other Kernel conversion functions share a name
        // with a constant, so they are resolved here rather than by letting
        // the identifier fall through to the class it collides with.
        if let Expression::Identifier { name, .. } = callee
            && crate::vm::native_methods::kernel_conversion::is_kernel_conversion(name)
        {
            let evaluated_args = self.evaluate_arguments(arguments)?;
            if let Some(result) = self.call_kernel_conversion(name, &evaluated_args, position)? {
                return Ok(result);
            }
        }

        let callable = self.evaluate_expression(callee);
        let evaluated_args = self.evaluate_arguments(arguments)?;
        let has_block = trailing_block.is_some();
        if let Some(block_expr) = trailing_block {
            self.pending_block = Some(self.evaluate_expression(block_expr)?);
        }
        match callable {
            Ok(func) => {
                let result = self.invoke_callable(func, evaluated_args, position);
                // `break <value>` in the attached block unwinds here.
                match result {
                    Err(MetorexError::BlockBreak { value, .. }) if has_block => Ok(value),
                    other => other,
                }
            }
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
