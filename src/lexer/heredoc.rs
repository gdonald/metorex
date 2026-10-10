// Heredoc literal lexing: `<<-IDENT`, `<<~IDENT` (indent-stripped).
//
// Limitations: heredocs are read greedily right after `<<-` / `<<~` is seen,
// which means any tokens between the heredoc opener and end-of-line are
// consumed as part of the parser stream after the body is returned.
//
// Interpolation: bodies of heredocs whose terminator is bare or
// double-quoted (`<<-EOS`, `<<-"EOS"`) interpolate `#{expr}`; bodies whose
// terminator is single-quoted (`<<-'EOS'`) are literal.

use super::{InterpolationPart, Lexer, TokenKind};

impl<'a> Lexer<'a> {
    /// Try to read a heredoc starting just after the `<<` has been consumed.
    /// Returns `Some(TokenKind::String(body))` if a heredoc was read, or
    /// `None` if the input doesn't look like a heredoc (in which case the
    /// caller falls back to emitting a `Shovel` token).
    pub(super) fn try_read_heredoc(&mut self) -> Option<TokenKind> {
        // Heredoc requires `-` or `~` immediately after `<<`. This avoids
        // breaking `arr << identifier` (the shovel operator).
        // - `<<-IDENT`: terminator may be indented; body content kept verbatim.
        // - `<<~IDENT`: terminator may be indented AND common leading whitespace
        //   is stripped from body lines.
        let (allow_indented_terminator, strip_indent) = match self.peek() {
            Some('-') => (true, false),
            Some('~') => (true, true),
            // A bare `<<TERMINATOR` is a heredoc only where a value is
            // expected: after an operand `<<` is the shovel operator. The
            // terminator has to follow immediately and start with an
            // uppercase letter, an underscore, or a quote.
            Some(character)
                if self.slash_is_regex()
                    && (character.is_ascii_uppercase()
                        || character == '_'
                        || character == '"'
                        || character == '\'') =>
            {
                (false, false)
            }
            _ => return None,
        };
        let bare = !matches!(self.peek(), Some('-') | Some('~'));
        // Save state so we can roll back if the next characters don't form
        // a valid heredoc terminator.
        let saved_chars = self.chars.clone();
        let saved_prepend = self.prepend.clone();
        let saved_line = self.line;
        let saved_column = self.column;
        let saved_offset = self.offset;

        if !bare {
            self.advance(); // consume `-` or `~`
        }

        // Optional quote around the terminator (`<<-"FOO"` / `<<-'FOO'`).
        // The quote choice controls interpolation: bare and double-quoted
        // both interpolate `#{...}`; single-quoted is literal (Ruby).
        let quote = match self.peek() {
            Some('\'') | Some('"') => {
                let q = self.peek().unwrap();
                self.advance();
                Some(q)
            }
            _ => None,
        };
        let interpolate = quote != Some('\'');

        // Read the terminator: a name, or between quotes anything up to the
        // closing quote, as in racc's `<<'.,.,'`.
        let mut terminator = String::new();
        while let Some(ch) = self.peek() {
            let belongs = match quote {
                Some(q) => ch != q && ch != '\n',
                None => ch.is_ascii_alphanumeric() || ch == '_',
            };
            if !belongs {
                break;
            }
            terminator.push(ch);
            self.advance();
        }
        // A quoted terminator has to close on the same line. One that does
        // not is no heredoc at all, and `<<` is left to stand on its own,
        // which is the syntax error Ruby reports.
        let quote_closed = match quote {
            Some(q) if self.peek() == Some(q) => {
                self.advance();
                true
            }
            Some(_) => false,
            None => true,
        };
        if terminator.is_empty() || !quote_closed {
            // Not actually a heredoc — restore the state and let the caller
            // fall back to the shovel operator.
            self.chars = saved_chars;
            self.prepend = saved_prepend;
            self.line = saved_line;
            self.column = saved_column;
            self.offset = saved_offset;
            return None;
        }

        // Capture anything on the same line after the heredoc opener. These
        // characters (e.g. `, TOPLEVEL_BINDING)` in `eval(<<-EOS, TOPLEVEL_BINDING)`)
        // need to be re-lexed as tokens AFTER the heredoc string is emitted,
        // then followed by the newline that originally terminated the line.
        let mut rest_of_line = String::new();
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                self.advance();
                break;
            }
            rest_of_line.push(ch);
            self.advance();
        }

        // Read body lines until a line that (after optional leading
        // whitespace if `strip_indent`) matches the terminator exactly.
        // We do NOT consume the newline that follows the terminator line —
        // the parser needs that newline to end the statement that opened
        // the heredoc.
        let mut lines: Vec<String> = Vec::new();
        loop {
            // End of source — treat as end of heredoc.
            if self.peek().is_none() {
                break;
            }

            // Read the next line's content (without consuming the newline yet).
            let mut line = String::new();
            while let Some(ch) = self.peek() {
                if ch == '\n' {
                    break;
                }
                line.push(ch);
                self.advance();
            }

            let trimmed = if allow_indented_terminator {
                line.trim_start()
            } else {
                line.as_str()
            };
            if trimmed == terminator {
                // Stop before consuming the newline so the parser sees a
                // statement terminator after the heredoc body string.
                break;
            }
            // Body line: consume the newline and append the line (with the
            // newline) to the body.
            if self.peek() == Some('\n') {
                self.advance();
            }
            lines.push(line);
        }

        // For `<<~`, strip the common leading whitespace from every line.
        let body = if strip_indent {
            let min_indent = lines
                .iter()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.chars().take_while(|c| c.is_whitespace()).count())
                .min()
                .unwrap_or(0);
            lines
                .into_iter()
                .map(|l| {
                    if l.len() >= min_indent {
                        l.chars().skip(min_indent).collect::<String>()
                    } else {
                        l
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            lines.join("\n")
        };
        // Heredoc bodies always have a trailing newline in Ruby.
        let body = if body.is_empty() {
            String::new()
        } else {
            format!("{}\n", body)
        };

        // Inject the captured rest-of-line (followed by a newline) so those
        // tokens are lexed next, before the scanner proceeds past the
        // terminator line. They belong to the opener's line, so rewind the
        // line counter for the injection and consume the terminator line's
        // newline now (the injected one replaces it as the statement
        // terminator); `restore_line` puts the counter back at the line
        // after the terminator once the injection drains.
        if !rest_of_line.is_empty() {
            if self.peek() == Some('\n') {
                self.advance();
            }
            self.restore_line = Some(self.line);
            self.line = saved_line;
            self.column = saved_column;
            let mut injection = rest_of_line;
            injection.push('\n');
            // `prepend` is LIFO (pop yields last), so push chars in reverse.
            for ch in injection.chars().rev() {
                self.prepend.push(ch);
            }
        }

        // If interpolation is enabled and the body contains `#{`, split it
        // into Text/Expression parts. Otherwise return the raw String.
        if interpolate {
            if body.contains("#{") {
                return Some(split_interpolated(
                    &body,
                    saved_line + 1,
                    self.binary_source,
                ));
            }
            return Some(plain_body(&body, self.binary_source));
        }
        Some(TokenKind::String(body))
    }
}

/// A heredoc body with its escapes read, the way a double-quoted string reads
/// them. A terminator written in quotes keeps its body as it stands, so only
/// the interpolating forms reach here. Alongside the text, whether escapes
/// named bytes the text stands for rather than characters.
pub(crate) fn unescaped(text: &str, binary_source: bool) -> (String, bool) {
    if !text.contains('\\') {
        return (text.to_string(), false);
    }
    let letters: Vec<char> = text.chars().collect();
    let mut held = String::new();
    let mut holds_bytes = false;
    let mut at = 0;
    while at < letters.len() {
        if letters[at] != '\\' || at + 1 >= letters.len() {
            held.push(letters[at]);
            at += 1;
            continue;
        }
        let escape = letters[at + 1];
        // A run of `\NNN` or of `\xNN` escapes names bytes, which together
        // may spell one character.
        if opens_byte_escape(&letters, at) {
            let (bytes, after) = escaped_byte_run(&letters, at, escape == 'x');
            at = after;
            match super::strings::binary_run(binary_source, &bytes) {
                Ok(spelled) => held.push_str(&spelled),
                Err(()) => {
                    holds_bytes = true;
                    super::strings::push_escaped_bytes(&mut held, &bytes);
                }
            }
            continue;
        }
        at += 2;
        match escape {
            'n' => held.push('\n'),
            't' => held.push('\t'),
            'r' => held.push('\r'),
            's' => held.push(' '),
            'a' => held.push('\u{7}'),
            'b' => held.push('\u{8}'),
            'e' => held.push('\u{1b}'),
            'f' => held.push('\u{c}'),
            'v' => held.push('\u{b}'),
            // A backslash at the end of a line joins it to the next, so
            // neither the backslash nor the newline stands in the text.
            '\n' => {}
            'u' => {
                let braced = letters.get(at) == Some(&'{');
                let mut points = vec![String::new()];
                if braced {
                    at += 1;
                    while at < letters.len() && letters[at] != '}' {
                        match letters[at] {
                            ' ' => points.push(String::new()),
                            digit => points.last_mut().expect("one point").push(digit),
                        }
                        at += 1;
                    }
                    at += 1;
                } else {
                    while points[0].len() < 4
                        && at < letters.len()
                        && letters[at].is_ascii_hexdigit()
                    {
                        points[0].push(letters[at]);
                        at += 1;
                    }
                }
                for point in points.iter().filter(|point| !point.is_empty()) {
                    match u32::from_str_radix(point, 16).ok().and_then(char::from_u32) {
                        Some(letter) => held.push(letter),
                        None => held.push('u'),
                    }
                }
            }
            other => held.push(other),
        }
    }
    (held, holds_bytes)
}

/// Whether the backslash at `at` opens an escape naming a byte: an octal
/// digit, or `x` with a hexadecimal digit after it.
fn opens_byte_escape(letters: &[char], at: usize) -> bool {
    match letters.get(at + 1) {
        Some('x') => letters.get(at + 2).is_some_and(char::is_ascii_hexdigit),
        Some(digit) => ('0'..='7').contains(digit),
        None => false,
    }
}

/// The bytes a run of escapes starting at `at` names, all octal or all
/// hexadecimal, and where the run ends. An octal escape past 255 keeps its
/// low byte.
fn escaped_byte_run(letters: &[char], mut at: usize, hexadecimal: bool) -> (Vec<u8>, usize) {
    let mut bytes = Vec::new();
    while letters.get(at) == Some(&'\\')
        && opens_byte_escape(letters, at)
        && (letters[at + 1] == 'x') == hexadecimal
    {
        at += if hexadecimal { 2 } else { 1 };
        let (width, radix) = if hexadecimal { (2, 16) } else { (3, 8) };
        let mut value = 0_u32;
        let mut read = 0;
        while read < width && at < letters.len() {
            let Some(digit) = letters[at].to_digit(radix) else {
                break;
            };
            value = value * radix + digit;
            read += 1;
            at += 1;
        }
        bytes.push(value as u8);
    }
    (bytes, at)
}

/// The token a heredoc body with no interpolation reads as: one whose escapes
/// named bytes is tagged as bytes, in the source's encoding when the source
/// is written in bytes.
fn plain_body(body: &str, binary_source: bool) -> TokenKind {
    match unescaped(body, binary_source) {
        (text, true) if binary_source => TokenKind::BinaryString(super::strings::byte_text(&text)),
        (text, true) => TokenKind::ByteString(super::strings::byte_text(&text)),
        (text, false) => TokenKind::String(text),
    }
}

/// Split a heredoc body into interpolation parts. Mirrors the `#{expr}`
/// scanner used for double-quoted strings: balances braces inside the
/// expression text, and respects `\#{` as an escape for a literal `#{`.
fn split_interpolated(body: &str, first_line: usize, binary_source: bool) -> TokenKind {
    let parts = split_interpolation_parts_from(body, first_line);
    if parts.is_empty() {
        TokenKind::String(String::new())
    } else if parts
        .iter()
        .all(|p| matches!(p, InterpolationPart::Text(_)))
    {
        // No actual expression parts — collapse back to a plain String.
        let mut joined = String::new();
        for p in parts {
            if let InterpolationPart::Text(t) = p {
                joined.push_str(&t);
            }
        }
        plain_body(&joined, binary_source)
    } else {
        let read = parts
            .into_iter()
            .map(|part| match part {
                InterpolationPart::Text(text) => {
                    InterpolationPart::Text(unescaped(&text, binary_source).0)
                }
                held => held,
            })
            .collect();
        TokenKind::InterpolatedString(read)
    }
}

/// Split a body containing `#{expr}` interpolations into its parts, balancing
/// braces inside each expression and treating `\#{` as a literal `#{`. The
/// line given is the one the body's first line sits on, so
/// each `#{` is recorded at the line it was written on. A zero line means the
/// caller has none to give.
pub(crate) fn split_interpolation_parts_from(
    body: &str,
    first_line: usize,
) -> Vec<InterpolationPart> {
    split_parts(body, first_line, false)
}

/// The same split for a regex, where `\#` stays as it was written: the
/// pattern reads it as an escaped `#`, which in extended mode is not the
/// start of a comment.
pub(crate) fn split_pattern_interpolation_parts(body: &str) -> Vec<InterpolationPart> {
    split_parts(body, 0, true)
}

fn split_parts(body: &str, first_line: usize, keeps_escaped_hash: bool) -> Vec<InterpolationPart> {
    let mut parts: Vec<InterpolationPart> = Vec::new();
    let mut current = String::new();
    let mut line = first_line;
    let mut chars = body.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\n' && first_line > 0 {
            line += 1;
        }
        if ch == '\\' {
            match chars.peek() {
                Some('#') => {
                    chars.next();
                    if keeps_escaped_hash {
                        current.push('\\');
                    }
                    current.push('#');
                }
                // `\c` names a control character by the one that follows it,
                // which is that character whatever it would otherwise mean,
                // so `\c#{name}` is the control-# escape, not a substitution.
                Some(&'c') => {
                    chars.next();
                    current.push('\\');
                    current.push('c');
                    if let Some(named) = chars.next() {
                        current.push(named);
                        if named == '\\'
                            && let Some(after) = chars.next()
                        {
                            current.push(after);
                        }
                    }
                }
                Some(&next) => {
                    current.push('\\');
                    current.push(next);
                    chars.next();
                }
                None => current.push('\\'),
            }
            continue;
        }
        if ch == '#' && chars.peek() == Some(&'{') {
            chars.next();
            if !current.is_empty() {
                parts.push(InterpolationPart::Text(std::mem::take(&mut current)));
            }
            let mut expr = String::new();
            let mut depth = 1;
            for ec in chars.by_ref() {
                if ec == '{' {
                    depth += 1;
                    expr.push(ec);
                } else if ec == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    expr.push(ec);
                } else {
                    expr.push(ec);
                }
            }
            parts.push(InterpolationPart::Expression(expr, line));
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        parts.push(InterpolationPart::Text(current));
    }
    parts
}
