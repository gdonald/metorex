// Whether a path matches a glob pattern, the way `File.fnmatch`
// reads both.

/// The flags `File.fnmatch` reads.
const FNM_NOESCAPE: i64 = 1;
const FNM_PATHNAME: i64 = 2;
const FNM_DOTMATCH: i64 = 4;
const FNM_CASEFOLD: i64 = 8;
const FNM_EXTGLOB: i64 = 16;

/// Whether a path matches a glob pattern, the way `File.fnmatch` reads both.
pub(crate) fn path_matches_pattern(pattern: &str, path: &str, flags: i64) -> bool {
    let patterns = if flags & FNM_EXTGLOB != 0 {
        expanded_braces(pattern, flags)
    } else {
        vec![pattern.to_string()]
    };
    patterns
        .iter()
        .any(|written| pattern_reaches(written, path, flags))
}

/// One pattern against one path, split into directory pieces when the flags
/// say a separator has to line up with a separator.
fn pattern_reaches(pattern: &str, path: &str, flags: i64) -> bool {
    if flags & FNM_PATHNAME == 0 {
        return segment_matches(pattern, path, flags);
    }
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let path_parts: Vec<&str> = path.split('/').collect();
    walk_segments(&pattern_parts, &path_parts, flags)
}

/// Match the directory pieces, where `**` followed by a separator stands for
/// any number of directories.
fn walk_segments(pattern: &[&str], path: &[&str], flags: i64) -> bool {
    let Some((first, rest)) = pattern.split_first() else {
        return path.is_empty();
    };
    if *first == "**" && !rest.is_empty() {
        for taken in 0..=path.len() {
            if taken > 0 {
                let swallowed = path[taken - 1];
                // A directory whose name opens with a period is not one a
                // wildcard reaches unless the flags say it is.
                if flags & FNM_DOTMATCH == 0 && swallowed.starts_with('.') {
                    return false;
                }
            }
            if walk_segments(rest, &path[taken..], flags) {
                return true;
            }
        }
        return false;
    }
    let Some((held, remaining)) = path.split_first() else {
        return false;
    };
    // A `**` with nothing after it reaches no further than a single `*`.
    let written = if *first == "**" { "*" } else { first };
    if !segment_matches(written, held, flags) {
        return false;
    }
    walk_segments(rest, remaining, flags)
}

/// One piece of the pattern against one piece of the path.
fn segment_matches(pattern: &str, text: &str, flags: i64) -> bool {
    let written: Vec<char> = pattern.chars().collect();
    let held: Vec<char> = text.chars().collect();
    // A name opening with a period is reached only by a pattern that opens
    // with one of its own.
    if flags & FNM_DOTMATCH == 0
        && held.first() == Some(&'.')
        && leading_letter(&written, flags) != Some('.')
    {
        return false;
    }
    letters_match(&written, &held, flags)
}

/// The character a pattern opens with, reading an escape as the character it
/// stands for.
fn leading_letter(pattern: &[char], flags: i64) -> Option<char> {
    match pattern.first() {
        Some('\\') if flags & FNM_NOESCAPE == 0 => pattern.get(1).copied(),
        other => other.copied(),
    }
}

/// The wildcard walk itself, over the characters of one piece.
fn letters_match(pattern: &[char], text: &[char], flags: i64) -> bool {
    let mut at = 0;
    let mut held = 0;
    while at < pattern.len() {
        match pattern[at] {
            '*' => {
                // A run of stars reaches the same as one.
                while pattern.get(at) == Some(&'*') {
                    at += 1;
                }
                if at == pattern.len() {
                    return !(flags & FNM_PATHNAME != 0 && text[held..].contains(&'/'));
                }
                for taken in held..=text.len() {
                    if flags & FNM_PATHNAME != 0 && text[held..taken].contains(&'/') {
                        break;
                    }
                    if letters_match(&pattern[at..], &text[taken..], flags) {
                        return true;
                    }
                }
                return false;
            }
            '?' => {
                if held >= text.len() || (flags & FNM_PATHNAME != 0 && text[held] == '/') {
                    return false;
                }
                at += 1;
                held += 1;
            }
            '[' => {
                if held >= text.len() {
                    return false;
                }
                let Some((reached, next)) = bracket_reaches(pattern, at, text[held], flags) else {
                    return false;
                };
                if !reached {
                    return false;
                }
                at = next;
                held += 1;
            }
            '\\' if flags & FNM_NOESCAPE == 0 && at + 1 < pattern.len() => {
                if held >= text.len() || !same_letter(pattern[at + 1], text[held], flags) {
                    return false;
                }
                at += 2;
                held += 1;
            }
            letter => {
                if held >= text.len() || !same_letter(letter, text[held], flags) {
                    return false;
                }
                at += 1;
                held += 1;
            }
        }
    }
    held == text.len()
}

/// Whether two characters stand for the same one, folding case when the
/// flags say to.
fn same_letter(written: char, held: char, flags: i64) -> bool {
    if flags & FNM_CASEFOLD != 0 {
        return written.to_lowercase().eq(held.to_lowercase());
    }
    written == held
}

/// Whether a bracket expression reaches a character, and where the pattern
/// carries on. None when the bracket is never closed.
pub(crate) fn bracket_reaches(
    pattern: &[char],
    at: usize,
    held: char,
    flags: i64,
) -> Option<(bool, usize)> {
    let mut cursor = at + 1;
    let negated = matches!(pattern.get(cursor), Some('^') | Some('!'));
    if negated {
        cursor += 1;
    }
    let mut reached = false;
    let mut first = true;
    while cursor < pattern.len() {
        if pattern[cursor] == ']' && !first {
            let answer = reached != negated;
            // A separator is never reached through a bracket when the flags
            // say a separator has to line up with a separator.
            if flags & FNM_PATHNAME != 0 && held == '/' {
                return Some((false, cursor + 1));
            }
            return Some((answer, cursor + 1));
        }
        first = false;
        let (low, next) = bracket_letter(pattern, cursor, flags)?;
        cursor = next;
        if pattern.get(cursor) == Some(&'-')
            && pattern.get(cursor + 1).is_some_and(|held| *held != ']')
        {
            let (high, after) = bracket_letter(pattern, cursor + 1, flags)?;
            cursor = after;
            if letter_in_range(held, low, high, flags) {
                reached = true;
            }
            continue;
        }
        if same_letter(low, held, flags) {
            reached = true;
        }
    }
    None
}

/// One character of a bracket expression, reading an escape as the character
/// it stands for.
fn bracket_letter(pattern: &[char], at: usize, flags: i64) -> Option<(char, usize)> {
    match pattern.get(at)? {
        '\\' if flags & FNM_NOESCAPE == 0 => Some((*pattern.get(at + 1)?, at + 2)),
        letter => Some((*letter, at + 1)),
    }
}

/// Whether a character falls between two others, folding case when the flags
/// say to.
fn letter_in_range(held: char, low: char, high: char, flags: i64) -> bool {
    if low <= held && held <= high {
        return true;
    }
    if flags & FNM_CASEFOLD == 0 {
        return false;
    }
    let folded: Vec<char> = held.to_lowercase().chain(held.to_uppercase()).collect();
    folded
        .iter()
        .any(|letter| low <= *letter && *letter <= high)
}

/// The patterns a `{a,b}` alternation stands for, with nesting and escapes
/// read the way the alternation itself is.
fn expanded_braces(pattern: &str, flags: i64) -> Vec<String> {
    let letters: Vec<char> = pattern.chars().collect();
    let Some(opening) = unescaped_brace(&letters, flags) else {
        return vec![pattern.to_string()];
    };
    let Some(closing) = matching_brace(&letters, opening, flags) else {
        return vec![pattern.to_string()];
    };
    let before: String = letters[..opening].iter().collect();
    let after: String = letters[closing + 1..].iter().collect();
    let inside: Vec<char> = letters[opening + 1..closing].to_vec();
    let mut made = Vec::new();
    for part in brace_alternatives(&inside, flags) {
        let joined = format!("{}{}{}", before, part, after);
        made.extend(expanded_braces(&joined, flags));
    }
    made
}

/// Where the first alternation opens, skipping anything escaped.
fn unescaped_brace(letters: &[char], flags: i64) -> Option<usize> {
    let mut at = 0;
    while at < letters.len() {
        if letters[at] == '\\' && flags & FNM_NOESCAPE == 0 {
            at += 2;
            continue;
        }
        if letters[at] == '{' {
            return Some(at);
        }
        at += 1;
    }
    None
}

/// Where the alternation that opened at `opening` closes.
fn matching_brace(letters: &[char], opening: usize, flags: i64) -> Option<usize> {
    let mut depth = 0;
    let mut at = opening;
    while at < letters.len() {
        if letters[at] == '\\' && flags & FNM_NOESCAPE == 0 {
            at += 2;
            continue;
        }
        match letters[at] {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(at);
                }
            }
            _ => {}
        }
        at += 1;
    }
    None
}

/// The pieces an alternation is written in, split at the commas that are not
/// inside a nested one.
fn brace_alternatives(inside: &[char], flags: i64) -> Vec<String> {
    let mut parts = Vec::new();
    let mut held = String::new();
    let mut depth = 0;
    let mut at = 0;
    while at < inside.len() {
        if inside[at] == '\\' && flags & FNM_NOESCAPE == 0 && at + 1 < inside.len() {
            held.push(inside[at]);
            held.push(inside[at + 1]);
            at += 2;
            continue;
        }
        match inside[at] {
            '{' => {
                depth += 1;
                held.push('{');
            }
            '}' => {
                depth -= 1;
                held.push('}');
            }
            ',' if depth == 0 => {
                parts.push(std::mem::take(&mut held));
            }
            letter => held.push(letter),
        }
        at += 1;
    }
    parts.push(held);
    parts
}
