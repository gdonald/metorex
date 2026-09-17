//! The pattern language `Dir.glob` reads a directory tree with: `*` and `?`
//! within one name, `[...]` for a set of characters, `{a,b}` for a list of
//! patterns, and `**` for a walk down through the subdirectories.

use std::path::{Path, PathBuf};

/// What a glob is allowed to match.
pub(crate) struct Options {
    /// Whether a name opening with a dot is matched by a pattern that does
    /// not open with one.
    pub(crate) dotmatch: bool,
    /// Whether a backslash stands for itself rather than escaping the
    /// character after it.
    pub(crate) noescape: bool,
    /// Whether two names that differ only in case match.
    pub(crate) casefold: bool,
}

/// Every pattern a written one stands for, with each `{a,b}` spread out in
/// the order it was written.
pub(crate) fn brace_expansions(pattern: &str) -> Vec<String> {
    let characters: Vec<char> = pattern.chars().collect();
    let Some((opens, closes)) = first_brace(&characters) else {
        return vec![pattern.to_string()];
    };
    let before: String = characters[..opens].iter().collect();
    let after: String = characters[closes + 1..].iter().collect();
    let inside = &characters[opens + 1..closes];
    let mut spread = Vec::new();
    for choice in brace_choices(inside) {
        spread.extend(brace_expansions(&format!("{before}{choice}{after}")));
    }
    spread
}

/// Where the first list written with braces opens and closes, counting the
/// lists written inside it.
fn first_brace(characters: &[char]) -> Option<(usize, usize)> {
    let mut at = 0;
    let mut opens = None;
    let mut depth = 0usize;
    while at < characters.len() {
        match characters[at] {
            '\\' => at += 1,
            '{' => {
                if depth == 0 {
                    opens = Some(at);
                }
                depth += 1;
            }
            '}' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    return opens.map(|held| (held, at));
                }
            }
            _ => {}
        }
        at += 1;
    }
    None
}

/// The pieces a list written with braces names, cut on the commas that stand
/// outside any list written inside it.
fn brace_choices(inside: &[char]) -> Vec<String> {
    let mut pieces = Vec::new();
    let mut held = String::new();
    let mut depth = 0usize;
    let mut at = 0;
    while at < inside.len() {
        let character = inside[at];
        match character {
            '\\' => {
                held.push(character);
                if at + 1 < inside.len() {
                    at += 1;
                    held.push(inside[at]);
                }
            }
            '{' => {
                depth += 1;
                held.push(character);
            }
            '}' => {
                depth = depth.saturating_sub(1);
                held.push(character);
            }
            ',' if depth == 0 => {
                pieces.push(std::mem::take(&mut held));
            }
            _ => held.push(character),
        }
        at += 1;
    }
    pieces.push(held);
    pieces
}

/// Whether one name matches one piece of a pattern.
pub(crate) fn segment_matches(name: &str, pattern: &str, options: &Options) -> bool {
    let name: Vec<char> = if options.casefold {
        name.to_lowercase().chars().collect()
    } else {
        name.chars().collect()
    };
    let pattern: Vec<char> = if options.casefold {
        pattern.to_lowercase().chars().collect()
    } else {
        pattern.chars().collect()
    };
    matches_from(&name, 0, &pattern, 0, options)
}

/// Whether the rest of a name matches the rest of a pattern.
fn matches_from(
    name: &[char],
    mut at: usize,
    pattern: &[char],
    mut pat_at: usize,
    options: &Options,
) -> bool {
    while pat_at < pattern.len() {
        match pattern[pat_at] {
            '*' => {
                // A run stands for any number of characters, so every place
                // the rest could carry on from is tried.
                while pat_at + 1 < pattern.len() && pattern[pat_at + 1] == '*' {
                    pat_at += 1;
                }
                if pat_at + 1 == pattern.len() {
                    return true;
                }
                for skip in at..=name.len() {
                    if matches_from(name, skip, pattern, pat_at + 1, options) {
                        return true;
                    }
                }
                return false;
            }
            '?' => {
                if at >= name.len() {
                    return false;
                }
                at += 1;
                pat_at += 1;
            }
            '[' => {
                if at >= name.len() {
                    return false;
                }
                let Some((held, after)) = set_matches(name[at], pattern, pat_at) else {
                    // An unclosed bracket stands for itself.
                    if name[at] != '[' {
                        return false;
                    }
                    at += 1;
                    pat_at += 1;
                    continue;
                };
                if !held {
                    return false;
                }
                at += 1;
                pat_at = after;
            }
            '\\' if !options.noescape && pat_at + 1 < pattern.len() => {
                if at >= name.len() || name[at] != pattern[pat_at + 1] {
                    return false;
                }
                at += 1;
                pat_at += 2;
            }
            held => {
                if at >= name.len() || name[at] != held {
                    return false;
                }
                at += 1;
                pat_at += 1;
            }
        }
    }
    at == name.len()
}

/// Whether a character is one of those a set names, and where the set ends.
/// None where the set never closes.
fn set_matches(held: char, pattern: &[char], opens: usize) -> Option<(bool, usize)> {
    let mut at = opens + 1;
    let negated = matches!(pattern.get(at), Some('^') | Some('!'));
    if negated {
        at += 1;
    }
    let mut found = false;
    let mut first = true;
    while at < pattern.len() {
        if pattern[at] == ']' && !first {
            return Some((found != negated, at + 1));
        }
        first = false;
        let low = pattern[at];
        // A dash between two characters names every character between them.
        if at + 2 < pattern.len() && pattern[at + 1] == '-' && pattern[at + 2] != ']' {
            let high = pattern[at + 2];
            if low <= held && held <= high {
                found = true;
            }
            at += 3;
            continue;
        }
        if low == held {
            found = true;
        }
        at += 1;
    }
    None
}

/// Whether a name opening with a dot is one this pattern may match.
fn dot_allowed(name: &str, pattern: &str, options: &Options) -> bool {
    if !name.starts_with('.') {
        return true;
    }
    options.dotmatch || pattern.starts_with('.')
}

/// The names a directory holds, in order, with `.` standing for the directory
/// itself. A directory that cannot be read holds nothing.
fn names_in(directory: &Path) -> Vec<String> {
    let Ok(reading) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut held: Vec<String> = reading
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();
    held.sort();
    held
}

/// Whether a path names a directory, following a link to one.
fn is_directory(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|held| held.is_dir())
        .unwrap_or(false)
}

/// Whether a path names a directory that is not a link, which is what a walk
/// down through the subdirectories follows.
fn is_plain_directory(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|held| held.is_dir())
        .unwrap_or(false)
}

/// Where a step stands in the pattern. A directory answers itself only where
/// the pattern has not walked down into it, and a name opening with a dot is
/// reached by a piece that opens with one only before any walk down.
#[derive(Clone, Copy)]
struct Place {
    at_self: bool,
    before_walk: bool,
}

/// The state a walk carries: where it started, what it is allowed to match,
/// and what it has found.
struct Walk<'a> {
    options: &'a Options,
    found: Vec<String>,
}

impl Walk<'_> {
    /// Match the rest of a pattern against the names a directory holds.
    fn step(&mut self, directory: &Path, prefix: &str, segments: &[&str], place: Place) {
        let Some(segment) = segments.first().copied() else {
            return;
        };
        let rest = &segments[1..];
        let only_directories = rest == [""];
        let last = rest.is_empty() || only_directories;

        if segment.is_empty() {
            // An empty piece comes from a doubled separator, which keeps the
            // place and the separator alike.
            self.step(directory, &format!("{prefix}/"), rest, place);
            return;
        }
        // A piece naming the directory itself takes no step down.
        if segment == "." && !last {
            self.step(directory, &format!("{prefix}./"), rest, place);
            return;
        }
        if segment == "**" && !rest.is_empty() {
            self.step_through(directory, prefix, rest, only_directories, place);
            return;
        }
        let segment = if segment == "**" { "*" } else { segment };
        for name in names_with_self(directory, segment, self.options, place) {
            if !dot_allowed(&name, segment, self.options) {
                continue;
            }
            if !segment_matches(&name, segment, self.options) {
                continue;
            }
            let path = directory.join(&name);
            if last {
                if only_directories {
                    if is_directory(&path) {
                        self.found.push(format!("{prefix}{name}/"));
                    }
                } else {
                    self.found.push(format!("{prefix}{name}"));
                }
                continue;
            }
            if is_directory(&path) {
                self.step(&path, &format!("{prefix}{name}/"), rest, place);
            }
        }
    }

    /// Match the rest of a pattern against this directory and every one
    /// beneath it, which is what `**/` asks for.
    fn step_through(
        &mut self,
        directory: &Path,
        prefix: &str,
        rest: &[&str],
        only_directories: bool,
        place: Place,
    ) {
        if only_directories {
            if !prefix.is_empty() {
                self.found.push(prefix.to_string());
            }
        } else {
            self.step(
                directory,
                prefix,
                rest,
                Place {
                    at_self: place.at_self,
                    before_walk: false,
                },
            );
        }
        let deeper = Place {
            at_self: false,
            before_walk: false,
        };
        for name in names_in(directory) {
            let path = directory.join(&name);
            if !is_plain_directory(&path) {
                continue;
            }
            if !dot_allowed(&name, "*", self.options) {
                continue;
            }
            let held = format!("{prefix}{name}/");
            self.step_through(&path, &held, rest, only_directories, deeper);
        }
    }
}

/// The names a directory holds, with `.` added where the pattern may reach
/// it. Only the directory a pattern opens against answers itself.
fn names_with_self(
    directory: &Path,
    segment: &str,
    options: &Options,
    place: Place,
) -> Vec<String> {
    let mut held = names_in(directory);
    let asks = options.dotmatch || (place.before_walk && segment.starts_with('.'));
    if place.at_self && asks {
        held.push(".".to_string());
        held.sort();
    }
    held
}

/// Every path a pattern names, read against `root`. The paths answered are
/// written the way the pattern was, so a pattern with no directory in it
/// answers plain names.
pub(crate) fn matching_paths(root: &Path, pattern: &str, options: &Options) -> Vec<String> {
    if pattern.is_empty() {
        return Vec::new();
    }
    // A pattern opening with a separator is read from the root of the
    // filesystem rather than from where the program stands.
    let (start, pattern, prefix) = match pattern.strip_prefix('/') {
        Some(rest) => (PathBuf::from("/"), rest.to_string(), "/".to_string()),
        None => (root.to_path_buf(), pattern.to_string(), String::new()),
    };
    let segments: Vec<&str> = pattern.split('/').collect();
    let mut walk = Walk {
        options,
        found: Vec::new(),
    };
    walk.step(
        &start,
        &prefix,
        &segments,
        Place {
            at_self: true,
            before_walk: true,
        },
    );
    walk.found
}
