// Parser module for Metorex
// Converts a stream of tokens into an Abstract Syntax Tree (AST)

mod error;
mod expressions;
pub(crate) use expressions::call::SAFE_CALL;
mod statements;
pub(crate) use statements::function::{
    ANONYMOUS_BLOCK, ANONYMOUS_KWREST, ANONYMOUS_SPLAT, SOLE_INSTANCE_RECEIVER,
};
mod token_stream;

use crate::ast::Statement;
use crate::error::MetorexError;
use crate::lexer::{Token, TokenKind};

use error::ErrorHandler;
use token_stream::TokenStream;

/// The parser converts a token stream into an AST
pub struct Parser {
    /// Token stream for navigation
    stream: TokenStream,
    /// Error handler for reporting and recovery
    error_handler: ErrorHandler,
    /// Track if we're currently parsing inside a class body
    in_class_body: bool,
    /// Depth of nested ternary expressions currently being parsed. Used to
    /// disambiguate `e.f?:sym` (symbol arg) from `cond ? e.f? : alt` (ternary).
    pub(crate) ternary_depth: usize,
    /// Depth of paren-less argument lists currently being parsed. When >0,
    /// identifier-valued arguments must NOT absorb a trailing `do...end` —
    /// the block belongs to the outer method call, per Ruby precedence.
    pub(crate) paren_less_arg_depth: usize,
    /// Depth of dictionary-literal key/value parsing currently active. Used
    /// to disambiguate `{x 1}` (dict-with-missing-colon, not a paren-less call)
    /// from `Class.new { attr o }` (brace block where `attr o` is a method call).
    pub(crate) dict_literal_depth: usize,
    /// Names the file binds somewhere: assignment targets, method parameters,
    /// and block parameters. `foo [1]` indexes a name in this set and passes
    /// an array to a name that is not, which is the rule Ruby applies.
    pub(crate) bound_names: std::collections::HashSet<String>,
}

/// Every name the token stream binds. Over-approximate on purpose: a name
/// counted here is treated as a variable, which is the reading metorex
/// already gave every name.
fn collect_bound_names(tokens: &[Token]) -> std::collections::HashSet<String> {
    use crate::lexer::TokenKind;
    let mut names = std::collections::HashSet::new();
    let mut in_parameters = false;
    let mut in_block_parameters = false;
    // The name a `def` is defining, and any receiver written before it, name
    // a method rather than a variable, so they are stepped over before the
    // parameter list starts.
    let mut naming_a_method = false;
    for (index, token) in tokens.iter().enumerate() {
        match &token.kind {
            TokenKind::Def => {
                naming_a_method = true;
                in_parameters = false;
            }
            TokenKind::Newline | TokenKind::Semicolon => {
                naming_a_method = false;
                in_parameters = false;
                in_block_parameters = false;
            }
            TokenKind::Dot | TokenKind::ColonColon if naming_a_method => {}
            TokenKind::Pipe => in_block_parameters = !in_block_parameters,
            TokenKind::Ident(name) if naming_a_method => {
                // A name followed by `.` is the receiver, so the name after
                // it is the one being defined.
                if !matches!(
                    tokens.get(index + 1).map(|next| &next.kind),
                    Some(TokenKind::Dot | TokenKind::ColonColon)
                ) {
                    naming_a_method = false;
                    in_parameters = true;
                }
                let _ = name;
            }
            TokenKind::Ident(name) => {
                let assigned = matches!(
                    tokens.get(index + 1).map(|next| &next.kind),
                    Some(
                        TokenKind::Equal
                            | TokenKind::PlusEqual
                            | TokenKind::MinusEqual
                            | TokenKind::StarEqual
                            | TokenKind::SlashEqual
                            | TokenKind::PercentEqual
                            | TokenKind::StarStarEqual
                            | TokenKind::PipeEqual
                            | TokenKind::AmpersandEqual
                            | TokenKind::CaretEqual
                            | TokenKind::ShovelEqual
                            | TokenKind::RightShiftEqual
                            | TokenKind::LogicalOrAssign
                            | TokenKind::LogicalAndAssign
                    )
                );
                let after_fat_arrow = index > 0
                    && matches!(tokens[index - 1].kind, TokenKind::FatArrow | TokenKind::In);
                if assigned || after_fat_arrow || in_parameters || in_block_parameters {
                    names.insert(name.clone());
                }
            }
            _ => {
                // An operator name (`def <=>`) is not an Ident, and the
                // parameter list starts right after it.
                if naming_a_method {
                    naming_a_method = false;
                    in_parameters = true;
                }
            }
        }
    }
    names
}

impl Parser {
    /// Create a new parser from a vector of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        let tokens_for_names = tokens.clone();
        Self {
            stream: TokenStream::new(tokens),
            error_handler: ErrorHandler::new(),
            in_class_body: false,
            ternary_depth: 0,
            paren_less_arg_depth: 0,
            dict_literal_depth: 0,
            bound_names: collect_bound_names(&tokens_for_names),
        }
    }

    /// Get the current token without consuming it
    fn peek(&self) -> &Token {
        self.stream.peek()
    }

    /// Peek ahead by offset tokens
    fn peek_ahead(&self, offset: usize) -> &Token {
        self.stream.peek_ahead(offset)
    }

    /// Get the previous token
    fn previous(&self) -> &Token {
        self.stream.previous()
    }

    /// Check if we're at the end of the token stream
    fn is_at_end(&self) -> bool {
        self.stream.is_at_end()
    }

    /// Advance to the next token and return the previous one
    fn advance(&mut self) -> Token {
        self.stream.advance()
    }

    /// Check if the current token matches any of the given kinds
    fn check(&self, kinds: &[TokenKind]) -> bool {
        self.stream.check(kinds)
    }

    /// Check if the current token matches a specific kind (handles complex matching)
    fn match_kind(&self, kind: &TokenKind) -> bool {
        self.stream.match_kind(kind)
    }

    /// Consume the current token if it matches any of the given kinds
    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        self.stream.match_token(kinds)
    }

    /// Expect a specific token kind and consume it, or report an error
    fn expect(&mut self, kind: TokenKind, message: &str) -> Result<Token, MetorexError> {
        if self.match_kind(&kind) {
            Ok(self.advance())
        } else {
            let _token = self.peek();
            Err(self.error_at_current(message))
        }
    }

    /// Skip newlines and comments
    fn skip_whitespace(&mut self) {
        self.stream.skip_whitespace()
    }

    /// Get a reference to the token stream for advanced operations
    pub(crate) fn stream(&self) -> &TokenStream {
        &self.stream
    }

    /// Create an error at the current token
    fn error_at_current(&self, message: &str) -> MetorexError {
        self.error_handler.error_at_current(message, self.peek())
    }

    /// Create an error at the previous token
    fn error_at_previous(&self, message: &str) -> MetorexError {
        self.error_handler
            .error_at_previous(message, self.previous())
    }

    /// Report an error and enter panic mode
    fn report_error(&mut self, error: MetorexError) {
        self.error_handler.report_error(error);
    }

    /// Synchronize after an error (panic mode recovery)
    /// Skip tokens until we find a statement boundary
    fn synchronize(&mut self) {
        self.error_handler.start_synchronize();

        while !self.is_at_end() {
            // If we just passed a newline or semicolon, we're at a statement boundary
            if matches!(
                self.previous().kind,
                TokenKind::Newline | TokenKind::Semicolon
            ) {
                return;
            }

            // Also synchronize at the start of a new statement
            match self.peek().kind {
                TokenKind::Class
                | TokenKind::Def
                | TokenKind::If
                | TokenKind::While
                | TokenKind::Do
                | TokenKind::End => return,
                _ => {}
            }

            self.advance();
        }
    }

    /// Parse a complete program (list of statements)
    pub fn parse(&mut self) -> Result<Vec<Statement>, Vec<MetorexError>> {
        let mut statements = Vec::new();

        // Skip leading whitespace
        self.skip_whitespace();

        while !self.is_at_end() {
            // Skip any whitespace between statements
            self.skip_whitespace();

            if self.is_at_end() {
                break;
            }

            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(err) => {
                    self.report_error(err);
                    self.synchronize();
                }
            }

            // Skip trailing whitespace after statement
            self.skip_whitespace();
        }

        if self.error_handler.has_errors() {
            Err(self.error_handler.errors().to_vec())
        } else {
            Ok(statements)
        }
    }
}
