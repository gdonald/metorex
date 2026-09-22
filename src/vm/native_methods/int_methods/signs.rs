// The magnitude and sign of a number, and the values it converts to.

use super::*;

impl VirtualMachine {
    /// The magnitude and sign of a number, and the values it converts to.
    pub(crate) fn call_int_sign_method(
        &mut self,
        receiver: &Object,
        n: &i64,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        match method_name {
            "abs" | "magnitude" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                // The widest negative number has no positive of its own
                // width, so its size is carried in the wider type.
                let held = num_bigint::BigInt::from(*n);
                Ok(Some(Object::integer(
                    if held < num_bigint::BigInt::from(0) {
                        -held
                    } else {
                        held
                    },
                )))
            }
            // The unary operators under the names `send` reaches them by.
            "-@" | "+@" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(match method_name {
                    // Negating the smallest Integer needs the wider type,
                    // since its opposite does not fit in the narrow one.
                    "-@" => match n.checked_neg() {
                        Some(negated) => Object::Int(negated),
                        None => Object::integer(-num_bigint::BigInt::from(*n)),
                    },
                    _ => Object::Int(*n),
                }))
            }
            "to_f" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Float(*n as f64)))
            }
            // An Integer is already the whole number these ask for.
            "to_i" | "to_int" | "ord" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(receiver.clone()))
            }
            "size" => {
                if !arguments.is_empty() {
                    return Err(method_argument_error(
                        method_name,
                        0,
                        arguments.len(),
                        position,
                    ));
                }
                Ok(Some(Object::Int(8)))
            }
            _ => Ok(None),
        }
    }
}
