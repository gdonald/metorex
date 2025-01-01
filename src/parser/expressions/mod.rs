// Expression parsing module
// Handles parsing of all expression types

mod binary;
mod call;
mod primary;
mod unary;

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

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
            let expr = self.parse_assignment()?;

            // Convert expression to a statement
            let body = vec![crate::ast::Statement::Expression {
                expression: expr,
                position: arrow_pos,
            }];

            return Ok(Expression::Lambda {
                parameters: Vec::new(),
                body,
                captured_vars: Some(Vec::new()), // Empty vec signals automatic capture
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
                        body,
                        captured_vars: Some(Vec::new()), // Empty vec signals automatic capture
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
                body,
                captured_vars: Some(Vec::new()), // Empty vec signals automatic capture
                position: expr.position(),
            });
        }

        Ok(expr)
    }

    /// Parse assignment (lowest precedence)
    pub(crate) fn parse_assignment(&mut self) -> Result<Expression, MetorexError> {
        self.parse_equality()
    }

    /// Parse a block: `do |param1, param2| ... end`
    pub(crate) fn parse_block(&mut self) -> Result<Expression, MetorexError> {
        let start_pos = self.peek().position;

        // Expect 'do' keyword
        self.expect(TokenKind::Do, "Expected 'do' to start block")?;
        self.skip_whitespace();

        // Parse block parameters (e.g., |x, y|)
        let parameters = if self.match_token(&[TokenKind::Pipe]) {
            let mut params = Vec::new();
            self.skip_whitespace();

            if !self.check(&[TokenKind::Pipe]) {
                loop {
                    self.skip_whitespace();
                    let param_token = self.advance();
                    match param_token.kind {
                        TokenKind::Ident(name) => params.push(name),
                        _ => return Err(self.error_at_previous("Expected parameter name")),
                    }
                    self.skip_whitespace();

                    if !self.match_token(&[TokenKind::Comma]) {
                        break;
                    }
                }
            }

            self.skip_whitespace();
            self.expect(TokenKind::Pipe, "Expected '|' after block parameters")?;
            params
        } else {
            Vec::new()
        };

        self.skip_whitespace();

        // Parse block body
        let mut body = Vec::new();
        while !self.check(&[TokenKind::End]) && !self.is_at_end() {
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        self.expect(TokenKind::End, "Expected 'end' to close block")?;

        Ok(Expression::Lambda {
            parameters,
            body,
            captured_vars: None, // Will be filled by semantic analysis
            position: start_pos,
        })
    }
}
