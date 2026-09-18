//! A pattern engine that gives back what it read when a piece further on
//! cannot carry on, which is what Ruby's patterns are written against.
//!
//! Lookbehind, backreferences, a group's name written more than once, calls
//! to a group by name, and possessive and atomic groups all need a match to
//! be able to give ground, so the engine walks the pattern rather than
//! building an automaton out of it.

mod classes;
mod matcher;
mod node;
mod parser;

pub use parser::{Flags, Trouble};

use matcher::{Found, Matcher};
use node::Node;

/// A run of text written so a pattern reads it as itself, with every
/// character that would otherwise mean something to a pattern spelled out.
pub fn escape(text: &str) -> String {
    let mut written = String::with_capacity(text.len());
    for letter in text.chars() {
        match letter {
            '\n' => written.push_str("\\n"),
            '\t' => written.push_str("\\t"),
            '\r' => written.push_str("\\r"),
            '\u{c}' => written.push_str("\\f"),
            '\u{b}' => written.push_str("\\v"),
            held if held.is_ascii_alphanumeric() || !held.is_ascii() || held == '_' => {
                written.push(held)
            }
            held => {
                written.push('\\');
                written.push(held);
            }
        }
    }
    written
}

/// A match that ran past the time it was given rather than failing on its
/// own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimedOut;

/// A pattern read and ready to match.
pub struct Pattern {
    root: Node,
    names: Vec<Option<String>>,
    /// What the pattern says that Ruby reads differently than it looks like
    /// it was meant.
    pub warnings: Vec<String>,
}

/// Where one group's text sits in the subject, in bytes, together with the
/// subject so the text itself can be read back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Match<'t> {
    subject: &'t str,
    start: usize,
    end: usize,
}

impl<'t> Match<'t> {
    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn as_str(&self) -> &'t str {
        &self.subject[self.start..self.end]
    }
}

/// What a match read, group by group. `'t` is how long the subject lives and
/// `'p` how long the pattern the groups were named in does.
pub struct Captures<'t, 'p> {
    subject: &'t str,
    spans: Vec<Option<(usize, usize)>>,
    names: &'p [Option<String>],
}

impl<'t> Captures<'t, '_> {
    /// Where group `index` sits, with zero naming the whole match.
    pub fn get(&self, index: usize) -> Option<Match<'t>> {
        let (start, end) = self.spans.get(index).copied().flatten()?;
        Some(Match {
            subject: self.subject,
            start,
            end,
        })
    }

    /// How many groups the pattern has, counting the whole match.
    pub fn len(&self) -> usize {
        self.spans.len()
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    /// What the group written under `name` read. A name may be written on
    /// more than one group, and the one that matched is the one it stands
    /// for.
    pub fn name(&self, name: &str) -> Option<Match<'t>> {
        let mut last = None;
        for (index, held) in self.names.iter().enumerate() {
            if held.as_deref() != Some(name) {
                continue;
            }
            last = Some(index);
            if self.spans.get(index).copied().flatten().is_some() {
                return self.get(index);
            }
        }
        last.and_then(|index| self.get(index))
    }
}

impl Pattern {
    /// Read a pattern's source under the flags it was written with.
    pub fn compile(source: &str, flags: Flags) -> Result<Self, Trouble> {
        let read = parser::Parser::new(source, flags).parse()?;
        Ok(Self {
            root: read.node,
            names: read.names,
            warnings: read.warnings,
        })
    }

    /// The name of each group by number, with None for the whole match and
    /// for a group written without one.
    pub fn capture_names(&self) -> impl Iterator<Item = Option<&str>> {
        self.names.iter().map(|held| held.as_deref())
    }

    /// How many groups the pattern has, counting the whole match as one.
    pub fn group_count(&self) -> usize {
        self.names.len()
    }

    /// The leftmost match at or after the byte offset `start`.
    pub fn captures_at<'t, 'p>(
        &'p self,
        subject: &'t str,
        start: usize,
    ) -> Option<Captures<'t, 'p>> {
        self.captures_within(subject, start, None).ok().flatten()
    }

    /// The same, given up on once `limit` has passed. Answers Err when the
    /// match was given up on rather than failing on its own.
    pub fn captures_within<'t, 'p>(
        &'p self,
        subject: &'t str,
        start: usize,
        limit: Option<std::time::Duration>,
    ) -> Result<Option<Captures<'t, 'p>>, TimedOut> {
        let reading = Reading::of(subject);
        let Some(from) = reading.char_index(start) else {
            return Ok(None);
        };
        let mut matcher = Matcher::new(&reading.letters, &self.names, &self.root);
        if let Some(limit) = limit {
            matcher.give_up_after(limit);
        }
        let Some(found) = matcher.search(&self.root, from) else {
            if matcher.gave_up() {
                return Err(TimedOut);
            }
            return Ok(None);
        };
        Ok(Some(reading.captures(subject, &self.names, &found)))
    }

    /// Where the leftmost match at or after `start` sits, without its groups.
    pub fn find_at<'t>(&self, subject: &'t str, start: usize) -> Option<Match<'t>> {
        self.captures_at(subject, start)?.get(0)
    }

    /// Whether a match begins at the byte offset `start` exactly.
    pub fn matches_at(&self, subject: &str, start: usize) -> bool {
        let reading = Reading::of(subject);
        let Some(from) = reading.char_index(start) else {
            return false;
        };
        let mut matcher = Matcher::new(&reading.letters, &self.names, &self.root);
        matcher.matches_at(&self.root, from)
    }

    /// Whether the subject holds a match at or after `start`.
    pub fn is_match_at(&self, subject: &str, start: usize) -> bool {
        self.captures_at(subject, start).is_some()
    }

    /// Whether the subject holds a match anywhere.
    pub fn is_match(&self, subject: &str) -> bool {
        self.is_match_at(subject, 0)
    }
}

/// A subject read as characters, with the byte offset each one sits at so a
/// match can be reported the way the rest of the interpreter counts.
struct Reading {
    letters: Vec<char>,
    offsets: Vec<usize>,
}

impl Reading {
    fn of(subject: &str) -> Self {
        let mut letters = Vec::with_capacity(subject.len());
        let mut offsets = Vec::with_capacity(subject.len() + 1);
        for (offset, letter) in subject.char_indices() {
            offsets.push(offset);
            letters.push(letter);
        }
        offsets.push(subject.len());
        Self { letters, offsets }
    }

    /// The character the byte offset `start` opens, or None when it sits past
    /// the end or inside a character.
    fn char_index(&self, start: usize) -> Option<usize> {
        self.offsets.binary_search(&start).ok()
    }

    fn byte_offset(&self, index: usize) -> usize {
        self.offsets
            .get(index)
            .copied()
            .unwrap_or_else(|| self.offsets.last().copied().unwrap_or(0))
    }

    fn captures<'t, 'p>(
        &self,
        subject: &'t str,
        names: &'p [Option<String>],
        found: &Found,
    ) -> Captures<'t, 'p> {
        let mut spans = vec![Some((
            self.byte_offset(found.start),
            self.byte_offset(found.end),
        ))];
        for slot in found.slots.iter().skip(1) {
            spans.push(slot.map(|(from, to)| (self.byte_offset(from), self.byte_offset(to))));
        }
        Captures {
            subject,
            spans,
            names,
        }
    }
}
