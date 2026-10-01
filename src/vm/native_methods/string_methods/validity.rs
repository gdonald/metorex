// Whether a run of bytes spells characters in the encoding it
// says it is written in, and whether two encodings can be read together.

use super::*;

/// Whether a string spells characters in the encoding it says it is written
/// in. Only a run of bytes tagged as text can fail to.
pub(crate) fn holds_valid_text(string_value: &crate::object::StringValue) -> bool {
    // An encoding Ruby names without converting anything through it reads its
    // text a byte at a time, so every run of bytes spells characters in it.
    // One that still pairs its bytes is read for that shape all the same.
    let named = string_value.encoding_name();
    if dummy_encoding(&named) && !pairs_its_bytes(&named) {
        return true;
    }
    match named.as_str() {
        "ASCII-8BIT" | "BINARY" => true,
        "US-ASCII" => string_value.as_str().is_ascii(),
        "UTF-8" if string_value.holds_bytes() => String::from_utf8(
            crate::vm::native_methods::pack_format::string_to_bytes(&string_value.as_str()),
        )
        .is_ok(),
        "UTF8-MAC" if string_value.holds_bytes() => String::from_utf8(
            crate::vm::native_methods::pack_format::string_to_bytes(&string_value.as_str()),
        )
        .is_ok(),
        "EUC-JP" => euc_jp_reads(&binary_bytes(string_value)),
        // Shift_JIS spells half-width katakana with one byte of its own, so
        // the shape of a run says more than a plain lead-and-trail pairing.
        named if spells_shift_jis(named) => shift_jis_reads(&binary_bytes(string_value)),
        // A fixed-width encoding reads whole units, so a run of bytes that
        // does not divide into them spells no characters at all, and a half
        // of a surrogate pair standing alone spells none either.
        "UTF-16" | "UTF-16BE" | "UTF-16LE" => utf16_reads(
            &binary_bytes(string_value),
            !string_value.encoding_name().ends_with("LE"),
        ),
        "UTF-32" | "UTF-32BE" | "UTF-32LE" => utf32_reads(
            &binary_bytes(string_value),
            !string_value.encoding_name().ends_with("LE"),
        ),
        named if pairs_its_bytes(named) => paired_bytes_read(&binary_bytes(string_value)),
        _ => true,
    }
}

/// Whether an encoding spells some characters with a lead byte and a trailing
/// one. Metorex reads the shape of such a run rather than the character it
/// stands for, which is what tells a broken run from a whole one.
pub(crate) fn pairs_its_bytes(named: &str) -> bool {
    matches!(
        named,
        "Big5"
            | "CP949"
            | "EUC-KR"
            | "EUC-TW"
            | "GB18030"
            | "GBK"
            | "GB2312"
            | "GB12345"
            | "Shift_JIS"
            | "Windows-31J"
            | "MacJapanese"
            | "Emacs-Mule"
            | "stateless-ISO-2022-JP"
            | "eucJP-ms"
            | "CP51932"
    )
}

/// Whether every lead byte in a run is followed by the byte it pairs with.
/// A byte under 0x80 stands for itself.
pub(crate) fn paired_bytes_read(bytes: &[u8]) -> bool {
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] < 0x80 {
            at += 1;
            continue;
        }
        if at + 1 >= bytes.len() {
            return false;
        }
        at += 2;
    }
    true
}

/// Whether a run of bytes spells characters in UTF-16: whole units, with each
/// high half of a surrogate pair followed by a low one.
pub(crate) fn utf16_reads(bytes: &[u8], big_endian: bool) -> bool {
    if !bytes.len().is_multiple_of(2) {
        return false;
    }
    let unit_at = |at: usize| -> u16 {
        let (first, second) = (bytes[at] as u16, bytes[at + 1] as u16);
        if big_endian {
            (first << 8) | second
        } else {
            (second << 8) | first
        }
    };
    let mut at = 0;
    while at < bytes.len() {
        let unit = unit_at(at);
        if (0xdc00..0xe000).contains(&unit) {
            return false;
        }
        if (0xd800..0xdc00).contains(&unit) {
            if at + 3 >= bytes.len() || !(0xdc00..0xe000).contains(&unit_at(at + 2)) {
                return false;
            }
            at += 4;
            continue;
        }
        at += 2;
    }
    true
}

/// Whether a run of bytes spells characters in UTF-32: whole units, each one
/// a code point Unicode names.
pub(crate) fn utf32_reads(bytes: &[u8], big_endian: bool) -> bool {
    if !bytes.len().is_multiple_of(4) {
        return false;
    }
    bytes.chunks(4).all(|unit| {
        let point = if big_endian {
            u32::from_be_bytes([unit[0], unit[1], unit[2], unit[3]])
        } else {
            u32::from_le_bytes([unit[0], unit[1], unit[2], unit[3]])
        };
        char::from_u32(point).is_some()
    })
}

/// Whether a run of bytes spells characters in EUC-JP. A byte under 0x80
/// stands for itself, 0x8E opens a half-width katakana pair, 0x8F opens a
/// three-byte run, and everything else pairs two bytes from 0xA1 to 0xFE.
pub(crate) fn euc_jp_reads(bytes: &[u8]) -> bool {
    let mut at = 0;
    while at < bytes.len() {
        let held = bytes[at];
        let width = match held {
            0x00..=0x7f => 1,
            0x8e => 2,
            0x8f => 3,
            0xa1..=0xfe => 2,
            _ => return false,
        };
        if at + width > bytes.len() {
            return false;
        }
        let trailing = &bytes[at + 1..at + width];
        let fits = match held {
            0x8e => trailing.iter().all(|byte| (0xa1..=0xdf).contains(byte)),
            _ => trailing.iter().all(|byte| (0xa1..=0xfe).contains(byte)),
        };
        if !fits {
            return false;
        }
        at += width;
    }
    true
}

/// Whether an encoding spells the ASCII letters one byte to a letter.
pub(crate) fn encoding_is_ascii_compatible(named: &str) -> bool {
    !matches!(
        named,
        "UTF-16" | "UTF-16BE" | "UTF-16LE" | "UTF-32" | "UTF-32BE" | "UTF-32LE"
    )
}

/// The ArgumentError Ruby raises for a string whose bytes spell nothing in
/// the encoding it is tagged with.
pub(crate) fn broken_text_error(
    string_value: &crate::object::StringValue,
    position: Position,
) -> MetorexError {
    let message = format!("invalid byte sequence in {}", string_value.encoding_name());
    crate::vm::errors::simple_exception("ArgumentError", &message, position)
}

/// Whether two strings are written in encodings that cannot be joined. Text
/// that is nothing but ASCII goes with anything.
pub(crate) fn encodings_clash(
    left: &crate::object::StringValue,
    right: &crate::object::StringValue,
) -> bool {
    if left.encoding_name() == right.encoding_name() {
        return false;
    }
    !left.as_str().is_ascii() && !right.as_str().is_ascii()
}

/// Whether two strings are written in encodings Ruby will compare. Empty
/// text goes with anything, as does text that is nothing but ASCII when the
/// other encoding spells ASCII one byte to a letter.
pub(crate) fn strings_comparable(
    left: &crate::object::StringValue,
    right: &crate::object::StringValue,
) -> bool {
    if left.as_str().is_empty() || right.as_str().is_empty() {
        return true;
    }
    if left.encoding_name() == right.encoding_name() {
        return true;
    }
    // Text tagged as raw bytes says nothing about the characters behind it,
    // so it compares against anything rather than against its own tag alone.
    let reads_as_bytes =
        |named: &str| matches!(named, "ASCII-8BIT" | "BINARY" | "ASCII-8BIT (BINARY)");
    if reads_as_bytes(&left.encoding_name()) || reads_as_bytes(&right.encoding_name()) {
        return true;
    }
    let left_plain =
        left.as_str().is_ascii() && encoding_is_ascii_compatible(&left.encoding_name());
    let right_plain =
        right.as_str().is_ascii() && encoding_is_ascii_compatible(&right.encoding_name());
    if left_plain && right_plain {
        return true;
    }
    if left_plain && encoding_is_ascii_compatible(&right.encoding_name()) {
        return true;
    }
    right_plain && encoding_is_ascii_compatible(&left.encoding_name())
}

/// The Encoding::CompatibilityError Ruby raises for joining text written in
/// two encodings that do not go together.
pub(crate) fn clashing_encodings_error(
    left: &crate::object::StringValue,
    right: &crate::object::StringValue,
    position: Position,
) -> MetorexError {
    let message = format!(
        "incompatible character encodings: {} and {}",
        left.encoding_name(),
        right.encoding_name()
    );
    crate::vm::errors::simple_exception("Encoding::CompatibilityError", &message, position)
}

/// How many bytes the character opening a run takes in UTF-8, or 0 when the
/// run does not open a whole one.
pub(crate) fn utf8_sequence_width(bytes: &[u8]) -> usize {
    let Some(first) = bytes.first().copied() else {
        return 0;
    };
    let width = match first {
        0x00..=0x7f => return 1,
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return 0,
    };
    if bytes.len() < width {
        return 0;
    }
    if bytes[1..width]
        .iter()
        .all(|held| (0x80..=0xbf).contains(held))
    {
        width
    } else {
        0
    }
}

/// How many characters a string holds, counted the way the encoding it is
/// tagged with reads them. A string that carries text rather than a run of
/// bytes is counted by its characters, since relabelling it leaves the text
/// as it was.
pub(crate) fn character_count(string_value: &crate::object::StringValue) -> i64 {
    let named = string_value.encoding_name();
    match named.as_str() {
        // A run of bytes has one character to the byte however it is tagged.
        "ASCII-8BIT" | "BINARY" => binary_bytes(string_value).len() as i64,
        // A fixed-width encoding reads whole units, and a unit left short at
        // the end still counts as the one broken character it spells.
        "UTF-16" | "UTF-16BE" | "UTF-16LE" if string_value.holds_bytes() => {
            utf16_unit_count(&binary_bytes(string_value), named.ends_with("BE"))
        }
        "UTF-32" | "UTF-32BE" | "UTF-32LE" if string_value.holds_bytes() => {
            binary_bytes(string_value).len().div_ceil(4) as i64
        }
        // Shift_JIS and EUC-JP pair some of their bytes, so a run of them
        // spells fewer characters than it has bytes.
        held if spells_shift_jis(held) && string_value.holds_bytes() => {
            shift_jis_characters(&binary_bytes(string_value)).len() as i64
        }
        "EUC-JP" if string_value.holds_bytes() => {
            euc_jp_characters(&binary_bytes(string_value)).len() as i64
        }
        "UTF-8" if string_value.holds_bytes() => {
            utf8_characters(&binary_bytes(string_value)).len() as i64
        }
        _ => string_value.as_str().chars().count() as i64,
    }
}

/// The characters of a string, each as the text that spells it. A Shift_JIS
/// or EUC-JP string held as bytes pairs some of them, so one character there
/// is the run of byte characters it takes.
pub(crate) fn character_units(string_value: &crate::object::StringValue) -> Vec<String> {
    let named = string_value.encoding_name();
    let grouped = |groups: Vec<Vec<u8>>| -> Vec<String> {
        groups
            .iter()
            .map(|group| group.iter().map(|byte| *byte as char).collect())
            .collect()
    };
    match named.as_str() {
        held if spells_shift_jis(held) && string_value.holds_bytes() => {
            grouped(shift_jis_characters(&binary_bytes(string_value)))
        }
        "EUC-JP" if string_value.holds_bytes() => {
            grouped(euc_jp_characters(&binary_bytes(string_value)))
        }
        "UTF-8" if string_value.holds_bytes() => {
            grouped(utf8_characters(&binary_bytes(string_value)))
        }
        _ => string_value.as_str().chars().map(String::from).collect(),
    }
}

/// The characters a run of UTF-8 bytes spells, each as the bytes it takes. A
/// byte that starts no character, or one cut short, stands alone.
pub(crate) fn utf8_characters(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut characters = Vec::new();
    let mut rest = bytes;
    while !rest.is_empty() {
        let taken = match std::str::from_utf8(rest) {
            Ok(_) => rest.len(),
            Err(trouble) => trouble.valid_up_to(),
        };
        let (valid, after) = rest.split_at(taken);
        let text = std::str::from_utf8(valid).unwrap_or_default();
        characters.extend(text.chars().map(|letter| letter.to_string().into_bytes()));
        if let Some((first, remaining)) = after.split_first() {
            characters.push(vec![*first]);
            rest = remaining;
        } else {
            rest = after;
        }
    }
    characters
}

/// How many characters a run of UTF-16 bytes spells. A high surrogate paired
/// with a low one stands for a single character, and one left on its own
/// stands for itself.
pub(crate) fn utf16_unit_count(bytes: &[u8], big_endian: bool) -> i64 {
    let unit_at = |at: usize| -> u16 {
        let (first, second) = (bytes[at] as u16, bytes[at + 1] as u16);
        if big_endian {
            (first << 8) | second
        } else {
            (second << 8) | first
        }
    };
    let mut counted = 0i64;
    let mut at = 0usize;
    while at + 1 < bytes.len() {
        let unit = unit_at(at);
        let paired = (0xd800..0xdc00).contains(&unit)
            && at + 3 < bytes.len()
            && (0xdc00..0xe000).contains(&unit_at(at + 2));
        at += if paired { 4 } else { 2 };
        counted += 1;
    }
    if at < bytes.len() {
        counted += 1;
    }
    counted
}
