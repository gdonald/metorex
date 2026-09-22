// Characters written out as an escape.

use super::*;

/// The escape a character is written as where the encoding an answer is
/// written in has no room for it.
pub(crate) fn escaped_point(character: char) -> String {
    let point = character as u32;
    if point > 0xffff {
        format!("\\u{{{point:X}}}")
    } else {
        format!("\\u{point:04X}")
    }
}

/// Text with every character an ASCII answer has no room for written as an
/// escape. Ruby writes a string this way where it would otherwise have to
/// refuse text in an encoding the answer is not being written in.
pub(crate) fn escaped_text(string_value: &crate::object::StringValue) -> Object {
    // An encoding that spells a character in more than one byte is carried as
    // the bytes themselves, so the characters are read back out of them.
    let held = match wide_encoding(&string_value.encoding_name()) {
        Some(shape) => wide_text(&binary_bytes(string_value), shape),
        None => string_value.to_text(),
    };
    let mut out = String::with_capacity(held.len());
    for character in held.chars() {
        match character {
            character if character.is_ascii() && !character.is_control() => out.push(character),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            character if character.is_control() => {
                out.push_str(&format!("\\x{:02X}", character as u32))
            }
            character => out.push_str(&escaped_point(character)),
        }
    }
    Object::String(Rc::new(crate::object::StringValue::with_encoding(
        out, "US-ASCII",
    )))
}

/// The letter Ruby writes a character as after a backslash, for the few that
/// have a name of their own.
pub(crate) fn named_escape(character: char) -> Option<char> {
    Some(match character {
        '\u{7}' => 'a',
        '\u{8}' => 'b',
        '\t' => 't',
        '\n' => 'n',
        '\u{b}' => 'v',
        '\u{c}' => 'f',
        '\r' => 'r',
        '\u{1b}' => 'e',
        _ => return None,
    })
}
