// Function and method call parsing
// Handles parsing of function calls, method calls, and array indexing

use crate::ast::Expression;
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse function calls and method calls
    pub(crate) fn parse_call(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&[TokenKind::LParen]) {
                // Function call with parentheses
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenKind::Dot]) {
                // Method call
                let method_name = match self.advance().kind {
                    TokenKind::Ident(name) => name,
                    _ => return Err(self.error_at_previous("Expected method name after '.'")),
                };

                // Check if there are arguments
                let arguments = if self.match_token(&[TokenKind::LParen]) {
                    self.parse_arguments()?
                } else {
                    Vec::new()
                };

                // Check for trailing block (both do...end and {...} syntax)
                let trailing_block = if self.check(&[TokenKind::Do]) {
                    Some(Box::new(self.parse_block()?))
                } else if self.check(&[TokenKind::LBrace]) {
                    Some(Box::new(self.parse_brace_block()?))
                } else {
                    None
                };

                let position = expr.position();
                expr = Expression::MethodCall {
                    receiver: Box::new(expr),
                    method: method_name,
                    arguments,
                    trailing_block,
                    position,
                };
            } else if self.match_token(&[TokenKind::LBracket]) {
                // Array indexing
                let index = self.parse_expression()?;
                self.expect(TokenKind::RBracket, "Expected ']' after array index")?;
                let position = expr.position();
                expr = Expression::Index {
                    array: Box::new(expr),
                    index: Box::new(index),
                    position,
                };
            } else if self.can_start_argument_for_call(&expr) {
                // Ruby-style function call without parentheses
                // Only parse this if we have an identifier as the callee
                if matches!(expr, Expression::Identifier { .. }) {
                    expr = self.finish_call_without_parens(expr)?;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Finish parsing a function call
    pub(crate) fn finish_call(&mut self, callee: Expression) -> Result<Expression, MetorexError> {
        let arguments = self.parse_arguments()?;

        // Check for trailing block (both do...end and {...} syntax)
        let trailing_block = if self.check(&[TokenKind::Do]) {
            Some(Box::new(self.parse_block()?))
        } else if self.check(&[TokenKind::LBrace]) {
            Some(Box::new(self.parse_brace_block()?))
        } else {
            None
        };

        let position = callee.position();

        Ok(Expression::Call {
            callee: Box::new(callee),
            arguments,
            trailing_block,
            position,
        })
    }

    /// Parse function/method arguments (with parentheses)
    pub(crate) fn parse_arguments(&mut self) -> Result<Vec<Expression>, MetorexError> {
        let mut arguments = Vec::new();
        self.skip_whitespace();

        if self.check(&[TokenKind::RParen]) {
            self.advance();
            return Ok(arguments);
        }

        loop {
            self.skip_whitespace();
            arguments.push(self.parse_expression()?);
            self.skip_whitespace();

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
        }

        self.skip_whitespace();
        self.expect(TokenKind::RParen, "Expected ')' after arguments")?;

        Ok(arguments)
    }

    /// Check if the next token can start an argument in a parentheses-less call
    /// Also checks if this looks like a dictionary context (value followed by colon)
    fn can_start_argument_for_call(&mut self, _callee: &Expression) -> bool {
        // Don't skip whitespace yet - we need to check if there's a newline first
        // Parentheses-less calls should be on the same line
        if matches!(self.peek().kind, TokenKind::Newline | TokenKind::Comment(_)) {
            return false;
        }

        self.skip_whitespace();

        // Don't parse as function call if we see operators or punctuation that
        // indicate we're in a different context (like dictionary key: value)
        // Also check for binary operators that shouldn't start an argument
        if matches!(
            self.peek().kind,
            TokenKind::Colon
                | TokenKind::RBrace
                | TokenKind::Comma
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Percent
                | TokenKind::Equal
                | TokenKind::EqualEqual
                | TokenKind::BangEqual
                | TokenKind::Less
                | TokenKind::Greater
                | TokenKind::LessEqual
                | TokenKind::GreaterEqual
        ) {
            return false;
        }

        // Check if next token can be an argument
        let can_be_arg = matches!(
            self.peek().kind,
            TokenKind::Ident(_)
                | TokenKind::Int(_)
                | TokenKind::Float(_)
                | TokenKind::String(_)
                | TokenKind::InterpolatedString(_)
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Nil
                | TokenKind::LBracket
                | TokenKind::InstanceVar(_)
                | TokenKind::ClassVar(_)
        );

        if !can_be_arg {
            return false;
        }

        // Look ahead to detect dictionary patterns
        // Pattern 1: <arg> ':' - clearly dict syntax like {x: 1}
        if matches!(self.peek_ahead(1).kind, TokenKind::Colon) {
            return false;
        }

        // Pattern 2: <arg> ',' - suggests dict with missing colon like {x 1, y: 2}
        // In valid code, function arguments are separated by commas, but they're inside
        // parentheses. At statement level, we don't expect immediate comma after arg.
        if matches!(self.peek_ahead(1).kind, TokenKind::Comma) {
            return false;
        }

        // Pattern 3: <arg> '}' - suggests dict with missing colon like {x 1}
        if matches!(self.peek_ahead(1).kind, TokenKind::RBrace) {
            return false;
        }

        true
    }

    /// Finish parsing a function call without parentheses (Ruby-style)
    fn finish_call_without_parens(
        &mut self,
        callee: Expression,
    ) -> Result<Expression, MetorexError> {
        let mut arguments = Vec::new();
        let position = callee.position();

        // Parse first argument
        self.skip_whitespace();
        arguments.push(self.parse_call()?);
        self.skip_whitespace();

        // After parsing the first argument, check if we see a colon
        // This would indicate we're in a dictionary literal, not a function call
        if self.check(&[TokenKind::Colon]) {
            // We misidentified this as a function call
            // Return an error - the dictionary parser will handle this correctly
            return Err(
                self.error_at_current("Expected function call but found dictionary-like syntax")
            );
        }

        // Parse remaining arguments if there are commas
        while self.match_token(&[TokenKind::Comma]) {
            self.skip_whitespace();

            // Stop if we hit a statement terminator or keyword
            if self.check(&[
                TokenKind::Newline,
                TokenKind::Semicolon,
                TokenKind::End,
                TokenKind::Else,
                TokenKind::Rescue,
                TokenKind::Ensure,
                TokenKind::Do,
            ]) || self.is_at_end()
            {
                break;
            }

            arguments.push(self.parse_call()?);
            self.skip_whitespace();
        }

        Ok(Expression::Call {
            callee: Box::new(callee),
            arguments,
            trailing_block: None,
            position,
        })
    }
}
