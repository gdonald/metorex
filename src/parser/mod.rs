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
use crate::lexer::{Position, Token, TokenKind};

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
    /// Where a `redo` was written with no loop or block around it yet. A
    /// `while` or `until` modifier after `begin ... end` makes that body a
    /// loop, so the refusal waits for the statement to end.
    pub(crate) unlooped_redos: Vec<Position>,
    /// Names the file binds: assignment targets, method parameters, block
    /// parameters and pattern variables, each with the tokens that bind it.
    /// `foo [1]` indexes a name bound before it and passes an array to any
    /// other, which is the rule Ruby applies.
    pub(crate) bound_name_tokens: std::collections::HashMap<String, Vec<usize>>,
    /// The lexical scope chain: where each `def`, `class` and `module` the
    /// walk is inside opened, innermost last. A scope of one of those sees
    /// no local bound before it opened.
    pub(crate) method_scope_starts: Vec<usize>,
    /// The tokens every scope read so far spans: blocks, methods, classes
    /// and modules. A name bound inside one is a local of that scope alone.
    pub(crate) closed_scope_spans: Vec<(usize, usize)>,
    /// The tokens each block read so far spans, with the names it takes as
    /// parameters.
    pub(crate) block_parameter_spans: Vec<(usize, usize, Vec<String>)>,
    /// The tokens every `def`, `class` and `module` read so far spans.
    pub(crate) closed_method_spans: Vec<(usize, usize)>,

    /// For each block the walk is inside, which anonymous parameters that
    /// block declared: `|*|`, `|**|`, `|&|`. Forwarding one of those on from
    /// inside the block is ambiguous, so Ruby refuses it.
    pub(crate) block_anonymous_params: Vec<AnonymousBlockParams>,

    /// For each method definition the walk is inside, whether it declared
    /// the anonymous block parameter, `&` or `...`, which is what a bare `&`
    /// in an argument list forwards.
    pub(crate) method_anonymous_block: Vec<bool>,

    /// Warnings the source earns as it is read, with where each was
    /// written, which a verbose run reports.
    pub(crate) warnings: Vec<(Position, String)>,

    /// The token range of every block body read so far. A bare `it` inside
    /// one of these belongs to that block, not to the block enclosing it.
    pub(crate) block_body_ranges: Vec<(usize, usize)>,

    /// How deep the walk is somewhere a name may not take arguments written
    /// without parentheses, such as the value of a `rescue` modifier.
    pub(crate) refuse_paren_less_args: usize,

    /// The token the statement being read opened on. A `rescue` modifier
    /// after an expression that opens the statement takes a command as its
    /// fallback, as `p value rescue p $!` does.
    pub(crate) statement_start: usize,

    /// Nonzero while the `rescue` modifier being read follows an expression
    /// that opened the statement.
    pub(crate) statement_rescue_depth: usize,

    /// Nonzero while the condition of a modifier `if`, `unless`, `while`, or
    /// `until` is read.
    pub(crate) modifier_condition_depth: usize,

    /// The token after the last argument list written without parentheses.
    /// A `rescue` modifier there follows a command, whose fallback may be a
    /// command too.
    pub(crate) command_arguments_end: usize,

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

/// Every name the token stream binds, with the index of each token that
/// binds it. A name is a variable from its binding on, and a call before it,
/// which is how Ruby's parser reads one.
fn collect_bound_names(tokens: &[Token]) -> std::collections::HashMap<String, Vec<usize>> {
    use crate::lexer::TokenKind;
    let mut names: std::collections::HashMap<String, Vec<usize>> = std::collections::HashMap::new();
    let mut in_parameters = false;
    let mut in_block_parameters = false;
    // The name a `def` is defining, and any receiver written before it, name
    // a method rather than a variable, so they are stepped over before the
    // parameter list starts.
    let mut naming_a_method = false;
    // How many parentheses deep the walk is inside the parameters of a
    // `->` lambda, which end at the closing parenthesis or, written without
    // any, at the body's opening.
    let mut lambda_parameters: Option<usize> = None;
    for (index, token) in tokens.iter().enumerate() {
        if let Some(depth) = lambda_parameters {
            lambda_parameters = match token.kind {
                TokenKind::LParen => Some(depth + 1),
                TokenKind::RParen if depth <= 1 => None,
                TokenKind::RParen => Some(depth - 1),
                TokenKind::LBrace | TokenKind::Do if depth == 0 => None,
                _ => Some(depth),
            };
        }
        match &token.kind {
            TokenKind::Arrow => lambda_parameters = Some(0),
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
            // `:name=` is a Symbol naming a writer, which the lexer hands
            // over as a colon, the name, and the `=`.
            TokenKind::Ident(_)
                if index > 0
                    && matches!(tokens[index - 1].kind, TokenKind::Colon)
                    && !token.had_leading_space => {}
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
                if assigned
                    || after_fat_arrow
                    || in_parameters
                    || in_block_parameters
                    || lambda_parameters.is_some()
                {
                    names.entry(name.clone()).or_default().push(index);
                }
            }
            // A regexp literal matched with `=~` binds a local for each group
            // it names.
            TokenKind::Regex(pattern, _)
                if matches!(
                    tokens.get(index + 1).map(|next| &next.kind),
                    Some(TokenKind::Match)
                ) =>
            {
                for name in named_groups(pattern) {
                    names.entry(name).or_default().push(index);
                }
            }
            // `lambda` is a method rather than syntax, so a program may name
            // a local after it. The lexer gives it a token of its own, so the
            // name is collected here rather than among the identifiers.
            TokenKind::Lambda
                if in_parameters
                    || in_block_parameters
                    || matches!(
                        tokens.get(index + 1).map(|next| &next.kind),
                        Some(TokenKind::Equal)
                    ) =>
            {
                names.entry("lambda".to_string()).or_default().push(index);
            }
            _ => {
                // An operator name (`def <=>`) is not an Ident, and the
                // parameter list starts right after it. A receiver such as
                // `self` or `@held` is followed by a `.`, and the name being
                // defined comes after that.
                let names_a_receiver = matches!(
                    tokens.get(index + 1).map(|next| &next.kind),
                    Some(TokenKind::Dot | TokenKind::ColonColon)
                );
                if naming_a_method && !names_a_receiver {
                    naming_a_method = false;
                    in_parameters = true;
                }
            }
        }
    }
    names
}

/// The names a block's parameter list binds, with the `*`, `**` and `&`
/// markers and any default value left out.
fn parameter_names(parameters: &[String]) -> Vec<String> {
    parameters
        .iter()
        .flat_map(|parameter| {
            parameter
                .split(|letter: char| !(letter.is_alphanumeric() || letter == '_'))
                .filter(|name| !name.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The names a regexp gives its groups with `(?<name>...)` that can be
/// local variables.
fn named_groups(pattern: &str) -> Vec<String> {
    pattern
        .split("(?<")
        .skip(1)
        .filter_map(|after| after.split_once('>').map(|(name, _)| name))
        .filter(|name| {
            name.chars()
                .next()
                .is_some_and(|first| first.is_lowercase() || first == '_')
                && name
                    .chars()
                    .all(|letter| letter.is_alphanumeric() || letter == '_')
        })
        .map(str::to_string)
        .collect()
}

impl Parser {
    /// Whether `name` is a local variable where the walk stands: bound by a
    /// pattern already, or bound before this point anywhere but inside a
    /// block that has closed since.
    pub(crate) fn names_a_local(&self, name: &str) -> bool {
        self.names_a_local_at(name, self.stream.current_position())
    }

    /// Whether `name` is a local variable at the token `here`.
    fn names_a_local_at(&self, name: &str, here: usize) -> bool {
        let scope_opened_at = self.method_scope_starts.last().copied().unwrap_or(0);
        self.bound_name_tokens.get(name).is_some_and(|bindings| {
            bindings.iter().any(|bound_at| {
                *bound_at < here
                    && *bound_at >= scope_opened_at
                    && !self
                        .closed_scope_spans
                        .iter()
                        .any(|(from, to)| (*from..*to).contains(bound_at) && *to <= here)
            })
        })
    }

    /// The locals of the scope around a block that the block's tokens, from
    /// `opened_at` up to `closed_at`, name, in the order they first name
    /// them. A name the block or a block inside it takes as a parameter is
    /// that parameter rather than the outer local.
    pub(crate) fn outer_locals_used(
        &self,
        opened_at: usize,
        closed_at: usize,
        parameters: &[String],
    ) -> Vec<(String, bool)> {
        let own = parameter_names(parameters);
        let tokens = self.stream.tokens();
        let mut found: Vec<(String, bool)> = Vec::new();
        for index in opened_at..closed_at {
            let TokenKind::Ident(name) = &tokens[index].kind else {
                continue;
            };
            let names_a_method = index > 0
                && matches!(
                    tokens[index - 1].kind,
                    TokenKind::Dot | TokenKind::SafeDot | TokenKind::ColonColon | TokenKind::Def
                );
            let labels_an_argument = matches!(
                tokens.get(index + 1),
                Some(next) if matches!(next.kind, TokenKind::Colon) && !next.had_leading_space
            );
            // A parameter of a block written inside this one is that
            // block's own, wherever in it the name is written.
            let a_nested_parameter = self.block_parameter_spans.iter().any(|(from, to, names)| {
                *from > opened_at
                    && *to <= closed_at
                    && (*from..*to).contains(&index)
                    && names.contains(name)
            });
            if names_a_method
                || name.starts_with(|first: char| first.is_ascii_uppercase())
                || labels_an_argument
                || a_nested_parameter
                || own.contains(name)
                || found.iter().any(|(seen, _)| seen == name)
                || !self.names_a_local_at(name, opened_at)
            {
                continue;
            }
            found.push((name.clone(), self.bound_more_than_once(name, opened_at)));
        }
        found
    }

    /// Whether the scope `name` is a local of at `here` binds it at more
    /// than one place, a block inside the scope included and a method,
    /// class or module inside it left out.
    fn bound_more_than_once(&self, name: &str, here: usize) -> bool {
        let scope_opened_at = self
            .method_scope_starts
            .iter()
            .rev()
            .find(|opened| **opened <= here)
            .copied()
            .unwrap_or(0);
        // Only a name already read as a local is asked about, so the scan
        // recorded where it is bound.
        self.bound_name_tokens[name]
            .iter()
            .filter(|bound_at| {
                **bound_at >= scope_opened_at
                    && !self.closed_method_spans.iter().any(|(from, to)| {
                        *from > scope_opened_at && (*from..*to).contains(*bound_at)
                    })
            })
            .count()
            > 1
    }

    /// Close a block whose tokens run from `opened_at` to here, noting the
    /// parameters it takes, and answer the outer locals it names.
    pub(crate) fn close_block_scope(
        &mut self,
        opened_at: usize,
        parameters: &[String],
    ) -> Vec<(String, bool)> {
        let closed_at = self.stream.current_position();
        let outer_locals = self.outer_locals_used(opened_at, closed_at, parameters);
        self.block_parameter_spans
            .push((opened_at, closed_at, parameter_names(parameters)));
        self.closed_scope_spans.push((opened_at, closed_at));
        outer_locals
    }

    /// Open the scope of a `def`, `class` or `module`, which sees no local
    /// bound outside it.
    pub(crate) fn open_method_scope(&mut self) {
        self.method_scope_starts
            .push(self.stream.current_position());
    }

    /// Close the scope `open_method_scope` opened, so what it bound is
    /// nobody else's local.
    pub(crate) fn close_method_scope(&mut self) {
        if let Some(opened_at) = self.method_scope_starts.pop() {
            let closed_at = self.stream.current_position();
            self.closed_scope_spans.push((opened_at, closed_at));
            self.closed_method_spans.push((opened_at, closed_at));
        }
    }

    /// Note that a pattern binds `name` at the token just read, so the scope
    /// it is in reads it as a local from here on.
    pub(crate) fn note_pattern_binding(&mut self, name: &str) {
        let bound_at = self.stream.current_position().saturating_sub(1);
        self.bound_name_tokens
            .entry(name.to_string())
            .or_default()
            .push(bound_at);
    }

    /// Create a new parser from a vector of tokens
    /// A parser for code handed to `eval`, `instance_eval`, or
    /// `class_eval`, which runs in the scope of the method that called it.
    /// Whether that method declared the anonymous block parameter is known
    /// only when the code runs, so a bare `&` at its top level is accepted.
    pub fn inside_eval(mut self) -> Self {
        self.method_anonymous_block.push(true);
        self
    }

    /// A parser for code written inside this one, such as an interpolated
    /// `#{}`, which reads in the method and blocks this one is inside.
    pub(crate) fn nested_parser(&self, tokens: Vec<Token>) -> Parser {
        let mut nested = Parser::new(tokens);
        nested.method_anonymous_block = self.method_anonymous_block.clone();
        nested.block_anonymous_params = self.block_anonymous_params.clone();
        nested
    }

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
            unlooped_redos: Vec::new(),
            bound_name_tokens: collect_bound_names(&tokens_for_names),
            method_scope_starts: Vec::new(),
            closed_scope_spans: Vec::new(),
            block_parameter_spans: Vec::new(),
            closed_method_spans: Vec::new(),
            block_anonymous_params: Vec::new(),
            method_anonymous_block: Vec::new(),
            warnings: Vec::new(),
            block_body_ranges: Vec::new(),
            lambda_default_depth: 0,
            seeded_primary: None,
            refuse_paren_less_args: 0,
            statement_start: usize::MAX,
            statement_rescue_depth: 0,
            modifier_condition_depth: 0,
            command_arguments_end: usize::MAX,
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
    /// The warnings the source earned as it was read.
    pub fn warnings(&self) -> &[(Position, String)] {
        &self.warnings
    }

    /// Whether the statement just read ends with a terminator and another
    /// statement of the same body follows it.
    pub(crate) fn another_statement_follows(&self) -> bool {
        let mut offset = 0;
        if !matches!(
            self.peek_ahead(offset).kind,
            TokenKind::Newline | TokenKind::Semicolon
        ) {
            return false;
        }
        while matches!(
            self.peek_ahead(offset).kind,
            TokenKind::Newline | TokenKind::Semicolon | TokenKind::Comment(_)
        ) {
            offset += 1;
        }
        !matches!(
            self.peek_ahead(offset).kind,
            TokenKind::EOF
                | TokenKind::End
                | TokenKind::RBrace
                | TokenKind::RParen
                | TokenKind::Else
                | TokenKind::Elsif
                | TokenKind::Rescue
                | TokenKind::Ensure
                | TokenKind::When
                | TokenKind::In
        )
    }

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

            let statement_start = self.peek().position.offset;
            match self.parse_statement().and_then(|stmt| {
                self.refuse_unlooped_redos_after(statement_start)
                    .map(|_| stmt)
            }) {
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
