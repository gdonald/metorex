// The text encodings a directive reads and writes: base64,
// quoted printable and uuencoding.

use super::*;

/// Read base64, ignoring anything that is not one of its characters.
/// Whether text holds only what strict base64 allows, which is the alphabet,
/// its padding, and the line breaks between them.
pub(crate) fn is_strict_base64(text: &str) -> bool {
    text.bytes().all(|character| {
        character.is_ascii_alphanumeric() || matches!(character, b'+' | b'/' | b'=' | b'\n' | b'\r')
    })
}

pub(crate) fn decode_base64(text: &str) -> Vec<u8> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let mut held: u32 = 0;
    let mut bits = 0;
    for character in text.bytes() {
        if character == b'=' {
            break;
        }
        let Some(value) = ALPHABET.iter().position(|entry| *entry == character) else {
            continue;
        };
        held = (held << 6) | value as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((held >> bits) & 0xff) as u8);
        }
    }
    out
}

/// Read quoted printable: `=XX` names a byte, and `=` at the end of a line
/// joins it to the next.
pub(crate) fn decode_quoted_printable(text: &str) -> Vec<u8> {
    let characters: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut index = 0;
    while index < characters.len() {
        if characters[index] != '=' {
            out.push(characters[index] as u8);
            index += 1;
            continue;
        }
        index += 1;
        if index < characters.len() && characters[index] == '\n' {
            index += 1;
            continue;
        }
        if index + 1 < characters.len() {
            let digits: String = characters[index..index + 2].iter().collect();
            if let Ok(value) = u8::from_str_radix(&digits, 16) {
                out.push(value);
                index += 2;
                continue;
            }
        }
        out.push(b'=');
    }
    out
}

/// Read uuencoding: each line begins with the count of bytes it holds, and
/// every character after that carries six bits with 32 added to it.
pub(crate) fn decode_uu(text: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for line in text.split('\n') {
        let characters: Vec<char> = line.chars().collect();
        let Some(marker) = characters.first() else {
            continue;
        };
        let length = ((*marker as u8).wrapping_sub(32) & 0x3f) as usize;
        if length == 0 {
            continue;
        }
        let mut written = 0;
        let mut index = 1;
        while index + 1 < characters.len() && written < length {
            let mut quad = [0u32; 4];
            for (slot, entry) in quad.iter_mut().enumerate() {
                *entry = match characters.get(index + slot) {
                    Some(character) => u32::from((*character as u8).wrapping_sub(32) & 0x3f),
                    None => 0,
                };
            }
            index += 4;
            let held = (quad[0] << 18) | (quad[1] << 12) | (quad[2] << 6) | quad[3];
            for shift in [16, 8, 0] {
                if written < length {
                    out.push(((held >> shift) & 0xff) as u8);
                    written += 1;
                }
            }
        }
    }
    out
}

/// Ruby refuses a format that asks for more items than the array holds.
pub(crate) fn too_few_items(position: Position) -> MetorexError {
    crate::vm::errors::simple_exception("ArgumentError", "too few arguments", position)
}

/// Write base64, in lines of sixty characters the way `pack("m")` does.
/// Write base64. `wrapped` asks for the line breaks `m` writes every sixty
/// characters and the newline that ends it, which `m0` leaves out.
pub(crate) fn encode_base64(bytes: &[u8], width: usize) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let groups = if width == 0 { usize::MAX } else { width / 3 };
    for (count, chunk) in bytes.chunks(3).enumerate() {
        if width > 0 && count > 0 && count % groups == 0 {
            out.push('\n');
        }
        let mut held = 0u32;
        for (slot, byte) in chunk.iter().enumerate() {
            held |= u32::from(*byte) << (16 - slot * 8);
        }
        for slot in 0..4 {
            if slot <= chunk.len() {
                out.push(ALPHABET[((held >> (18 - slot * 6)) & 0x3f) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    if width > 0 && !bytes.is_empty() {
        out.push('\n');
    }
    out
}

/// How many bytes one base64 line holds. A count of three or more names it,
/// rounded down to whole groups, and zero asks for no line breaks at all.
pub(crate) fn base64_line_length(count: &Count) -> usize {
    match count {
        Count::Exactly(0) => 0,
        Count::Exactly(named) if *named >= 3 => (*named / 3) * 3,
        _ => 45,
    }
}

/// Write quoted printable, escaping what is not plain text.
pub(crate) fn encode_quoted_printable(bytes: &[u8], width: usize) -> String {
    let mut out = String::new();
    let mut standing = 0usize;
    let mut before: Option<u8> = None;
    for byte in bytes {
        let byte = *byte;
        if byte > 126 || (byte < 32 && byte != b'\n' && byte != b'\t') || byte == b'=' {
            out.push_str(&format!("={byte:02X}"));
            standing += 3;
            before = None;
        } else if byte == b'\n' {
            // A line ending on a space or a tab writes that character in a
            // soft break, since the space would otherwise be lost.
            if matches!(before, Some(b' ') | Some(b'\t')) {
                out.push_str("=\n");
            }
            out.push('\n');
            standing = 0;
            before = Some(byte);
        } else {
            out.push(char::from(byte));
            standing += 1;
            before = Some(byte);
        }
        if standing > width {
            out.push_str("=\n");
            standing = 0;
            before = Some(b'\n');
        }
    }
    if standing > 0 {
        out.push_str("=\n");
    }
    out
}

/// How wide a quoted-printable line runs. A count names it, and one under two
/// leaves the default in place.
pub(crate) fn quoted_line_length(count: &Count) -> usize {
    match count {
        Count::Exactly(named) if *named >= 2 => *named,
        _ => 72,
    }
}

/// Write uuencoding: each line begins with the count of bytes it holds, and
/// every character after that carries six bits with 32 added to it.
pub(crate) fn encode_uu(bytes: &[u8], line_length: usize) -> String {
    let mut out = String::new();
    for line in bytes.chunks(line_length) {
        out.push(uu_char(line.len() as u8));
        for chunk in line.chunks(3) {
            let mut held = 0u32;
            for (slot, byte) in chunk.iter().enumerate() {
                held |= u32::from(*byte) << (16 - slot * 8);
            }
            for slot in 0..4 {
                out.push(uu_char(((held >> (18 - slot * 6)) & 0x3f) as u8));
            }
        }
        out.push('\n');
    }
    out
}

/// The encoding a packed string carries: Unicode where a directive wrote code
/// points, ASCII where every directive wrote text that is ASCII by its nature,
/// and bytes otherwise.
pub(crate) fn pack_result_encoding(directives: &[Directive]) -> &'static str {
    if directives.iter().all(|held| held.code == 'U') {
        return "UTF-8";
    }
    if directives
        .iter()
        .all(|held| matches!(held.code, 'u' | 'm' | 'M'))
    {
        return "US-ASCII";
    }
    "ASCII-8BIT"
}

/// How many bytes one uuencoded line holds. A count names it, rounded down to
/// whole groups of three, and a count too small for one group leaves the
/// default in place.
pub(crate) fn uu_line_length(count: &Count) -> usize {
    match count {
        Count::Exactly(named) if *named >= 3 => (*named / 3) * 3,
        _ => 45,
    }
}

/// One six-bit group as the character uuencoding writes it.
pub(crate) fn uu_char(value: u8) -> char {
    if value == 0 {
        '`'
    } else {
        (value + 32) as char
    }
}

/// Text tagged US-ASCII, which is how `unpack` answers the bit and nibble
/// directives.
pub(crate) fn ascii_string(text: String) -> Object {
    Object::String(std::rc::Rc::new(crate::object::StringValue::with_encoding(
        text, "US-ASCII",
    )))
}
