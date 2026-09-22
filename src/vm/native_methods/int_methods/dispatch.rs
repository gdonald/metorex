// The order an Integer method is looked for in. Each step answers `None`
// when the name is none of its own, and the next step is tried.

use super::*;

/// A step that reads the receiver as it stands, whichever width it is.
type WholeStep = fn(
    &mut VirtualMachine,
    &Object,
    &str,
    &[Object],
    Position,
) -> Result<Option<Object>, MetorexError>;

/// A step that reads a receiver already known to fit in a machine word.
type NarrowStep = fn(
    &mut VirtualMachine,
    &Object,
    &i64,
    &str,
    &[Object],
    Position,
) -> Result<Option<Object>, MetorexError>;

const WHOLE_STEPS: &[WholeStep] = &[
    VirtualMachine::call_int_bit_method,
    VirtualMachine::call_int_shared_method,
];

const NARROW_STEPS: &[NarrowStep] = &[
    VirtualMachine::call_int_bit_operation,
    VirtualMachine::call_int_sign_method,
    VirtualMachine::call_int_text_method,
    VirtualMachine::call_int_walk_method,
];

impl VirtualMachine {
    /// The Integer method `method_name` names, run against `receiver`, or
    /// `None` when no group answers that name.
    pub(crate) fn call_int_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        for step in WHOLE_STEPS {
            if let Some(result) = step(self, receiver, method_name, arguments, position)? {
                return Ok(Some(result));
            }
        }
        if let Object::BigInt(value) = receiver {
            return self.call_big_int_method(value, method_name, arguments, position);
        }
        let Object::Int(n) = receiver else {
            return Ok(None);
        };
        // These read the same whether or not the receiver fits in a machine
        // word, so the arbitrary-width implementation answers for both.
        if matches!(
            method_name,
            "zero?"
                | "positive?"
                | "negative?"
                | "even?"
                | "odd?"
                | "succ"
                | "next"
                | "pred"
                | "integer?"
        ) {
            let value = Rc::new(num_bigint::BigInt::from(*n));
            return self.call_big_int_method(&value, method_name, arguments, position);
        }
        for step in NARROW_STEPS {
            if let Some(result) = step(self, receiver, n, method_name, arguments, position)? {
                return Ok(Some(result));
            }
        }
        Ok(None)
    }
}
