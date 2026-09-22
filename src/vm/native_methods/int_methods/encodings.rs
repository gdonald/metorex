// The character a code names in an encoding.

use super::*;

/// A string of one byte, tagged with the encoding it is read in.
pub(crate) fn one_byte_string(byte: u8, named: &str) -> Object {
    let made = crate::object::StringValue::with_encoding((byte as char).to_string(), named);
    if !byte.is_ascii() {
        made.mark_bytes();
    }
    Object::String(std::rc::Rc::new(made))
}

/// Whether an encoding spells some of its characters in more than one byte.
fn spells_several_bytes(named: &str) -> bool {
    let upper = named.to_ascii_uppercase();
    upper.starts_with("UTF")
        || upper.starts_with("EUC")
        || upper.starts_with("ISO-2022")
        || upper.starts_with("CESU")
        || matches!(
            upper.as_str(),
            "SHIFT_JIS"
                | "WINDOWS-31J"
                | "MACJAPANESE"
                | "BIG5"
                | "GBK"
                | "GB18030"
                | "GB2312"
                | "CP949"
                | "EMACS-MULE"
                | "STATELESS-ISO-2022-JP"
                | "CP50221"
        )
}

/// Whether a run of bytes spells one Shift_JIS character. A byte in the lead
/// ranges is read together with the byte after it, and the rest stand alone.
fn shift_jis_ok(bytes: &[u8]) -> bool {
    match bytes {
        [one] => matches!(one, 0x00..=0x7f | 0xa1..=0xdf),
        [lead, trail] => {
            matches!(lead, 0x81..=0x9f | 0xe0..=0xfc) && matches!(trail, 0x40..=0x7e | 0x80..=0xfc)
        }
        _ => false,
    }
}

/// The character a code names in an encoding, or None where the encoding has
/// no character of that number.
pub(crate) fn character_in_encoding(code: i64, named: &str) -> Option<Object> {
    let upper = named.to_ascii_uppercase();
    if upper == "UTF-8" {
        let letter = u32::try_from(code).ok().and_then(char::from_u32)?;
        return Some(Object::String(std::rc::Rc::new(
            crate::object::StringValue::with_encoding(letter.to_string(), named),
        )));
    }
    // CESU-8 spells a character outside the first plane as the two halves a
    // surrogate pair names, each written the way UTF-8 writes a character.
    if upper == "CESU-8" {
        let point = u32::try_from(code).ok()?;
        char::from_u32(point)?;
        let mut bytes = Vec::with_capacity(6);
        if point >= 0x10000 {
            let above = point - 0x10000;
            for half in [0xd800 + (above >> 10), 0xdc00 + (above & 0x3ff)] {
                bytes.push(0xe0 | (half >> 12) as u8);
                bytes.push(0x80 | ((half >> 6) & 0x3f) as u8);
                bytes.push(0x80 | (half & 0x3f) as u8);
            }
        } else {
            let mut room = [0u8; 4];
            let letter = char::from_u32(point)?;
            bytes.extend_from_slice(letter.encode_utf8(&mut room).as_bytes());
        }
        let made = crate::object::StringValue::from_bytes(
            crate::vm::native_methods::string_methods::bytes_as_text(&bytes),
        );
        made.set_encoding(named);
        return Some(Object::String(std::rc::Rc::new(made)));
    }
    if upper == "US-ASCII" {
        if !(0..=0x7f).contains(&code) {
            return None;
        }
        return Some(one_byte_string(code as u8, named));
    }
    if upper == "ASCII-8BIT" || upper == "BINARY" {
        if !(0..=0xff).contains(&code) {
            return None;
        }
        return Some(one_byte_string(code as u8, "ASCII-8BIT"));
    }
    if !spells_several_bytes(named) {
        // One byte to a character, so the number names that byte.
        if !(0..=0xff).contains(&code) {
            return None;
        }
        return Some(one_byte_string(code as u8, named));
    }
    // A UTF encoding names a code point, and the halves a surrogate pair is
    // written with name no character of their own.
    if upper.starts_with("UTF") {
        let point = u32::try_from(code).ok()?;
        char::from_u32(point)?;
    }
    // An encoding that spells a character in bytes of its own reads the
    // number as those bytes.
    let mut bytes: Vec<u8> = u32::try_from(code).ok()?.to_be_bytes().to_vec();
    while bytes.len() > 1 && bytes[0] == 0 {
        bytes.remove(0);
    }
    if upper == "EUC-JP" {
        let (_, width) = crate::vm::native_methods::euc_jp_table::euc_jp_character(&bytes)?;
        if width != bytes.len() {
            return None;
        }
    }
    if matches!(upper.as_str(), "SHIFT_JIS" | "WINDOWS-31J" | "MACJAPANESE")
        && !shift_jis_ok(&bytes)
    {
        return None;
    }
    let made = crate::object::StringValue::from_bytes(
        crate::vm::native_methods::string_methods::bytes_as_text(&bytes),
    );
    made.set_encoding(named);
    Some(Object::String(std::rc::Rc::new(made)))
}
