// Checking a pattern source before the regexp engine sees it.

use super::*;

/// One string written so a pattern matches it and nothing else. Ruby escapes
/// every character a pattern reads as punctuation, the space among them, and
/// spells out the whitespace that has no printable form.
pub(crate) fn quoted_for_pattern(source: &str) -> String {
    let mut written = String::with_capacity(source.len());
    for character in source.chars() {
        match character {
            '[' | ']' | '{' | '}' | '(' | ')' | '|' | '-' | '*' | '.' | '\\' | '?' | '+' | '^'
            | '$' | '#' | ' ' => {
                written.push('\\');
                written.push(character);
            }
            '\n' => written.push_str("\\n"),
            '\r' => written.push_str("\\r"),
            '\t' => written.push_str("\\t"),
            '\u{b}' => written.push_str("\\v"),
            '\u{c}' => written.push_str("\\f"),
            other => written.push(other),
        }
    }
    written
}

/// Whether a pattern's source names anything outside ASCII, counting the
/// escapes that stand for a byte or a codepoint as well as the characters
/// written directly.
pub(crate) fn pattern_reaches_beyond_ascii(source: &str) -> bool {
    let letters: Vec<char> = source.chars().collect();
    let mut at = 0;
    while at < letters.len() {
        let letter = letters[at];
        if !letter.is_ascii() {
            return true;
        }
        if letter == '\\' && at + 1 < letters.len() {
            match letters[at + 1] {
                // A `\u` escape names a codepoint, which reaches past ASCII
                // only when the codepoint itself does.
                'u' => {
                    let (points, next) = unicode_escape_points(&letters, at + 2);
                    if points.iter().any(|held| *held >= 0x80) {
                        return true;
                    }
                    at = next;
                    continue;
                }
                'x' => {
                    let digits: String = letters[at + 2..]
                        .iter()
                        .take(2)
                        .take_while(|held| held.is_ascii_hexdigit())
                        .collect();
                    if let Ok(value) = u32::from_str_radix(&digits, 16)
                        && value >= 0x80
                    {
                        return true;
                    }
                    at += 2 + digits.len();
                    continue;
                }
                _ => {}
            }
            at += 2;
            continue;
        }
        at += 1;
    }
    false
}

/// The RegexpError a pattern Ruby refuses raises, checked before the engine
/// underneath is asked to compile it.
pub(crate) fn refuse_bad_pattern(source: &str, position: Position) -> Result<(), MetorexError> {
    let letters: Vec<char> = source.chars().collect();
    let refuse = |named: &str| {
        let message = format!("{}: /{}/", named, source);
        Err(crate::vm::errors::simple_exception(
            "RegexpError",
            &message,
            position,
        ))
    };
    let mut at = 0;
    let mut class_opened = false;
    let mut depth = 0usize;
    while at < letters.len() {
        match letters[at] {
            '\\' => {
                let Some(escaped) = letters.get(at + 1) else {
                    return refuse("too short escape sequence");
                };
                match escaped {
                    'x' => {
                        let digits = letters[at + 2..]
                            .iter()
                            .take(2)
                            .take_while(|held| held.is_ascii_hexdigit())
                            .count();
                        if digits == 0 {
                            return refuse("invalid hex escape");
                        }
                        at += 2 + digits;
                        continue;
                    }
                    'u' if letters.get(at + 2) == Some(&'{') => {
                        let Some(closing) = letters[at + 3..].iter().position(|held| *held == '}')
                        else {
                            return refuse("invalid Unicode list");
                        };
                        let inside = &letters[at + 3..at + 3 + closing];
                        if inside.is_empty()
                            || inside
                                .iter()
                                .any(|held| !held.is_ascii_hexdigit() && !held.is_whitespace())
                        {
                            return refuse("invalid Unicode list");
                        }
                        if inside
                            .iter()
                            .filter(|held| held.is_ascii_hexdigit())
                            .count()
                            > 6
                        {
                            return refuse("invalid Unicode range");
                        }
                        at += 4 + closing;
                        continue;
                    }
                    'u' => {
                        let digits = letters[at + 2..]
                            .iter()
                            .take(4)
                            .take_while(|held| held.is_ascii_hexdigit())
                            .count();
                        if digits < 4 {
                            return refuse("invalid Unicode escape");
                        }
                        at += 2 + digits;
                        continue;
                    }
                    _ => {
                        at += 2;
                        continue;
                    }
                }
            }
            '[' if !class_opened => class_opened = true,
            ']' if class_opened => class_opened = false,
            '(' if !class_opened => {
                // `(?#...)` is a comment, which runs to the first `)` and
                // carries no group of its own.
                if letters.get(at + 1) == Some(&'?') && letters.get(at + 2) == Some(&'#') {
                    let Some(closing) = letters[at + 3..].iter().position(|held| *held == ')')
                    else {
                        return refuse("end pattern with unmatched parenthesis");
                    };
                    at += 4 + closing;
                    continue;
                }
                if let Some(opener) = group_name_opener(&letters, at) {
                    let named: String = letters[at + 3..]
                        .iter()
                        .take_while(|held| **held != opener)
                        .collect();
                    if named.is_empty() {
                        return refuse("group name is empty");
                    }
                    if named.starts_with(|held: char| held.is_ascii_digit() || held == '-') {
                        return refuse(&format!("invalid group name <{named}>"));
                    }
                }
                depth += 1;
            }
            ')' if !class_opened => {
                if depth == 0 {
                    return refuse("unmatched close parenthesis");
                }
                depth -= 1;
            }
            _ => {}
        }
        at += 1;
    }
    if class_opened {
        return refuse("premature end of char-class");
    }
    if depth > 0 {
        return refuse("end pattern with unmatched parenthesis");
    }
    // Whatever the checks above let through, the engine still has to be able
    // to read the pattern before there is anything to match with.
    if let Err(trouble) = crate::vm::native_methods::regexp_methods::read_pattern(source, "") {
        return refuse(&trouble);
    }
    Ok(())
}

/// The character that closes the name of a group opening at `at`, for
/// `(?<name>` and `(?'name'`. A lookbehind is written `(?<=` and `(?<!`, so
/// those carry no name.
fn group_name_opener(letters: &[char], at: usize) -> Option<char> {
    if letters.get(at + 1) != Some(&'?') {
        return None;
    }
    match letters.get(at + 2) {
        Some('<') if !matches!(letters.get(at + 3), Some('=') | Some('!')) => Some('>'),
        Some('\'') => Some('\''),
        _ => None,
    }
}

/// The codepoints a `\u` escape names, and where the pattern carries on.
fn unicode_escape_points(letters: &[char], at: usize) -> (Vec<u32>, usize) {
    if letters.get(at) == Some(&'{') {
        let Some(closing) = letters[at + 1..].iter().position(|held| *held == '}') else {
            return (Vec::new(), at + 1);
        };
        let inside: String = letters[at + 1..at + 1 + closing].iter().collect();
        let points = inside
            .split_whitespace()
            .filter_map(|held| u32::from_str_radix(held, 16).ok())
            .collect();
        return (points, at + closing + 2);
    }
    let digits: String = letters[at..]
        .iter()
        .take(4)
        .take_while(|held| held.is_ascii_hexdigit())
        .collect();
    let counted = digits.len();
    (
        u32::from_str_radix(&digits, 16).ok().into_iter().collect(),
        at + counted,
    )
}
