//! The text a Ruby String holds, with the encoding it is tagged with.
//!
//! Metorex keeps every string's characters as Rust text. The encoding is a
//! label carried alongside: `force_encoding` changes what a string says it
//! is without touching what it holds, which is what Ruby does for a string
//! whose bytes are already right.

use std::cell::RefCell;
use std::ops::Deref;

/// The encoding a string literal carries.
pub const DEFAULT_ENCODING: &str = "UTF-8";

#[derive(Debug, Clone)]
pub struct StringValue {
    text: String,
    encoding: RefCell<String>,
    /// The id this string answers, handed out the first time one is asked
    /// for. A counter rather than an address, since an address a dropped
    /// string leaves behind can be handed to the next one.
    object_id: std::cell::Cell<u64>,
}

impl StringValue {
    /// Text tagged with the encoding a literal carries.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            encoding: RefCell::new(DEFAULT_ENCODING.to_string()),
            object_id: std::cell::Cell::new(0),
        }
    }

    /// Text tagged with an encoding named outright, which is what a run of
    /// bytes with no character meaning is built as.
    pub fn with_encoding(text: impl Into<String>, encoding: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            encoding: RefCell::new(encoding.into()),
            object_id: std::cell::Cell::new(0),
        }
    }

    /// The name of the encoding this string says it is in.
    pub fn encoding_name(&self) -> String {
        self.encoding.borrow().clone()
    }

    /// Tag the string as being in another encoding, leaving what it holds
    /// alone. Every reference to the same string sees the new tag, the way
    /// Ruby's `force_encoding` changes the string itself.
    pub fn set_encoding(&self, encoding: impl Into<String>) {
        *self.encoding.borrow_mut() = encoding.into();
    }

    /// The text itself.
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// The id this string answers. The first ask takes the next one from the
    /// counter handed in, and every ask after answers the same.
    pub fn object_id(&self, next: impl FnOnce() -> u64) -> u64 {
        if self.object_id.get() == 0 {
            self.object_id.set(next());
        }
        self.object_id.get()
    }
}

/// Reading a StringValue reads its text, so everything written against a
/// `String` keeps working.
impl Deref for StringValue {
    type Target = String;

    fn deref(&self) -> &String {
        &self.text
    }
}

impl PartialEq for StringValue {
    /// Two strings are equal when they hold the same text. Ruby compares an
    /// ASCII-only string across encodings the same way.
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
    }
}

impl Eq for StringValue {}

/// Strings order by their text, which is what `<` and `<=>` compare.
impl PartialOrd for StringValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StringValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.text.cmp(&other.text)
    }
}

/// Comparing against plain text reads the text this holds.
impl PartialEq<String> for StringValue {
    fn eq(&self, other: &String) -> bool {
        &self.text == other
    }
}

impl PartialEq<StringValue> for String {
    fn eq(&self, other: &StringValue) -> bool {
        self == &other.text
    }
}

impl PartialEq<str> for StringValue {
    fn eq(&self, other: &str) -> bool {
        self.text == other
    }
}

impl PartialEq<&str> for StringValue {
    fn eq(&self, other: &&str) -> bool {
        self.text == *other
    }
}

impl std::hash::Hash for StringValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.text.hash(state);
    }
}

impl std::fmt::Display for StringValue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.text)
    }
}

/// Reading a StringValue out as plain text, which is what everything that
/// wants a `String` from one does.
impl From<StringValue> for String {
    fn from(held: StringValue) -> Self {
        held.text
    }
}

impl From<String> for StringValue {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

impl From<&str> for StringValue {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}
