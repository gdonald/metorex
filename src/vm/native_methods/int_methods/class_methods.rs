// The Integer methods that belong to the class.

use super::*;

impl VirtualMachine {
    /// `Integer.sqrt` and `Integer.try_convert`, which belong to the class.
    pub(crate) fn integer_class_method(
        &mut self,
        method_name: &str,
        argument: &Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        if method_name == "try_convert" {
            // An Integer answers itself, an object with `to_int` answers what
            // that gives, and anything else is not an Integer at all.
            if matches!(argument, Object::Int(_) | Object::BigInt(_)) {
                return Ok(argument.clone());
            }
            if !self.responds_to(argument, "to_int") {
                return Ok(Object::Nil);
            }
            let source = self.builtins().class_of(argument).name().to_string();
            let converted = self.send_to_object(argument.clone(), "to_int", vec![], position)?;
            return match converted {
                Object::Nil | Object::Int(_) | Object::BigInt(_) => Ok(converted),
                other => {
                    let message = format!(
                        "can't convert {} into Integer ({}#to_int gives {})",
                        source,
                        source,
                        self.builtins().class_of(&other).name()
                    );
                    Err(MetorexError::UncaughtException {
                        exception: Object::exception("TypeError", message.clone()),
                        location: crate::vm::utils::position_to_location(position),
                        message,
                    })
                }
            };
        }
        // `sqrt` takes the whole part of the square root, so it answers an
        // Integer for a number of any width.
        let value = match argument {
            Object::Float(number) => num_bigint::BigInt::from(*number as i64),
            other => self.coerce_integer_argument(other, position)?,
        };
        if value < num_bigint::BigInt::from(0) {
            return Err(crate::vm::errors::simple_exception(
                "Math::DomainError",
                "Numerical argument is out of domain - \"isqrt\"",
                position,
            ));
        }
        Ok(Object::integer(integer_square_root(value)))
    }
}
