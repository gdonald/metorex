// The `+` and `-` written in front of a value.

use super::*;

impl VirtualMachine {
    /// The method a program of its own put on Integer or Float for an
    /// operator, or None where the numbers answer for themselves.
    pub(crate) fn redefined_number_operator(
        &mut self,
        held: &Object,
        named: &str,
    ) -> Option<(Rc<crate::class::Class>, Rc<crate::object::Method>)> {
        let class = self.builtins().class_of(held);
        let found = class.find_own_method(named)?;
        if found.is_undefined || found.body.is_empty() {
            return None;
        }
        Some((class, found))
    }

    /// The arithmetic a number answers for an operator on its own, reached
    /// without consulting a redefinition. An UnboundMethod cut from Integer
    /// before the program replaced `+` calls through here, so binding it back
    /// runs the original rather than the replacement.
    pub(crate) fn builtin_number_operator(
        &mut self,
        named: &str,
        left: Object,
        right: Object,
        position: Position,
    ) -> Option<Result<Object, MetorexError>> {
        if !matches!(left, Object::Int(_) | Object::BigInt(_) | Object::Float(_)) {
            return None;
        }
        // Anything but a number on the right is asked to `coerce`, which is
        // the ordinary dispatch rather than the arithmetic underneath.
        if !matches!(right, Object::Int(_) | Object::BigInt(_) | Object::Float(_)) {
            return None;
        }
        match named {
            "+" => Some(self.evaluate_addition(left, right, position)),
            "-" => Some(self.evaluate_numeric_binary(&BinaryOp::Subtract, left, right, position)),
            "*" => Some(self.evaluate_numeric_binary(&BinaryOp::Multiply, left, right, position)),
            "/" => Some(self.evaluate_numeric_binary(&BinaryOp::Divide, left, right, position)),
            "%" => Some(self.evaluate_numeric_binary(&BinaryOp::Modulo, left, right, position)),
            "**" => Some(self.evaluate_numeric_binary(&BinaryOp::Power, left, right, position)),
            _ => None,
        }
    }

    /// Evaluate a unary operation (`+` or `-`).
    pub(crate) fn evaluate_unary_operation(
        &self,
        op: &UnaryOp,
        value: Object,
        position: Position,
    ) -> Result<Object, MetorexError> {
        match op {
            UnaryOp::Plus => match value {
                // Numeric `+x` is a no-op identity.
                Object::Int(_) | Object::BigInt(_) | Object::Float(_) => Ok(value),
                // `+str` asks for a string that changes, so a frozen one and
                // one carrying notice that it will be frozen both answer a
                // copy, and any other answers itself.
                Object::String(ref text) => {
                    if !text.is_frozen() && !text.is_chilled() {
                        return Ok(value.clone());
                    }
                    Ok(Object::String(std::rc::Rc::new(
                        crate::object::StringValue::with_encoding(
                            text.to_text(),
                            text.encoding_name(),
                        ),
                    )))
                }
                _ => Err(unary_type_error(op, &value, position)),
            },
            UnaryOp::Minus => match value {
                // Negating i64::MIN needs the wider type.
                Object::Int(v) => Ok(Object::integer(-num_bigint::BigInt::from(v))),
                Object::BigInt(v) => Ok(Object::integer(-(*v).clone())),
                Object::Float(v) => Ok(Object::Float(-v)),
                // `-str` asks for a string that does not change, so one
                // already frozen answers itself and any other answers a
                // frozen copy.
                Object::String(ref text) => {
                    if text.is_frozen() {
                        text.mark_deduplicated();
                        return Ok(value.clone());
                    }
                    let copy = crate::object::StringValue::with_encoding(
                        text.to_text(),
                        text.encoding_name(),
                    );
                    if text.holds_bytes() {
                        copy.mark_bytes();
                    }
                    copy.mark_deduplicated();
                    Ok(Object::String(std::rc::Rc::new(copy)))
                }
                _ => Err(unary_type_error(op, &value, position)),
            },
            UnaryOp::Not => Ok(Object::Bool(matches!(
                value,
                Object::Bool(false) | Object::Nil
            ))),
        }
    }
}
