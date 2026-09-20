// Lexer module for tokenizing Metorex source code.
//
// Implementation is split across files by responsibility:
//   - cursor:      character cursor primitives (advance, peek, position)
//   - identifiers: identifiers, keywords, instance/class/global vars
//   - numbers:     integer and float literals
//   - strings:     quoted strings, escape sequences, interpolation, comments
//   - regex_lit:   `/.../` regex literals and slash-vs-regex disambiguation
//   - percent:     `%`-prefixed literals (`%r`, `%w`, `%Q`, `%[...]`, etc.)
//   - dispatch:    the main `next_token_inner` switch over leading characters

pub mod token;

mod cursor;
mod dispatch;
mod heredoc;
pub(crate) use heredoc::split_interpolation_parts;
mod identifiers;
mod numbers;
mod percent;
mod regex_lit;
mod strings;

pub use token::{InterpolationPart, Position, Token, TokenKind};

use std::iter::Peekable;
use std::str::Chars;

/// The lexer converts source code into a stream of tokens
pub struct Lexer<'a> {
    /// Peekable iterator over the characters
    pub(super) chars: Peekable<Chars<'a>>,
    /// Characters injected back into the stream (consumed before `chars`).
    /// Stored in reverse order so `pop` yields the next char.
    pub(super) prepend: Vec<char>,
    /// Current position in the source
    pub(super) line: usize,
    pub(super) column: usize,
    pub(super) offset: usize,
    /// Last significant token kind (for regex vs division disambiguation)
    pub(super) prev_significant: Option<TokenKind>,
    /// Source offset just past the last significant token, which tells a
    /// symbol naming an operator (`:/`) from a colon that happens to be
    /// followed by one (`condition ? a : /re/`).
    pub(super) prev_significant_end: usize,
    /// Line number to restore once `prepend` drains. Heredoc lexing rewinds
    /// `line` to the opener's line while the rest of that line is re-lexed
    /// from `prepend`; this puts the counter back at the line following the
    /// heredoc terminator afterwards.
    pub(super) restore_line: Option<usize>,
    /// Whether this lexer is reading the core library rather than a program.
    pub(super) prelude: bool,
    /// Whether the source says it is written in bytes rather than in text. A
    /// run of numeric escapes then names those bytes one by one instead of
    /// spelling a character between them.
    pub(super) binary_source: bool,
    /// Whether the source asked for its string literals to be frozen.
    pub(super) frozen_literals: bool,
    /// Whether the source asked outright for its string literals to stay
    /// mutable. A source that says nothing either way gets the literals Ruby
    /// hands back with notice that a later release will freeze them.
    pub(super) mutable_literals: bool,
    /// The encoding the source says it is written in, which is what
    /// `__ENCODING__` answers where it is written.
    pub(super) source_encoding: String,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source code
    pub fn new(source: &'a str) -> Self {
        Self::with_start_line(source, 1)
    }

    /// A lexer over the core library metorex loads at startup. Every position
    /// it hands out says so, which is how a tracepoint knows to pass over the
    /// library's own statements the way Ruby passes over its C code.
    /// A lexer over a library metorex carries. Like the prelude, it is
    /// metorex's own source rather than the program's, so a setting the
    /// program was started with says nothing about its literals.
    pub fn for_embedded_library(source: &'a str) -> Self {
        let mut made = Self::with_start_line(source, 1);
        made.frozen_literals = false;
        made.mutable_literals = false;
        made
    }

    /// Say what encoding the source is written in, for code that arrives as a
    /// string rather than as a file: an eval takes the encoding of the string
    /// it was handed, unless the code names one of its own.
    pub fn with_source_encoding(mut self, named: Option<String>) -> Self {
        if let Some(named) = named {
            self.source_encoding = named;
        }
        self
    }

    pub fn for_prelude(source: &'a str) -> Self {
        let mut made = Self::with_start_line(source, 1);
        made.prelude = true;
        // The library metorex loads at startup is its own rather than the
        // program's, so a setting the program was started with says nothing
        // about it.
        made.frozen_literals = false;
        made.mutable_literals = false;
        made
    }

    /// Create a lexer whose first line is numbered `start_line`. Used by
    /// `eval`/`class_eval`/`module_eval` so `__LINE__` reflects the optional
    /// `lineno` argument (e.g. `class_eval("...", "file", 102)`).
    pub fn with_start_line(source: &'a str, start_line: usize) -> Self {
        // A file written with a byte order mark opens with one, and it names
        // the encoding rather than anything the program says.
        let source = source.strip_prefix('\u{feff}').unwrap_or(source);
        let binary_source = names_binary_encoding(source);
        // A magic comment decides for the source it is written in; without
        // one the setting the program was started with decides.
        let (frozen_literals, mutable_literals) = if names_string_literal_setting(source) {
            let frozen = freezes_string_literals(source);
            (frozen, !frozen)
        } else {
            match literal_default() {
                LiteralDefault::Frozen => (true, false),
                LiteralDefault::Mutable => (false, true),
                LiteralDefault::Chilled => (false, false),
            }
        };
        Self {
            chars: source.chars().peekable(),
            prepend: Vec::new(),
            line: start_line,
            column: 1,
            offset: 0,
            prev_significant: None,
            prev_significant_end: 0,
            restore_line: None,
            prelude: false,
            binary_source,
            frozen_literals,
            mutable_literals,
            source_encoding: named_source_encoding(source).unwrap_or_else(default_source_encoding),
        }
    }

    /// Peek at the next token without consuming it
    pub fn peek_token(&mut self) -> Token {
        // Save current state
        let saved_chars = self.chars.clone();
        let saved_prepend = self.prepend.clone();
        let saved_line = self.line;
        let saved_column = self.column;
        let saved_offset = self.offset;
        let saved_prev = self.prev_significant.clone();
        let saved_prev_end = self.prev_significant_end;
        let saved_restore_line = self.restore_line;

        // Get the next token
        let token = self.next_token();

        // Restore state
        self.chars = saved_chars;
        self.prepend = saved_prepend;
        self.line = saved_line;
        self.column = saved_column;
        self.offset = saved_offset;
        self.prev_significant = saved_prev;
        self.prev_significant_end = saved_prev_end;
        self.restore_line = saved_restore_line;

        token
    }

    /// Collect all tokens from the lexer
    pub fn tokenize(mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token.kind == TokenKind::EOF {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }

    /// Get the next token from the source code, updating regex disambiguation state.
    pub fn next_token(&mut self) -> Token {
        let token = self.next_token_inner();
        match &token.kind {
            // Newlines reset expression context for slash-vs-regex purposes:
            // a `/` at the start of a fresh line should be parsed as a regex
            // literal, not division applied to whatever ended the prior line.
            TokenKind::Newline => {
                self.prev_significant = None;
            }
            // Comments and EOF leave the previous significant token alone.
            TokenKind::Comment(_) | TokenKind::EOF => {}
            other => {
                self.prev_significant = Some(other.clone());
                self.prev_significant_end = self.offset;
            }
        }
        token
    }
}

/// Iterator implementation for Lexer
/// This allows using the lexer in for loops and with iterator methods
impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.next_token();
        if token.kind == TokenKind::EOF {
            None
        } else {
            Some(token)
        }
    }
}

/// Whether a magic comment on one of the first two lines says the source is
/// written in bytes. Ruby reads such a file as bytes, so what a numeric
/// escape names is a byte rather than part of a character.
/// Whether the source says its string literals are frozen, which the magic
/// comment `# frozen_string_literal: true` asks for.
fn freezes_string_literals(source: &str) -> bool {
    for line in source.lines().take(3) {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.starts_with('#') {
            break;
        }
        let lowered = trimmed.to_ascii_lowercase();
        let Some(at) = lowered.find("frozen_string_literal") else {
            continue;
        };
        let named = lowered[at + "frozen_string_literal".len()..].trim_start();
        let named = named.strip_prefix(':').unwrap_or(named).trim_start();
        return named.starts_with("true");
    }
    false
}

/// What a literal is in a source that says nothing about frozen string
/// literals: frozen, mutable, or handed back with notice that a later
/// release will freeze it.
#[derive(Clone, Copy, PartialEq)]
pub enum LiteralDefault {
    Frozen,
    Mutable,
    Chilled,
}

static LITERAL_DEFAULT: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// The encoding a source is read as when it names none of its own, which is
/// what `-K` and `-U` say for the whole run.
static DEFAULT_SOURCE_ENCODING: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);

/// Say what encoding a source is written in when it names none of its own.
pub fn set_default_source_encoding(named: &str) {
    if let Ok(mut held) = DEFAULT_SOURCE_ENCODING.write() {
        *held = Some(named.to_string());
    }
}

fn default_source_encoding() -> String {
    DEFAULT_SOURCE_ENCODING
        .read()
        .ok()
        .and_then(|held| held.clone())
        .unwrap_or_else(|| "UTF-8".to_string())
}

/// Say what a literal is in a source that names no setting of its own, which
/// is what `--enable-frozen-string-literal` and its opposite decide.
pub fn set_literal_default(setting: LiteralDefault) {
    let held = match setting {
        LiteralDefault::Chilled => 0,
        LiteralDefault::Frozen => 1,
        LiteralDefault::Mutable => 2,
    };
    LITERAL_DEFAULT.store(held, std::sync::atomic::Ordering::Relaxed);
}

fn literal_default() -> LiteralDefault {
    match LITERAL_DEFAULT.load(std::sync::atomic::Ordering::Relaxed) {
        1 => LiteralDefault::Frozen,
        2 => LiteralDefault::Mutable,
        _ => LiteralDefault::Chilled,
    }
}

/// Whether a `frozen_string_literal` magic comment was written after code
/// began, where it settles nothing and Ruby says so under `$VERBOSE`.
pub fn frozen_string_literal_after_a_token(source: &str) -> bool {
    let mut a_token_came_first = false;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        let Some(comment) = trimmed.strip_prefix('#') else {
            a_token_came_first = true;
            continue;
        };
        if a_token_came_first
            && comment
                .to_ascii_lowercase()
                .contains("frozen_string_literal")
        {
            return true;
        }
    }
    false
}

/// Whether a magic comment says anything at all about frozen string
/// literals, whichever way it says it.
fn names_string_literal_setting(source: &str) -> bool {
    for line in source.lines().take(3) {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        if !trimmed.starts_with('#') {
            break;
        }
        if trimmed
            .to_ascii_lowercase()
            .contains("frozen_string_literal")
        {
            return true;
        }
    }
    false
}

/// The source up to a line reading `__END__`, and where the data after that
/// line starts. Everything from there on is the program's data rather than
/// its code.
pub fn source_before_data_section(source: &str) -> (&str, Option<usize>) {
    let mut at = 0;
    while at <= source.len() {
        let line_end = source[at..]
            .find('\n')
            .map(|held| at + held)
            .unwrap_or(source.len());
        if &source[at..line_end] == "__END__" {
            let after = if line_end < source.len() {
                line_end + 1
            } else {
                line_end
            };
            return (&source[..at], Some(after));
        }
        if line_end >= source.len() {
            break;
        }
        at = line_end + 1;
    }
    (source, None)
}

/// What a comment names after `coding`, which Ruby reads only when a `:` or
/// an `=` separates the two. A comment merely holding the word, as in
/// "encoding-related", names no encoding at all.
fn encoding_after_coding(comment: &str) -> Option<String> {
    let lowered = comment.to_ascii_lowercase();
    let mut from = 0usize;
    while let Some(found) = lowered[from..].find("coding") {
        let after = from + found + "coding".len();
        let rest = lowered[after..].trim_start();
        if let Some(named) = rest.strip_prefix(':').or_else(|| rest.strip_prefix('=')) {
            return Some(named.trim_start().to_string());
        }
        from = after;
    }
    None
}

/// The encoding the source's magic comment names, or None when it names none.
pub fn named_source_encoding(source: &str) -> Option<String> {
    let trimmed = magic_comment_line(source)?;
    let named = encoding_after_coding(trimmed)?;
    let spelled: String = named
        .chars()
        .take_while(|held| held.is_alphanumeric() || *held == '-' || *held == '_' || *held == '.')
        .collect();
    (!spelled.is_empty()).then_some(spelled)
}

fn names_binary_encoding(source: &str) -> bool {
    let Some(trimmed) = magic_comment_line(source) else {
        return false;
    };
    match encoding_after_coding(trimmed) {
        Some(named) => named.starts_with("binary") || named.starts_with("ascii-8bit"),
        None => false,
    }
}

/// The one line a magic comment can be written on: the first, or the second
/// when a shebang takes the first. A comment further down names nothing, so
/// an encoding comment after another magic comment is ignored.
fn magic_comment_line(source: &str) -> Option<&str> {
    let mut lines = source.lines();
    let first = lines.next()?.trim_start();
    let line = if first.starts_with("#!") {
        lines.next()?.trim_start()
    } else {
        first
    };
    line.starts_with('#').then_some(line)
}
