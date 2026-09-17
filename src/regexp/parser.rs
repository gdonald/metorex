//! Reading a pattern's source into the pieces it names.

use super::node::{Class, ClassItem, Greed, Named, Node, Target};

/// The flags in force where a piece of the pattern is written. Ruby lets a
/// group turn them on and off for its body alone, so they travel with the
/// reader rather than sitting on the pattern as a whole.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Flags {
    pub folded: bool,
    pub dot_reads_newline: bool,
    pub extended: bool,
    /// Whether a class covers ASCII alone, which `(?a)` asks for.
    pub ascii_classes: bool,
    /// Whether the backslash escapes cover every character their name fits
    /// rather than ASCII alone, which `(?u)` asks for.
    pub unicode_escapes: bool,
}

impl Flags {
    /// The flags the letters after `(?` turn on, and the ones after a `-`
    /// they turn off.
    fn apply(&mut self, letters: &str, on: bool) {
        for letter in letters.chars() {
            match letter {
                'i' => self.folded = on,
                'm' => self.dot_reads_newline = on,
                'x' => self.extended = on,
                // The three that say how wide a named run reaches are a
                // choice of one rather than a set of switches.
                'a' => {
                    self.ascii_classes = on;
                    self.unicode_escapes = false;
                }
                'd' => {
                    self.ascii_classes = false;
                    self.unicode_escapes = false;
                }
                'u' => {
                    self.ascii_classes = false;
                    self.unicode_escapes = on;
                }
                _ => {}
            }
        }
    }
}

/// What a pattern's source could not be read as.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trouble(pub String);

type Read<T> = Result<T, Trouble>;

/// What a read pattern holds beyond its pieces.
pub struct Parsed {
    pub node: Node,
    /// The name of each group, by the number it answers to.
    pub names: Vec<Option<String>>,
    /// What the pattern says that Ruby reads, but reads differently than it
    /// looks like it was meant.
    pub warnings: Vec<String>,
}

pub struct Parser {
    letters: Vec<char>,
    at: usize,
    groups: usize,
    names: Vec<Option<String>>,
    /// Whether the pattern folds case where a piece is written, which a
    /// literal carries into the pieces it builds.
    flags: Flags,
    /// Whether any group has been given a name, which stops a number from
    /// naming one.
    any_named: bool,
    warnings: Vec<String>,
}

impl Parser {
    pub fn new(source: &str, flags: Flags) -> Self {
        Self {
            letters: source.chars().collect(),
            at: 0,
            groups: 0,
            names: vec![None],
            flags,
            any_named: false,
            warnings: Vec::new(),
        }
    }

    /// Read the whole pattern.
    pub fn parse(mut self) -> Read<Parsed> {
        let node = self.parse_alternation()?;
        if self.at < self.letters.len() {
            return Err(Trouble("unmatched close parenthesis".to_string()));
        }
        Ok(Parsed {
            node,
            names: self.names,
            warnings: self.warnings,
        })
    }

    fn peek(&self) -> Option<char> {
        self.letters.get(self.at).copied()
    }

    fn peek_at(&self, ahead: usize) -> Option<char> {
        self.letters.get(self.at + ahead).copied()
    }

    fn take(&mut self) -> Option<char> {
        let held = self.peek();
        if held.is_some() {
            self.at += 1;
        }
        held
    }

    fn eat(&mut self, letter: char) -> bool {
        if self.peek() == Some(letter) {
            self.at += 1;
            return true;
        }
        false
    }

    /// Whether the letters at the cursor spell `run`, and step past them when
    /// they do.
    fn eat_run(&mut self, run: &str) -> bool {
        let wanted: Vec<char> = run.chars().collect();
        if self.letters[self.at..].starts_with(&wanted) {
            self.at += wanted.len();
            return true;
        }
        false
    }

    /// Step past whitespace and comments, which count for nothing under `/x`.
    fn skip_ignored(&mut self) {
        if !self.flags.extended {
            return;
        }
        while let Some(letter) = self.peek() {
            if letter.is_whitespace() {
                self.at += 1;
                continue;
            }
            if letter == '#' {
                while let Some(held) = self.peek() {
                    self.at += 1;
                    if held == '\n' {
                        break;
                    }
                }
                continue;
            }
            break;
        }
    }

    fn parse_alternation(&mut self) -> Read<Node> {
        let mut branches = vec![self.parse_branch()?];
        while self.peek() == Some('|') {
            self.at += 1;
            branches.push(self.parse_branch()?);
        }
        Ok(Node::alternate(branches))
    }

    fn parse_branch(&mut self) -> Read<Node> {
        let mut pieces = Vec::new();
        loop {
            self.skip_ignored();
            match self.peek() {
                None | Some('|') | Some(')') => break,
                _ => {}
            }
            let piece = self.parse_piece()?;
            pieces.push(piece);
        }
        Ok(Node::concat(pieces))
    }

    /// One item with whatever repetition follows it.
    fn parse_piece(&mut self) -> Read<Node> {
        let atom = self.parse_atom()?;
        self.parse_repetition(atom)
    }

    fn parse_repetition(&mut self, atom: Node) -> Read<Node> {
        self.skip_ignored();
        let (least, most) = match self.peek() {
            Some('*') => {
                self.at += 1;
                (0, None)
            }
            Some('+') => {
                self.at += 1;
                (1, None)
            }
            Some('?') => {
                self.at += 1;
                (0, Some(1))
            }
            Some('{') => match self.read_counted()? {
                // After an exact count, anything that follows opens another
                // repetition rather than saying how this one gives ground.
                Some((least, most)) if most == Some(least) => {
                    let repeated = self.repeat_node(atom, least, most, Greed::Greedy);
                    return self.parse_repetition(repeated);
                }
                // A `+` after a counted range is a repetition of a
                // repetition rather than a refusal to give ground, which
                // Ruby says so about.
                Some((least, most)) if self.peek() == Some('+') => {
                    self.warnings.push(
                        "nested repeat operator '+' and '{n}' was replaced with '*'".to_string(),
                    );
                    let repeated = self.repeat_node(atom, least, most, Greed::Greedy);
                    return self.parse_repetition(repeated);
                }
                Some(counted) => counted,
                None => return Ok(atom),
            },
            _ => return Ok(atom),
        };
        // An anchor and a lookaround read nothing, so repeating one would
        // stand still forever. Ruby reads the repetition and matches it once.
        let greed = match self.peek() {
            Some('?') => {
                self.at += 1;
                Greed::Lazy
            }
            Some('+') => {
                self.at += 1;
                Greed::Possessive
            }
            _ => Greed::Greedy,
        };
        if let Some(most) = most
            && most < least
        {
            return Err(Trouble("min repeat greater than max repeat".to_string()));
        }
        let repeated = self.repeat_node(atom, least, most, greed);
        // `a**` is a repetition of a repetition, which Ruby reads as one.
        self.parse_repetition(repeated)
    }

    /// One piece repeated, remembering which group the repetition repeats.
    fn repeat_node(&self, atom: Node, least: u32, most: Option<u32>, greed: Greed) -> Node {
        let clears = match &atom {
            Node::Capture { index, .. } => Some(*index),
            _ => None,
        };
        Node::Repeat {
            node: Box::new(atom),
            least,
            most,
            greed,
            clears,
        }
    }

    /// `{n}`, `{n,}`, `{,m}`, or `{n,m}`. A brace that opens none of those is
    /// an ordinary character, so the cursor is left where it was.
    fn read_counted(&mut self) -> Read<Option<(u32, Option<u32>)>> {
        let opened = self.at;
        self.at += 1;
        let low: String = self.read_digits();
        let (high, comma) = if self.eat(',') {
            (self.read_digits(), true)
        } else {
            (String::new(), false)
        };
        if self.peek() != Some('}') || (low.is_empty() && high.is_empty()) {
            self.at = opened;
            return Ok(None);
        }
        self.at += 1;
        let least = low.parse::<u32>().unwrap_or(0);
        let most = if !comma {
            Some(least)
        } else if high.is_empty() {
            None
        } else {
            Some(high.parse::<u32>().unwrap_or(u32::MAX))
        };
        Ok(Some((least, most)))
    }

    fn read_digits(&mut self) -> String {
        let mut held = String::new();
        while let Some(letter) = self.peek() {
            if !letter.is_ascii_digit() {
                break;
            }
            held.push(letter);
            self.at += 1;
        }
        held
    }

    fn parse_atom(&mut self) -> Read<Node> {
        let Some(letter) = self.peek() else {
            return Ok(Node::Empty);
        };
        match letter {
            '(' => self.parse_group(),
            '[' => {
                self.at += 1;
                let class = self.parse_class()?;
                Ok(Node::Class(class))
            }
            '.' => {
                self.at += 1;
                Ok(Node::AnyChar {
                    newline: self.flags.dot_reads_newline,
                })
            }
            '^' => {
                self.at += 1;
                Ok(Node::LineStart)
            }
            '$' => {
                self.at += 1;
                Ok(Node::LineEnd)
            }
            '\\' => self.parse_escape(),
            '*' | '+' | '?' => Err(Trouble(
                "target of repeat operator is not specified".to_string(),
            )),
            _ => {
                self.at += 1;
                Ok(self.letter_node(letter))
            }
        }
    }

    /// A single character, folded when the flags in force say to.
    fn letter_node(&self, letter: char) -> Node {
        if self.flags.folded {
            return Node::Class(Class {
                negated: false,
                ascii_only: self.flags.ascii_classes,
                parts: vec![folded_letters(letter)],
            });
        }
        Node::Letter(letter)
    }

    fn parse_group(&mut self) -> Read<Node> {
        self.at += 1;
        if !self.eat('?') {
            let index = self.next_group(None);
            let inner = self.parse_alternation()?;
            self.close_group()?;
            return Ok(Node::Capture {
                index,
                node: Box::new(inner),
            });
        }
        match self.peek() {
            Some('#') => {
                while let Some(letter) = self.take() {
                    if letter == ')' {
                        return Ok(Node::Empty);
                    }
                }
                Err(Trouble("end pattern in group".to_string()))
            }
            Some(':') => {
                self.at += 1;
                let inner = self.parse_alternation()?;
                self.close_group()?;
                Ok(Node::Group(Box::new(inner)))
            }
            Some('>') => {
                self.at += 1;
                let inner = self.parse_alternation()?;
                self.close_group()?;
                Ok(Node::Atomic(Box::new(inner)))
            }
            Some('=') => {
                self.at += 1;
                self.parse_look(false, false)
            }
            Some('!') => {
                self.at += 1;
                self.parse_look(false, true)
            }
            Some('~') => {
                self.at += 1;
                let inner = self.parse_alternation()?;
                self.close_group()?;
                Ok(Node::Absent(Box::new(inner)))
            }
            Some('<') if matches!(self.peek_at(1), Some('=')) => {
                self.at += 2;
                self.parse_look(true, false)
            }
            Some('<') if matches!(self.peek_at(1), Some('!')) => {
                self.at += 2;
                self.parse_look(true, true)
            }
            Some('<') => self.parse_named_group('>'),
            Some('\'') => self.parse_named_group('\''),
            Some('(') => self.parse_conditional(),
            _ => self.parse_flag_group(),
        }
    }

    fn parse_look(&mut self, behind: bool, negated: bool) -> Read<Node> {
        let inner = self.parse_alternation()?;
        self.close_group()?;
        Ok(Node::Look {
            behind,
            negated,
            node: Box::new(inner),
        })
    }

    fn parse_named_group(&mut self, closer: char) -> Read<Node> {
        self.at += 1;
        let mut name = String::new();
        loop {
            match self.take() {
                Some(letter) if letter == closer => break,
                Some(letter) => name.push(letter),
                None => {
                    return Err(Trouble(
                        "end pattern with unmatched parenthesis".to_string(),
                    ));
                }
            }
        }
        if name.is_empty() {
            return Err(Trouble("group name is empty".to_string()));
        }
        if name.starts_with(|held: char| held.is_ascii_digit() || held == '-') {
            return Err(Trouble(format!("invalid group name <{name}>")));
        }
        let index = self.next_group(Some(name.clone()));
        let inner = self.parse_alternation()?;
        self.close_group()?;
        Ok(Node::Capture {
            index,
            node: Box::new(inner),
        })
    }

    /// `(?(1)yes|no)` and `(?(<name>)yes|no)`.
    fn parse_conditional(&mut self) -> Read<Node> {
        self.at += 1;
        let mut written = String::new();
        loop {
            match self.take() {
                Some(')') => break,
                Some(letter) => written.push(letter),
                None => {
                    return Err(Trouble(
                        "end pattern with unmatched parenthesis".to_string(),
                    ));
                }
            }
        }
        let target = condition_target(&written)?;
        let target = self.with_level_split(target)?;
        let then_node = self.parse_branch()?;
        let else_node = if self.eat('|') {
            self.parse_branch()?
        } else {
            Node::Empty
        };
        self.close_group()?;
        Ok(Node::Conditional {
            target,
            then_node: Box::new(then_node),
            else_node: Box::new(else_node),
        })
    }

    /// `(?imx-imx)` sets the flags for the rest of the enclosing group, and
    /// `(?imx-imx:...)` sets them for its own body alone.
    fn parse_flag_group(&mut self) -> Read<Node> {
        let mut on = String::new();
        while matches!(
            self.peek(),
            Some('i') | Some('m') | Some('x') | Some('a') | Some('d') | Some('u')
        ) {
            on.push(self.take().unwrap_or('i'));
        }
        let mut off = String::new();
        if self.eat('-') {
            while matches!(self.peek(), Some('i') | Some('m') | Some('x')) {
                off.push(self.take().unwrap_or('i'));
            }
        }
        let mut inner_flags = self.flags;
        inner_flags.apply(&on, true);
        inner_flags.apply(&off, false);
        match self.take() {
            Some(':') => {
                let outer = std::mem::replace(&mut self.flags, inner_flags);
                let inner = self.parse_alternation();
                self.flags = outer;
                let inner = inner?;
                self.close_group()?;
                Ok(Node::Group(Box::new(inner)))
            }
            Some(')') => {
                self.flags = inner_flags;
                Ok(Node::Empty)
            }
            _ => Err(Trouble("undefined group option".to_string())),
        }
    }

    fn close_group(&mut self) -> Read<()> {
        self.skip_ignored();
        if self.eat(')') {
            return Ok(());
        }
        Err(Trouble(
            "end pattern with unmatched parenthesis".to_string(),
        ))
    }

    fn next_group(&mut self, name: Option<String>) -> usize {
        self.any_named = self.any_named || name.is_some();
        self.groups += 1;
        self.names.push(name);
        self.groups
    }

    fn parse_escape(&mut self) -> Read<Node> {
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
                let target = self.with_level_split(target)?;
                self.refuse_bad_reference(&target)?;
                Ok(Node::Backreference {
                    target,
                    folded: self.flags.folded,
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
    fn named_run(&self, letter: char) -> Named {
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
    fn read_property(&mut self) -> Read<(String, bool)> {
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
    fn read_reference_name(&mut self) -> Read<Target> {
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
    fn with_level_split(&self, target: Target) -> Read<Target> {
        let Target::Named(written) = &target else {
            return Ok(target);
        };
        let Some(at) = written.rfind(['+', '-']) else {
            if self
                .names
                .iter()
                .any(|held| held.as_deref() == Some(written.as_str()))
            {
                return Ok(target);
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
        Ok(Target::Named(name.to_string()))
    }

    /// Whether a reference points at a group there could be.
    fn refuse_bad_reference(&self, target: &Target) -> Read<()> {
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
    fn read_escaped_letter(&mut self) -> Read<char> {
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

    fn read_hex_escape(&mut self) -> Read<char> {
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

    fn read_unicode_escape(&mut self) -> Read<char> {
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

    fn read_control_escape(&mut self) -> Read<char> {
        let held = match self.peek() {
            Some('\\') => self.read_escaped_letter()?,
            _ => self
                .take()
                .ok_or_else(|| Trouble("invalid control-code syntax".to_string()))?,
        };
        let point = (held as u32) & 0x9f;
        char::from_u32(point).ok_or_else(|| Trouble("invalid control-code syntax".to_string()))
    }

    /// The body of a `[...]`, with the cursor just past the opening bracket.
    fn parse_class(&mut self) -> Read<Class> {
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
    fn read_posix_class(&mut self) -> Read<Option<ClassItem>> {
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
    fn class_member(&mut self) -> Read<Member> {
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
enum Member {
    Letter(char),
    Named(Named, bool),
}

/// The items a span stands for, folded when the pattern folds case. A folded
/// span covers the other case of every letter in it, which is more than
/// folding the two ends would give.
fn span_items(low: char, high: char, folded: bool) -> Vec<ClassItem> {
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

fn posix_run(name: &str) -> Option<Named> {
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
fn name_target(written: &str) -> Read<Target> {
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
fn condition_target(written: &str) -> Read<Target> {
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
