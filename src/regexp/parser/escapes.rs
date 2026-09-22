// The character or the run a backslash escape names.

use super::*;

impl Parser {
    pub(crate) fn close_group(&mut self) -> Read<()> {
        self.skip_ignored();
        if self.eat(')') {
            return Ok(());
        }
        Err(Trouble(
            "end pattern with unmatched parenthesis".to_string(),
        ))
    }

    pub(crate) fn next_group(&mut self, name: Option<String>) -> usize {
        self.any_named = self.any_named || name.is_some();
        self.groups += 1;
        self.names.push(name);
        self.groups
    }

    pub(crate) fn parse_escape(&mut self) -> Read<Node> {
        self.at += 1;
        let Some(letter) = self.take() else {
            return Err(Trouble("too short escape sequence".to_string()));
        };
        match letter {
            'A' => Ok(Node::SubjectStart),
            'z' => Ok(Node::SubjectEnd),
            'Z' => Ok(Node::SubjectEndBeforeNewline),
            'G' => Ok(Node::SearchStart),
            'K' => Ok(Node::KeepOut),
            'b' => Ok(Node::WordBoundary { negated: false }),
            'B' => Ok(Node::WordBoundary { negated: true }),
            'R' => Ok(Node::alternate(vec![
                Node::concat(vec![Node::Letter('\r'), Node::Letter('\n')]),
                Node::Class(Class {
                    negated: false,
                    ascii_only: false,
                    parts: vec![vec![
                        ClassItem::Letter('\n'),
                        ClassItem::Letter('\r'),
                        ClassItem::Letter('\u{b}'),
                        ClassItem::Letter('\u{c}'),
                        ClassItem::Letter('\u{85}'),
                        ClassItem::Letter('\u{2028}'),
                        ClassItem::Letter('\u{2029}'),
                    ]],
                }),
            ])),
            'X' => Ok(Node::Grapheme),
            'd' | 'D' | 'w' | 'W' | 's' | 'S' | 'h' | 'H' => Ok(Node::Class(Class {
                negated: letter.is_uppercase(),
                ascii_only: false,
                parts: vec![vec![ClassItem::Named(self.named_run(letter), false)]],
            })),
            'p' | 'P' => {
                let (name, negated) = self.read_property()?;
                Ok(Node::Class(Class {
                    negated: negated != (letter == 'P'),
                    ascii_only: false,
                    parts: vec![vec![ClassItem::Named(Named::Property(name), false)]],
                }))
            }
            'k' => {
                let target = self.read_reference_name()?;
                let (target, level) = self.with_level_split(target)?;
                self.refuse_bad_reference(&target)?;
                Ok(Node::Backreference {
                    target,
                    folded: self.flags.folded,
                    level,
                })
            }
            'g' => {
                let target = self.read_reference_name()?;
                Ok(Node::Call(target))
            }
            '1'..='9' => {
                let mut digits = letter.to_string();
                while let Some(next) = self.peek() {
                    if !next.is_ascii_digit() {
                        break;
                    }
                    digits.push(next);
                    self.at += 1;
                }
                let counted: usize = digits.parse().unwrap_or(0);
                // A number names a group only once that group has been
                // opened. Before then the digits spell a character: an octal
                // escape where they can, and themselves where they cannot.
                // A single digit names a group written anywhere in the
                // pattern, while a longer number names only one already
                // opened.
                if counted <= self.groups || digits.len() == 1 {
                    if self.any_named {
                        return Err(Trouble(
                            "numbered backref/call is not allowed. (use name)".to_string(),
                        ));
                    }
                    return Ok(Node::Backreference {
                        target: Target::Numbered(counted),
                        folded: self.flags.folded,
                        level: None,
                    });
                }
                if ('0'..='7').contains(&letter) {
                    let point = u32::from_str_radix(&digits, 8).unwrap_or(0);
                    let held = char::from_u32(point)
                        .ok_or_else(|| Trouble("invalid octal escape".to_string()))?;
                    return Ok(self.letter_node(held));
                }
                Ok(Node::concat(
                    digits.chars().map(|held| self.letter_node(held)).collect(),
                ))
            }
            _ => {
                // The cursor sits past the letter, and reading the escape
                // starts from the backslash that opened it.
                self.at -= 2;
                let held = self.read_escaped_letter()?;
                Ok(self.letter_node(held))
            }
        }
    }

    /// The run a backslash escape names. It covers ASCII alone unless the
    /// pattern asked for every character the name fits.
    pub(crate) fn named_run(&self, letter: char) -> Named {
        let wide = self.flags.unicode_escapes;
        match letter.to_ascii_lowercase() {
            'd' if wide => Named::Digit,
            'd' => Named::AsciiDigit,
            'w' if wide => Named::Word,
            'w' => Named::AsciiWord,
            'h' => Named::XDigit,
            _ if wide => Named::Space,
            _ => Named::AsciiSpace,
        }
    }

    /// The name inside `\p{...}`, and whether the `^` in front of it says the
    /// property stands for everything outside itself.
    pub(crate) fn read_property(&mut self) -> Read<(String, bool)> {
        if !self.eat('{') {
            return Err(Trouble("invalid character property".to_string()));
        }
        let negated = self.eat('^');
        let mut name = String::new();
        loop {
            match self.take() {
                Some('}') => break,
                Some(letter) => name.push(letter),
                None => return Err(Trouble("invalid character property".to_string())),
            }
        }
        Ok((name, negated))
    }

    /// What `\k<...>` or `\g<...>` points at.
    pub(crate) fn read_reference_name(&mut self) -> Read<Target> {
        let closer = match self.take() {
            Some('<') => '>',
            Some('\'') => '\'',
            _ => return Err(Trouble("invalid backref number/name".to_string())),
        };
        let mut written = String::new();
        loop {
            match self.take() {
                Some(letter) if letter == closer => break,
                Some(letter) => written.push(letter),
                None => return Err(Trouble("invalid backref number/name".to_string())),
            }
        }
        name_target(&written)
    }

    /// A `\k<name+1>` names a group together with how deep a call it belongs
    /// to. The level is not part of the name, so it is taken off, and what is
    /// left has to name a group the pattern wrote.
    pub(crate) fn with_level_split(&self, target: Target) -> Read<(Target, Option<isize>)> {
        let Target::Named(written) = &target else {
            return Ok((target, None));
        };
        let Some(at) = written.rfind(['+', '-']) else {
            if self
                .names
                .iter()
                .any(|held| held.as_deref() == Some(written.as_str()))
            {
                return Ok((target, None));
            }
            return Err(Trouble("invalid backref number/name".to_string()));
        };
        let (name, level) = written.split_at(at);
        if name.is_empty() || !level[1..].chars().all(|held| held.is_ascii_digit()) {
            return Err(Trouble("invalid backref number/name".to_string()));
        }
        if level[1..].is_empty() {
            return Err(Trouble("invalid backref number/name".to_string()));
        }
        if !self.names.iter().any(|held| held.as_deref() == Some(name)) {
            return Err(Trouble("invalid backref number/name".to_string()));
        }
        let counted: isize = level[1..].parse().unwrap_or(0);
        let signed = if level.starts_with('-') {
            -counted
        } else {
            counted
        };
        Ok((Target::Named(name.to_string()), Some(signed)))
    }

    /// Whether a reference points at a group there could be.
    pub(crate) fn refuse_bad_reference(&self, target: &Target) -> Read<()> {
        match target {
            Target::Numbered(0) => Err(Trouble("invalid backref number/name".to_string())),
            Target::Numbered(_) if self.any_named => Err(Trouble(
                "numbered backref/call is not allowed. (use name)".to_string(),
            )),
            _ => Ok(()),
        }
    }

    /// The character an escape names, with the cursor sitting on the
    /// backslash.
    pub(crate) fn read_escaped_letter(&mut self) -> Read<char> {
        self.at += 1;
        let Some(letter) = self.take() else {
            return Err(Trouble("too short escape sequence".to_string()));
        };
        match letter {
            'n' => Ok('\n'),
            't' => Ok('\t'),
            'r' => Ok('\r'),
            'f' => Ok('\u{c}'),
            'v' => Ok('\u{b}'),
            'a' => Ok('\u{7}'),
            'e' => Ok('\u{1b}'),
            '0'..='7' => {
                // An octal escape names a character by up to three digits,
                // counting the one already read.
                self.at -= 1;
                let mut digits = String::new();
                while digits.len() < 3 {
                    match self.peek() {
                        Some(held) if ('0'..='7').contains(&held) => {
                            digits.push(held);
                            self.at += 1;
                        }
                        _ => break,
                    }
                }
                let point = u32::from_str_radix(&digits, 8).unwrap_or(0);
                char::from_u32(point).ok_or_else(|| Trouble("invalid octal escape".to_string()))
            }
            'x' => self.read_hex_escape(),
            'u' => self.read_unicode_escape(),
            'c' => self.read_control_escape(),
            'C' => {
                if !self.eat('-') {
                    return Err(Trouble("invalid control-code syntax".to_string()));
                }
                self.read_control_escape()
            }
            'M' => {
                if !self.eat('-') {
                    return Err(Trouble("invalid meta-code syntax".to_string()));
                }
                let held = match self.peek() {
                    Some('\\') => self.read_escaped_letter()?,
                    _ => self
                        .take()
                        .ok_or_else(|| Trouble("invalid meta-code syntax".to_string()))?,
                };
                Ok(char::from_u32((held as u32) | 0x80).unwrap_or(held))
            }
            held => Ok(held),
        }
    }

    pub(crate) fn read_hex_escape(&mut self) -> Read<char> {
        if self.eat('{') {
            let mut digits = String::new();
            loop {
                match self.take() {
                    Some('}') => break,
                    Some(letter) => digits.push(letter),
                    None => return Err(Trouble("invalid hex escape".to_string())),
                }
            }
            let point = u32::from_str_radix(digits.trim(), 16)
                .map_err(|_| Trouble("invalid hex escape".to_string()))?;
            return char::from_u32(point).ok_or_else(|| Trouble("invalid hex escape".to_string()));
        }
        let mut digits = String::new();
        while digits.len() < 2 {
            match self.peek() {
                Some(letter) if letter.is_ascii_hexdigit() => {
                    digits.push(letter);
                    self.at += 1;
                }
                _ => break,
            }
        }
        if digits.is_empty() {
            return Err(Trouble("invalid hex escape".to_string()));
        }
        let point = u32::from_str_radix(&digits, 16).unwrap_or(0);
        char::from_u32(point).ok_or_else(|| Trouble("invalid hex escape".to_string()))
    }

    pub(crate) fn read_unicode_escape(&mut self) -> Read<char> {
        if self.eat('{') {
            let mut digits = String::new();
            loop {
                match self.take() {
                    Some('}') => break,
                    Some(letter) => digits.push(letter),
                    None => return Err(Trouble("invalid Unicode escape".to_string())),
                }
            }
            let first = digits.split_whitespace().next().unwrap_or("");
            let point = u32::from_str_radix(first, 16)
                .map_err(|_| Trouble("invalid Unicode escape".to_string()))?;
            return char::from_u32(point)
                .ok_or_else(|| Trouble("invalid Unicode escape".to_string()));
        }
        let mut digits = String::new();
        while digits.len() < 4 {
            match self.peek() {
                Some(letter) if letter.is_ascii_hexdigit() => {
                    digits.push(letter);
                    self.at += 1;
                }
                _ => break,
            }
        }
        if digits.len() < 4 {
            return Err(Trouble("invalid Unicode escape".to_string()));
        }
        let point = u32::from_str_radix(&digits, 16).unwrap_or(0);
        char::from_u32(point).ok_or_else(|| Trouble("invalid Unicode escape".to_string()))
    }

    pub(crate) fn read_control_escape(&mut self) -> Read<char> {
        let held = match self.peek() {
            Some('\\') => self.read_escaped_letter()?,
            _ => self
                .take()
                .ok_or_else(|| Trouble("invalid control-code syntax".to_string()))?,
        };
        let point = (held as u32) & 0x9f;
        char::from_u32(point).ok_or_else(|| Trouble("invalid control-code syntax".to_string()))
    }
}
