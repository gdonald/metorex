// The protocol a number follows for an operand it does not know.

use super::*;

impl VirtualMachine {
    /// Ruby's coercion protocol: a number handed an operand it does not know
    /// asks that operand to `coerce` it, then applies the operator to the pair
    /// it answers. An error raised inside `coerce` belongs to the caller, so it
    /// travels out rather than becoming a TypeError here.
    pub(crate) fn coerce_binary_operand(
        &mut self,
        op: &BinaryOp,
        left: &Object,
        right: &Object,
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // A Rational or Complex asks for coercion too, since its own
        // arithmetic only knows the numbers it can already work with.
        let numeric_left =
            is_number(left) || crate::vm::native_methods::rational_parts(left).is_some();
        if !coerces_its_operand(op) || !numeric_left || !takes_coercion(right) {
            return Ok(None);
        }
        // The operand is asked whether it coerces, the way Ruby asks, so an
        // object that answers `respond_to?` for itself is heard.
        if !self.answers_to(right, "coerce", position)? {
            return Ok(None);
        }
        let pair = self.send_to_object(right.clone(), "coerce", vec![left.clone()], position)?;
        // A `coerce` that answers no pair leaves the operator with nothing to
        // apply, which for an ordering means the two have no order at all.
        let parts = match &pair {
            Object::Array(parts) if parts.borrow().len() == 2 => parts.borrow().clone(),
            // A `coerce` that answers no pair leaves the operator with
            // nothing to apply. Ruby refuses it for a Float, where the
            // comparison is a relational one, and reports no order at all
            // for the rest.
            _ if matches!(op, BinaryOp::Spaceship) && !matches!(left, Object::Float(_)) => {
                return Ok(Some(Object::Nil));
            }
            _ if matches!(op, BinaryOp::Spaceship) => {
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    "coerce must return [x, y]",
                    position,
                ));
            }
            _ => return Ok(None),
        };
        let name = crate::vm::native_methods::ast_methods::binary_op_str(op);
        self.send_to_object(parts[0].clone(), name, vec![parts[1].clone()], position)
            .map(Some)
    }
}

/// Whether the operator consults `coerce` for an operand it does not know.
/// Equality does not: Ruby answers it by asking the other object instead.
fn coerces_its_operand(op: &BinaryOp) -> bool {
    matches!(
        op,
        BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo
            | BinaryOp::Power
            | BinaryOp::BitwiseAnd
            | BinaryOp::BitwiseOr
            | BinaryOp::Xor
            | BinaryOp::Less
            | BinaryOp::Greater
            | BinaryOp::LessEqual
            | BinaryOp::GreaterEqual
            | BinaryOp::Spaceship
    )
}

/// Whether this operand is one of the numbers the operators handle directly.
pub(crate) fn is_number(value: &Object) -> bool {
    matches!(value, Object::Int(_) | Object::BigInt(_) | Object::Float(_))
}

/// Whether an operand is a candidate for coercion. Rational and Complex carry
/// their own arithmetic, so they are left to it.
pub(crate) fn takes_coercion(value: &Object) -> bool {
    match value {
        Object::Instance(instance) => {
            !matches!(instance.borrow().class.name(), "Rational" | "Complex")
        }
        _ => false,
    }
}

/// The nearest Float to an arbitrary-precision integer, which is what Ruby
/// answers when one meets a Float in arithmetic. A magnitude past the Float
/// range becomes an infinity, as Ruby's does.
pub(crate) fn big_to_float(value: &num_bigint::BigInt) -> f64 {
    use std::str::FromStr;
    f64::from_str(&value.to_string()).unwrap_or(f64::INFINITY)
}

/// The name a numeric operator is written under, for looking one up that the
/// program defined of its own.
pub(crate) fn comparison_free_operator_name(op: &BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("+"),
        BinaryOp::Subtract => Some("-"),
        BinaryOp::Multiply => Some("*"),
        BinaryOp::Divide => Some("/"),
        BinaryOp::Modulo => Some("%"),
        BinaryOp::Power => Some("**"),
        _ => None,
    }
}
