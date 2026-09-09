//! The directive machine `Array#pack` writes with and `String#unpack` reads
//! back. Both walk the same format string, so a directive means the same
//! thing whichever way the bytes are travelling.

use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;

/// How many items a directive applies to.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Count {
    /// No count was written, so the directive applies once.
    One,
    /// A number was written.
    Exactly(usize),
    /// `*` was written: every item that remains.
    Rest,
}

/// One directive with the count that follows it.
pub(crate) struct Directive {
    pub(crate) code: char,
    pub(crate) count: Count,
    /// The byte order a `<` or `>` modifier asked for, where the directive
    /// admits one. None means the directive's own order.
    pub(crate) order: Option<Order>,
    /// Whether `!` or `_` asked for the platform's own width.
    pub(crate) native: bool,
}

/// Which end of a number its first byte holds.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Order {
    Little,
    Big,
}

/// Split a format string into its directives. Whitespace between them is
/// ignored, a `#` runs a comment to the end of the line, and a directive Ruby
/// does not know is refused by name.
pub(crate) fn parse_format(
    format: &str,
    verb: &str,
    position: Position,
) -> Result<Vec<Directive>, MetorexError> {
    let characters: Vec<char> = format.chars().collect();
    let mut directives = Vec::new();
    let mut index = 0;
    while index < characters.len() {
        let code = characters[index];
        if code.is_whitespace() {
            index += 1;
            continue;
        }
        if code == '#' {
            while index < characters.len() && characters[index] != '\n' {
                index += 1;
            }
            continue;
        }
        if !is_known_directive(code) {
            return Err(unknown_directive(code, format, verb, position));
        }
        index += 1;
        // `!` and `_` ask for the platform's own width, and `<` and `>` name
        // the byte order. Only the integer directives admit either, and both
        // may be written, in either order.
        let sized_integer = matches!(
            code,
            's' | 'S' | 'i' | 'I' | 'l' | 'L' | 'q' | 'Q' | 'j' | 'J'
        );
        let mut order = None;
        let mut native = false;
        while let Some(modifier) = characters.get(index) {
            match modifier {
                '!' | '_' => {
                    if !sized_integer {
                        return Err(unknown_directive(*modifier, format, verb, position));
                    }
                    native = true;
                }
                '<' | '>' => {
                    if !sized_integer {
                        return Err(unknown_directive(*modifier, format, verb, position));
                    }
                    order = Some(if *modifier == '<' {
                        Order::Little
                    } else {
                        Order::Big
                    });
                }
                _ => break,
            }
            index += 1;
        }
        let count = if index < characters.len() && characters[index] == '*' {
            index += 1;
            Count::Rest
        } else if index < characters.len() && characters[index].is_ascii_digit() {
            let start = index;
            while index < characters.len() && characters[index].is_ascii_digit() {
                index += 1;
            }
            let digits: String = characters[start..index].iter().collect();
            match digits.parse::<usize>() {
                Ok(number) => Count::Exactly(number),
                Err(_) => {
                    let message = format!("{verb} length too big");
                    return Err(crate::vm::errors::simple_exception(
                        "RangeError",
                        &message,
                        position,
                    ));
                }
            }
        } else {
            Count::One
        };
        directives.push(Directive {
            code,
            count,
            order,
            native,
        });
    }
    Ok(directives)
}

/// Every directive Ruby reads. A count or a width modifier is not one.
fn is_known_directive(code: char) -> bool {
    matches!(
        code,
        'a' | 'A'
            | 'Z'
            | 'b'
            | 'B'
            | 'h'
            | 'H'
            | 'c'
            | 'C'
            | 's'
            | 'S'
            | 'l'
            | 'L'
            | 'q'
            | 'Q'
            | 'j'
            | 'J'
            | 'i'
            | 'I'
            | 'n'
            | 'N'
            | 'v'
            | 'V'
            | 'U'
            | 'w'
            | 'm'
            | 'M'
            | 'u'
            | 'f'
            | 'F'
            | 'd'
            | 'D'
            | 'e'
            | 'E'
            | 'g'
            | 'G'
            | 'x'
            | 'X'
            | '@'
            | 'p'
            | 'P'
    )
}

fn unknown_directive(code: char, format: &str, verb: &str, position: Position) -> MetorexError {
    let message = format!("unknown {verb} directive '{code}' in '{format}'");
    crate::vm::errors::simple_exception("ArgumentError", &message, position)
}

/// How wide one item of an integer directive is, in bytes. `native` asks for
/// the platform's own width, which widens a long to eight bytes here and
/// leaves an int at four.
pub(crate) fn integer_width(code: char, native: bool) -> Option<usize> {
    match code {
        'c' | 'C' => Some(1),
        's' | 'S' | 'v' | 'n' => Some(2),
        'l' | 'L' if native => Some(std::mem::size_of::<std::os::raw::c_long>()),
        'l' | 'L' | 'V' | 'N' | 'i' | 'I' => Some(4),
        'q' | 'Q' | 'j' | 'J' => Some(8),
        _ => None,
    }
}

/// Whether a directive reads its bytes with the most significant one first.
pub(crate) fn is_big_endian(code: char) -> bool {
    matches!(code, 'n' | 'N')
}

/// Whether a directive answers a signed number.
pub(crate) fn is_signed(code: char) -> bool {
    matches!(code, 'c' | 's' | 'l' | 'q' | 'j' | 'i')
}

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
    Object::binary_string(bytes.iter().map(|byte| *byte as char).collect::<String>())
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

/// Read base64, ignoring anything that is not one of its characters.
fn decode_base64(text: &str) -> Vec<u8> {
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
fn decode_quoted_printable(text: &str) -> Vec<u8> {
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
fn decode_uu(text: &str) -> Vec<u8> {
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
fn too_few_items(position: Position) -> MetorexError {
    crate::vm::errors::simple_exception("ArgumentError", "too few arguments", position)
}

/// Write base64, in lines of sixty characters the way `pack("m")` does.
/// Write base64. `wrapped` asks for the line breaks `m` writes every sixty
/// characters and the newline that ends it, which `m0` leaves out.
fn encode_base64(bytes: &[u8], wrapped: bool) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for (count, chunk) in bytes.chunks(3).enumerate() {
        if wrapped && count > 0 && count % 20 == 0 {
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
    if wrapped {
        out.push('\n');
    }
    out
}

/// Write quoted printable, escaping what is not plain text.
fn encode_quoted_printable(bytes: &[u8]) -> String {
    let mut out = String::new();
    for byte in bytes {
        if byte.is_ascii_graphic() && *byte != b'=' || *byte == b' ' || *byte == b'\t' {
            out.push(*byte as char);
        } else if *byte == b'\n' {
            out.push('\n');
        } else {
            out.push_str(&format!("={:02X}", byte));
        }
    }
    out.push_str("=\n");
    out
}

/// Write uuencoding: each line begins with the count of bytes it holds, and
/// every character after that carries six bits with 32 added to it.
fn encode_uu(bytes: &[u8]) -> String {
    let mut out = String::new();
    for line in bytes.chunks(45) {
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

/// One six-bit group as the character uuencoding writes it.
fn uu_char(value: u8) -> char {
    if value == 0 {
        '`'
    } else {
        (value + 32) as char
    }
}

use crate::vm::VirtualMachine;

impl VirtualMachine {
    /// `String#unpack` — read the bytes back as the directives describe them.
    pub(crate) fn string_unpack(
        &mut self,
        text: &str,
        format: &str,
        position: Position,
    ) -> Result<Vec<Object>, MetorexError> {
        let bytes = string_to_bytes(text);
        let directives = parse_format(format, "unpack", position)?;
        let mut out: Vec<Object> = Vec::new();
        let mut at = 0usize;
        for directive in &directives {
            let remaining = bytes.len().saturating_sub(at);
            match directive.code {
                // A run of bytes, read as text. 'a' keeps what is there, 'A'
                // drops the trailing spaces and nulls, and 'Z' stops at the
                // first null.
                'a' | 'A' | 'Z' => {
                    // `Z*` reads up to the first null and steps past it, so
                    // the next directive starts on what follows.
                    if directive.code == 'Z' && directive.count == Count::Rest {
                        let end = bytes[at..]
                            .iter()
                            .position(|byte| *byte == 0)
                            .map_or(bytes.len(), |offset| at + offset);
                        let held: String =
                            bytes[at..end].iter().map(|byte| *byte as char).collect();
                        at = (end + 1).min(bytes.len());
                        out.push(Object::string(held));
                        continue;
                    }
                    let taken = match directive.count {
                        Count::Rest => remaining,
                        Count::One => remaining.min(1),
                        Count::Exactly(n) => remaining.min(n),
                    };
                    let slice = &bytes[at..at + taken];
                    at += taken;
                    let held: String = slice.iter().map(|byte| *byte as char).collect();
                    let settled = match directive.code {
                        'A' => held.trim_end_matches(['\0', ' ']).to_string(),
                        'Z' => match held.find('\0') {
                            Some(end) => held[..end].to_string(),
                            None => held,
                        },
                        _ => held,
                    };
                    out.push(Object::string(settled));
                }
                // Bits, most or least significant first.
                'b' | 'B' => {
                    let wanted = match directive.count {
                        Count::Rest => remaining * 8,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    let available = remaining * 8;
                    let taken = wanted.min(available);
                    let mut written = String::with_capacity(taken);
                    for step in 0..taken {
                        let byte = bytes[at + step / 8];
                        let bit = if directive.code == 'B' {
                            (byte >> (7 - step % 8)) & 1
                        } else {
                            (byte >> (step % 8)) & 1
                        };
                        written.push(if bit == 1 { '1' } else { '0' });
                    }
                    at += taken.div_ceil(8);
                    out.push(Object::string(written));
                }
                // Nibbles, high or low half first.
                'h' | 'H' => {
                    let wanted = match directive.count {
                        Count::Rest => remaining * 2,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    let available = remaining * 2;
                    let taken = wanted.min(available);
                    let mut written = String::with_capacity(taken);
                    for step in 0..taken {
                        let byte = bytes[at + step / 2];
                        let nibble = if directive.code == 'H' {
                            if step % 2 == 0 { byte >> 4 } else { byte & 0xf }
                        } else if step % 2 == 0 {
                            byte & 0xf
                        } else {
                            byte >> 4
                        };
                        written.push(char::from_digit(u32::from(nibble), 16).unwrap_or('0'));
                    }
                    at += taken.div_ceil(2);
                    out.push(Object::string(written));
                }
                // A UTF-8 character read back as its code point.
                'U' => {
                    let held: String = bytes[at..].iter().map(|byte| *byte as char).collect();
                    let source = String::from_utf8_lossy(&bytes[at..]).to_string();
                    let _ = held;
                    let wanted = match directive.count {
                        Count::Rest => usize::MAX,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    for (read, character) in source.chars().enumerate() {
                        if read >= wanted {
                            break;
                        }
                        out.push(Object::Int(character as i64));
                        at += character.len_utf8();
                    }
                }
                // Skip forward, back, or to a fixed place.
                // `x` steps forward, and `x*` runs to the end.
                'x' => {
                    let step = match directive.count {
                        Count::Rest => remaining,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    if at + step > bytes.len() {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            "x outside of string",
                            position,
                        ));
                    }
                    at += step;
                }
                // `X` steps back, and `X*` steps back by however many bytes
                // are left, which there has to be room for.
                'X' => {
                    let step = match directive.count {
                        Count::Rest => remaining,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    if step > at {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            "X outside of string",
                            position,
                        ));
                    }
                    at -= step;
                }
                // `@` moves to a fixed place, and `@*` names no place, so it
                // leaves the index where it stands.
                '@' => {
                    if let Count::Exactly(place) = directive.count {
                        if place > bytes.len() {
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                "@ outside of string",
                                position,
                            ));
                        }
                        at = place;
                    } else if directive.count == Count::One {
                        at = 0;
                    }
                }
                // The integer directives, which differ only in width, order,
                // and whether they are signed.
                code if integer_width(code, directive.native).is_some() => {
                    let width =
                        integer_width(code, directive.native).expect("the guard read a width");
                    let wanted = match directive.count {
                        Count::Rest => remaining / width,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    for _ in 0..wanted {
                        if at + width > bytes.len() {
                            // Ruby fills the count a format asked for with
                            // nil once the bytes run out, and answers nothing
                            // at all for `*`.
                            if directive.count != Count::Rest {
                                out.push(Object::Nil);
                            }
                            at = bytes.len();
                            continue;
                        }
                        let value = read_integer(&bytes[at..at + width], code, directive.order);
                        at += width;
                        out.push(Object::integer(num_bigint::BigInt::from(value)));
                    }
                }
                // The floating point directives. 'e' and 'g' name the two
                // byte orders outright, and 'f'/'d' follow the platform's.
                'f' | 'F' | 'e' | 'd' | 'D' | 'E' | 'g' | 'G' => {
                    let width = if matches!(directive.code, 'f' | 'F' | 'e' | 'g') {
                        4
                    } else {
                        8
                    };
                    let big = matches!(directive.code, 'g' | 'G');
                    let wanted = match directive.count {
                        Count::Rest => remaining / width,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    for _ in 0..wanted {
                        if at + width > bytes.len() {
                            if directive.count != Count::Rest {
                                out.push(Object::Nil);
                            }
                            at = bytes.len();
                            continue;
                        }
                        let mut slice = bytes[at..at + width].to_vec();
                        if big {
                            slice.reverse();
                        }
                        at += width;
                        let value = if width == 4 {
                            f64::from(f32::from_le_bytes(
                                slice.try_into().expect("four bytes were taken"),
                            ))
                        } else {
                            f64::from_le_bytes(slice.try_into().expect("eight bytes were taken"))
                        };
                        out.push(Object::Float(value));
                    }
                }
                // A BER compressed integer: seven bits per byte, most
                // significant first, with the top bit set on every byte but
                // the last.
                'w' => {
                    let wanted = match directive.count {
                        Count::Rest => usize::MAX,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    let mut read = 0;
                    while read < wanted && at < bytes.len() {
                        let mut value = num_bigint::BigInt::from(0);
                        while let Some(byte) = bytes.get(at) {
                            at += 1;
                            value = value * 128 + num_bigint::BigInt::from(byte & 0x7f);
                            if byte & 0x80 == 0 {
                                break;
                            }
                        }
                        out.push(Object::integer(value));
                        read += 1;
                    }
                }
                // Base64, in the strict form 'm0' asks for and the lenient
                // one every other count admits.
                'm' => {
                    let held: String = bytes[at..].iter().map(|byte| *byte as char).collect();
                    at = bytes.len();
                    out.push(bytes_to_string(&decode_base64(&held)));
                }
                // Quoted printable.
                'M' => {
                    let held: String = bytes[at..].iter().map(|byte| *byte as char).collect();
                    at = bytes.len();
                    out.push(bytes_to_string(&decode_quoted_printable(&held)));
                }
                // Uuencoding, as `uuencode` writes it.
                'u' => {
                    let held: String = bytes[at..].iter().map(|byte| *byte as char).collect();
                    at = bytes.len();
                    out.push(bytes_to_string(&decode_uu(&held)));
                }
                other => {
                    return Err(unknown_directive(other, format, "unpack", position));
                }
            }
        }
        Ok(out)
    }
}

impl VirtualMachine {
    /// `Array#pack` — write the items out as the directives describe them.
    pub(crate) fn array_pack(
        &mut self,
        items: &[Object],
        format: &str,
        position: Position,
    ) -> Result<Object, MetorexError> {
        let directives = parse_format(format, "pack", position)?;
        // Nothing written is nothing but ASCII, which is what Ruby tags an
        // empty result as.
        if directives.is_empty() {
            return Ok(Object::String(std::rc::Rc::new(
                crate::object::StringValue::with_encoding(String::new(), "US-ASCII"),
            )));
        }
        let mut out: Vec<u8> = Vec::new();
        let mut taken = 0usize;
        for directive in &directives {
            let left = items.len().saturating_sub(taken);
            match directive.code {
                // A run of bytes written from one string. 'a' pads with
                // nulls, 'A' with spaces, and 'Z' adds a null of its own.
                'a' | 'A' | 'Z' => {
                    let held = match items.get(taken) {
                        Some(Object::String(text)) => text.as_str().to_string(),
                        // nil packs as nothing, which the padding then fills.
                        Some(Object::Nil) => String::new(),
                        Some(other) => self.coerce_pack_string(other, position)?,
                        None => return Err(too_few_items(position)),
                    };
                    taken += 1;
                    let source = string_to_bytes(&held);
                    let width = match directive.count {
                        Count::Rest => source.len() + usize::from(directive.code == 'Z'),
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    let filler = if directive.code == 'A' { b' ' } else { 0 };
                    for step in 0..width {
                        out.push(source.get(step).copied().unwrap_or(filler));
                    }
                }
                // Bits and nibbles, written from a string of '0'/'1' or of
                // hexadecimal digits.
                'b' | 'B' | 'h' | 'H' => {
                    let held = match items.get(taken) {
                        Some(Object::String(text)) => text.as_str().to_string(),
                        Some(Object::Nil) => String::new(),
                        Some(other) => self.coerce_pack_string(other, position)?,
                        None => return Err(too_few_items(position)),
                    };
                    taken += 1;
                    let digits: Vec<char> = held.chars().collect();
                    let wanted = match directive.count {
                        Count::Rest => digits.len(),
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    let bits_each = if matches!(directive.code, 'b' | 'B') {
                        1
                    } else {
                        4
                    };
                    let per_byte = 8 / bits_each;
                    let mut current = 0u8;
                    for step in 0..wanted {
                        let value = match digits.get(step) {
                            // A bit is the low bit of the character that
                            // names it, so '0' and '1' read as themselves and
                            // any other character contributes its own.
                            Some(digit) if bits_each == 1 => (*digit as u8) & 1,
                            // A letter counts nine past its own low half, so
                            // 'a' through 'f' land on ten through fifteen and
                            // every other letter follows the same step. Any
                            // other character contributes its low half alone.
                            Some(digit) => {
                                let low = (*digit as u8) & 0xf;
                                if digit.is_ascii_alphabetic() {
                                    (low + 9) & 0xf
                                } else {
                                    low
                                }
                            }
                            None => 0,
                        };
                        let slot = step % per_byte;
                        let shift = match directive.code {
                            'B' => 7 - slot,
                            'b' => slot,
                            'H' => 4 - slot * 4,
                            _ => slot * 4,
                        };
                        current |= value << shift;
                        if slot == per_byte - 1 {
                            out.push(current);
                            current = 0;
                        }
                    }
                    if wanted % per_byte != 0 {
                        out.push(current);
                    }
                }
                // A code point written as UTF-8.
                'U' => {
                    let wanted = match directive.count {
                        Count::Rest => left,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    for _ in 0..wanted {
                        let point = match items.get(taken) {
                            Some(Object::Int(number)) => *number as u32,
                            Some(_) => 0,
                            None => return Err(too_few_items(position)),
                        };
                        taken += 1;
                        let character = char::from_u32(point).unwrap_or('\u{fffd}');
                        let mut buffer = [0u8; 4];
                        out.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
                    }
                }
                // A BER compressed integer.
                'w' => {
                    let wanted = match directive.count {
                        Count::Rest => left,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    for _ in 0..wanted {
                        let Some(item) = items.get(taken) else {
                            return Err(too_few_items(position));
                        };
                        let mut value = self.coerce_pack_integer(item, position)?;
                        // A compressed integer has no sign to carry, so a
                        // negative one cannot be written at all.
                        if value < 0 {
                            return Err(crate::vm::errors::simple_exception(
                                "ArgumentError",
                                "can't compress negative numbers",
                                position,
                            ));
                        }
                        taken += 1;
                        let mut written = vec![(value & 0x7f) as u8];
                        value >>= 7;
                        while value > 0 {
                            written.push(((value & 0x7f) as u8) | 0x80);
                            value >>= 7;
                        }
                        written.reverse();
                        out.extend_from_slice(&written);
                    }
                }
                // Padding and placement, which take no item.
                'x' => {
                    let step = match directive.count {
                        Count::Rest => 0,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    out.extend(std::iter::repeat_n(0u8, step));
                }
                'X' => {
                    let step = match directive.count {
                        Count::Rest => 0,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    if step > out.len() {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            "X outside of string",
                            position,
                        ));
                    }
                    out.truncate(out.len() - step);
                }
                '@' => {
                    let place = match directive.count {
                        Count::Rest => 0,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    if place > out.len() {
                        out.resize(place, 0);
                    } else {
                        out.truncate(place);
                    }
                }
                // The floating point directives.
                'f' | 'F' | 'e' | 'd' | 'D' | 'E' | 'g' | 'G' => {
                    let width = if matches!(directive.code, 'f' | 'F' | 'e' | 'g') {
                        4
                    } else {
                        8
                    };
                    let big = matches!(directive.code, 'g' | 'G');
                    let wanted = match directive.count {
                        Count::Rest => left,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    for _ in 0..wanted {
                        let Some(item) = items.get(taken) else {
                            return Err(too_few_items(position));
                        };
                        let value = self.coerce_pack_float(item, position)?;
                        taken += 1;
                        let mut written = if width == 4 {
                            (value as f32).to_le_bytes().to_vec()
                        } else {
                            value.to_le_bytes().to_vec()
                        };
                        if big {
                            written.reverse();
                        }
                        out.extend_from_slice(&written);
                    }
                }
                // The integer directives.
                code if integer_width(code, directive.native).is_some() => {
                    let width =
                        integer_width(code, directive.native).expect("the guard read a width");
                    let wanted = match directive.count {
                        Count::Rest => left,
                        Count::One => 1,
                        Count::Exactly(n) => n,
                    };
                    for _ in 0..wanted {
                        let Some(item) = items.get(taken) else {
                            return Err(too_few_items(position));
                        };
                        let value = self.coerce_pack_integer(item, position)?;
                        taken += 1;
                        out.extend_from_slice(&write_integer(value, width, code, directive.order));
                    }
                }
                // Uuencoding, written in lines of the length the count names.
                'u' | 'm' | 'M' => {
                    let held = match items.get(taken) {
                        Some(Object::String(text)) => text.as_str().to_string(),
                        Some(other) => self.coerce_pack_string(other, position)?,
                        None => return Err(too_few_items(position)),
                    };
                    taken += 1;
                    let source = string_to_bytes(&held);
                    let written = match directive.code {
                        'u' => encode_uu(&source),
                        'M' => encode_quoted_printable(&source),
                        // `m0` asks for base64 with no line breaks in it.
                        _ => encode_base64(&source, directive.count != Count::Exactly(0)),
                    };
                    out.extend_from_slice(written.as_bytes());
                }
                other => {
                    return Err(unknown_directive(other, format, "pack", position));
                }
            }
        }
        Ok(bytes_to_string(&out))
    }

    /// One item an integer directive was given, read as a number.
    fn coerce_pack_integer(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<i128, MetorexError> {
        match value {
            Object::Int(number) => return Ok(*number as i128),
            Object::Float(number) => return Ok(*number as i128),
            Object::BigInt(number) => {
                // Only the low bytes are written, so a number too wide for
                // the directive is taken modulo what fits.
                let wrapped: num_bigint::BigInt =
                    number.as_ref() & num_bigint::BigInt::from(u128::MAX);
                return Ok(wrapped.try_into().unwrap_or(0));
            }
            _ => {}
        }
        if self.responds_to(value, "to_int")
            && let Object::Int(number) =
                self.send_to_object(value.clone(), "to_int", vec![], position)?
        {
            return Ok(number as i128);
        }
        let message = format!(
            "no implicit conversion of {} into Integer",
            self.builtins().class_of(value).name()
        );
        Err(crate::vm::errors::simple_exception(
            "TypeError",
            &message,
            position,
        ))
    }

    /// One item a float directive was given, read as a number.
    fn coerce_pack_float(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<f64, MetorexError> {
        match value {
            Object::Float(number) => return Ok(*number),
            Object::Int(number) => return Ok(*number as f64),
            _ => {}
        }
        if self.responds_to(value, "to_f")
            && let Object::Float(number) =
                self.send_to_object(value.clone(), "to_f", vec![], position)?
        {
            return Ok(number);
        }
        let message = format!(
            "no implicit conversion of {} into Float",
            self.builtins().class_of(value).name()
        );
        Err(crate::vm::errors::simple_exception(
            "TypeError",
            &message,
            position,
        ))
    }

    /// One item a string directive was given, read as a String.
    fn coerce_pack_string(
        &mut self,
        value: &Object,
        position: Position,
    ) -> Result<String, MetorexError> {
        if self.responds_to(value, "to_str")
            && let Object::String(text) =
                self.send_to_object(value.clone(), "to_str", vec![], position)?
        {
            return Ok(text.as_str().to_string());
        }
        let message = format!(
            "no implicit conversion of {} into String",
            self.builtins().class_of(value).name()
        );
        Err(crate::vm::errors::simple_exception(
            "TypeError",
            &message,
            position,
        ))
    }
}
