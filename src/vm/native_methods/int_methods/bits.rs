// The bits of an integer read as a two's-complement run.

use super::*;

impl VirtualMachine {
    /// The bits of an integer read as a two's-complement run.
    pub(crate) fn call_int_bit_method(
        &mut self,
        receiver: &Object,
        method_name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Object>, MetorexError> {
        // Integer#[] reads the bits of the two's-complement representation:
        // one bit by position, or a run of them named by a length or a range.
        if method_name == "[]" && matches!(receiver, Object::Int(_) | Object::BigInt(_)) {
            // A bit position of a machine-word Integer is taken through
            // `to_int`, which names nil as any other value without one.
            if let (Object::Int(_), [Object::Nil]) = (receiver, arguments) {
                return Err(crate::vm::errors::simple_exception(
                    "TypeError",
                    "no implicit conversion of nil into Integer",
                    position,
                ));
            }
            return self
                .integer_bits_at(
                    &receiver.as_big_integer().expect("integer-kinded"),
                    arguments,
                    position,
                )
                .map(Some);
        }
        // Integer#allbits? / #anybits? / #nobits? — how the bits of a mask sit
        // in the receiver. Both sides are compared as arbitrary-width integers,
        // so a bignum and a negative number answer the same way a fixnum does.
        if matches!(method_name, "allbits?" | "anybits?" | "nobits?")
            && matches!(receiver, Object::Int(_) | Object::BigInt(_))
        {
            if arguments.len() != 1 {
                return Err(method_argument_error(
                    method_name,
                    1,
                    arguments.len(),
                    position,
                ));
            }
            let mask = self.coerce_integer_argument(&arguments[0], position)?;
            let masked = receiver.as_big_integer().expect("integer-kinded") & &mask;
            let zero = num_bigint::BigInt::from(0);
            return Ok(Some(Object::Bool(match method_name {
                "allbits?" => masked == mask,
                "anybits?" => masked != zero,
                _ => masked == zero,
            })));
        }
        // An Integer is the whole of its own fraction, so it is its own
        // numerator over a denominator of one, and the Rational those name.
        if matches!(
            method_name,
            "numerator" | "denominator" | "to_r" | "rationalize"
        ) && matches!(receiver, Object::Int(_) | Object::BigInt(_))
        {
            // `rationalize` takes an optional precision, which cannot move a
            // number that is already exact. The others take nothing at all.
            let allowed = usize::from(method_name == "rationalize");
            if arguments.len() > allowed {
                return Err(method_argument_error(
                    method_name,
                    allowed,
                    arguments.len(),
                    position,
                ));
            }
            let value = receiver.as_big_integer().expect("integer-kinded");
            return match method_name {
                "numerator" => Ok(Some(Object::integer(value))),
                "denominator" => Ok(Some(Object::Int(1))),
                _ => self
                    .make_rational(value, num_bigint::BigInt::from(1), position)
                    .map(Some),
            };
        }
        Ok(None)
    }
}
