// One `tr`-style character set, and what it names.

/// One `tr`-style character set: the characters it names, and whether the set
/// was written with a leading `^` and so means every character but those.
pub(crate) struct CharacterSet {
    members: Vec<char>,
    negated: bool,
}

impl CharacterSet {
    /// Read a set written the way `tr`, `count`, `delete`, and `squeeze` take
    /// one: `a-z` is a range, a leading `^` negates, and `\` escapes the
    /// character after it. A range whose end comes before its start is
    /// refused, naming the range as Ruby names it.
    pub(crate) fn parse(source: &str) -> Result<Self, String> {
        let characters: Vec<char> = source.chars().collect();
        let negated = characters.len() > 1 && characters[0] == '^';
        let mut members = Vec::new();
        let mut index = if negated { 1 } else { 0 };
        while index < characters.len() {
            if characters[index] == '\\' && index + 1 < characters.len() {
                members.push(characters[index + 1]);
                index += 2;
                continue;
            }
            // `a-z` names every character between the two ends, while a `-`
            // at either end of the set is a character of its own.
            if index + 2 < characters.len() && characters[index + 1] == '-' {
                let (start, end) = (characters[index], characters[index + 2]);
                if start > end {
                    return Err(format!(
                        "invalid range \"{}-{}\" in string transliteration",
                        start, end
                    ));
                }
                for point in (start as u32)..=(end as u32) {
                    if let Some(member) = char::from_u32(point) {
                        members.push(member);
                    }
                }
                index += 3;
                continue;
            }
            members.push(characters[index]);
            index += 1;
        }
        Ok(Self { members, negated })
    }

    pub(crate) fn holds(&self, character: char) -> bool {
        self.members.contains(&character) != self.negated
    }
}

/// Whether every set given holds the character, which is how `count`,
/// `delete`, and `squeeze` read more than one.
pub(crate) fn all_hold(sets: &[CharacterSet], character: char) -> bool {
    sets.iter().all(|set| set.holds(character))
}

/// The replacement `tr` puts in place of a character, or None when the
/// destination set is empty and the character is dropped instead.
pub(crate) fn translation_for(
    from: &CharacterSet,
    to: &CharacterSet,
    character: char,
) -> Option<char> {
    if to.members.is_empty() {
        return None;
    }
    if from.negated {
        return Some(*to.members.last().expect("a non-empty destination set"));
    }
    let at = from
        .members
        .iter()
        .position(|member| *member == character)?;
    Some(
        *to.members
            .get(at)
            .unwrap_or_else(|| to.members.last().expect("a non-empty destination set")),
    )
}
