// The body of a `[...]`, and the items a span stands for.

use super::*;

impl Parser {
    /// The body of a `[...]`, with the cursor just past the opening bracket.
    pub(crate) fn parse_class(&mut self) -> Read<Class> {
        let negated = self.eat('^');
        let mut parts: Vec<Vec<ClassItem>> = Vec::new();
        let mut items: Vec<ClassItem> = Vec::new();
        let mut first = true;
        loop {
            // A `]` written first is an ordinary character rather than the
            // one that closes the class.
            if self.peek() == Some(']') && !first {
                self.at += 1;
                break;
            }
            first = false;
            let Some(letter) = self.peek() else {
                return Err(Trouble("premature end of char-class".to_string()));
            };
            if letter == '&' && self.peek_at(1) == Some('&') {
                self.at += 2;
                parts.push(std::mem::take(&mut items));
                first = true;
                continue;
            }
            if letter == '['
                && self.peek_at(1) == Some(':')
                && let Some(named) = self.read_posix_class()?
            {
                if self.peek() == Some('-') && !matches!(self.peek_at(1), None | Some(']')) {
                    return Err(Trouble("char-class must not be a range start".to_string()));
                }
                items.push(named);
                continue;
            }
            if letter == '[' {
                self.at += 1;
                let nested = self.parse_class()?;
                items.push(ClassItem::Nested(Box::new(nested)));
                continue;
            }
            let low = match self.class_member()? {
                Member::Letter(held) => held,
                Member::Named(name, negated) => {
                    // A named run stands for many characters, so it names
                    // neither end of a span.
                    if self.peek() == Some('-') && !matches!(self.peek_at(1), None | Some(']')) {
                        return Err(Trouble("char-class must not be a range start".to_string()));
                    }
                    items.push(ClassItem::Named(name, negated));
                    continue;
                }
            };
            // A `-` between two characters spans them, while one written last
            // is an ordinary character.
            if self.peek() == Some('-')
                && !matches!(self.peek_at(1), None | Some(']'))
                && !(self.peek_at(1) == Some('&') && self.peek_at(2) == Some('&'))
            {
                self.at += 1;
                match self.class_member()? {
                    Member::Letter(high) => {
                        if high < low {
                            return Err(Trouble("empty range in char class".to_string()));
                        }
                        items.extend(span_items(low, high, self.flags.folded));
                        continue;
                    }
                    Member::Named(_, _) => {
                        return Err(Trouble("char-class must not be a range end".to_string()));
                    }
                }
            }
            if self.flags.folded {
                items.extend(folded_letters(low));
            } else {
                items.push(ClassItem::Letter(low));
            }
        }
        parts.push(items);
        Ok(Class {
            negated,
            ascii_only: self.flags.ascii_classes,
            parts,
        })
    }

    /// `[:alpha:]` and its negated form, or None when the brackets open a
    /// nested class instead.
    pub(crate) fn read_posix_class(&mut self) -> Read<Option<ClassItem>> {
        let opened = self.at;
        self.at += 2;
        let negated = self.eat('^');
        let mut name = String::new();
        while let Some(letter) = self.peek() {
            if !letter.is_ascii_alphabetic() {
                break;
            }
            name.push(letter);
            self.at += 1;
        }
        if !self.eat_run(":]") {
            self.at = opened;
            return Ok(None);
        }
        match posix_run(&name) {
            Some(named) => Ok(Some(ClassItem::Named(named, negated))),
            None => Err(Trouble(format!("invalid POSIX bracket type: {name}"))),
        }
    }

    /// One character inside a class, or the run a `\d`-style escape names.
    pub(crate) fn class_member(&mut self) -> Read<Member> {
        if self.peek() != Some('\\') {
            let held = self
                .take()
                .ok_or_else(|| Trouble("premature end of char-class".to_string()))?;
            return Ok(Member::Letter(held));
        }
        match self.peek_at(1) {
            Some(letter @ ('d' | 'D' | 'w' | 'W' | 's' | 'S' | 'h' | 'H')) => {
                self.at += 2;
                Ok(Member::Named(self.named_run(letter), letter.is_uppercase()))
            }
            Some(letter @ ('p' | 'P')) => {
                self.at += 2;
                let (name, negated) = self.read_property()?;
                Ok(Member::Named(
                    Named::Property(name),
                    negated != (letter == 'P'),
                ))
            }
            Some('b') => {
                self.at += 2;
                Ok(Member::Letter('\u{8}'))
            }
            _ => Ok(Member::Letter(self.read_escaped_letter()?)),
        }
    }
}

/// What reading one item of a character class answered.
pub(crate) enum Member {
    Letter(char),
    Named(Named, bool),
}

/// The items a span stands for, folded when the pattern folds case. A folded
/// span covers the other case of every letter in it, which is more than
/// folding the two ends would give.
pub(crate) fn span_items(low: char, high: char, folded: bool) -> Vec<ClassItem> {
    let mut items = vec![ClassItem::Span(low, high)];
    if !folded {
        return items;
    }
    for point in (low as u32)..=(high as u32) {
        let Some(letter) = char::from_u32(point) else {
            continue;
        };
        for other in folded_letters(letter) {
            if let ClassItem::Letter(held) = other
                && (held < low || held > high)
            {
                items.push(ClassItem::Letter(held));
            }
        }
    }
    items
}

/// A letter together with the other cases it stands for.
pub fn folded_letters(letter: char) -> Vec<ClassItem> {
    let mut items = vec![ClassItem::Letter(letter)];
    for other in letter.to_lowercase().chain(letter.to_uppercase()) {
        if other != letter
            && !items
                .iter()
                .any(|held| matches!(held, ClassItem::Letter(had) if *had == other))
        {
            items.push(ClassItem::Letter(other));
        }
    }
    items
}

pub(crate) fn posix_run(name: &str) -> Option<Named> {
    Some(match name {
        "alpha" => Named::Alpha,
        "alnum" => Named::Alnum,
        "digit" => Named::Digit,
        "upper" => Named::Upper,
        "lower" => Named::Lower,
        "space" => Named::Space,
        "punct" => Named::Punct,
        "print" => Named::Print,
        "graph" => Named::Graph,
        "cntrl" => Named::Cntrl,
        "blank" => Named::Blank,
        "xdigit" => Named::XDigit,
        "ascii" => Named::Ascii,
        "word" => Named::Word,
        _ => return None,
    })
}

/// What a name written inside `\k<...>` or `\g<...>` points at.
pub(crate) fn name_target(written: &str) -> Read<Target> {
    if written.is_empty() {
        return Err(Trouble("invalid backref number/name".to_string()));
    }
    if let Some(rest) = written.strip_prefix('+')
        && let Ok(counted) = rest.parse::<isize>()
    {
        return Ok(Target::Relative(counted));
    }
    if written.starts_with('-')
        && let Ok(counted) = written.parse::<isize>()
    {
        return Ok(Target::Relative(counted));
    }
    if let Ok(counted) = written.parse::<usize>() {
        return Ok(Target::Numbered(counted));
    }
    Ok(Target::Named(written.to_string()))
}

/// What the condition of a `(?(...)...)` points at. A number stands on its
/// own, while a name has to be written inside `<>` or `''`.
pub(crate) fn condition_target(written: &str) -> Read<Target> {
    let inside = match (written.chars().next(), written.chars().last()) {
        (Some('<'), Some('>')) | (Some('\''), Some('\'')) if written.len() >= 2 => {
            &written[1..written.len() - 1]
        }
        _ => {
            return name_target(written).and_then(|target| match target {
                Target::Named(_) => Err(Trouble("invalid group name".to_string())),
                held => Ok(held),
            });
        }
    };
    name_target(inside)
}
