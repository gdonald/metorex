// The place values of a number in a base.

use super::*;

impl VirtualMachine {
    /// The place values of an integer in `base`, least significant first.
    pub(crate) fn integer_digits(
        &mut self,
        receiver: &Object,
        base: num_bigint::BigInt,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let zero = num_bigint::BigInt::from(0);
        let value = receiver.as_big_integer().expect("integer-kinded");
        if value < zero {
            return Err(crate::vm::errors::simple_exception(
                "Math::DomainError",
                "out of domain",
                position,
            ));
        }
        if base < num_bigint::BigInt::from(2) {
            let message = if base < zero {
                "negative radix".to_string()
            } else {
                format!("invalid radix {}", base)
            };
            return Err(crate::vm::errors::simple_exception(
                "ArgumentError",
                &message,
                position,
            ));
        }
        let mut digits = Vec::new();
        let mut remaining = value;
        while remaining > zero {
            digits.push(Object::integer(&remaining % &base));
            remaining /= &base;
        }
        if digits.is_empty() {
            digits.push(Object::Int(0));
        }
        Ok(Object::array(digits))
    }

    /// The Float a `coerce` argument stands for. Ruby reads a String as a
    /// number and asks anything else for `to_f`.
    pub(crate) fn coerce_to_float(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<f64, MetorexError> {
        if let Object::String(text) = argument {
            return text.as_str().trim().parse::<f64>().map_err(|_| {
                let message = format!("invalid value for Float(): {:?}", text.as_str());
                MetorexError::UncaughtException {
                    exception: Object::exception("ArgumentError", message.clone()),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                }
            });
        }
        if let Object::Float(value) = argument {
            return Ok(*value);
        }
        if self.responds_to(argument, "to_f")
            && let Object::Float(value) =
                self.send_to_object(argument.clone(), "to_f", vec![], position)?
        {
            return Ok(value);
        }
        Err(self.uncoercible(argument, position))
    }
}
