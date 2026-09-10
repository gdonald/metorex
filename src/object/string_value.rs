//! The text a Ruby String holds, with the encoding it is tagged with.
//!
//! Metorex keeps every string's characters as Rust text. The encoding is a
//! label carried alongside: `force_encoding` changes what a string says it
//! is without touching what it holds, which is what Ruby does for a string
//! whose bytes are already right.

use std::cell::RefCell;

/// The encoding a string literal carries.
pub const DEFAULT_ENCODING: &str = "UTF-8";

#[derive(Debug)]
pub struct StringValue {
    /// What the string holds. Ruby lets a string change what it holds while
    /// every reference to it sees the change, so the text sits behind a cell
    /// rather than being fixed when the string is made.
    text: RefCell<String>,
    encoding: RefCell<String>,
    /// The id this string answers, handed out the first time one is asked
    /// for. A counter rather than an address, since an address a dropped
    /// string leaves behind can be handed to the next one.
    object_id: std::cell::Cell<u64>,
    /// Whether the string refuses to change. `freeze` sets it, and every
    /// method that would change the text checks it first.
    frozen: std::cell::Cell<bool>,
    /// Whether the characters stand for bytes rather than for text. A run of
    /// bytes read off a file or unpacked from a format is built this way, and
    /// keeps saying so even after it is tagged with a text encoding.
    holds_bytes: std::cell::Cell<bool>,
}

impl StringValue {
    /// Text tagged with the encoding a literal carries.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: RefCell::new(text.into()),
            encoding: RefCell::new(DEFAULT_ENCODING.to_string()),
            object_id: std::cell::Cell::new(0),
            frozen: std::cell::Cell::new(false),
            holds_bytes: std::cell::Cell::new(false),
        }
    }

    /// Text tagged with an encoding named outright, which is what a run of
    /// bytes with no character meaning is built as.
    pub fn with_encoding(text: impl Into<String>, encoding: impl Into<String>) -> Self {
        Self {
            text: RefCell::new(text.into()),
            encoding: RefCell::new(encoding.into()),
            object_id: std::cell::Cell::new(0),
            frozen: std::cell::Cell::new(false),
            holds_bytes: std::cell::Cell::new(false),
        }
    }

    /// Text whose characters stand for the bytes they were read from.
    pub fn from_bytes(text: impl Into<String>) -> Self {
        let made = Self::with_encoding(text, "ASCII-8BIT");
        made.holds_bytes.set(true);
        made
    }

    /// Whether the characters stand for bytes rather than for text.
    pub fn holds_bytes(&self) -> bool {
        self.holds_bytes.get()
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

    /// The text itself. The borrow lasts as long as the value is held, so a
    /// caller that changes the string has to let it go first.
    pub fn as_str(&self) -> std::cell::Ref<'_, str> {
        std::cell::Ref::map(self.text.borrow(), |held| held.as_str())
    }

    /// A copy of the text, for a caller that keeps it past a change.
    pub fn to_text(&self) -> String {
        self.text.borrow().clone()
    }

    /// Replace what the string holds. Every reference to the same string
    /// sees the new text, which is what Ruby's in-place methods do.
    pub fn replace_text(&self, text: impl Into<String>) {
        *self.text.borrow_mut() = text.into();
    }

    /// Add text to the end of what the string holds.
    pub fn append_text(&self, extra: &str) {
        self.text.borrow_mut().push_str(extra);
    }

    /// Change the text through a function of what it holds now.
    pub fn update_text(&self, change: impl FnOnce(&str) -> String) {
        let made = change(self.text.borrow().as_str());
        *self.text.borrow_mut() = made;
    }

    /// Whether the string refuses to change.
    pub fn is_frozen(&self) -> bool {
        self.frozen.get()
    }

    /// Refuse every change from here on. Every reference to the same string
    /// sees it, which is what Ruby's `freeze` does.
    pub fn freeze(&self) {
        self.frozen.set(true);
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

impl Clone for StringValue {
    /// A copy holds the same text under an id of its own, since Ruby's `dup`
    /// answers a string that changes on its own.
    fn clone(&self) -> Self {
        Self {
            text: RefCell::new(self.text.borrow().clone()),
            encoding: RefCell::new(self.encoding.borrow().clone()),
            object_id: std::cell::Cell::new(0),
            frozen: std::cell::Cell::new(false),
            holds_bytes: std::cell::Cell::new(self.holds_bytes.get()),
        }
    }
}

impl PartialEq for StringValue {
    /// Two strings are equal when they hold the same text. Ruby compares an
    /// ASCII-only string across encodings the same way.
    fn eq(&self, other: &Self) -> bool {
        *self.text.borrow() == *other.text.borrow()
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
        let held = self.text.borrow();
        let theirs = other.text.borrow();
        held.cmp(&theirs)
    }
}

/// Comparing against plain text reads the text this holds.
impl PartialEq<String> for StringValue {
    fn eq(&self, other: &String) -> bool {
        &*self.text.borrow() == other
    }
}

impl PartialEq<StringValue> for String {
    fn eq(&self, other: &StringValue) -> bool {
        self == &*other.text.borrow()
    }
}

impl PartialEq<str> for StringValue {
    fn eq(&self, other: &str) -> bool {
        self.text.borrow().as_str() == other
    }
}

impl PartialEq<&str> for StringValue {
    fn eq(&self, other: &&str) -> bool {
        self.text.borrow().as_str() == *other
    }
}

impl std::hash::Hash for StringValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.text.borrow().hash(state);
    }
}

impl std::fmt::Display for StringValue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.text.borrow())
    }
}

/// Reading a StringValue out as plain text, which is what everything that
/// wants a `String` from one does.
impl From<StringValue> for String {
    fn from(held: StringValue) -> Self {
        held.text.into_inner()
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
