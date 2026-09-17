//! What a pattern says, once it has been read.

/// One item a character class holds.
#[derive(Debug, Clone)]
pub enum ClassItem {
    /// A single character the class names.
    Letter(char),
    /// Every character from the first through the second.
    Span(char, char),
    /// A named run of characters, such as `\d` or `[:alpha:]`, and whether the
    /// name stands for everything outside it.
    Named(Named, bool),
    /// A class written inside another one.
    Nested(Box<Class>),
}

/// A run of characters named rather than listed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Named {
    /// The runs the backslash escapes name, which cover ASCII alone.
    AsciiDigit,
    AsciiWord,
    AsciiSpace,
    /// The runs the bracket names stand for, which cover every character the
    /// name fits.
    Digit,
    Word,
    Space,
    Alpha,
    Alnum,
    Upper,
    Lower,
    Punct,
    Print,
    Graph,
    Cntrl,
    Blank,
    XDigit,
    Ascii,
    /// A Unicode property named with `\p{...}`.
    Property(String),
}

/// A set of characters, which may be written as the union of several sets and
/// as the part two sets share.
#[derive(Debug, Clone)]
pub struct Class {
    pub negated: bool,
    /// Whether the class covers ASCII alone, which `(?a)` asks for.
    pub ascii_only: bool,
    /// The items, grouped by `&&`. A character belongs to the class when it
    /// belongs to every group.
    pub parts: Vec<Vec<ClassItem>>,
}

/// Where a group's name points, for a backreference or a call.
#[derive(Debug, Clone)]
pub enum Target {
    /// The group at this number, counting from one.
    Numbered(usize),
    /// Every group written under this name.
    Named(String),
    /// The group this many back from where the reference is written.
    Relative(isize),
}

/// How many times a piece repeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Greed {
    /// Take as many as possible, giving them back one at a time.
    Greedy,
    /// Take as few as possible, taking one more at a time.
    Lazy,
    /// Take as many as possible and give none back.
    Possessive,
}

/// A piece of a pattern.
#[derive(Debug, Clone)]
pub enum Node {
    /// Matches without reading anything.
    Empty,
    Letter(char),
    /// A run of letters that follow one another, which is the common shape and
    /// the one a match can step through without recursing.
    Letters(Vec<char>),
    /// `.`, which reads any one character, and a newline too under `/m`.
    AnyChar {
        newline: bool,
    },
    Class(Class),
    /// The start of a line, or of the subject alone under `\A`.
    LineStart,
    LineEnd,
    SubjectStart,
    SubjectEnd,
    /// `\Z`, the end of the subject or the newline that closes it.
    SubjectEndBeforeNewline,
    /// Where the search last left off, which `\G` stands for.
    SearchStart,
    WordBoundary {
        negated: bool,
    },
    /// `\K`, which drops everything read so far from what the match reports.
    KeepOut,
    /// `\X`, one character as a reader sees it: a base together with every
    /// mark, tone, and selector carried on it, and the runs those are joined
    /// into.
    Grapheme,
    Concat(Vec<Node>),
    Alternate(Vec<Node>),
    Repeat {
        node: Box<Node>,
        least: u32,
        most: Option<u32>,
        greed: Greed,
        /// The group the repetition repeats, when it repeats one. It is
        /// forgotten at the start of every turn, so a backreference cannot
        /// reach the turn before. A group written inside that one keeps what
        /// it read.
        clears: Option<usize>,
    },
    /// A group that reports what it read, at `index` counting from one. The
    /// name it may have been written with is kept with the other names, by
    /// the number it answers to.
    Capture {
        index: usize,
        node: Box<Node>,
    },
    /// A group that reports nothing, which may turn flags on for its body.
    Group(Box<Node>),
    /// A group that gives nothing back once it has matched.
    Atomic(Box<Node>),
    /// Text that has to be there, or has to not be, without reading it.
    Look {
        behind: bool,
        negated: bool,
        node: Box<Node>,
    },
    /// The text a group read earlier.
    Backreference {
        target: Target,
        folded: bool,
    },
    /// The pattern a group holds, matched again here.
    Call(Target),
    /// `(?~x)`, which reads as far as it can without any of it matching `x`.
    Absent(Box<Node>),
    /// A branch that runs only when a group has matched.
    Conditional {
        target: Target,
        then_node: Box<Node>,
        else_node: Box<Node>,
    },
}

impl Node {
    /// Fold a list of pieces into one, flattening the single-piece cases so a
    /// match does not step through a wrapper that says nothing.
    pub fn concat(mut pieces: Vec<Node>) -> Node {
        // A run of single letters reads faster as one piece, and the matcher
        // walks it without recursing once per letter.
        let mut folded: Vec<Node> = Vec::with_capacity(pieces.len());
        for piece in pieces.drain(..) {
            match (folded.last_mut(), piece) {
                (Some(Node::Letters(run)), Node::Letter(next)) => run.push(next),
                (Some(Node::Letters(run)), Node::Letters(next)) => run.extend(next),
                (_, Node::Letter(next)) => folded.push(Node::Letters(vec![next])),
                (_, other) => folded.push(other),
            }
        }
        for piece in &mut folded {
            if let Node::Letters(run) = piece
                && run.len() == 1
            {
                *piece = Node::Letter(run[0]);
            }
        }
        match folded.len() {
            0 => Node::Empty,
            1 => folded.pop().unwrap_or(Node::Empty),
            _ => Node::Concat(folded),
        }
    }

    /// Fold a list of branches into one, dropping the wrapper when there is
    /// only the one branch.
    pub fn alternate(mut branches: Vec<Node>) -> Node {
        match branches.len() {
            0 => Node::Empty,
            1 => branches.pop().unwrap_or(Node::Empty),
            _ => Node::Alternate(branches),
        }
    }
}
