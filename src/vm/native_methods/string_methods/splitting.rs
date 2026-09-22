// The pieces a `split` cuts a string into.

use super::*;

/// The paragraphs a string holds: each one runs up to a break of two or more
/// newlines, keeps two of them, and the rest of the break is dropped.
pub(crate) fn paragraphs_of(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut collected = Vec::new();
    let mut start = 0;
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] != b'\n' {
            at += 1;
            continue;
        }
        let mut run = at;
        while run < bytes.len() && bytes[run] == b'\n' {
            run += 1;
        }
        if run - at < 2 {
            at = run;
            continue;
        }
        collected.push(text[start..at + 2].to_string());
        start = run;
        at = run;
    }
    if start < text.len() {
        collected.push(text[start..].to_string());
    }
    collected
}

/// One line with its separator taken back off. Written without a separator,
/// the walk cuts on newlines and takes a carriage return off with them.
pub(crate) fn chomped_line(piece: String, separator: Option<&str>) -> String {
    match separator {
        Some("\n") | None => piece
            .strip_suffix('\n')
            .map(|line| line.strip_suffix('\r').unwrap_or(line))
            .unwrap_or(&piece)
            .to_string(),
        Some(separator) => piece.strip_suffix(separator).unwrap_or(&piece).to_string(),
    }
}

/// A replacement string with its backslash sequences filled in from the
/// match: the numbered groups, the whole match, what sits either side of it,
/// and the last group that took part.
pub(crate) fn fill_replacement(
    written: &str,
    subject: &str,
    captured: &crate::regexp::Captures<'_, '_>,
) -> String {
    let letters: Vec<char> = written.chars().collect();
    let whole = captured.get(0).expect("a match was found");
    let mut built = String::new();
    let mut at = 0;
    while at < letters.len() {
        if letters[at] != '\\' || at + 1 >= letters.len() {
            built.push(letters[at]);
            at += 1;
            continue;
        }
        let marker = letters[at + 1];
        at += 2;
        match marker {
            '0'..='9' => {
                let index = marker as usize - '0' as usize;
                if let Some(part) = captured.get(index) {
                    built.push_str(part.as_str());
                }
            }
            '&' => built.push_str(whole.as_str()),
            '`' => built.push_str(&subject[..whole.start()]),
            '\'' => built.push_str(&subject[whole.end()..]),
            '+' => {
                // The last group that took part in the match, which is not
                // always the last one written.
                let found = (1..captured.len())
                    .rev()
                    .find_map(|index| captured.get(index));
                if let Some(part) = found {
                    built.push_str(part.as_str());
                }
            }
            '\\' => built.push('\\'),
            'k' if letters.get(at) == Some(&'<') => {
                let Some(closing) = letters[at + 1..].iter().position(|held| *held == '>') else {
                    built.push('\\');
                    built.push('k');
                    continue;
                };
                let name: String = letters[at + 1..at + 1 + closing].iter().collect();
                if let Some(part) = captured.name(&name) {
                    built.push_str(part.as_str());
                }
                at += closing + 2;
            }
            other => {
                built.push('\\');
                built.push(other);
            }
        }
    }
    built
}

/// The bytes a `chomp` keeps. Without a separator the string is left as it
/// is, an empty one takes off every line ending at the end, and the line
/// ending itself takes off one of any of the three shapes it has.
pub(crate) fn chomped_bytes(bytes: &[u8], named: Option<&str>, encoding: &str) -> Vec<u8> {
    let Some(named) = named else {
        return bytes.to_vec();
    };
    let spelled = |text: &str| bytes_in_encoding(text, encoding);
    if named.is_empty() {
        let (carriage, line) = (spelled("\r\n"), spelled("\n"));
        let mut kept = bytes;
        loop {
            if let Some(shorter) = kept.strip_suffix(carriage.as_slice()) {
                kept = shorter;
                continue;
            }
            if let Some(shorter) = kept.strip_suffix(line.as_slice()) {
                kept = shorter;
                continue;
            }
            return kept.to_vec();
        }
    }
    if named == "\n" {
        for ending in ["\r\n", "\n", "\r"] {
            if let Some(kept) = bytes.strip_suffix(spelled(ending).as_slice()) {
                return kept.to_vec();
            }
        }
        return bytes.to_vec();
    }
    match bytes.strip_suffix(spelled(named).as_slice()) {
        Some(kept) => kept.to_vec(),
        None => bytes.to_vec(),
    }
}

/// The bytes an encoding spells a run of text with, or None where metorex
/// carries no table for it.
pub(crate) fn spelled_bytes(text: &str, named: &str) -> Option<Vec<u8>> {
    if let Some(shape) = wide_encoding(named) {
        return Some(wide_bytes(text, shape));
    }
    if named == "EUC-JP" {
        return crate::vm::native_methods::euc_jp_table::euc_jp_bytes(text).ok();
    }
    if named == "ISO-2022-JP" {
        return crate::vm::native_methods::euc_jp_table::iso_2022_jp_bytes(text).ok();
    }
    if spells_shift_jis(named) {
        return crate::vm::native_methods::shift_jis_table::shift_jis_bytes(text).ok();
    }
    match latin_bytes(text, named) {
        Some(Ok(spelled)) => Some(spelled),
        _ => None,
    }
}

/// The bytes an encoding spells a run of text with, falling back on the ones
/// the text is held as where the encoding has no table of its own.
pub(crate) fn bytes_in_encoding(text: &str, named: &str) -> Vec<u8> {
    spelled_bytes(text, named).unwrap_or_else(|| text.as_bytes().to_vec())
}

/// What a `split` cuts on.
pub(crate) enum Separator {
    /// Runs of whitespace, with none kept at either end.
    Whitespace,
    /// Between every character.
    Characters,
    /// A run of text, matched as it stands.
    Literal(String),
    /// A pattern, whose groups are kept alongside the pieces.
    Pattern(crate::regexp::Pattern),
    /// The place a run starts, cut in front of rather than taken out.
    Before(crate::regexp::Pattern),
}

/// The run a pattern that is nothing but a look-ahead names.
pub(crate) fn looks_ahead(pattern: &str, flags: &str) -> Option<crate::regexp::Pattern> {
    let inside = pattern.strip_prefix("(?=")?.strip_suffix(')')?;
    if inside.contains(')') && !inside.contains('(') {
        return None;
    }
    crate::vm::native_methods::regexp_methods::compile(inside, flags)
}

/// The separator a written one stands for. A single space asks for the
/// whitespace reading, and an empty one cuts between characters.
pub(crate) fn separator_of(written: &str) -> Separator {
    if written == " " {
        return Separator::Whitespace;
    }
    if written.is_empty() {
        return Separator::Characters;
    }
    Separator::Literal(written.to_string())
}

/// The pieces a split answers. A count of one keeps the whole text, a
/// positive count leaves the rest in the last piece, zero drops the empty
/// pieces at the end, and a negative one keeps them.
pub(crate) fn split_pieces(text: &str, separator: &Separator, limit: i64) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }
    if limit == 1 {
        return vec![text.to_string()];
    }
    let mut pieces = match separator {
        Separator::Whitespace => split_on_whitespace(text, limit),
        Separator::Characters => split_between_characters(text, limit),
        Separator::Literal(held) => split_on_literal(text, held, limit),
        Separator::Pattern(held) => split_on_pattern(text, held, limit),
        Separator::Before(held) => split_before(text, held, limit),
    };
    if limit == 0 {
        while pieces.last().is_some_and(|piece| piece.is_empty()) {
            pieces.pop();
        }
    }
    pieces
}

/// Words, with the whitespace between them dropped along with any at either
/// end. A count leaves the rest of the text, spaces and all, in the last
/// piece.
fn split_on_whitespace(text: &str, limit: i64) -> Vec<String> {
    let mut pieces = Vec::new();
    let bytes = text.as_bytes();
    let mut skipping = true;
    let mut from = 0usize;
    let mut ends = 0usize;
    let mut fields = 1i64;
    let mut at = 0usize;
    while at < bytes.len() {
        let width = next_character(text, at).min(text.len()) - at;
        let space = spells_space(bytes[at]);
        if skipping {
            if space {
                from = at + width;
            } else {
                ends = at + width;
                skipping = false;
                if limit > 0 && limit <= fields {
                    break;
                }
            }
        } else if space {
            pieces.push(text[from..ends].to_string());
            skipping = true;
            from = at + width;
            fields += 1;
        } else {
            ends = at + width;
        }
        at += width;
    }
    if !text.is_empty() && (limit != 0 || text.len() > from) {
        pieces.push(text[from..].to_string());
    }
    pieces
}

/// Whether a byte is one of the six characters Ruby counts as whitespace.
fn spells_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

/// One piece for each character.
fn split_between_characters(text: &str, limit: i64) -> Vec<String> {
    let mut pieces: Vec<String> = Vec::new();
    for (at, held) in text.char_indices() {
        if limit > 0 && pieces.len() as i64 == limit - 1 {
            pieces.push(text[at..].to_string());
            return pieces;
        }
        pieces.push(held.to_string());
    }
    if limit != 0 {
        pieces.push(String::new());
    }
    pieces
}

/// Pieces cut on a run of text matched as it stands.
fn split_on_literal(text: &str, held: &str, limit: i64) -> Vec<String> {
    let mut pieces = Vec::new();
    let mut from = 0usize;
    let mut at = 0usize;
    while let Some(found) = text[at..].find(held) {
        let cut = at + found;
        pieces.push(text[from..cut].to_string());
        from = cut + held.len();
        at = from;
        if limit > 0 && pieces.len() as i64 >= limit - 1 {
            break;
        }
    }
    pieces.push(text[from..].to_string());
    pieces
}

/// Pieces cut on a pattern, with the text each of its groups matched kept
/// alongside them. A pattern that matches nothing at all cuts between
/// characters, which is what Ruby does for an empty match.
fn split_on_pattern(text: &str, held: &crate::regexp::Pattern, limit: i64) -> Vec<String> {
    let mut pieces: Vec<String> = Vec::new();
    let mut fields = 1i64;
    let mut from = 0usize;
    let mut at = 0usize;
    let mut after_empty = false;
    while at <= text.len() {
        let Some(found) = held.captures_at(text, at) else {
            break;
        };
        let whole = found.get(0).expect("a match holds its whole run");
        let (starts, ends) = (whole.start(), whole.end());
        if at == starts && starts == ends {
            if !after_empty {
                // An empty match at the reading point steps on by one
                // character before it counts as a cut.
                at = next_character(text, at);
                after_empty = true;
                continue;
            }
            let ends_at = next_character(text, from);
            pieces.push(text[from..ends_at].to_string());
            from = at;
        } else {
            pieces.push(text[from..starts].to_string());
            from = ends;
            at = ends;
        }
        after_empty = false;
        for group in 1..found.len() {
            if let Some(taken) = found.get(group) {
                pieces.push(taken.as_str().to_string());
            }
        }
        fields += 1;
        if limit > 0 && limit <= fields {
            break;
        }
    }
    if text.len() > from || limit != 0 {
        pieces.push(text[from..].to_string());
    }
    pieces
}

/// Pieces cut in front of every place a run starts, which is what a pattern
/// written as a look-ahead asks for.
fn split_before(text: &str, held: &crate::regexp::Pattern, limit: i64) -> Vec<String> {
    let mut pieces = Vec::new();
    let mut from = 0usize;
    let mut at = 0usize;
    let mut fields = 1i64;
    while at <= text.len() {
        let Some(found) = held.find_at(text, at) else {
            break;
        };
        let starts = found.start();
        if starts > from {
            pieces.push(text[from..starts].to_string());
            from = starts;
            fields += 1;
            if limit > 0 && limit <= fields {
                break;
            }
        }
        at = next_character(text, starts);
    }
    if text.len() > from || limit != 0 {
        pieces.push(text[from..].to_string());
    }
    pieces
}

/// Where the character at `at` ends.
fn next_character(text: &str, at: usize) -> usize {
    if at >= text.len() {
        return text.len() + 1;
    }
    text[at..]
        .chars()
        .next()
        .map(|held| at + held.len_utf8())
        .unwrap_or(text.len())
}
