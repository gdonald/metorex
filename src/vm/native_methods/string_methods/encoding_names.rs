// The name Ruby knows an encoding by, and what its bytes look like.

use super::*;

/// The name Ruby knows an encoding by, whatever spelling or case it was named
/// with. A name nothing in the table matches is left as it was given.
pub(crate) fn canonical_encoding_name(named: &str) -> String {
    let wanted = named.replace('-', "_").to_ascii_uppercase();
    let same = |held: &str| held.replace('-', "_").to_ascii_uppercase() == wanted;
    crate::vm::init::ENCODING_NAMES
        .iter()
        .find(|(spelled, canonical, _)| same(spelled) || same(canonical))
        .map(|(_, canonical, _)| canonical.to_string())
        .unwrap_or_else(|| named.to_string())
}

/// Whether an encoding name is one of the Shift_JIS family.
pub(crate) fn spells_shift_jis(named: &str) -> bool {
    matches!(named, "Shift_JIS" | "Windows-31J" | "MacJapanese")
}

/// Whether a run of bytes spells characters in Shift_JIS. A byte under 0x80
/// stands for itself, 0xA1 through 0xDF is a half-width katakana of its own,
/// and 0x81 through 0x9F or 0xE0 through 0xEF opens a pair.
pub(crate) fn shift_jis_reads(bytes: &[u8]) -> bool {
    let mut at = 0;
    while at < bytes.len() {
        let held = bytes[at];
        let width = match held {
            0x00..=0x7f | 0xa1..=0xdf => 1,
            0x81..=0x9f | 0xe0..=0xef => 2,
            _ => return false,
        };
        if width == 2 {
            match bytes.get(at + 1) {
                Some(0x40..=0x7e | 0x80..=0xfc) => {}
                _ => return false,
            }
        }
        at += width;
    }
    true
}

/// Whether a run of bytes spells characters in an encoding. Used where bytes
/// arrive from outside the program already, such as a name read off the file
/// system, rather than as the text a String holds.
pub(crate) fn encoding_reads_bytes(bytes: &[u8], named: &str) -> bool {
    match named {
        "ASCII-8BIT" | "BINARY" => true,
        "US-ASCII" => bytes.is_ascii(),
        "UTF-8" | "CESU-8" => std::str::from_utf8(bytes).is_ok(),
        "EUC-JP" => euc_jp_reads(bytes),
        "Shift_JIS" | "Windows-31J" | "MacJapanese" => shift_jis_reads(bytes),
        "UTF-16" | "UTF-16BE" | "UTF-16LE" => bytes.len().is_multiple_of(2),
        "UTF-32" | "UTF-32BE" | "UTF-32LE" => bytes.len().is_multiple_of(4),
        _ => true,
    }
}

// The C library's one-way hash, which `String#crypt` answers with. BSD keeps
// it in the C library itself, while Linux keeps it in a library of its own
// that has to be named for the linker to find it.
#[cfg_attr(target_os = "linux", link(name = "crypt"))]
unsafe extern "C" {
    pub(crate) fn crypt(key: *const libc::c_char, salt: *const libc::c_char) -> *mut libc::c_char;
}
