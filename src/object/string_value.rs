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
    /// Whether something else is reading the string right now and refuses to
    /// have it changed underneath. An IO::Buffer over a string sets this for
    /// as long as it is in use.
    borrowed: std::cell::Cell<bool>,
    /// Whether the string is one of the interned copies `String#-@` hands
    /// back. Ruby refuses a singleton class on one of those, since the same
    /// string stands for every use of that text.
    deduplicated: std::cell::Cell<bool>,
    /// The warning a change to this string prints, for one Ruby hands back
    /// with notice that it will be frozen in a later release. Cleared once
    /// the warning has been given, so it is printed only the first time.
    chilled: RefCell<Option<String>>,
    /// The pointer a `pack` with 'P' or 'p' left on the string it built, which
    /// names the string those directives wrote the address of. Zero where the
    /// string carries none, which is every string but a packed one.
    pointer: std::cell::Cell<u64>,
    /// Where the literal that made this string was written, as `file:line`.
    /// Only `--debug-frozen-string-literal` records it, and everything else
    /// leaves it empty.
    created_at: RefCell<Option<String>>,
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
            borrowed: std::cell::Cell::new(false),
            deduplicated: std::cell::Cell::new(false),
            chilled: RefCell::new(None),
            pointer: std::cell::Cell::new(0),
            created_at: RefCell::new(None),
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
            borrowed: std::cell::Cell::new(false),
            deduplicated: std::cell::Cell::new(false),
            chilled: RefCell::new(None),
            pointer: std::cell::Cell::new(0),
            created_at: RefCell::new(None),
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

    /// Say that the characters stand for bytes, which is what a copy built
    /// from one that does has to carry.
    pub fn mark_bytes(&self) {
        self.holds_bytes.set(true);
    }

    /// Say that the characters stand for text again, which is what reading a
    /// run of bytes back through an encoding that spells them leaves.
    pub fn clear_bytes(&self) {
        self.holds_bytes.set(false);
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

    /// Whether something else is reading the string and refuses a change.
    pub fn is_borrowed(&self) -> bool {
        self.borrowed.get()
    }

    /// Say that something else is reading the string, or that it is done.
    pub fn set_borrowed(&self, borrowed: bool) {
        self.borrowed.set(borrowed);
    }

    /// Give the string notice that it will be frozen in a later release, so
    /// the first change made to it says so.
    pub fn chill(&self, warning: String) {
        *self.chilled.borrow_mut() = Some(warning);
    }

    /// The pointer `pack` left on this string, or zero where it left none.
    pub fn pointer(&self) -> u64 {
        self.pointer.get()
    }

    /// Mark this string as naming the run of text a pointer stands for.
    pub fn set_pointer(&self, named: u64) {
        self.pointer.set(named);
    }

    /// Whether the string carries notice that it will be frozen in a later
    /// release.
    pub fn is_chilled(&self) -> bool {
        self.chilled.borrow().is_some()
    }

    /// The notice a change to this string prints, taken so it prints once.
    pub fn take_chill(&self) -> Option<String> {
        self.chilled.borrow_mut().take()
    }

    /// Record where the literal that made this string was written.
    pub fn set_created_at(&self, written_at: String) {
        *self.created_at.borrow_mut() = Some(written_at);
    }

    /// Where the literal that made this string was written, for a run that
    /// asked to be told.
    pub fn created_at(&self) -> Option<String> {
        self.created_at.borrow().clone()
    }

    /// Whether the string is one of the interned copies `String#-@` answers.
    pub fn is_deduplicated(&self) -> bool {
        self.deduplicated.get()
    }

    /// Say the string is one of the interned copies.
    pub fn mark_deduplicated(&self) {
        self.deduplicated.set(true);
        self.frozen.set(true);
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
            borrowed: std::cell::Cell::new(false),
            deduplicated: std::cell::Cell::new(false),
            chilled: RefCell::new(None),
            pointer: std::cell::Cell::new(self.pointer.get()),
            created_at: RefCell::new(self.created_at.borrow().clone()),
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
