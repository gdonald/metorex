// Reading a number off the front of a string.

use super::*;

impl VirtualMachine {
    /// The number a slice argument names, which Ruby reads through `to_int`
    /// and refuses when the object names none.
    pub(crate) fn coerce_slice_number(
        &mut self,
        given: &Object,
        method_name: &str,
        position: Position,
    ) -> Result<i64, MetorexError> {
        match given {
            Object::Int(number) => Ok(*number),
            Object::Float(number) => Ok(*number as i64),
            // A number past what a machine word holds names no place in a
            // string, which Ruby reports as a range rather than a type.
            Object::BigInt(_) => Err(crate::vm::errors::simple_exception(
                "RangeError",
                "bignum too big to convert into 'long'",
                position,
            )),
            other if self.answers_to(other, "to_int", position)? => {
                match self.send_to_object(other.clone(), "to_int", vec![], position)? {
                    Object::Int(number) => Ok(number),
                    Object::Float(number) => Ok(number as i64),
                    _ => Err(method_argument_type_error(
                        method_name,
                        "Integer",
                        given,
                        position,
                    )),
                }
            }
            other => Err(method_argument_type_error(
                method_name,
                "Integer",
                other,
                position,
            )),
        }
    }
}

/// The Float the leading characters of a string spell, which is 0.0 when they
/// spell none. Underscores separate digits, and a trailing `e`, `.`, or sign
/// that names no digits is left off rather than refused.
pub(crate) fn leading_float(text: &str) -> f64 {
    let letters: Vec<char> = text.trim_start().chars().collect();
    let mut taken = String::new();
    let mut index = 0;
    if matches!(letters.first(), Some('+') | Some('-')) {
        taken.push(letters[0]);
        index = 1;
    }
    // An underscore separates digits, so one that does not sit between two of
    // them ends the number.
    let mut digits = 0;
    while index < letters.len() {
        if letters[index].is_ascii_digit() {
            taken.push(letters[index]);
            digits += 1;
            index += 1;
            continue;
        }
        if letters[index] == '_'
            && digits > 0
            && letters
                .get(index + 1)
                .is_some_and(|next| next.is_ascii_digit())
        {
            index += 1;
            continue;
        }
        break;
    }
    if index < letters.len() && letters[index] == '.' {
        let mut fraction = String::new();
        let mut cursor = index + 1;
        while cursor < letters.len() {
            if letters[cursor].is_ascii_digit() {
                fraction.push(letters[cursor]);
                cursor += 1;
                continue;
            }
            if letters[cursor] == '_'
                && !fraction.is_empty()
                && letters
                    .get(cursor + 1)
                    .is_some_and(|next| next.is_ascii_digit())
            {
                cursor += 1;
                continue;
            }
            break;
        }
        if !fraction.is_empty() || digits > 0 {
            taken.push('.');
            taken.push_str(&fraction);
            digits += fraction.len();
            index = cursor;
        }
    }
    if digits == 0 {
        return 0.0;
    }
    // An exponent counts only when digits follow it.
    if index < letters.len() && (letters[index] == 'e' || letters[index] == 'E') {
        let mut cursor = index + 1;
        let mut exponent = String::new();
        if matches!(letters.get(cursor), Some('+') | Some('-')) {
            exponent.push(letters[cursor]);
            cursor += 1;
        }
        let mut exponent_digits = 0;
        while cursor < letters.len() {
            if letters[cursor].is_ascii_digit() {
                exponent.push(letters[cursor]);
                exponent_digits += 1;
                cursor += 1;
                continue;
            }
            if letters[cursor] == '_'
                && exponent_digits > 0
                && letters
                    .get(cursor + 1)
                    .is_some_and(|next| next.is_ascii_digit())
            {
                cursor += 1;
                continue;
            }
            break;
        }
        if exponent_digits > 0 {
            taken.push('e');
            taken.push_str(&exponent);
        }
    }
    taken.parse().unwrap_or(0.0)
}

/// Read a number off the front of `text` in `default_radix`, honoring a base
/// prefix (`0x`, `0b`, `0o`, `0d`) and treating an underscore between digits
/// as a separator. Anything the number does not start with answers 0.
pub(crate) fn leading_radix_number(text: &str, default_radix: u32) -> i64 {
    let trimmed = text.trim_start();
    let (sign, rest) = match trimmed.strip_prefix('-') {
        Some(rest) => (-1i64, rest),
        None => (1i64, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };
    // `hex` reads only the `0x` prefix, since `0b` and `0d` are themselves
    // hex digits. `oct` reads every base prefix.
    let (radix, rest) = match rest.get(..2).map(str::to_ascii_lowercase).as_deref() {
        Some("0x") => (16, &rest[2..]),
        Some("0b") if default_radix == 8 => (2, &rest[2..]),
        Some("0o") if default_radix == 8 => (8, &rest[2..]),
        Some("0d") if default_radix == 8 => (10, &rest[2..]),
        _ => (default_radix, rest),
    };
    let mut digits = String::new();
    let mut previous_was_digit = false;
    for character in rest.chars() {
        if character == '_' && previous_was_digit {
            previous_was_digit = false;
            continue;
        }
        if !character.is_digit(radix) {
            break;
        }
        previous_was_digit = true;
        digits.push(character);
    }
    match i64::from_str_radix(&digits, radix) {
        Ok(value) => sign * value,
        Err(_) => 0,
    }
}

/// The number a string starts with, the way `String#to_i` reads one: leading
/// whitespace, one sign, a radix prefix that agrees with the base, then
/// digits with lone underscores between them. Reading stops at the first
/// character the base does not name, and a string that starts with none is
/// zero.
pub(crate) fn leading_integer(text: &str, base: u32) -> num_bigint::BigInt {
    let mut rest = text.trim_start_matches(|c: char| c.is_whitespace());
    let mut negative = false;
    if let Some(stripped) = rest.strip_prefix('+') {
        rest = stripped;
    } else if let Some(stripped) = rest.strip_prefix('-') {
        negative = true;
        rest = stripped;
    }

    let lowered = rest.to_ascii_lowercase();
    let prefix_radix = if lowered.starts_with("0x") {
        Some(16)
    } else if lowered.starts_with("0b") {
        Some(2)
    } else if lowered.starts_with("0o") {
        Some(8)
    } else if lowered.starts_with("0d") {
        Some(10)
    } else {
        None
    };

    let mut radix = base;
    match prefix_radix {
        Some(prefix) if base == 0 || base == prefix => {
            radix = prefix;
            rest = &rest[2..];
        }
        // Only a base left to the string reads a bare leading zero as octal.
        _ if base == 0 && rest.len() > 1 && rest.starts_with('0') => {
            radix = 8;
            rest = &rest[1..];
        }
        _ => {}
    }
    if radix == 0 {
        radix = 10;
    }

    let mut digits = String::with_capacity(rest.len());
    let mut previous_underscore = false;
    for character in rest.chars() {
        if character == '_' {
            // A pair of them ends the number, and so does one before any
            // digit at all.
            if digits.is_empty() || previous_underscore {
                break;
            }
            previous_underscore = true;
            continue;
        }
        if character.to_digit(radix).is_none() {
            break;
        }
        digits.push(character);
        previous_underscore = false;
    }
    let magnitude = num_bigint::BigInt::parse_bytes(digits.as_bytes(), radix).unwrap_or_default();
    if negative { -magnitude } else { magnitude }
}
