// Expression parsing module
// Handles parsing of all expression types

mod binary;
pub(crate) mod call;
pub(crate) mod primary;
mod unary;

use crate::ast::{Expression, Statement};
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

/// Parsed block parameter list: names (with `*` / `&` prefixes preserved)
/// paired with default-value expressions keyed by parameter index.
type BlockParams = (Vec<String>, Vec<(usize, Expression)>);

impl Parser {
    /// The names inside a destructuring group, written back out so the binder
    /// can walk them. A group may hold groups of its own, and one name in it
    /// may take whatever the others leave.
    pub(crate) fn read_parameter_group(&mut self) -> Result<String, MetorexError> {
        let mut names: Vec<String> = Vec::new();
        loop {
            self.skip_whitespace();
            if self.match_token(&[TokenKind::RParen]) {
                break;
            }
            if self.match_token(&[TokenKind::LParen]) {
                let nested = self.read_parameter_group()?;
                names.push(format!("({})", nested));
            } else {
                let star = if self.match_token(&[TokenKind::Star]) {
                    "*"
                } else {
                    ""
                };
                self.skip_whitespace();
                // `(*)` takes what the named parts leave without naming it.
                if !star.is_empty() && self.check(&[TokenKind::RParen, TokenKind::Comma]) {
                    names.push("*".to_string());
                } else {
                    match self.advance().kind {
                        TokenKind::Ident(name) => {
                            self.declare_local(&name);
                            names.push(format!("{}{}", star, name))
                        }
                        _ => {
                            return Err(self.error_at_previous("Expected parameter name in group"));
                        }
                    }
                }
            }
            self.skip_whitespace();
            if !self.match_token(&[TokenKind::Comma]) {
                self.skip_whitespace();
                self.expect(TokenKind::RParen, "Expected ')' after group")?;
                break;
            }
        }
        Ok(names.join(","))
    }

    /// Parse an expression using operator precedence climbing
    pub(crate) fn parse_expression(&mut self) -> Result<Expression, MetorexError> {
        let opens_the_statement = self.stream.current_position() == self.statement_start;
        let expr = self.parse_assignment()?;
        if opens_the_statement && self.call_argument_depth == 0 {
            self.statement_rescue_depth += 1;
            let wrapped = self.wrap_with_rescue_modifier(expr);
            self.statement_rescue_depth -= 1;
            return wrapped;
        }
        self.wrap_with_rescue_modifier(expr)
    }

    /// `value => pattern` and `value in pattern` written on their own, which
    /// test the value against the pattern rather than opening a `case`. Both
    /// stand where a statement does, so nothing inside an expression takes
    /// the `=>` of a hash pair or the `in` of a `for` as one of these.
    pub(crate) fn wrap_with_pattern_test(
        &mut self,
        expr: Expression,
    ) -> Result<Expression, MetorexError> {
        if self.refuse_pattern_test > 0 {
            return Ok(expr);
        }
        let refuses = match self.peek().kind {
            TokenKind::FatArrow => true,
            TokenKind::In => false,
            _ => return Ok(expr),
        };
        let position = self.advance().position;
        self.skip_whitespace();
        let pattern = self.parse_in_pattern()?;
        Ok(Expression::PatternTest {
            value: Box::new(expr),
            pattern: Box::new(pattern),
            refuses,
            position,
        })
    }

    /// `expr rescue fallback` answers `fallback` when `expr` raises a
    /// StandardError. The `rescue` has to follow the expression directly, so
    /// the one that opens a clause inside `begin ... end` is left alone.
    pub(crate) fn wrap_with_rescue_modifier(
        &mut self,
        expr: Expression,
    ) -> Result<Expression, MetorexError> {
        // A modifier binds to the expression it follows, so nothing may come
        // between them. A `rescue` on its own line opens a clause, and so
        // does one after a semicolon, even though that shares the line.
        // A `rescue` after an argument written without parentheses, or after
        // a modifier's condition, belongs to the statement around it.
        if self.ternary_branch_depth > 0
            || self.paren_less_arg_depth > 0
            || self.modifier_condition_depth > 0
        {
            return Ok(expr);
        }
        if !self.check(&[TokenKind::Rescue])
            || self.peek().position.line != self.previous().position.line
            || matches!(
                self.previous().kind,
                TokenKind::Semicolon | TokenKind::Newline
            )
        {
            return Ok(expr);
        }
        // A `rescue` modifier written straight into an argument list is
        // ambiguous, so Ruby asks for parentheses around it.
        if self.call_argument_depth > 0 {
            return Err(MetorexError::syntax_error(
                "a rescue modifier in an argument list needs parentheses of its own",
                crate::error::SourceLocation::new(
                    self.peek().position.line,
                    self.peek().position.column,
                    self.peek().position.offset,
                ),
            ));
        }
        // After a command, or where the expression opened the statement, the
        // fallback may be a command too. Anywhere else it is one expression,
        // so a name followed by an argument written without parentheses is
        // not one Ruby reads here.
        let takes_a_command = self.statement_rescue_depth > 0
            || self.stream.current_position() == self.command_arguments_end;
        let position = self.advance().position;
        self.skip_whitespace();
        let fallback = if takes_a_command {
            let enclosing = std::mem::take(&mut self.statement_rescue_depth);
            let fallback = self.parse_assignment();
            self.statement_rescue_depth = enclosing;
            fallback?
        } else {
            self.refuse_paren_less_args += 1;
            let fallback = self.parse_assignment();
            self.refuse_paren_less_args -= 1;
            fallback?
        };
        if !self.check(&[
            TokenKind::Newline,
            TokenKind::Semicolon,
            TokenKind::EOF,
            TokenKind::RParen,
            TokenKind::RBrace,
            TokenKind::RBracket,
            TokenKind::Comma,
            TokenKind::End,
            TokenKind::Dot,
            TokenKind::Then,
            TokenKind::Rescue,
            TokenKind::Ensure,
            TokenKind::Else,
        ]) {
            return Err(MetorexError::syntax_error(
                "unexpected argument after a rescue modifier",
                crate::error::SourceLocation::new(
                    self.peek().position.line,
                    self.peek().position.column,
                    self.peek().position.offset,
                ),
            ));
        }
        Ok(Expression::BeginRescue {
            body: vec![Statement::Expression {
                expression: expr,
                position,
            }],
            rescue_clauses: vec![crate::ast::RescueClause {
                exception_types: vec!["StandardError".to_string()],
                variable_name: None,
                variable_target: None,
                splatted_types: Vec::new(),
                body: vec![Statement::Expression {
                    expression: fallback,
                    position,
                }],
                position,
            }],
            else_clause: None,
            ensure_block: None,
            position,
        })
    }

    /// An expression where a value is read, a `->` lambda among them.
    pub(crate) fn parse_expression_with_lambda(&mut self) -> Result<Expression, MetorexError> {
        if self.check(&[TokenKind::Arrow]) {
            return self.parse_expression();
        }
        self.parse_assignment()
    }

    /// A condition, with Ruby's warnings for a literal written where a test
    /// goes: a regexp or a string, or an integer as an end of a flip-flop.
    pub(crate) fn parse_tested_condition(&mut self) -> Result<Expression, MetorexError> {
        let condition = self.parse_condition()?;
        self.warn_literal_condition(&condition);
        Ok(condition)
    }

    /// Note the warnings a literal in a condition earns, looking through
    /// `and`, `or` and parentheses.
    pub(crate) fn warn_literal_condition(&mut self, condition: &Expression) {
        match condition {
            Expression::RegexLiteral { position, .. } => self
                .default_warnings
                .push((*position, "regex literal in condition".to_string())),
            Expression::StringLiteral { position, .. } => self
                .default_warnings
                .push((*position, "string literal in condition".to_string())),
            Expression::Range { start, end, .. } => {
                for bound in [start, end] {
                    if let Expression::IntLiteral { position, .. } = bound.as_ref() {
                        self.default_warnings
                            .push((*position, "integer literal in flip-flop".to_string()));
                    }
                }
            }
            Expression::BinaryOp {
                op: crate::ast::BinaryOp::And | crate::ast::BinaryOp::Or,
                left,
                right,
                ..
            } => {
                self.warn_literal_condition(left);
                self.warn_literal_condition(right);
            }
            Expression::Grouped { expression, .. } => self.warn_literal_condition(expression),
            _ => {}
        }
    }

    /// Parse assignment (lowest precedence)
    /// Parse a condition expression that may contain an inline assignment.
    /// Used in if/unless/while conditions where `var = expr` is valid.
    pub(crate) fn parse_condition(&mut self) -> Result<Expression, MetorexError> {
        self.condition_depth += 1;
        let parsed = self.parse_expression();
        self.condition_depth -= 1;
        let expr = parsed?;

        // Check for inline assignment: ident = expr (in condition context)
        if matches!(
            expr,
            Expression::Identifier { .. }
                | Expression::InstanceVariable { .. }
                | Expression::ClassVariable { .. }
                | Expression::GlobalVariable { .. }
        ) && self.check(&[TokenKind::Equal])
        {
            let position = self.advance().position;
            self.skip_whitespace();
            // The value is read as part of the condition, so a `do` after it
            // opens the loop body rather than a block on the value.
            self.condition_depth += 1;
            self.assignment_rhs_depth += 1;
            let read = self.parse_expression();
            self.assignment_rhs_depth -= 1;
            self.condition_depth -= 1;
            let value = read?;
            // `while line = gets and line.size > 1` tests the assignment and
            // then the rest.
            return self.fold_keyword_logic_onto(Expression::BinaryOp {
                op: crate::ast::BinaryOp::Assign,
                left: Box::new(expr),
                right: Box::new(value),
                position,
            });
        }

        // `case held[key] += 1` and `while (count += 1) < 3` take a compound
        // assignment as the value, which reads the way a statement does.
        if crate::parser::statements::is_assignable(&expr)
            && let Some(operation) = self.compound_assignment_ahead()
        {
            let position = self.advance().position;
            self.skip_whitespace();
            self.condition_depth += 1;
            let read = self.parse_expression();
            self.condition_depth -= 1;
            let value = read?;
            let reading = crate::parser::statements::or_assign_reading(&operation, expr.clone());
            return Ok(Expression::BinaryOp {
                op: crate::ast::BinaryOp::Assign,
                left: Box::new(expr),
                right: Box::new(Expression::BinaryOp {
                    op: operation,
                    left: Box::new(reading),
                    right: Box::new(value),
                    position,
                }),
                position,
            });
        }

        Ok(expr)
    }

    pub(crate) fn parse_assignment(&mut self) -> Result<Expression, MetorexError> {
        let expr = self.parse_range()?;

        // Note: assignment (`=`) is handled at the statement level.
        // parse_assignment only deals with ternary operators.

        // Ternary operator: condition ? true_expr : false_expr
        if self.check(&[TokenKind::Question]) {
            let position = self.advance().position; // consume ?
            self.skip_whitespace();
            self.ternary_depth += 1;
            self.ternary_branch_depth += 1;
            // A branch may be an assignment, which answers what it assigned:
            // `flag ? hash[key] = 1 : hash[key] = 2`.
            let then_expr = self.parse_expression_with_assignment();
            self.ternary_depth -= 1;
            let then_expr = match then_expr {
                Ok(held) => held,
                Err(error) => {
                    self.ternary_branch_depth -= 1;
                    return Err(error);
                }
            };
            self.skip_whitespace();
            let expected = self.expect(TokenKind::Colon, "Expected ':' in ternary expression");
            if let Err(error) = expected {
                self.ternary_branch_depth -= 1;
                return Err(error);
            }
            self.skip_whitespace();
            let else_expr = self.parse_expression_with_assignment();
            self.ternary_branch_depth -= 1;
            let else_expr = else_expr?;
            return Ok(Expression::If {
                condition: Box::new(expr),
                then_branch: vec![Statement::Expression {
                    expression: then_expr,
                    position,
                }],
                elsif_branches: vec![],
                else_branch: Some(vec![Statement::Expression {
                    expression: else_expr,
                    position,
                }]),
                position,
            });
        }

        Ok(expr)
    }

    /// Parse a block's `|params|` list (or `||`), returning the names (with
    /// `*` / `&` modifiers preserved as prefixes) and default-value
    /// expressions keyed by parameter index. Defaults parse below the `|`
    /// operator level so the closing pipe terminates them. Accepts a
    /// trailing comma (`|a,|`).
    pub(crate) fn parse_block_pipe_params(&mut self) -> Result<BlockParams, MetorexError> {
        let mut params = Vec::new();
        let mut defaults = Vec::new();
        self.wrote_block_parameter_list = false;
        if self.match_token(&[TokenKind::LogicalOr]) {
            // Empty parameter list: ||
            self.wrote_block_parameter_list = true;
            return Ok((params, defaults));
        }
        if !self.match_token(&[TokenKind::Pipe]) {
            return Ok((params, defaults));
        }
        self.wrote_block_parameter_list = true;
        self.skip_line_breaks();
        if !self.check(&[TokenKind::Pipe, TokenKind::Semicolon]) {
            loop {
                self.skip_line_breaks();
                // `|**nil|` says the block takes no keyword arguments, which
                // is a declaration rather than a parameter.
                if self.check(&[TokenKind::StarStar])
                    && matches!(self.peek_ahead(1).kind, TokenKind::Nil)
                {
                    self.advance();
                    self.advance();
                    params.push(crate::object::NO_KEYWORDS_PARAM.to_string());
                    self.skip_line_breaks();
                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                    continue;
                }
                let prefix = if self.match_token(&[TokenKind::StarStar]) {
                    "**"
                } else if self.match_token(&[TokenKind::Star]) {
                    "*"
                } else if self.match_token(&[TokenKind::Ampersand]) {
                    "&"
                } else {
                    ""
                };
                self.skip_line_breaks();
                // `|(a, b)|` spreads one array argument across the names in
                // the group, which the binder undoes by the marker.
                if prefix.is_empty() && self.match_token(&[TokenKind::LParen]) {
                    let names = self.read_parameter_group()?;
                    params.push(format!(
                        "{}{}",
                        crate::object::DESTRUCTURED_GROUP_PREFIX,
                        names
                    ));
                } else if !prefix.is_empty() && self.check(&[TokenKind::Pipe, TokenKind::Comma]) {
                    // `|*|`, `|**|`, and `|&|` take the values without naming
                    // them, so the prefix stands alone.
                    params.push(prefix.to_string());
                } else {
                    let param_token = self.advance();
                    match param_token.kind {
                        TokenKind::Ident(name) => {
                            self.declare_local(&name);
                            // `|x:|` names a keyword parameter, which takes
                            // its value from the keyword arguments.
                            if prefix.is_empty()
                                && self.check(&[TokenKind::Colon])
                                && !self.peek().had_leading_space
                            {
                                self.advance();
                                params.push(format!(
                                    "{}{}",
                                    crate::object::KEYWORD_PARAM_PREFIX,
                                    name
                                ));
                                // `|x: 1|` gives the keyword a default, which
                                // follows the colon directly.
                                self.skip_line_breaks();
                                if !self.check(&[TokenKind::Comma, TokenKind::Pipe]) {
                                    let default = self.parse_bitwise_and()?;
                                    defaults.push((params.len() - 1, default));
                                }
                            } else {
                                params.push(format!("{}{}", prefix, name));
                            }
                        }
                        _ => return Err(self.error_at_previous("Expected parameter name")),
                    }
                }
                if self.check(&[TokenKind::Semicolon]) {
                    break;
                }
                self.skip_line_breaks();
                if self.match_token(&[TokenKind::Equal]) {
                    self.skip_line_breaks();
                    let default = self.parse_bitwise_and()?;
                    defaults.push((params.len() - 1, default));
                    self.skip_line_breaks();
                }
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
                self.skip_line_breaks();
                // `|a,|` — trailing comma before the closing pipe. Record it:
                // it is what makes a lone array argument destructure.
                if self.check(&[TokenKind::Pipe]) {
                    params.push(crate::object::TRAILING_COMMA_PARAM.to_string());
                    break;
                }
            }
        }
        // `|a; held|` names locals of the block's own, which start as nil and
        // never take an argument.
        if self.match_token(&[TokenKind::Semicolon]) {
            loop {
                self.skip_line_breaks();
                if self.check(&[TokenKind::Pipe]) {
                    break;
                }
                match self.advance().kind {
                    TokenKind::Ident(name) => {
                        self.declare_local(&name);
                        params.push(format!("{}{}", crate::object::BLOCK_LOCAL_PREFIX, name))
                    }
                    _ => return Err(self.error_at_previous("Expected a name after ';'")),
                }
                self.skip_line_breaks();
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        self.skip_line_breaks();
        if self.check(&[TokenKind::Semicolon]) {
            return Err(self.error_at_current("syntax error, unexpected ';'"));
        }
        self.expect(TokenKind::Pipe, "Expected '|' after block parameters")?;
        self.refuse_duplicate_parameters(&params)?;
        Ok((params, defaults))
    }

    /// Skip the line breaks and comments inside a parameter list, where a
    /// `;` has a meaning of its own rather than ending a statement.
    fn skip_line_breaks(&mut self) {
        while matches!(self.peek().kind, TokenKind::Newline | TokenKind::Comment(_)) {
            self.advance();
        }
    }

    /// Refuse a parameter list that names one parameter twice. A name
    /// starting with `_` may repeat, and the first one binds.
    pub(crate) fn refuse_duplicate_parameters(
        &self,
        params: &[String],
    ) -> Result<(), MetorexError> {
        let mut seen = std::collections::HashSet::new();
        let names = params.iter().flat_map(|param| {
            param
                .split(|letter: char| !(letter.is_alphanumeric() || letter == '_'))
                .filter(|name| !name.is_empty() && name != &"nil")
                .map(str::to_string)
                .collect::<Vec<_>>()
        });
        for name in names {
            if !name.starts_with('_') && !seen.insert(name) {
                return Err(self.error_at_previous("duplicated argument name"));
            }
        }
        Ok(())
    }

    /// Parse a block: `do |param1, param2| ... end`
    pub(crate) fn parse_block(&mut self) -> Result<Expression, MetorexError> {
        // A block body is its own run of statements, so `and` and `or` bind
        // there the way they do anywhere else.
        let held = std::mem::take(&mut self.assignment_rhs_depth);
        let parsed = self.parse_block_body();
        self.assignment_rhs_depth = held;
        parsed
    }

    pub(crate) fn parse_block_body(&mut self) -> Result<Expression, MetorexError> {
        let start_pos = self.peek().position;
        let block_opened_at = self.stream.current_position();

        // Expect 'do' keyword
        self.expect(TokenKind::Do, "Expected 'do' to start block")?;
        self.skip_whitespace();

        let (parameters, parameter_defaults) = self.parse_block_pipe_params()?;
        let wrote_parameter_list = self.wrote_block_parameter_list;

        self.skip_whitespace();

        let body_opened_at = self.stream.current_position();
        self.jump_target_depth += 1;
        self.enter_block_parameters(&parameters);
        let body = self.parse_block_body_with_optional_rescue_ensure(start_pos);
        self.leave_block_parameters();
        self.jump_target_depth -= 1;
        let body = body?;
        let body_closed_at = self.stream.current_position();
        self.expect(TokenKind::End, "Expected 'end' to close block")?;
        let outer_locals = self.close_block_scope(block_opened_at, &parameters);
        let parameters = self.block_parameters_or_refuse(
            parameters,
            wrote_parameter_list,
            body_opened_at,
            body_closed_at,
        )?;

        Ok(Expression::Lambda {
            parameters,
            parameter_defaults,
            body,
            captured_vars: None, // Will be filled by semantic analysis
            outer_locals,
            is_lambda: false,
            position: start_pos,
        })
    }

    /// Parse the body of a block/do/def up to `end`, allowing an implicit
    /// `begin...end` wrapper — i.e. `rescue`/`else`/`ensure` clauses right
    /// inside the block (Ruby 2.5+ semantics). When any of those clauses are
    /// present the body is wrapped in a `BeginRescue` expression statement.
    /// Consumes body + trailing clauses; caller must still consume `end`.
    pub(crate) fn parse_block_body_with_optional_rescue_ensure(
        &mut self,
        start_pos: crate::lexer::Position,
    ) -> Result<Vec<Statement>, MetorexError> {
        let mut body = Vec::new();
        while !self.check(&[
            TokenKind::End,
            TokenKind::Rescue,
            TokenKind::Else,
            TokenKind::Ensure,
        ]) && !self.is_at_end()
        {
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        let has_clauses = self.check(&[TokenKind::Rescue, TokenKind::Else, TokenKind::Ensure]);
        if !has_clauses {
            return Ok(body);
        }

        let mut rescue_clauses = Vec::new();
        while self.match_token(&[TokenKind::Rescue]) {
            rescue_clauses.push(self.parse_rescue_clause()?);
            self.skip_whitespace();
        }

        let else_clause = if self.match_token(&[TokenKind::Else]) {
            self.skip_whitespace();
            let mut else_body = Vec::new();
            while !self.check(&[TokenKind::Ensure, TokenKind::End]) && !self.is_at_end() {
                else_body.push(self.parse_statement()?);
                self.skip_whitespace();
            }
            Some(else_body)
        } else {
            None
        };

        let ensure_block = if self.match_token(&[TokenKind::Ensure]) {
            self.skip_whitespace();
            let mut ensure_body = Vec::new();
            while !self.check(&[TokenKind::End]) && !self.is_at_end() {
                ensure_body.push(self.parse_statement()?);
                self.skip_whitespace();
            }
            Some(ensure_body)
        } else {
            None
        };

        let begin_rescue = Expression::BeginRescue {
            body,
            rescue_clauses,
            else_clause,
            ensure_block,
            position: start_pos,
        };
        Ok(vec![Statement::Expression {
            expression: begin_rescue,
            position: start_pos,
        }])
    }

    /// Parse a block with brace syntax: { |x| ... }
    /// The parameters a block declares, or the numbered ones its body names
    /// when it declares none. `{ _1 + _2 }` takes two parameters, spelled
    /// `_1` and `_2`, which is how Ruby reads a block written that way.
    /// The parameters a block takes, refusing a body that mixes `it` with
    /// parameters written in pipes or with numbered ones.
    fn block_parameters_or_refuse(
        &mut self,
        declared: Vec<String>,
        wrote_parameter_list: bool,
        opened_at: usize,
        closed_at: usize,
    ) -> Result<Vec<String>, MetorexError> {
        let mentions_it = self.mentions_implicit_it(opened_at, closed_at);
        let position = self.peek().position;
        // The body has been read, so a bare `it` inside a block written
        // within it belongs to that block rather than to this one.
        self.block_body_ranges.push((opened_at, closed_at));
        // A block that names `it` among its parameters means that name where
        // it is written, so the body reads it as an ordinary variable.
        let names_it = declared.iter().any(|held| {
            held.trim_start_matches(['*', '&']) == "it"
                || held.trim_start_matches(crate::object::KEYWORD_PARAM_PREFIX) == "it"
        });
        if mentions_it && wrote_parameter_list && !names_it {
            return Err(MetorexError::syntax_error(
                "'it' is not allowed when an ordinary parameter is defined".to_string(),
                crate::error::SourceLocation::new(position.line, position.column, position.offset),
            ));
        }
        let numbered_at = self.mentions_a_numbered_parameter(opened_at, closed_at);
        if numbered_at.is_some() && wrote_parameter_list {
            return Err(MetorexError::syntax_error(
                "a numbered parameter is not allowed when an ordinary parameter is defined"
                    .to_string(),
                crate::error::SourceLocation::new(position.line, position.column, position.offset),
            ));
        }
        // A block whose body reads a numbered parameter cannot hold another
        // that reads one: which block the name belongs to is ambiguous.
        if numbered_at.is_some()
            && self.numbered_parameter_ranges.iter().any(|(from, to)| {
                (*from, *to) != (opened_at, closed_at) && *from >= opened_at && *to <= closed_at
            })
        {
            return Err(MetorexError::syntax_error(
                "numbered parameter is already used in an outer block".to_string(),
                crate::error::SourceLocation::new(position.line, position.column, position.offset),
            ));
        }
        if numbered_at.is_some() {
            self.numbered_parameter_ranges.push((opened_at, closed_at));
        }
        if mentions_it && self.numbered_parameter_comes_first(opened_at, closed_at) {
            return Err(MetorexError::syntax_error(
                "'it' is not allowed when a numbered parameter is already used".to_string(),
                crate::error::SourceLocation::new(position.line, position.column, position.offset),
            ));
        }
        let settled = self.with_numbered_parameters(declared, opened_at, closed_at);
        if mentions_it && settled.first().is_some_and(|held| held.starts_with('_')) {
            return Err(MetorexError::syntax_error(
                "numbered parameters are not allowed when 'it' is already used".to_string(),
                crate::error::SourceLocation::new(position.line, position.column, position.offset),
            ));
        }
        Ok(settled)
    }

    /// Where this block's body reads a numbered parameter, ignoring the
    /// blocks written inside it, which take numbered parameters of their own.
    fn mentions_a_numbered_parameter(&self, opened_at: usize, closed_at: usize) -> Option<usize> {
        let tokens = self.stream.tokens();
        for (offset, token) in tokens[opened_at..closed_at].iter().enumerate() {
            let at = opened_at + offset;
            let TokenKind::Ident(name) = &token.kind else {
                continue;
            };
            if !crate::parser::names_a_numbered_parameter(name) {
                continue;
            }
            // `:_1` names a symbol and `held._1` a method, neither of which
            // is the numbered parameter.
            if at > 0
                && matches!(
                    tokens[at - 1].kind,
                    TokenKind::Colon | TokenKind::Dot | TokenKind::SafeDot | TokenKind::Def
                )
            {
                continue;
            }
            if self.inside_a_nested_block(at, opened_at, closed_at) {
                continue;
            }
            return Some(at);
        }
        None
    }

    /// Whether a numbered parameter is written before the first bare `it` in
    /// this block's body. Ruby names whichever came first in the refusal.
    fn numbered_parameter_comes_first(&self, opened_at: usize, closed_at: usize) -> bool {
        let tokens = self.stream.tokens();
        for (offset, token) in tokens[opened_at..closed_at].iter().enumerate() {
            let at = opened_at + offset;
            if self.inside_a_nested_block(at, opened_at, closed_at) {
                continue;
            }
            let TokenKind::Ident(name) = &token.kind else {
                continue;
            };
            if name == "it" {
                return false;
            }
            if let Some(digit) = name.strip_prefix('_')
                && digit.len() == 1
                && digit
                    .chars()
                    .next()
                    .is_some_and(|held| held.is_ascii_digit())
            {
                return true;
            }
        }
        false
    }

    /// Whether the token at `at` sits inside a block written within the
    /// body running from `opened_at` to `closed_at`.
    fn inside_a_nested_block(&self, at: usize, opened_at: usize, closed_at: usize) -> bool {
        self.block_body_ranges.iter().any(|(from, to)| {
            // A body read twice, as the walk does when it backtracks over a
            // lambda's parameters, records the same range again. That is
            // this body, not a block written inside it.
            (*from, *to) != (opened_at, closed_at)
                && *from >= opened_at
                && *to <= closed_at
                && at >= *from
                && at < *to
        })
    }

    fn with_numbered_parameters(
        &self,
        declared: Vec<String>,
        opened_at: usize,
        closed_at: usize,
    ) -> Vec<String> {
        if !declared.is_empty() {
            return declared;
        }
        let mut highest = 0;
        for token in &self.stream.tokens()[opened_at..closed_at] {
            if let TokenKind::Ident(name) = &token.kind
                && let Some(digit) = name.strip_prefix('_')
                && digit.len() == 1
                && let Some(place) = digit.chars().next().and_then(|c| c.to_digit(10))
                && place >= 1
            {
                highest = highest.max(place as usize);
            }
        }
        if highest > 0 {
            return (1..=highest).map(|place| format!("_{}", place)).collect();
        }
        // A block that names no parameters and mentions a bare `it` takes the
        // first argument under that name, which is the implicit parameter
        // Ruby gives it.
        if self.mentions_implicit_it(opened_at, closed_at) {
            return vec![crate::object::IMPLICIT_IT_PARAM.to_string()];
        }
        Vec::new()
    }

    /// Whether a block body reads a bare `it` as a value. A name written
    /// where a call goes, after a dot, or as the target of an assignment is
    /// not the implicit parameter.
    fn mentions_implicit_it(&self, opened_at: usize, closed_at: usize) -> bool {
        // A local named `it` bound before the block opens is what a bare
        // `it` in the block reads.
        if self.names_a_local_at("it", opened_at) {
            return false;
        }
        let tokens = self.stream.tokens();
        // An assignment to `it` makes it a local from there on, so only a
        // mention ahead of the first one is the implicit parameter.
        let first_assignment = tokens[opened_at..closed_at]
            .windows(2)
            .enumerate()
            .find(|(offset, pair)| {
                matches!(&pair[0].kind, TokenKind::Ident(name) if name == "it")
                    && matches!(pair[1].kind, TokenKind::Equal)
                    && !self.inside_a_nested_block(opened_at + offset, opened_at, closed_at)
            })
            .map_or(closed_at, |(offset, _)| opened_at + offset);
        for (offset, token) in tokens[opened_at..first_assignment].iter().enumerate() {
            let TokenKind::Ident(name) = &token.kind else {
                continue;
            };
            if name != "it" {
                continue;
            }
            let at = opened_at + offset;
            // A bare `it` inside a block written in this body is that
            // block's implicit parameter, not this one's.
            if self.inside_a_nested_block(at, opened_at, closed_at) {
                continue;
            }
            if at > 0
                && matches!(
                    tokens[at - 1].kind,
                    TokenKind::Dot | TokenKind::SafeDot | TokenKind::Def
                )
            {
                continue;
            }
            // `it` stands for the argument only where a value stands. A name
            // followed by anything that could open an argument list or a
            // block is the method of that name being called.
            let reads_as_value = matches!(
                tokens.get(at + 1).map(|held| &held.kind),
                None | Some(TokenKind::Newline)
                    | Some(TokenKind::Semicolon)
                    | Some(TokenKind::Comment(_))
                    | Some(TokenKind::RBrace)
                    | Some(TokenKind::RParen)
                    | Some(TokenKind::RBracket)
                    | Some(TokenKind::Comma)
                    | Some(TokenKind::End)
                    | Some(TokenKind::Dot)
                    | Some(TokenKind::SafeDot)
                    | Some(TokenKind::Plus)
                    | Some(TokenKind::Minus)
                    | Some(TokenKind::Star)
                    | Some(TokenKind::Slash)
                    | Some(TokenKind::Percent)
                    | Some(TokenKind::EqualEqual)
                    | Some(TokenKind::BangEqual)
                    | Some(TokenKind::Less)
                    | Some(TokenKind::Greater)
                    | Some(TokenKind::LessEqual)
                    | Some(TokenKind::GreaterEqual)
                    | Some(TokenKind::Spaceship)
                    | Some(TokenKind::LogicalAnd)
                    | Some(TokenKind::LogicalOr)
                    | Some(TokenKind::KeywordAnd)
                    | Some(TokenKind::KeywordOr)
                    | Some(TokenKind::Question)
                    | Some(TokenKind::Then)
                    | Some(TokenKind::If)
                    | Some(TokenKind::Unless)
                    | Some(TokenKind::While)
                    | Some(TokenKind::Until)
                    | Some(TokenKind::Rescue)
            );
            if !reads_as_value {
                continue;
            }
            return true;
        }
        false
    }

    pub(crate) fn parse_brace_block(&mut self) -> Result<Expression, MetorexError> {
        // A block body is its own run of statements, so `and` and `or` bind
        // there the way they do anywhere else.
        let held = std::mem::take(&mut self.assignment_rhs_depth);
        self.block_body_depth += 1;
        let parsed = self.parse_brace_block_body();
        self.block_body_depth -= 1;
        self.assignment_rhs_depth = held;
        parsed
    }

    pub(crate) fn parse_brace_block_body(&mut self) -> Result<Expression, MetorexError> {
        let start_pos = self.peek().position;
        let block_opened_at = self.stream.current_position();

        // Expect '{' to start block
        self.expect(TokenKind::LBrace, "Expected '{' to start block")?;
        self.skip_whitespace();

        let (parameters, parameter_defaults) = self.parse_block_pipe_params()?;
        let wrote_parameter_list = self.wrote_block_parameter_list;

        self.skip_whitespace();

        // Parse block body (single expression or statements)
        let body_opened_at = self.stream.current_position();
        let mut body = Vec::new();
        self.jump_target_depth += 1;
        self.enter_block_parameters(&parameters);
        let collected = (|| -> Result<(), MetorexError> {
            while !self.check(&[TokenKind::RBrace]) && !self.is_at_end() {
                // For brace blocks, we typically expect a single expression
                // but we'll parse statements to be flexible
                body.push(self.parse_statement()?);
                self.skip_whitespace();
            }
            Ok(())
        })();
        self.leave_block_parameters();
        self.jump_target_depth -= 1;
        collected?;
        let body_closed_at = self.stream.current_position();

        self.expect(TokenKind::RBrace, "Expected '}' to close block")?;
        let outer_locals = self.close_block_scope(block_opened_at, &parameters);
        let parameters = self.block_parameters_or_refuse(
            parameters,
            wrote_parameter_list,
            body_opened_at,
            body_closed_at,
        )?;

        Ok(Expression::Lambda {
            parameters,
            parameter_defaults,
            body,
            captured_vars: None, // Will be filled by semantic analysis
            outer_locals,
            is_lambda: false,
            position: start_pos,
        })
    }
}
