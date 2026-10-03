// Encodings that spell a character in more than one byte.

/// Whether a string holds one character per byte rather than text: one
/// built from bytes, one tagged binary, or one in an encoding that spells a
/// character in more than one byte.
pub(crate) fn bytes_are_characters(string_value: &crate::object::StringValue) -> bool {
    string_value.holds_bytes()
        || match string_value.encoding_name().as_str() {
            "ASCII-8BIT" | "BINARY" => true,
            named => wide_encoding(named).is_some(),
        }
}

/// The bytes a string stands for. A string tagged binary holds one character
/// per byte, so its characters are its bytes rather than their UTF-8 form.
pub(crate) fn binary_bytes(string_value: &crate::object::StringValue) -> Vec<u8> {
    if bytes_are_characters(string_value) {
        return crate::vm::native_methods::pack_format::string_to_bytes(&string_value.as_str());
    }
    string_value.as_str().as_bytes().to_vec()
}

/// How an encoding lays a character out: the width of a code unit, whether
/// the high byte comes first, and whether a byte order mark opens the text.
#[derive(Clone, Copy)]
pub(crate) struct WideShape {
    unit: usize,
    big_endian: bool,
    marked: bool,
}

impl WideShape {
    /// How many bytes one code unit takes.
    pub(crate) fn unit(self) -> usize {
        self.unit
    }
}

/// The shape of an encoding that spells a character in more than one byte,
/// or None for one that spells it in a single byte.
pub(crate) fn wide_encoding(named: &str) -> Option<WideShape> {
    let shape = match named {
        "UTF-16BE" => (2, true, false),
        "UTF-16LE" => (2, false, false),
        "UTF-16" => (2, true, true),
        "UTF-32BE" => (4, true, false),
        "UTF-32LE" => (4, false, false),
        "UTF-32" => (4, true, true),
        _ => return None,
    };
    Some(WideShape {
        unit: shape.0,
        big_endian: shape.1,
        marked: shape.2,
    })
}

/// Text written out in a wide encoding, one code unit at a time, with the
/// surrogate pair a character above the basic plane calls for.
pub(crate) fn wide_bytes(text: &str, shape: WideShape) -> Vec<u8> {
    let mut units: Vec<u32> = Vec::new();
    if shape.marked {
        units.push(0xfeff);
    }
    for letter in text.chars() {
        let point = letter as u32;
        if shape.unit == 2 && point > 0xffff {
            let carried = point - 0x10000;
            units.push(0xd800 + (carried >> 10));
            units.push(0xdc00 + (carried & 0x3ff));
        } else {
            units.push(point);
        }
    }
    let mut bytes = Vec::with_capacity(units.len() * shape.unit);
    for unit in units {
        let written = unit.to_be_bytes();
        let taken = &written[4 - shape.unit..];
        if shape.big_endian {
            bytes.extend_from_slice(taken);
        } else {
            bytes.extend(taken.iter().rev());
        }
    }
    bytes
}

/// The text a run of bytes spells in a wide encoding, reading the byte order
/// mark when the encoding opens with one.
pub(crate) fn wide_text(bytes: &[u8], shape: WideShape) -> String {
    let mut big_endian = shape.big_endian;
    let mut at = 0;
    let mut units: Vec<u32> = Vec::new();
    while at + shape.unit <= bytes.len() {
        let taken = &bytes[at..at + shape.unit];
        let mut value = 0u32;
        if big_endian {
            for byte in taken {
                value = (value << 8) | *byte as u32;
            }
        } else {
            for byte in taken.iter().rev() {
                value = (value << 8) | *byte as u32;
            }
        }
        at += shape.unit;
        if units.is_empty() && shape.marked && value == 0xfeff {
            continue;
        }
        if units.is_empty() && shape.marked && value == 0xfffe0000 {
            big_endian = !big_endian;
            continue;
        }
        units.push(value);
    }
    let mut text = String::new();
    let mut index = 0;
    while index < units.len() {
        let unit = units[index];
        index += 1;
        if shape.unit == 2 && (0xd800..0xdc00).contains(&unit) && index < units.len() {
            let low = units[index];
            index += 1;
            let point = 0x10000 + ((unit - 0xd800) << 10) + (low - 0xdc00);
            if let Some(letter) = char::from_u32(point) {
                text.push(letter);
            }
            continue;
        }
        if let Some(letter) = char::from_u32(unit) {
            text.push(letter);
        }
    }
    text
}

/// A run of bytes held as text, one character to a byte, which is how a
/// string tagged with a wide encoding carries what it spells.
pub(crate) fn bytes_as_text(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| *byte as char).collect()
}
