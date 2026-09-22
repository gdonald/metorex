// The flags in force, the reader itself, and what a failed read
// answers.

use super::*;

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
    pub(crate) fn apply(&mut self, letters: &str, on: bool) {
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

pub(crate) type Read<T> = Result<T, Trouble>;

/// What a read pattern holds beyond its pieces.
pub struct Parsed {
    pub(crate) node: Node,
    /// The name of each group, by the number it answers to.
    pub(crate) names: Vec<Option<String>>,
    /// What the pattern says that Ruby reads, but reads differently than it
    /// looks like it was meant.
    pub(crate) warnings: Vec<String>,
}

pub struct Parser {
    pub(crate) letters: Vec<char>,
    pub(crate) at: usize,
    pub(crate) groups: usize,
    pub(crate) names: Vec<Option<String>>,
    /// Whether the pattern folds case where a piece is written, which a
    /// literal carries into the pieces it builds.
    pub(crate) flags: Flags,
    /// Whether any group has been given a name, which stops a number from
    /// naming one.
    pub(crate) any_named: bool,
    pub(crate) warnings: Vec<String>,
}
