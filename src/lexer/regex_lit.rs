// Regex literal lexing and slash-vs-regex disambiguation.

use super::{Lexer, TokenKind};

impl<'a> Lexer<'a> {
    /// Check whether `/` should start a regex literal based on the previous
    /// significant token.  After value-producing tokens (identifiers, numbers,
    /// closing brackets, `end`, `true`, etc.) `/` is division; otherwise it
    /// starts a regex.
    pub(super) fn slash_is_regex(&self) -> bool {
        match &self.prev_significant {
            None => true, // start of file
            Some(kind) => !matches!(
                kind,
                // Value-producing tokens: / after these is division
                TokenKind::Ident(_)
                    | TokenKind::Int(_)
                    | TokenKind::Float(_)
                    | TokenKind::String(_)
                    | TokenKind::InterpolatedString(_)
                    | TokenKind::Regex(_, _)
                    | TokenKind::True
                    | TokenKind::False
                    | TokenKind::Nil
                    | TokenKind::RParen
                    | TokenKind::RBracket
                    | TokenKind::RBrace
                    | TokenKind::End
                    | TokenKind::InstanceVar(_)
                    | TokenKind::ClassVar(_)
                    | TokenKind::GlobalVar(_)
                    | TokenKind::MagicFile
                    | TokenKind::MagicLine
                    // After `def`, / is an operator method name, not regex
                    | TokenKind::Def
            ),
        }
    }

    /// Read a regex literal: `/pattern/flags`
    /// Called after the opening `/` has been consumed.
    pub(super) fn read_regex(&mut self) -> TokenKind {
        let mut pattern = String::new();
        let mut escaped = false;
        let mut in_char_class = false;

        while let Some(ch) = self.peek() {
            if escaped {
                pattern.push('\\');
                pattern.push(ch);
                self.advance();
                escaped = false;
                continue;
            }
            match ch {
                '\\' => {
                    self.advance();
                    escaped = true;
                }
                '/' if !in_char_class => {
                    self.advance(); // consume closing /
                    break;
                }
                '[' => {
                    in_char_class = true;
                    pattern.push(ch);
                    self.advance();
                }
                ']' => {
                    in_char_class = false;
                    pattern.push(ch);
                    self.advance();
                }
                '\n' => break, // unterminated regex
                _ => {
                    pattern.push(ch);
                    self.advance();
                }
            }
        }

        // Read optional flags (i, m, x, etc.)
        let mut flags = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphabetic() {
                flags.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        TokenKind::Regex(pattern, flags)
    }
}
