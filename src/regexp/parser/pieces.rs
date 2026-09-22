// Reading the pattern: alternation, repetition, and one atom at a time.

use super::*;

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

    pub(crate) fn peek(&self) -> Option<char> {
        self.letters.get(self.at).copied()
    }

    pub(crate) fn peek_at(&self, ahead: usize) -> Option<char> {
        self.letters.get(self.at + ahead).copied()
    }

    pub(crate) fn take(&mut self) -> Option<char> {
        let held = self.peek();
        if held.is_some() {
            self.at += 1;
        }
        held
    }

    pub(crate) fn eat(&mut self, letter: char) -> bool {
        if self.peek() == Some(letter) {
            self.at += 1;
            return true;
        }
        false
    }

    /// Whether the letters at the cursor spell `run`, and step past them when
    /// they do.
    pub(crate) fn eat_run(&mut self, run: &str) -> bool {
        let wanted: Vec<char> = run.chars().collect();
        if self.letters[self.at..].starts_with(&wanted) {
            self.at += wanted.len();
            return true;
        }
        false
    }

    /// Step past whitespace and comments, which count for nothing under `/x`.
    pub(crate) fn skip_ignored(&mut self) {
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

    pub(crate) fn parse_alternation(&mut self) -> Read<Node> {
        let mut branches = vec![self.parse_branch()?];
        while self.peek() == Some('|') {
            self.at += 1;
            branches.push(self.parse_branch()?);
        }
        Ok(Node::alternate(branches))
    }

    pub(crate) fn parse_branch(&mut self) -> Read<Node> {
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
    pub(crate) fn parse_piece(&mut self) -> Read<Node> {
        let atom = self.parse_atom()?;
        self.parse_repetition(atom)
    }

    pub(crate) fn parse_repetition(&mut self, atom: Node) -> Read<Node> {
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
    pub(crate) fn repeat_node(
        &self,
        atom: Node,
        least: u32,
        most: Option<u32>,
        greed: Greed,
    ) -> Node {
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
    pub(crate) fn read_counted(&mut self) -> Read<Option<(u32, Option<u32>)>> {
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

    pub(crate) fn read_digits(&mut self) -> String {
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

    pub(crate) fn parse_atom(&mut self) -> Read<Node> {
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
    pub(crate) fn letter_node(&self, letter: char) -> Node {
        if self.flags.folded {
            return Node::Class(Class {
                negated: false,
                ascii_only: self.flags.ascii_classes,
                parts: vec![folded_letters(letter)],
            });
        }
        Node::Letter(letter)
    }

    pub(crate) fn parse_group(&mut self) -> Read<Node> {
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

    pub(crate) fn parse_look(&mut self, behind: bool, negated: bool) -> Read<Node> {
        let inner = self.parse_alternation()?;
        self.close_group()?;
        Ok(Node::Look {
            behind,
            negated,
            node: Box::new(inner),
        })
    }

    pub(crate) fn parse_named_group(&mut self, closer: char) -> Read<Node> {
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
    pub(crate) fn parse_conditional(&mut self) -> Read<Node> {
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
        let (target, _) = self.with_level_split(target)?;
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
    pub(crate) fn parse_flag_group(&mut self) -> Read<Node> {
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
}
