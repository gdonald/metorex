// The whole part of a square root.

use super::*;

/// The whole part of a square root, found by Newton's method so a number wider
/// than a Float still answers exactly.
pub(crate) fn integer_square_root(value: num_bigint::BigInt) -> num_bigint::BigInt {
    let zero = num_bigint::BigInt::from(0);
    let one = num_bigint::BigInt::from(1);
    if value <= one {
        return value;
    }
    let mut guess = value.clone();
    let mut next = (&guess + &value / &guess) / 2;
    while next < guess {
        guess = next;
        next = (&guess + &value / &guess) / 2;
    }
    let _ = zero;
    guess
}

/// The digits of a number as a String, which Ruby hands back in US-ASCII
/// whatever the default encodings are set to.
pub(crate) fn ascii_string(written: String) -> Object {
    let made = crate::object::StringValue::new(written);
    made.set_encoding("US-ASCII");
    Object::String(std::rc::Rc::new(made))
}
