// The clusters a string breaks into, each one character as a reader
// sees it.

use super::*;

/// The character that joins what stands on either side of it into one
/// cluster, which is how a flag and a rainbow spell one emoji.
pub(crate) const ZERO_WIDTH_JOINER: char = '\u{200d}';

/// The clusters a string breaks into: one leading character, then every
/// combining mark that attaches to it, and whatever a zero width joiner
/// pulls along after it.
pub(crate) fn grapheme_clusters(text: &str) -> Vec<String> {
    let mut clusters: Vec<String> = Vec::new();
    let mut joining = false;
    for character in text.chars() {
        let attaches = joining || is_combining_mark(character) || character == ZERO_WIDTH_JOINER;
        joining = character == ZERO_WIDTH_JOINER;
        if attaches && let Some(last) = clusters.last_mut() {
            last.push(character);
            continue;
        }
        clusters.push(character.to_string());
    }
    clusters
}

/// The clusters a string breaks into, each tagged with the string's own
/// encoding. An encoding that spells a character in more than one byte is
/// read out of those bytes and written back into them.
pub(crate) fn encoded_grapheme_clusters(string_value: &crate::object::StringValue) -> Vec<Object> {
    use crate::vm::native_methods::string_methods::{
        binary_bytes, bytes_as_text, wide_bytes, wide_encoding, wide_text,
    };
    let named = string_value.encoding_name();
    if let Some(shape) = wide_encoding(&named)
        && !crate::vm::native_methods::string_methods::dummy_encoding(&named)
    {
        return grapheme_clusters(&wide_text(&binary_bytes(string_value), shape))
            .iter()
            .map(|cluster| {
                let made = crate::object::StringValue::from_bytes(bytes_as_text(&wide_bytes(
                    cluster, shape,
                )));
                made.set_encoding(named.clone());
                Object::String(std::rc::Rc::new(made))
            })
            .collect();
    }
    // An encoding that has one character to the byte has one cluster to the
    // byte with it, since nothing there attaches to anything.
    if matches!(named.as_str(), "ASCII-8BIT" | "BINARY")
        || crate::vm::native_methods::string_methods::dummy_encoding(&named)
        || matches!(named.as_str(), "Shift_JIS" | "Windows-31J" | "MacJapanese")
    {
        return crate::vm::native_methods::string_methods::encoded_characters(string_value);
    }
    grapheme_clusters(&string_value.as_str())
        .into_iter()
        .map(|cluster| {
            Object::String(std::rc::Rc::new(crate::object::StringValue::with_encoding(
                cluster,
                named.clone(),
            )))
        })
        .collect()
}

/// Whether a character attaches to the one before it rather than standing on
/// its own, which is what keeps a cluster together.
pub(crate) fn is_combining_mark(character: char) -> bool {
    matches!(character as u32,
        0x0300..=0x036F
        | 0x0483..=0x0489
        | 0x0591..=0x05BD
        | 0x0610..=0x061A
        | 0x064B..=0x065F
        | 0x0670
        | 0x06D6..=0x06DC
        | 0x0900..=0x0903
        | 0x093A..=0x094F
        | 0x0951..=0x0957
        | 0x1AB0..=0x1AFF
        | 0x1DC0..=0x1DFF
        | 0x20D0..=0x20F0
        | 0xFE00..=0xFE0F
        | 0xFE20..=0xFE2F)
}
