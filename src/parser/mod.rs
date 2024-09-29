// Parser module for Metorex
// Converts a stream of tokens into an Abstract Syntax Tree (AST)

mod error;
mod statements;
mod token_stream;

use crate::ast::{BinaryOp, Expression, Statement, UnaryOp};
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
}

impl Parser {
    /// Create a new parser from a vector of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            stream: TokenStream::new(tokens),
            error_handler: ErrorHandler::new(),
        }
    }

    /// Get the current token without consuming it
    fn peek(&self) -> &Token {
        self.stream.peek()
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

    /// Parse an expression using operator precedence climbing
    fn parse_expression(&mut self) -> Result<Expression, MetorexError> {
        self.parse_assignment()
    }

    /// Parse assignment (lowest precedence)
    fn parse_assignment(&mut self) -> Result<Expression, MetorexError> {
        self.parse_equality()
    }

    /// Parse equality operators (==, !=)
    fn parse_equality(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_comparison()?;

        while self.check(&[TokenKind::EqualEqual, TokenKind::BangEqual]) {
            let op_token = self.advance();
            let op = match op_token.kind {
                TokenKind::EqualEqual => BinaryOp::Equal,
                TokenKind::BangEqual => BinaryOp::NotEqual,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            expr = Expression::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// Parse comparison operators (<, >, <=, >=)
    fn parse_comparison(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_term()?;

        while self.check(&[
            TokenKind::Less,
            TokenKind::Greater,
            TokenKind::LessEqual,
            TokenKind::GreaterEqual,
        ]) {
            let op_token = self.advance();
            let op = match op_token.kind {
                TokenKind::Less => BinaryOp::Less,
                TokenKind::Greater => BinaryOp::Greater,
                TokenKind::LessEqual => BinaryOp::LessEqual,
                TokenKind::GreaterEqual => BinaryOp::GreaterEqual,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            expr = Expression::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// Parse addition and subtraction
    fn parse_term(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_factor()?;

        while self.check(&[TokenKind::Plus, TokenKind::Minus]) {
            let op_token = self.advance();
            let op = match op_token.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            expr = Expression::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// Parse multiplication, division, and modulo
    fn parse_factor(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_unary()?;

        while self.check(&[TokenKind::Star, TokenKind::Slash, TokenKind::Percent]) {
            let op_token = self.advance();
            let op = match op_token.kind {
                TokenKind::Star => BinaryOp::Multiply,
                TokenKind::Slash => BinaryOp::Divide,
                TokenKind::Percent => BinaryOp::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            expr = Expression::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// Parse unary operators (+, -)
    fn parse_unary(&mut self) -> Result<Expression, MetorexError> {
        if self.check(&[TokenKind::Plus, TokenKind::Minus]) {
            let op_token = self.advance();
            let op = match op_token.kind {
                TokenKind::Plus => UnaryOp::Plus,
                TokenKind::Minus => UnaryOp::Minus,
                _ => unreachable!(),
            };
            let operand = self.parse_unary()?;
            Ok(Expression::UnaryOp {
                op,
                operand: Box::new(operand),
                position: op_token.position,
            })
        } else {
            self.parse_call()
        }
    }

    /// Parse function calls and method calls
    fn parse_call(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(&[TokenKind::LParen]) {
                // Function call
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

                let position = expr.position();
                expr = Expression::MethodCall {
                    receiver: Box::new(expr),
                    method: method_name,
                    arguments,
                    trailing_block: None,
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
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Finish parsing a function call
    fn finish_call(&mut self, callee: Expression) -> Result<Expression, MetorexError> {
        let arguments = self.parse_arguments()?;
        let position = callee.position();

        Ok(Expression::Call {
            callee: Box::new(callee),
            arguments,
            trailing_block: None,
            position,
        })
    }

    /// Parse function/method arguments
    fn parse_arguments(&mut self) -> Result<Vec<Expression>, MetorexError> {
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

    /// Parse primary expressions (literals, identifiers, groups)
    fn parse_primary(&mut self) -> Result<Expression, MetorexError> {
        let token = self.advance();

        match token.kind {
            // Literals
            TokenKind::Int(value) => Ok(Expression::IntLiteral {
                value,
                position: token.position,
            }),
            TokenKind::Float(value) => Ok(Expression::FloatLiteral {
                value,
                position: token.position,
            }),
            TokenKind::String(value) => Ok(Expression::StringLiteral {
                value,
                position: token.position,
            }),
            TokenKind::InterpolatedString(parts) => {
                // Convert token interpolation parts to AST interpolation parts
                let mut ast_parts = Vec::new();
                for part in parts {
                    match part {
                        crate::lexer::InterpolationPart::Text(text) => {
                            ast_parts.push(crate::ast::node::InterpolationPart::Text(text));
                        }
                        crate::lexer::InterpolationPart::Expression(expr_str) => {
                            // Parse the expression string
                            // For now, we'll create a simple parser for the embedded expression
                            let expr_lexer = crate::lexer::Lexer::new(&expr_str);
                            let expr_tokens = expr_lexer.tokenize();
                            let mut expr_parser = Parser::new(expr_tokens);
                            let expr = expr_parser.parse_expression()?;
                            ast_parts.push(crate::ast::node::InterpolationPart::Expression(
                                Box::new(expr),
                            ));
                        }
                    }
                }
                Ok(Expression::InterpolatedString {
                    parts: ast_parts,
                    position: token.position,
                })
            }
            TokenKind::True => Ok(Expression::BoolLiteral {
                value: true,
                position: token.position,
            }),
            TokenKind::False => Ok(Expression::BoolLiteral {
                value: false,
                position: token.position,
            }),
            TokenKind::Nil => Ok(Expression::NilLiteral {
                position: token.position,
            }),

            // Identifiers and variables
            TokenKind::Ident(name) => Ok(Expression::Identifier {
                name,
                position: token.position,
            }),
            TokenKind::InstanceVar(name) => Ok(Expression::InstanceVariable {
                name,
                position: token.position,
            }),
            TokenKind::ClassVar(name) => Ok(Expression::ClassVariable {
                name,
                position: token.position,
            }),

            // Grouped expression
            TokenKind::LParen => {
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen, "Expected ')' after expression")?;
                Ok(Expression::Grouped {
                    expression: Box::new(expr),
                    position: token.position,
                })
            }

            // Array literal
            TokenKind::LBracket => {
                let mut elements = Vec::new();
                self.skip_whitespace();

                if !self.check(&[TokenKind::RBracket]) {
                    loop {
                        self.skip_whitespace();
                        elements.push(self.parse_expression()?);
                        self.skip_whitespace();

                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }

                self.skip_whitespace();
                self.expect(TokenKind::RBracket, "Expected ']' after array elements")?;

                Ok(Expression::Array {
                    elements,
                    position: token.position,
                })
            }

            // Dictionary literal
            TokenKind::LBrace => {
                let mut entries = Vec::new();
                self.skip_whitespace();

                if !self.check(&[TokenKind::RBrace]) {
                    loop {
                        self.skip_whitespace();
                        let key = self.parse_expression()?;
                        self.skip_whitespace();
                        self.expect(TokenKind::Colon, "Expected ':' after dictionary key")?;
                        self.skip_whitespace();
                        let value = self.parse_expression()?;
                        entries.push((key, value));
                        self.skip_whitespace();

                        if !self.match_token(&[TokenKind::Comma]) {
                            break;
                        }
                    }
                }

                self.skip_whitespace();
                self.expect(TokenKind::RBrace, "Expected '}' after dictionary entries")?;

                Ok(Expression::Dictionary {
                    entries,
                    position: token.position,
                })
            }

            _ => Err(self.error_at_previous(&format!("Unexpected token: {:?}", token.kind))),
        }
    }
}
