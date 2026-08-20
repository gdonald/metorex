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
        let allow_indented_terminator;
        let strip_indent;
        match self.peek() {
            Some('-') => {
                allow_indented_terminator = true;
                strip_indent = false;
            }
            Some('~') => {
                allow_indented_terminator = true;
                strip_indent = true;
            }
            _ => return None,
        }
        // Save state so we can roll back if the next characters don't form
        // a valid heredoc terminator.
        let saved_chars = self.chars.clone();
        let saved_prepend = self.prepend.clone();
        let saved_line = self.line;
        let saved_column = self.column;
        let saved_offset = self.offset;

        self.advance(); // consume `-` or `~`

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

        // Read the terminator identifier.
        let mut terminator = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                terminator.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        if let Some(q) = quote
            && self.peek() == Some(q)
        {
            self.advance();
        }
        if terminator.is_empty() {
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
        if interpolate && body.contains("#{") {
            return Some(split_interpolated(&body));
        }
        Some(TokenKind::String(body))
    }
}

/// Split a heredoc body into interpolation parts. Mirrors the `#{expr}`
/// scanner used for double-quoted strings: balances braces inside the
/// expression text, and respects `\#{` as an escape for a literal `#{`.
fn split_interpolated(body: &str) -> TokenKind {
    let parts = split_interpolation_parts(body);
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
        TokenKind::String(joined)
    } else {
        TokenKind::InterpolatedString(parts)
    }
}

/// Split a body containing `#{expr}` interpolations into its parts. Balances
/// braces inside each expression, and treats `\#{` as a literal `#{`.
pub(crate) fn split_interpolation_parts(body: &str) -> Vec<InterpolationPart> {
    let mut parts: Vec<InterpolationPart> = Vec::new();
    let mut current = String::new();
    let mut chars = body.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.peek() {
                Some('#') => {
                    chars.next();
                    current.push('#');
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
            parts.push(InterpolationPart::Expression(expr));
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        parts.push(InterpolationPart::Text(current));
    }
    parts
}
