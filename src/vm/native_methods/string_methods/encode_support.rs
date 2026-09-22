// The pieces `encode` and `inspect` are built from.

use super::*;

/// The arguments an `encode` was written with, split from what it says to put
/// in place of a byte the source encoding cannot read. `invalid: :replace`
/// asks for that, and `replace:` names the text, which defaults to `"?"`.
pub(crate) fn encode_options(arguments: &[Object]) -> (&[Object], Option<String>) {
    let Some(Object::Dict(pairs)) = arguments.last() else {
        return (arguments, None);
    };
    let held = pairs.borrow();
    if !held.contains_key("__MX_KWARGS__") {
        return (arguments, None);
    }
    let replacing = matches!(
        held.get(":invalid"),
        Some(Object::Symbol(named)) if &*named.as_str() == "replace"
    ) || matches!(
        held.get(":undef"),
        Some(Object::Symbol(named)) if &*named.as_str() == "replace"
    );
    let stands_in = match held.get(":replace") {
        Some(Object::String(text)) => text.as_str().to_string(),
        _ => "?".to_string(),
    };
    drop(held);
    let rest = &arguments[..arguments.len() - 1];
    (rest, replacing.then_some(stands_in))
}

/// The text a run of EUC-JP bytes spells, with a byte that opens no character
/// written as the text that stands in for one.
pub(crate) fn euc_jp_text_replacing(bytes: &[u8], stands_in: &str) -> String {
    let mut written = String::with_capacity(bytes.len());
    let mut at = 0usize;
    while at < bytes.len() {
        match crate::vm::native_methods::euc_jp_table::euc_jp_character(&bytes[at..]) {
            Some((character, width)) => {
                written.push(character);
                at += width;
            }
            None => {
                written.push_str(stands_in);
                at += 1;
            }
        }
    }
    written
}

/// Whether Ruby names this encoding without converting anything through it.
/// A string tagged with one has a character to the byte, since nothing reads
/// its bytes as text.
/// A symbol written the way `inspect` writes one, reading its name through
/// the encoding it carries. A name that does not read as plain text is
/// quoted, with the bytes it holds written out.
pub(crate) fn symbol_inspect_text(value: &crate::object::StringValue) -> String {
    let named = value.encoding_name();
    let bytes = binary_bytes(value);
    // An encoding metorex names without reading text through it spells its
    // name byte by byte.
    if dummy_encoding(&named) {
        let written: String = bytes.iter().map(|byte| format!("\\x{byte:02X}")).collect();
        return format!(":\"{written}\"");
    }
    // An encoding that does not spell ASCII the way ASCII does is read
    // through, and the characters it names are quoted.
    if let Some(shape) = wide_encoding(&named) {
        return format!(":{:?}", wide_text(&bytes, shape));
    }
    if matches!(named.as_str(), "ASCII-8BIT" | "BINARY") && !bytes.is_ascii() {
        let mut written = String::from(":\"");
        for byte in &bytes {
            if byte.is_ascii_graphic() || *byte == b' ' {
                written.push(char::from(*byte));
            } else {
                written.push_str(&format!("\\x{byte:02X}"));
            }
        }
        written.push('"');
        return written;
    }
    crate::object::inspect_symbol(&value.as_str())
}

pub(crate) fn dummy_encoding(named: &str) -> bool {
    crate::vm::init::ENCODING_NAMES
        .iter()
        .any(|(_, canonical, dummy)| *dummy && *canonical == named)
}

/// Where each Shift_JIS character starts. A byte in the lead ranges is read
/// together with the byte after it, and every other byte stands alone.
pub(crate) fn shift_jis_characters(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut characters = Vec::new();
    let mut at = 0usize;
    while at < bytes.len() {
        let leads = matches!(bytes[at], 0x81..=0x9f | 0xe0..=0xfc) && at + 1 < bytes.len();
        let width = if leads { 2 } else { 1 };
        characters.push(bytes[at..at + width].to_vec());
        at += width;
    }
    characters
}

/// The characters a string spells, each tagged with the string's own
/// encoding. What counts as a character depends on that encoding: a run of
/// bytes and an encoding Ruby converts nothing through both split at every
/// byte, Shift_JIS reads a lead byte together with the one after it, and the
/// rest read text.
pub(crate) fn encoded_characters(string_value: &crate::object::StringValue) -> Vec<Object> {
    let named = string_value.encoding_name();
    let from_bytes = |bytes: &[u8]| {
        let spelled: String = bytes.iter().map(|byte| *byte as char).collect();
        let made = crate::object::StringValue::from_bytes(spelled);
        made.set_encoding(named.clone());
        Object::String(Rc::new(made))
    };
    let per_byte = || {
        binary_bytes(string_value)
            .iter()
            .map(|byte| from_bytes(&[*byte]))
            .collect()
    };
    match named.as_str() {
        "ASCII-8BIT" | "BINARY" => per_byte(),
        held if dummy_encoding(held) => per_byte(),
        "Shift_JIS" | "Windows-31J" | "MacJapanese" => {
            shift_jis_characters(&binary_bytes(string_value))
                .iter()
                .map(|character| from_bytes(character))
                .collect()
        }
        _ => string_value
            .as_str()
            .chars()
            .map(|character| {
                let made =
                    crate::object::StringValue::with_encoding(character.to_string(), named.clone());
                Object::String(Rc::new(made))
            })
            .collect(),
    }
}
