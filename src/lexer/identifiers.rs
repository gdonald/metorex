// Identifier and variable lexing: identifiers, instance/class/global vars, keywords.

use super::{Lexer, TokenKind};

impl<'a> Lexer<'a> {
    /// Check if a character can start an identifier (letter or underscore).
    /// Every non-ASCII character is allowed, matching Ruby's treatment of
    /// multibyte characters in identifiers (e.g. `CS_CONSTλ`).
    pub(super) fn is_identifier_start(ch: char) -> bool {
        ch.is_alphabetic() || ch == '_' || !ch.is_ascii()
    }

    /// Check if a character can continue an identifier (letter, digit, or underscore)
    pub(super) fn is_identifier_continue(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_' || !ch.is_ascii()
    }

    /// Read an identifier or keyword
    pub(super) fn read_identifier(&mut self) -> TokenKind {
        let mut ident = String::new();

        // Read identifier characters
        while let Some(ch) = self.peek() {
            if Self::is_identifier_continue(ch) {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check for trailing ? or ! (Ruby-style method names). An `=`
        // right after it starts `!=` instead, as in `a!=b` and `:a!=> 1`,
        // unless it opens `==` or `=~`.
        if let Some(ch) = self.peek()
            && (ch == '?' || ch == '!')
            && !(self.peek_second() == Some('=') && self.third_char_is_not_eq_or_match())
        {
            ident.push(ch);
            self.advance();
        }

        // A keyword written as a label, `next: 1`, is a name like any other.
        // After a ternary `?` the colon belongs to the ternary instead.
        let kind = self.keyword_or_identifier(ident.clone());
        if !matches!(kind, TokenKind::Ident(_))
            && !ident.ends_with('?')
            && self.peek() == Some(':')
            && self.peek_second() != Some(':')
            && !matches!(self.prev_significant, Some(TokenKind::Question))
        {
            return TokenKind::Ident(ident);
        }
        kind
    }

    /// Whether the character two past the cursor leaves an `=` standing
    /// alone, rather than opening `==` or `=~`.
    fn third_char_is_not_eq_or_match(&self) -> bool {
        let mut ahead = self.chars.clone();
        let skip = 2usize.saturating_sub(self.prepend.len());
        for _ in 0..skip {
            ahead.next();
        }
        let third = if self.prepend.len() >= 3 {
            Some(self.prepend[self.prepend.len() - 3])
        } else {
            ahead.next()
        };
        !matches!(third, Some('=') | Some('~'))
    }

    /// Read an instance or class variable (@var or @@var)
    pub(super) fn read_variable(&mut self) -> TokenKind {
        // Skip the first @
        self.advance();

        // Check if it's a class variable (@@)
        if self.peek() == Some('@') {
            self.advance();
            // Read the identifier part
            let mut ident = String::new();
            while let Some(ch) = self.peek() {
                if Self::is_identifier_continue(ch) {
                    ident.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            TokenKind::ClassVar(ident)
        } else {
            // Instance variable (@)
            let mut ident = String::new();
            while let Some(ch) = self.peek() {
                if Self::is_identifier_continue(ch) {
                    ident.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
            TokenKind::InstanceVar(ident)
        }
    }

    /// Read a global variable ($var, $:, $0, etc.)
    pub(super) fn read_global_variable(&mut self) -> TokenKind {
        // Skip the $
        self.advance();
        // The flags a command line wrote read back under their own letter,
        // so `-a` is `$-a`.
        if self.peek() == Some('-') {
            self.advance();
            let flag = self.peek().unwrap_or('-');
            self.advance();
            return TokenKind::GlobalVar(format!("-{flag}"));
        }
        // `$1` and the rest name a capture group, and a pattern may hold
        // more than nine of them, so the digits are read as one number.
        if self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            let mut digits = String::new();
            while let Some(ch) = self.peek() {
                if !ch.is_ascii_digit() {
                    break;
                }
                digits.push(ch);
                self.advance();
            }
            return TokenKind::GlobalVar(digits);
        }
        if let Some(ch) = self.peek() {
            // Special single-character globals: $: $; $, $/ $\ $! $@ $~ $& $' $` $+ $. $< $> $" $_ $* $$ $?
            if !Self::is_identifier_start(ch) {
                self.advance();
                return TokenKind::GlobalVar(ch.to_string());
            }
        }
        let mut ident = String::new();
        while let Some(ch) = self.peek() {
            if Self::is_identifier_continue(ch) {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        TokenKind::GlobalVar(ident)
    }

    /// Convert a string to a keyword token or identifier
    pub(super) fn keyword_or_identifier(&self, ident: String) -> TokenKind {
        match ident.as_str() {
            "def" => TokenKind::Def,
            "class" => TokenKind::Class,
            "if" => TokenKind::If,
            "elsif" => TokenKind::Elsif,
            "else" => TokenKind::Else,
            "unless" => TokenKind::Unless,
            "while" => TokenKind::While,
            "until" => TokenKind::Until,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "end" => TokenKind::End,
            "do" => TokenKind::Do,
            "begin" => TokenKind::Begin,
            "rescue" => TokenKind::Rescue,
            "ensure" => TokenKind::Ensure,
            "raise" => TokenKind::Raise,
            "break" => TokenKind::Break,
            "next" => TokenKind::Continue,
            "redo" => TokenKind::Redo,
            "retry" => TokenKind::Retry,
            "return" => TokenKind::Return,
            "lambda" => TokenKind::Lambda,
            "super" => TokenKind::Super,
            "yield" => TokenKind::Yield,
            "defined?" => TokenKind::Defined,
            "case" => TokenKind::Case,
            "when" => TokenKind::When,
            "then" => TokenKind::Then,
            "attr_reader" => TokenKind::AttrReader,
            "attr_writer" => TokenKind::AttrWriter,
            "attr_accessor" => TokenKind::AttrAccessor,
            "module" => TokenKind::Module,
            "include" => TokenKind::Include,
            "extend" => TokenKind::Extend,
            "alias" => TokenKind::Alias,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "nil" => TokenKind::Nil,
            "__FILE__" => TokenKind::MagicFile,
            "__LINE__" => TokenKind::MagicLine,
            "__dir__" => TokenKind::MagicDir,
            "__ENCODING__" => TokenKind::SourceEncoding(self.source_encoding.clone()),
            "and" => TokenKind::KeywordAnd,
            "or" => TokenKind::KeywordOr,
            "not" => TokenKind::NotKeyword,
            _ => TokenKind::Ident(ident),
        }
    }
}
