// Parser module for Metorex
// Converts a stream of tokens into an Abstract Syntax Tree (AST)

use crate::ast::{BinaryOp, Expression, Parameter, Statement, UnaryOp};
use crate::error::{MetorexError, SourceLocation};
use crate::lexer::{Position, Token, TokenKind};

/// The parser converts a token stream into an AST
pub struct Parser {
    /// The tokens to parse
    tokens: Vec<Token>,
    /// Current position in the token stream
    current: usize,
    /// Errors encountered during parsing
    errors: Vec<MetorexError>,
    /// Flag indicating if we're in panic mode (error recovery)
    panic_mode: bool,
}

impl Parser {
    /// Create a new parser from a vector of tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
            panic_mode: false,
        }
    }

    /// Get the current token without consuming it
    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or_else(|| {
            // If we're past the end, return the last token (should be EOF)
            self.tokens.last().unwrap()
        })
    }

    /// Get the token at an offset from the current position
    #[allow(dead_code)]
    fn peek_ahead(&self, offset: usize) -> &Token {
        self.tokens
            .get(self.current + offset)
            .unwrap_or_else(|| self.tokens.last().unwrap())
    }

    /// Get the previous token
    fn previous(&self) -> &Token {
        if self.current > 0 {
            &self.tokens[self.current - 1]
        } else {
            &self.tokens[0]
        }
    }

    /// Check if we're at the end of the token stream
    fn is_at_end(&self) -> bool {
        matches!(self.peek().kind, TokenKind::EOF)
    }

    /// Advance to the next token and return the previous one
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous().clone()
    }

    /// Check if the current token matches any of the given kinds
    fn check(&self, kinds: &[TokenKind]) -> bool {
        if self.is_at_end() {
            return false;
        }
        kinds.iter().any(|kind| self.match_kind(kind))
    }

    /// Check if the current token matches a specific kind (handles complex matching)
    fn match_kind(&self, kind: &TokenKind) -> bool {
        let current = &self.peek().kind;
        match (kind, current) {
            (TokenKind::Ident(_), TokenKind::Ident(_)) => true,
            (TokenKind::Int(_), TokenKind::Int(_)) => true,
            (TokenKind::Float(_), TokenKind::Float(_)) => true,
            (TokenKind::String(_), TokenKind::String(_)) => true,
            (TokenKind::InterpolatedString(_), TokenKind::InterpolatedString(_)) => true,
            (TokenKind::InstanceVar(_), TokenKind::InstanceVar(_)) => true,
            (TokenKind::ClassVar(_), TokenKind::ClassVar(_)) => true,
            (TokenKind::Comment(_), TokenKind::Comment(_)) => true,
            _ => kind == current,
        }
    }

    /// Consume the current token if it matches any of the given kinds
    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        if self.check(kinds) {
            self.advance();
            true
        } else {
            false
        }
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

    /// Skip any newline tokens
    #[allow(dead_code)]
    fn skip_newlines(&mut self) {
        while self.match_token(&[TokenKind::Newline]) {
            // Keep consuming newlines
        }
    }

    /// Skip any comment tokens
    #[allow(dead_code)]
    fn skip_comments(&mut self) {
        while matches!(self.peek().kind, TokenKind::Comment(_)) {
            self.advance();
        }
    }

    /// Skip newlines and comments
    fn skip_whitespace(&mut self) {
        while let TokenKind::Newline | TokenKind::Comment(_) = &self.peek().kind {
            self.advance();
        }
    }

    /// Convert a Position to a SourceLocation
    fn position_to_location(&self, position: Position) -> SourceLocation {
        SourceLocation::new(position.line, position.column, position.offset)
    }

    /// Create an error at the current token
    fn error_at_current(&self, message: &str) -> MetorexError {
        let location = self.position_to_location(self.peek().position);
        MetorexError::syntax_error(message, location)
    }

    /// Create an error at the previous token
    fn error_at_previous(&self, message: &str) -> MetorexError {
        let location = self.position_to_location(self.previous().position);
        MetorexError::syntax_error(message, location)
    }

    /// Report an error and enter panic mode
    fn report_error(&mut self, error: MetorexError) {
        if !self.panic_mode {
            self.panic_mode = true;
            self.errors.push(error);
        }
    }

    /// Synchronize after an error (panic mode recovery)
    /// Skip tokens until we find a statement boundary
    fn synchronize(&mut self) {
        self.panic_mode = false;

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

        if self.errors.is_empty() {
            Ok(statements)
        } else {
            Err(self.errors.clone())
        }
    }

    /// Parse a single statement
    fn parse_statement(&mut self) -> Result<Statement, MetorexError> {
        // Skip leading whitespace
        self.skip_whitespace();

        let token = self.peek().clone();
        match &token.kind {
            TokenKind::Class => self.parse_class_def(),
            TokenKind::Def => self.parse_function_def(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            _ => {
                // Try to parse as an expression or assignment
                let expr = self.parse_expression()?;

                // Check if this is an assignment
                if self.check(&[
                    TokenKind::Equal,
                    TokenKind::PlusEqual,
                    TokenKind::MinusEqual,
                    TokenKind::StarEqual,
                    TokenKind::SlashEqual,
                ]) {
                    let op_token = self.advance();
                    let value = self.parse_expression()?;

                    // Convert compound assignment to regular assignment with binary op
                    let final_value = match op_token.kind {
                        TokenKind::PlusEqual => Expression::BinaryOp {
                            op: BinaryOp::Add,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::MinusEqual => Expression::BinaryOp {
                            op: BinaryOp::Subtract,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::StarEqual => Expression::BinaryOp {
                            op: BinaryOp::Multiply,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::SlashEqual => Expression::BinaryOp {
                            op: BinaryOp::Divide,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::Equal => value,
                        _ => unreachable!(),
                    };

                    Ok(Statement::Assignment {
                        target: expr,
                        value: final_value,
                        position: token.position,
                    })
                } else {
                    // It's just an expression statement
                    Ok(Statement::Expression {
                        expression: expr,
                        position: token.position,
                    })
                }
            }
        }
    }

    /// Parse a class definition
    fn parse_class_def(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::Class, "Expected 'class'")?.position;
        self.skip_whitespace();

        let name = match self.advance().kind {
            TokenKind::Ident(name) => name,
            _ => return Err(self.error_at_previous("Expected class name")),
        };

        self.skip_whitespace();

        // Check for superclass
        let superclass = if self.match_token(&[TokenKind::Less]) {
            self.skip_whitespace();
            match self.advance().kind {
                TokenKind::Ident(parent) => Some(parent),
                _ => return Err(self.error_at_previous("Expected superclass name")),
            }
        } else {
            None
        };

        self.skip_whitespace();

        // Parse class body
        let mut body = Vec::new();
        while !self.check(&[TokenKind::End]) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(&[TokenKind::End]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        self.expect(TokenKind::End, "Expected 'end' after class body")?;

        Ok(Statement::ClassDef {
            name,
            superclass,
            body,
            position: start_pos,
        })
    }

    /// Parse a function definition
    fn parse_function_def(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::Def, "Expected 'def'")?.position;
        self.skip_whitespace();

        let name = match self.advance().kind {
            TokenKind::Ident(name) => name,
            _ => return Err(self.error_at_previous("Expected function name")),
        };

        self.skip_whitespace();

        // Parse parameters (optional parentheses)
        let parameters = if self.match_token(&[TokenKind::LParen]) {
            self.parse_parameters()?
        } else {
            Vec::new()
        };

        self.skip_whitespace();

        // Parse function body
        let mut body = Vec::new();
        while !self.check(&[TokenKind::End]) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(&[TokenKind::End]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        self.expect(TokenKind::End, "Expected 'end' after function body")?;

        Ok(Statement::FunctionDef {
            name,
            parameters,
            body,
            position: start_pos,
        })
    }

    /// Parse function parameters
    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, MetorexError> {
        let mut params = Vec::new();
        self.skip_whitespace();

        if self.check(&[TokenKind::RParen]) {
            self.advance();
            return Ok(params);
        }

        loop {
            self.skip_whitespace();

            let param_pos = self.peek().position;

            // Check for variadic parameter (*args)
            if self.match_token(&[TokenKind::Star]) {
                let name = match self.advance().kind {
                    TokenKind::Ident(name) => name,
                    _ => return Err(self.error_at_previous("Expected parameter name after '*'")),
                };
                params.push(Parameter::variadic(name, param_pos));
            } else {
                let name = match self.advance().kind {
                    TokenKind::Ident(name) => name,
                    _ => return Err(self.error_at_previous("Expected parameter name")),
                };

                // Check for default value
                if self.match_token(&[TokenKind::Equal]) {
                    let default = self.parse_expression()?;
                    params.push(Parameter::with_default(name, default, param_pos));
                } else {
                    params.push(Parameter::simple(name, param_pos));
                }
            }

            self.skip_whitespace();

            if !self.match_token(&[TokenKind::Comma]) {
                break;
            }
        }

        self.skip_whitespace();
        self.expect(TokenKind::RParen, "Expected ')' after parameters")?;

        Ok(params)
    }

    /// Parse an if statement
    fn parse_if_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::If, "Expected 'if'")?.position;
        self.skip_whitespace();

        let condition = self.parse_expression()?;
        self.skip_whitespace();

        // Parse then branch
        let mut then_branch = Vec::new();
        while !self.check(&[TokenKind::Else, TokenKind::End]) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(&[TokenKind::Else, TokenKind::End]) {
                break;
            }
            then_branch.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        // Parse optional else branch
        let else_branch = if self.match_token(&[TokenKind::Else]) {
            self.skip_whitespace();
            let mut else_stmts = Vec::new();
            while !self.check(&[TokenKind::End]) && !self.is_at_end() {
                self.skip_whitespace();
                if self.check(&[TokenKind::End]) {
                    break;
                }
                else_stmts.push(self.parse_statement()?);
                self.skip_whitespace();
            }
            Some(else_stmts)
        } else {
            None
        };

        self.expect(TokenKind::End, "Expected 'end' after if statement")?;

        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
            position: start_pos,
        })
    }

    /// Parse a while loop
    fn parse_while_statement(&mut self) -> Result<Statement, MetorexError> {
        let start_pos = self.expect(TokenKind::While, "Expected 'while'")?.position;
        self.skip_whitespace();

        let condition = self.parse_expression()?;
        self.skip_whitespace();

        // Optionally consume 'do'
        self.match_token(&[TokenKind::Do]);
        self.skip_whitespace();

        // Parse loop body
        let mut body = Vec::new();
        while !self.check(&[TokenKind::End]) && !self.is_at_end() {
            self.skip_whitespace();
            if self.check(&[TokenKind::End]) {
                break;
            }
            body.push(self.parse_statement()?);
            self.skip_whitespace();
        }

        self.expect(TokenKind::End, "Expected 'end' after while loop")?;

        Ok(Statement::While {
            condition,
            body,
            position: start_pos,
        })
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
