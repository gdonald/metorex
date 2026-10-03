// The number an argument stands for.

use super::*;

impl VirtualMachine {
    /// An Integer argument: one as it stands, and anything else through
    /// `to_int`. Ruby refuses everything else, `coerce` included, since a bit
    /// operation is not arithmetic between two numbers.
    pub(crate) fn coerce_integer_argument(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<num_bigint::BigInt, MetorexError> {
        if let Some(value) = argument.as_big_integer() {
            return Ok(value);
        }
        let source = self.builtins().class_of(argument).name().to_string();
        let refuse = |message: String| MetorexError::UncaughtException {
            exception: Object::exception("TypeError", message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        };
        let Some((class, method)) = self.lookup_method(argument, "to_int") else {
            if matches!(argument, Object::Nil) {
                return Err(refuse(
                    "no implicit conversion from nil to integer".to_string(),
                ));
            }
            return Err(refuse(format!(
                "no implicit conversion of {} into Integer",
                self.conversion_name(argument)
            )));
        };
        let converted =
            self.invoke_method(class, method, argument.clone(), Vec::new(), position)?;
        converted.as_big_integer().ok_or_else(|| {
            refuse(format!(
                "can't convert {} to Integer ({}#to_int gives {})",
                source,
                source,
                self.builtins().class_of(&converted).name()
            ))
        })
    }

    /// The precision argument to a rounding method: an Integer as it stands,
    /// and anything else through `to_int`. One too large to count digits with
    /// is a RangeError rather than a number to round by.
    pub(crate) fn coerce_precision_argument(
        &mut self,
        argument: &Object,
        position: Position,
    ) -> Result<i64, MetorexError> {
        let refuse = |class_name: &str, message: String| MetorexError::UncaughtException {
            exception: Object::exception(class_name, message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        };
        if matches!(argument, Object::Float(value) if !value.is_finite()) {
            return Err(refuse(
                "RangeError",
                "float infinity out of range of integer".to_string(),
            ));
        }
        let value = self.coerce_integer_argument(argument, position)?;
        i32::try_from(&value).map(i64::from).map_err(|_| {
            refuse(
                "RangeError",
                format!("integer {} out of range of int", value),
            )
        })
    }

    /// The Float an operand answers through the coercion protocol, when it
    /// takes part in one at all.
    pub(crate) fn coerced_float(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<Option<f64>, MetorexError> {
        if !self.responds_to(value, "coerce") {
            return Ok(None);
        }
        let pair = self.send_to_object(value.clone(), "coerce", vec![Object::Int(1)], position)?;
        let Object::Array(parts) = pair else {
            return Ok(None);
        };
        let parts = parts.borrow().clone();
        if parts.len() != 2 {
            return Ok(None);
        }
        match self.send_to_object(parts[1].clone(), "to_f", vec![], position)? {
            Object::Float(number) => Ok(Some(number)),
            Object::Int(number) => Ok(Some(number as f64)),
            _ => Ok(None),
        }
    }

    /// The TypeError Ruby raises for an operand that cannot become a number.
    pub(crate) fn uncoercible(&mut self, value: &Object, position: Position) -> MetorexError {
        let message = format!(
            "{} can't be coerced into Integer",
            self.builtins().class_of(value).name()
        );
        MetorexError::UncaughtException {
            exception: Object::exception("TypeError", message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        }
    }
}
