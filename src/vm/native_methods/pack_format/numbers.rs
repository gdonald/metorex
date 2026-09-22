// Reading and writing a number as a run of bytes.

use super::*;

/// Read `width` bytes as one number, in the order the directive asks for.
pub(crate) fn read_integer(bytes: &[u8], code: char, order: Option<Order>) -> i128 {
    let mut value: u128 = 0;
    let big = match order {
        Some(Order::Big) => true,
        Some(Order::Little) => false,
        None => is_big_endian(code),
    };
    if big {
        for byte in bytes {
            value = (value << 8) | u128::from(*byte);
        }
    } else {
        for byte in bytes.iter().rev() {
            value = (value << 8) | u128::from(*byte);
        }
    }
    let width = bytes.len();
    if is_signed(code) && width > 0 {
        let sign_bit = 1u128 << (width * 8 - 1);
        if value & sign_bit != 0 {
            return (value as i128) - (1i128 << (width * 8));
        }
    }
    value as i128
}

/// Write one number as `width` bytes, in the order the directive asks for.
pub(crate) fn write_integer(
    value: i128,
    width: usize,
    code: char,
    order: Option<Order>,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(width);
    let unsigned = value as u128;
    for step in 0..width {
        bytes.push(((unsigned >> (step * 8)) & 0xff) as u8);
    }
    let big = match order {
        Some(Order::Big) => true,
        Some(Order::Little) => false,
        None => is_big_endian(code),
    };
    if big {
        bytes.reverse();
    }
    bytes
}

/// A packed result as a String. Every byte stands for one character, which is
/// how metorex holds a run of bytes, and the result carries no character
/// meaning, which is what ASCII-8BIT says.
pub(crate) fn bytes_to_string(bytes: &[u8]) -> Object {
    Object::String(std::rc::Rc::new(crate::object::StringValue::from_bytes(
        bytes.iter().map(|byte| *byte as char).collect::<String>(),
    )))
}

/// The bytes a String stands for, one per character.
pub(crate) fn string_to_bytes(text: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    for character in text.chars() {
        if (character as u32) < 256 {
            bytes.push(character as u8);
        } else {
            let mut buffer = [0u8; 4];
            bytes.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
        }
    }
    bytes
}
