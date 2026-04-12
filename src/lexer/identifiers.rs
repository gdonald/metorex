// Identifier and variable lexing: identifiers, instance/class/global vars, keywords.

use super::{Lexer, TokenKind};

impl<'a> Lexer<'a> {
    /// Check if a character can start an identifier (letter or underscore)
    pub(super) fn is_identifier_start(ch: char) -> bool {
        ch.is_ascii_alphabetic() || ch == '_'
    }

    /// Check if a character can continue an identifier (letter, digit, or underscore)
    pub(super) fn is_identifier_continue(ch: char) -> bool {
        ch.is_ascii_alphanumeric() || ch == '_'
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

        // Check for trailing ? or ! (Ruby-style method names)
        if let Some(ch) = self.peek()
            && (ch == '?' || ch == '!')
        {
            ident.push(ch);
            self.advance();
        }

        // Check if it's a keyword
        self.keyword_or_identifier(ident)
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
        if let Some(ch) = self.peek() {
            // Special single-character globals: $: $; $, $/ $\ $! $@ $~ $& $' $` $+ $. $< $> $" $_ $* $$ $? $0-$9
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
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "end" => TokenKind::End,
            "do" => TokenKind::Do,
            "begin" => TokenKind::Begin,
            "rescue" => TokenKind::Rescue,
            "ensure" => TokenKind::Ensure,
            "raise" => TokenKind::Raise,
            "break" => TokenKind::Break,
            "continue" | "next" => TokenKind::Continue,
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
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "nil" => TokenKind::Nil,
            "__FILE__" => TokenKind::MagicFile,
            "__LINE__" => TokenKind::MagicLine,
            "__dir__" => TokenKind::MagicDir,
            "and" => TokenKind::LogicalAnd,
            "or" => TokenKind::LogicalOr,
            "not" => TokenKind::Bang,
            _ => TokenKind::Ident(ident),
        }
    }
}
