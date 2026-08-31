// Expression parsing module
// Handles parsing of all expression types

mod binary;
mod call;
mod primary;
mod unary;

use crate::ast::{Expression, Statement};
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

/// Parsed block parameter list: names (with `*` / `&` prefixes preserved)
/// paired with default-value expressions keyed by parameter index.
type BlockParams = (Vec<String>, Vec<(usize, Expression)>);

impl Parser {
    /// Parse an expression using operator precedence climbing
    pub(crate) fn parse_expression(&mut self) -> Result<Expression, MetorexError> {
        self.parse_assignment()
    }

    /// Parse expression with arrow lambda support (for top-level expressions only)
    pub(crate) fn parse_expression_with_lambda(&mut self) -> Result<Expression, MetorexError> {
        self.parse_arrow_lambda()
    }

    /// Parse arrow lambda syntax: x -> expr, (x, y) -> expr, or -> expr
    pub(crate) fn parse_arrow_lambda(&mut self) -> Result<Expression, MetorexError> {
        // Check for zero-param arrow lambda: -> expr
        if self.check(&[TokenKind::Arrow]) {
            let arrow_pos = self.advance().position;
            self.skip_whitespace();

            // -> { body } or -> do body end — parse as brace/do block
            if self.check(&[TokenKind::LBrace]) {
                let block = self.parse_brace_block()?;
                if let Expression::Lambda {
                    parameters,
                    body,
                    position,
                    ..
                } = block
                {
                    let lambda = Expression::Lambda {
                        parameters,
                        parameter_defaults: Vec::new(),
                        body,
                        captured_vars: Some(Vec::new()),
                        is_lambda: true,
                        position,
                    };
                    return self.parse_postfix_calls(lambda);
                }
            } else if self.check(&[TokenKind::Do]) {
                let block = self.parse_block()?;
                if let Expression::Lambda {
                    parameters,
                    body,
                    position,
                    ..
                } = block
                {
                    let lambda = Expression::Lambda {
                        parameters,
                        parameter_defaults: Vec::new(),
                        body,
                        captured_vars: Some(Vec::new()),
                        is_lambda: true,
                        position,
                    };
                    return self.parse_postfix_calls(lambda);
                }
            }

            // -> (params) { body } or -> (params) do body end
            if self.check(&[TokenKind::LParen]) {
                self.advance(); // consume (
                let mut params = Vec::new();
                self.skip_whitespace();
                if !self.check(&[TokenKind::RParen]) {
                    loop {
                        self.skip_whitespace();
                        params.extend(self.parse_lambda_parameter());
                        self.skip_whitespace();
                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RParen, "Expected ')' after lambda parameters")?;
                self.skip_whitespace();
                if self.check(&[TokenKind::LBrace]) {
                    let block = self.parse_brace_block()?;
                    if let Expression::Lambda { body, position, .. } = block {
                        return Ok(Expression::Lambda {
                            parameters: params,
                            parameter_defaults: Vec::new(),
                            body,
                            captured_vars: Some(Vec::new()),
                            is_lambda: true,
                            position,
                        });
                    }
                } else if self.check(&[TokenKind::Do]) {
                    let block = self.parse_block()?;
                    if let Expression::Lambda { body, position, .. } = block {
                        return Ok(Expression::Lambda {
                            parameters: params,
                            parameter_defaults: Vec::new(),
                            body,
                            captured_vars: Some(Vec::new()),
                            is_lambda: true,
                            position,
                        });
                    }
                }
                let expr = self.parse_assignment()?;
                return Ok(Expression::Lambda {
                    parameters: params,
                    parameter_defaults: Vec::new(),
                    body: vec![crate::ast::Statement::Expression {
                        expression: expr,
                        position: arrow_pos,
                    }],
                    captured_vars: Some(Vec::new()),
                    is_lambda: true,
                    position: arrow_pos,
                });
            }

            // Paren-less params: `-> e { ... }` / `-> a, b { ... }`. Collect
            // a comma-separated identifier list; if a `{` or `do` follows,
            // those are the lambda's parameters. Otherwise backtrack to the
            // bare `-> expr` form below.
            if matches!(self.peek().kind, TokenKind::Ident(_) | TokenKind::Star) {
                let saved_position = self.stream().current_position();
                let mut params = Vec::new();
                loop {
                    match self.parse_lambda_parameter() {
                        Some(param) => params.push(param),
                        None => {
                            params.clear();
                            break;
                        }
                    }
                    self.skip_whitespace();
                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                    self.skip_whitespace();
                }
                if !params.is_empty() && self.check(&[TokenKind::LBrace, TokenKind::Do]) {
                    let block = if self.check(&[TokenKind::LBrace]) {
                        self.parse_brace_block()?
                    } else {
                        self.parse_block()?
                    };
                    if let Expression::Lambda { body, position, .. } = block {
                        return Ok(Expression::Lambda {
                            parameters: params,
                            parameter_defaults: Vec::new(),
                            body,
                            captured_vars: Some(Vec::new()),
                            is_lambda: true,
                            position,
                        });
                    }
                }
                let stream = &mut self.stream;
                stream.restore_position(saved_position);
            }

            let expr = self.parse_assignment()?;

            let body = vec![crate::ast::Statement::Expression {
                expression: expr,
                position: arrow_pos,
            }];

            return Ok(Expression::Lambda {
                parameters: Vec::new(),
                parameter_defaults: Vec::new(),
                body,
                captured_vars: Some(Vec::new()),
                is_lambda: true,
                position: arrow_pos,
            });
        }

        // Special case: check for multi-param arrow lambda: (x, y) -> expr
        // Use lookahead to avoid consuming tokens unless it's definitely a lambda
        if self.check(&[TokenKind::LParen]) {
            let saved_position = self.stream().current_position();
            let start_pos = self.peek().position;

            self.advance(); // consume '('
            self.skip_whitespace();

            // Try to parse as comma-separated identifiers
            let mut params = Vec::new();
            let mut is_param_list = true;

            if !self.check(&[TokenKind::RParen]) {
                loop {
                    self.skip_whitespace();
                    if let TokenKind::Ident(name) = self.peek().kind.clone() {
                        params.push(name);
                        self.advance();
                    } else {
                        is_param_list = false;
                        break;
                    }

                    self.skip_whitespace();
                    if self.match_token(&[TokenKind::Comma]) {
                        continue;
                    } else {
                        break;
                    }
                }
            }

            self.skip_whitespace();

            // Check if this looks like a parameter list: ) followed by ->
            if is_param_list && self.check(&[TokenKind::RParen]) {
                self.advance(); // consume ')'
                self.skip_whitespace();

                if self.check(&[TokenKind::Arrow]) {
                    // It's a multi-param arrow lambda!
                    let arrow_pos = self.advance().position;
                    self.skip_whitespace();

                    let body_expr = self.parse_assignment()?;
                    let body = vec![crate::ast::Statement::Expression {
                        expression: body_expr,
                        position: arrow_pos,
                    }];

                    return Ok(Expression::Lambda {
                        parameters: params,
                        parameter_defaults: Vec::new(),
                        body,
                        captured_vars: Some(Vec::new()), // Empty vec signals automatic capture
                        is_lambda: true,
                        position: start_pos,
                    });
                }
            }

            // Not a multi-param lambda, backtrack and parse normally
            let stream = &mut self.stream;
            stream.restore_position(saved_position);
        }

        // Try to parse as regular expression first
        let expr = self.parse_assignment()?;

        // Check if there's an arrow after the expression
        if self.check(&[TokenKind::Arrow]) {
            let arrow_pos = self.advance().position;
            self.skip_whitespace();

            // Extract parameters from the left side
            let parameters = match &expr {
                // Single parameter: x -> expr
                Expression::Identifier { name, .. } => {
                    vec![name.clone()]
                }
                // Multiple parameters: (x, y) -> expr
                Expression::Grouped { expression, .. } => {
                    // Check if it's a tuple of identifiers (we'll handle this as comma-separated for now)
                    // For now, we'll just support single grouped identifier
                    if let Expression::Identifier { name, .. } = expression.as_ref() {
                        vec![name.clone()]
                    } else {
                        return Err(
                            self.error_at_current("Arrow lambda parameters must be identifiers")
                        );
                    }
                }
                _ => {
                    return Err(self.error_at_current("Left side of arrow must be parameter(s)"));
                }
            };

            // Parse the lambda body
            let body_expr = self.parse_assignment()?;
            let body = vec![crate::ast::Statement::Expression {
                expression: body_expr,
                position: arrow_pos,
            }];

            return Ok(Expression::Lambda {
                parameters,
                parameter_defaults: Vec::new(),
                body,
                captured_vars: Some(Vec::new()), // Empty vec signals automatic capture
                is_lambda: true,
                position: expr.position(),
            });
        }

        Ok(expr)
    }

    /// Parse assignment (lowest precedence)
    /// Parse a condition expression that may contain an inline assignment.
    /// Used in if/unless/while conditions where `var = expr` is valid.
    pub(crate) fn parse_condition(&mut self) -> Result<Expression, MetorexError> {
        let expr = self.parse_expression()?;

        // Check for inline assignment: ident = expr (in condition context)
        if matches!(expr, Expression::Identifier { .. }) && self.check(&[TokenKind::Equal]) {
            let position = self.advance().position;
            self.skip_whitespace();
            let value = self.parse_expression()?;
            return Ok(Expression::BinaryOp {
                op: crate::ast::BinaryOp::Assign,
                left: Box::new(expr),
                right: Box::new(value),
                position,
            });
        }

        Ok(expr)
    }

    pub(crate) fn parse_assignment(&mut self) -> Result<Expression, MetorexError> {
        let expr = self.parse_logical_or()?;

        // Note: assignment (`=`) is handled at the statement level.
        // parse_assignment only deals with ternary operators.

        // Ternary operator: condition ? true_expr : false_expr
        if self.check(&[TokenKind::Question]) {
            let position = self.advance().position; // consume ?
            self.skip_whitespace();
            self.ternary_depth += 1;
            let then_expr = self.parse_assignment()?;
            self.ternary_depth -= 1;
            self.skip_whitespace();
            self.expect(TokenKind::Colon, "Expected ':' in ternary expression")?;
            self.skip_whitespace();
            let else_expr = self.parse_assignment()?;
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
        if self.match_token(&[TokenKind::LogicalOr]) {
            // Empty parameter list: ||
            return Ok((params, defaults));
        }
        if !self.match_token(&[TokenKind::Pipe]) {
            return Ok((params, defaults));
        }
        self.skip_whitespace();
        if !self.check(&[TokenKind::Pipe]) {
            loop {
                self.skip_whitespace();
                let prefix = if self.match_token(&[TokenKind::Star]) {
                    "*"
                } else if self.match_token(&[TokenKind::Ampersand]) {
                    "&"
                } else {
                    ""
                };
                self.skip_whitespace();
                let param_token = self.advance();
                match param_token.kind {
                    TokenKind::Ident(name) => {
                        params.push(format!("{}{}", prefix, name));
                    }
                    _ => return Err(self.error_at_previous("Expected parameter name")),
                }
                self.skip_whitespace();
                if self.match_token(&[TokenKind::Equal]) {
                    self.skip_whitespace();
                    let default = self.parse_range()?;
                    defaults.push((params.len() - 1, default));
                    self.skip_whitespace();
                }
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
                self.skip_whitespace();
                // `|a,|` — trailing comma before the closing pipe. Record it:
                // it is what makes a lone array argument destructure.
                if self.check(&[TokenKind::Pipe]) {
                    params.push(crate::object::TRAILING_COMMA_PARAM.to_string());
                    break;
                }
            }
        }
        self.skip_whitespace();
        self.expect(TokenKind::Pipe, "Expected '|' after block parameters")?;
        Ok((params, defaults))
    }

    /// Parse a block: `do |param1, param2| ... end`
    pub(crate) fn parse_block(&mut self) -> Result<Expression, MetorexError> {
        let start_pos = self.peek().position;

        // Expect 'do' keyword
        self.expect(TokenKind::Do, "Expected 'do' to start block")?;
        self.skip_whitespace();

        let (parameters, parameter_defaults) = self.parse_block_pipe_params()?;

        self.skip_whitespace();

        let body = self.parse_block_body_with_optional_rescue_ensure(start_pos)?;
        self.expect(TokenKind::End, "Expected 'end' to close block")?;

        Ok(Expression::Lambda {
            parameters,
            parameter_defaults,
            body,
            captured_vars: None, // Will be filled by semantic analysis
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
    pub(crate) fn parse_brace_block(&mut self) -> Result<Expression, MetorexError> {
        let start_pos = self.peek().position;

        // Expect '{' to start block
        self.expect(TokenKind::LBrace, "Expected '{' to start block")?;
        self.skip_whitespace();

        let (parameters, parameter_defaults) = self.parse_block_pipe_params()?;

        self.skip_whitespace();

        // Parse block body (single expression or statements)
        let mut body = Vec::new();
        while !self.check(&[TokenKind::RBrace]) && !self.is_at_end() {
            // For brace blocks, we typically expect a single expression
            // but we'll parse statements to be flexible
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        self.expect(TokenKind::RBrace, "Expected '}' to close block")?;

        Ok(Expression::Lambda {
            parameters,
            parameter_defaults,
            body,
            captured_vars: None, // Will be filled by semantic analysis
            is_lambda: false,
            position: start_pos,
        })
    }
}
