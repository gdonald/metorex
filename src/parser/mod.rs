// Parser module for Metorex
// Converts a stream of tokens into an Abstract Syntax Tree (AST)

mod error;
mod expressions;
pub(crate) use expressions::call::SAFE_CALL;
mod statements;
pub(crate) use statements::function::{
    ANONYMOUS_BLOCK, ANONYMOUS_KWREST, ANONYMOUS_SPLAT, SOLE_INSTANCE_RECEIVER,
};
pub(crate) use statements::names_a_numbered_parameter;
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
    /// How many conditional branches the parse is inside. A `rescue` modifier
    /// binds looser than `? :`, so one written in a branch belongs to the
    /// whole conditional rather than to that branch.
    pub(crate) ternary_branch_depth: usize,
    /// How many range operands the parse is inside. `or` and `and` bind
    /// looser than a range, so neither belongs to one of its ends.
    pub(crate) range_operand_depth: usize,
    /// Depth of paren-less argument lists currently being parsed. When >0,
    /// identifier-valued arguments must NOT absorb a trailing `do...end` —
    /// the block belongs to the outer method call, per Ruby precedence.
    pub(crate) paren_less_arg_depth: usize,

    /// How deep the walk is inside a `while`, `until` or `for` condition,
    /// where a `do` closes the condition rather than opening a block.
    pub(crate) condition_depth: usize,

    /// Whether the walk is reading a `when` clause, where a `*` spreads a
    /// list of choices rather than naming a rest pattern.
    pub(crate) in_when_clause: bool,

    /// How deep the walk is inside a rescue body, which is the only place a
    /// `retry` may be written.
    pub(crate) rescue_depth: usize,

    /// Whether the walk is reading the argument of `defined?`, where a jump
    /// is reported on rather than taken and so is written freely.
    pub(crate) in_defined_argument: bool,

    /// How deep the walk is inside the right-hand side of an assignment,
    /// where `and` and `or` bind more loosely than the assignment itself.
    pub(crate) assignment_rhs_depth: usize,
    /// Depth of dictionary-literal key/value parsing currently active. Used
    /// to disambiguate `{x 1}` (dict-with-missing-colon, not a paren-less call)
    /// from `Class.new { attr o }` (brace block where `attr o` is a method call).
    pub(crate) dict_literal_depth: usize,
    /// How deep the walk is inside a `def` body, where an `END` block is
    /// registered once for every call rather than once for the program.
    pub(crate) def_body_depth: usize,
    /// How many block bodies the parser sits inside. `BEGIN` belongs to the
    /// top level of a code unit, so one written inside a block is refused.
    pub(crate) block_body_depth: usize,
    /// While above zero, a trailing `=> pattern` or `in pattern` is left for
    /// the construct being read rather than taken as a pattern test. A
    /// `case` subject and a hash's `=>` both rely on this.
    pub(crate) refuse_pattern_test: usize,
    /// The names the `in` clause being read binds, so a repeat among them is
    /// refused where it is written.
    pub(crate) pattern_names: std::collections::HashSet<String>,
    /// How deep the walk is inside something a `next` belongs to: a loop
    /// body or a block. A `next` written straight in a method body, with
    /// none of those around it, has nothing to jump to.
    pub(crate) jump_target_depth: usize,
    /// Names the file binds somewhere: assignment targets, method parameters,
    /// and block parameters. `foo [1]` indexes a name in this set and passes
    /// an array to a name that is not, which is the rule Ruby applies.
    pub(crate) bound_names: std::collections::HashSet<String>,

    /// For each block the walk is inside, which anonymous parameters that
    /// block declared: `|*|`, `|**|`, `|&|`. Forwarding one of those on from
    /// inside the block is ambiguous, so Ruby refuses it.
    pub(crate) block_anonymous_params: Vec<AnonymousBlockParams>,

    /// The token range of every block body read so far. A bare `it` inside
    /// one of these belongs to that block, not to the block enclosing it.
    pub(crate) block_body_ranges: Vec<(usize, usize)>,

    /// How deep the walk is somewhere a name may not take arguments written
    /// without parentheses, such as the value of a `rescue` modifier.
    pub(crate) refuse_paren_less_args: usize,

    /// How deep the walk is inside the parentheses of a call's arguments,
    /// where a `rescue` modifier needs parentheses of its own.
    pub(crate) call_argument_depth: usize,

    /// An expression already read that the next primary stands for. Used to
    /// carry a definition read as a statement into the expression parser, so
    /// `class Held; end.name` chains onto what the body answered.
    pub(crate) seeded_primary: Option<crate::ast::Expression>,

    /// How deep the walk is inside the default value of a lambda parameter,
    /// where the `{` that follows opens the lambda's body rather than a
    /// block for the call the default is made of.
    pub(crate) lambda_default_depth: usize,

    /// The token range of every block body that took numbered parameters.
    /// A block holding one of these cannot take numbered parameters itself.
    pub(crate) numbered_parameter_ranges: Vec<(usize, usize)>,

    /// Whether the block last read wrote a parameter list at all. `{ || it }`
    /// declares no parameters but did write the list, which is enough to
    /// rule out the implicit `it`.
    pub(crate) wrote_block_parameter_list: bool,
}

/// The anonymous parameters one block declared.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct AnonymousBlockParams {
    pub(crate) rest: bool,
    pub(crate) keyword_rest: bool,
    pub(crate) block: bool,
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
            // `lambda` is a method rather than syntax, so a program may name
            // a local after it. The lexer gives it a token of its own, so the
            // name is collected here rather than among the identifiers.
            TokenKind::Lambda
                if matches!(
                    tokens.get(index + 1).map(|next| &next.kind),
                    Some(TokenKind::Equal)
                ) =>
            {
                names.insert("lambda".to_string());
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
            ternary_branch_depth: 0,
            range_operand_depth: 0,
            paren_less_arg_depth: 0,
            condition_depth: 0,
            in_when_clause: false,
            rescue_depth: 0,
            in_defined_argument: false,
            assignment_rhs_depth: 0,
            dict_literal_depth: 0,
            def_body_depth: 0,
            block_body_depth: 0,
            refuse_pattern_test: 0,
            pattern_names: std::collections::HashSet::new(),
            jump_target_depth: 0,
            bound_names: collect_bound_names(&tokens_for_names),
            block_anonymous_params: Vec::new(),
            block_body_ranges: Vec::new(),
            lambda_default_depth: 0,
            seeded_primary: None,
            refuse_paren_less_args: 0,
            call_argument_depth: 0,
            numbered_parameter_ranges: Vec::new(),
            wrote_block_parameter_list: false,
        }
    }

    /// Whether a `{` here opens a block for the call just read. Inside a
    /// lambda parameter's default it opens the lambda's body instead.
    pub(crate) fn starts_brace_block(&self) -> bool {
        self.check(&[TokenKind::LBrace]) && self.lambda_default_depth == 0
    }

    /// Record what anonymous parameters the block now being read declared,
    /// so an attempt to forward one on from inside it can be refused.
    pub(crate) fn enter_block_parameters(&mut self, parameters: &[String]) {
        self.block_anonymous_params.push(AnonymousBlockParams {
            rest: parameters.iter().any(|name| name == "*"),
            keyword_rest: parameters.iter().any(|name| name == "**"),
            block: parameters.iter().any(|name| name == "&"),
        });
    }

    /// Leave the block whose parameters `enter_block_parameters` recorded.
    pub(crate) fn leave_block_parameters(&mut self) {
        self.block_anonymous_params.pop();
    }

    /// Whether an enclosing block declared the anonymous parameter a bare
    /// `*`, `**`, or `&` in an argument list would forward on.
    pub(crate) fn block_declares_anonymous(
        &self,
        which: fn(&AnonymousBlockParams) -> bool,
    ) -> bool {
        self.block_anonymous_params.iter().any(which)
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

    /// Expect a token, reporting a miss where the construct it closes was
    /// opened rather than where the walk gave up. A body that runs off the
    /// end of the file points at the keyword that opened it, which is where
    /// the missing `end` belongs.
    fn expect_closing(
        &mut self,
        kind: TokenKind,
        message: &str,
        opened_at: crate::lexer::Position,
    ) -> Result<Token, MetorexError> {
        if self.match_kind(&kind) {
            return Ok(self.advance());
        }
        Err(MetorexError::syntax_error(
            message,
            crate::error::SourceLocation::new(opened_at.line, opened_at.column, opened_at.offset),
        ))
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
