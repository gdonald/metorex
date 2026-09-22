// The string that follows this one.

/// Ruby's `String#succ`: the rightmost alphanumeric character is bumped, and a
/// carry moves left, growing the string when the leftmost one wraps. A string
/// with no alphanumeric character bumps its last byte instead.
pub(crate) fn successor_of(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let mut letters: Vec<char> = text.chars().collect();
    let alphanumeric: Vec<usize> = letters
        .iter()
        .enumerate()
        .filter(|(_, letter)| letter.is_ascii_alphanumeric())
        .map(|(index, _)| index)
        .collect();
    if alphanumeric.is_empty() {
        // With no letters or digits the characters count up as the bytes
        // they stand for, carrying from the end. A carry past the front puts
        // one more character there.
        if letters.iter().all(|letter| (*letter as u32) < 256) {
            let mut at = letters.len();
            loop {
                if at == 0 {
                    letters.insert(0, '\u{1}');
                    break;
                }
                at -= 1;
                if letters[at] == '\u{ff}' {
                    letters[at] = '\0';
                    continue;
                }
                letters[at] = char::from_u32(letters[at] as u32 + 1).unwrap_or(letters[at]);
                break;
            }
            return letters.into_iter().collect();
        }
        let last = letters.len() - 1;
        let bumped = (letters[last] as u32).wrapping_add(1);
        if let Some(letter) = char::from_u32(bumped) {
            letters[last] = letter;
        }
        return letters.into_iter().collect();
    }
    for (step, position) in alphanumeric.iter().rev().enumerate() {
        let (next, carried) = bump(letters[*position]);
        letters[*position] = next;
        if !carried {
            return letters.into_iter().collect();
        }
        if step + 1 == alphanumeric.len() {
            // Every character carried, so one more is prepended: "zz" grows
            // into "aaa" and "99" into "100".
            let leading = if letters[*position].is_ascii_digit() {
                '1'
            } else {
                letters[*position]
            };
            letters.insert(*position, leading);
        }
    }
    letters.into_iter().collect()
}

/// One character of a `succ`, answering what it becomes and whether the bump
/// carried past the end of its run.
pub(crate) fn bump(letter: char) -> (char, bool) {
    match letter {
        'z' => ('a', true),
        'Z' => ('A', true),
        '9' => ('0', true),
        other => (char::from_u32(other as u32 + 1).unwrap_or(other), false),
    }
}
