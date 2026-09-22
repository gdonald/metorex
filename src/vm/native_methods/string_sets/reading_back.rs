// Reading back what `dump` wrote, and the strings `upto` yields.

use super::*;

impl VirtualMachine {
    /// Refuse an `upto` between two strings whose encodings have no common
    /// ground, which is what Ruby refuses before it compares them.
    pub(crate) fn check_upto_encodings(
        &mut self,
        left: &crate::object::StringValue,
        right: &crate::object::StringValue,
        position: Position,
    ) -> Result<(), MetorexError> {
        let (from, to) = (left.encoding_name(), right.encoding_name());
        if from == to {
            return Ok(());
        }
        if !NOT_ASCII_COMPATIBLE.contains(&from.as_str())
            && !NOT_ASCII_COMPATIBLE.contains(&to.as_str())
        {
            return Ok(());
        }
        let message = format!("incompatible character encodings: {} and {}", from, to);
        Err(MetorexError::UncaughtException {
            exception: Object::exception("Encoding::CompatibilityError", message.clone()),
            location: crate::vm::utils::position_to_location(position),
            message,
        })
    }
}

/// Read back what `dump` wrote, answering None when the text is not something
/// `dump` could have produced.
pub(crate) fn undump(text: &str) -> Result<(Vec<u8>, Option<String>), &'static str> {
    // A dump of a string in an encoding of its own carries the name after the
    // quoted run, as `"...".force_encoding("NAME")`.
    let (body, named) = match text.rfind("\".force_encoding(") {
        None => (text, None),
        Some(at) => {
            let (body, tail) = text.split_at(at + 1);
            let inside = tail
                .strip_prefix(".force_encoding(")
                .and_then(|held| held.strip_suffix(')'))
                .ok_or("invalid dumped string")?;
            let name = inside
                .strip_prefix('"')
                .and_then(|held| held.strip_suffix('"'))
                .ok_or("invalid dumped string")?;
            (body, Some(name.to_string()))
        }
    };
    let inner = body.strip_prefix('"').ok_or("invalid dumped string")?;
    let inner = inner
        .strip_suffix('"')
        .ok_or("unterminated dumped string")?;
    let mut bytes: Vec<u8> = Vec::new();
    let mut characters = inner.chars().peekable();
    while let Some(character) = characters.next() {
        if character != '\\' {
            match character {
                '"' => return Err("invalid dumped string"),
                '\0' => return Err("string contains null byte"),
                held if !held.is_ascii() => return Err("non-ASCII character detected"),
                held => bytes.push(held as u8),
            }
            continue;
        }
        let escaped = characters.next().ok_or("invalid dumped string")?;
        match escaped {
            'n' => bytes.push(b'\n'),
            't' => bytes.push(b'\t'),
            'r' => bytes.push(b'\r'),
            '0' => bytes.push(0),
            'a' => bytes.push(7),
            'b' => bytes.push(8),
            'v' => bytes.push(11),
            'f' => bytes.push(12),
            'e' => bytes.push(27),
            's' => bytes.push(b' '),
            '\\' | '"' | '#' => bytes.push(escaped as u8),
            'u' => {
                let mut digits = String::new();
                if characters.peek() == Some(&'{') {
                    characters.next();
                    let mut closed = false;
                    for digit in characters.by_ref() {
                        if digit == '}' {
                            closed = true;
                            break;
                        }
                        digits.push(digit);
                    }
                    if !closed {
                        return Err("invalid Unicode escape");
                    }
                } else {
                    for _ in 0..4 {
                        digits.push(characters.next().ok_or("invalid Unicode escape")?);
                    }
                }
                let point =
                    u32::from_str_radix(&digits, 16).map_err(|_| "invalid Unicode escape")?;
                let spelled = char::from_u32(point).ok_or("invalid Unicode escape")?;
                let mut room = [0u8; 4];
                bytes.extend(spelled.encode_utf8(&mut room).as_bytes().iter().copied());
            }
            'x' => {
                let mut digits = String::new();
                for _ in 0..2 {
                    match characters.peek() {
                        Some(digit) if digit.is_ascii_hexdigit() => {
                            digits.push(*digit);
                            characters.next();
                        }
                        _ => break,
                    }
                }
                if digits.len() != 2 {
                    return Err("invalid hex escape");
                }
                bytes.push(u8::from_str_radix(&digits, 16).map_err(|_| "invalid hex escape")?);
            }
            _ => return Err("invalid dumped string"),
        }
    }
    Ok((bytes, named))
}

/// The encodings that carry no ASCII characters at all, so a string tagged
/// with one cannot be compared against a string tagged with another.
pub(crate) const NOT_ASCII_COMPATIBLE: &[&str] = &[
    "ISO-2022-JP",
    "UTF-16",
    "UTF-16BE",
    "UTF-16LE",
    "UTF-32",
    "UTF-32BE",
    "UTF-32LE",
];

/// Every string `upto` yields, from `first` up to `last`.
pub(crate) fn upto_sequence(first: &str, last: &str, exclusive: bool) -> Vec<String> {
    let held = first.chars().count();
    let target = last.chars().count();
    if held > target || (held == target && first > last) {
        return Vec::new();
    }
    // Two single characters step by code point, so `"9".upto("A")` walks the
    // punctuation between them rather than counting.
    if held == 1 && target == 1 {
        let from = first.chars().next().expect("one character");
        let to = last.chars().next().expect("one character");
        let stop = if exclusive { to as u32 } else { to as u32 + 1 };
        return (from as u32..stop)
            .filter_map(char::from_u32)
            .map(String::from)
            .collect();
    }
    let mut steps = Vec::new();
    let mut current = first.to_string();
    loop {
        if exclusive && current == last {
            break;
        }
        steps.push(current.clone());
        if current == last {
            break;
        }
        let next = crate::vm::native_methods::string_methods::successor_of(&current);
        if next.chars().count() > target {
            break;
        }
        current = next;
    }
    steps
}

/// Fold only the ASCII letters, which is the comparison `casecmp` makes.
pub(crate) fn fold_ascii_case(text: &str) -> String {
    text.chars()
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

/// The encoding two strings can both be read in, or None when they hold
/// characters their encodings disagree about.
pub(crate) fn compatible_encoding(
    left: &str,
    left_encoding: &str,
    right: &str,
    right_encoding: &str,
) -> Option<String> {
    if left_encoding == right_encoding {
        return Some(left_encoding.to_string());
    }
    // A string that is all ASCII reads the same under either encoding, so the
    // other one decides.
    if left.is_ascii() {
        return Some(right_encoding.to_string());
    }
    if right.is_ascii() {
        return Some(left_encoding.to_string());
    }
    None
}

/// Whether an encoding spells the whole of Unicode, which is what decides
/// whether a letter outside ASCII maps onto another case of itself.
pub(crate) fn unicode_encoding(named: &str) -> bool {
    named.starts_with("UTF-") || named == "US-ASCII" || named == "CESU-8"
}
