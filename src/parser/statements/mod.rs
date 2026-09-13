// Statement parsing module
// Handles parsing of all statement types

mod attributes;
mod class;
mod control_flow;
mod exception;
pub(crate) mod function;

use crate::ast::{BinaryOp, Expression, Statement};
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

/// Whether an expression names something an assignment can write to. Ruby
/// rejects anything else while parsing, so `1 + 1 = 2` never reaches the VM.
fn is_assignable(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::Identifier { .. }
            | Expression::TopLevelConstant { .. }
            | Expression::InstanceVariable { .. }
            | Expression::ClassVariable { .. }
            | Expression::GlobalVariable { .. }
            | Expression::ScopeResolution { .. }
            | Expression::Index { .. }
            | Expression::MethodCall { .. }
            | Expression::Splat { .. }
    )
}

impl Parser {
    /// Parse a single statement
    pub(crate) fn parse_statement(&mut self) -> Result<Statement, MetorexError> {
        // A statement is never itself part of a paren-less argument list, even
        // when it sits in a block body inside one. Clearing the depth here is
        // what lets `guard -> { a or b }` read the `or` as its own.
        let enclosing_arg_depth = self.paren_less_arg_depth;
        self.paren_less_arg_depth = 0;
        let parsed = self.parse_statement_inner();
        self.paren_less_arg_depth = enclosing_arg_depth;
        parsed
    }

    fn parse_statement_inner(&mut self) -> Result<Statement, MetorexError> {
        // Skip leading whitespace
        self.skip_whitespace();

        // Nothing at all is not a statement. A group left open at the end of
        // the source reaches here, and stops rather than reading on.
        if self.is_at_end() {
            return Err(self.error_at_previous("Unexpected end of input"));
        }

        // `BEGIN { ... }` runs its body once before the rest of the file,
        // and `END { ... }` runs its body once when the program ends, which
        // is what `at_exit` does.
        if let TokenKind::Ident(name) = &self.peek().kind
            && matches!(name.as_str(), "BEGIN" | "END")
            && matches!(self.peek_ahead(1).kind, TokenKind::LBrace)
        {
            let position = self.peek().position;
            let opens = name == "BEGIN";
            // Ruby says so where an `END` block sits inside a method, since
            // the block is registered again on every call.
            if !opens && self.def_body_depth > 0 {
                eprintln!("{}: warning: END in method; use at_exit", position.line);
            }
            self.advance();
            let block = self.parse_brace_block()?;
            let called = if opens {
                "__begin_once__"
            } else {
                "__end_once__"
            };
            return Ok(Statement::Expression {
                expression: Expression::Call {
                    callee: Box::new(Expression::Identifier {
                        name: called.to_string(),
                        position,
                    }),
                    arguments: Vec::new(),
                    trailing_block: Some(Box::new(block)),
                    position,
                },
                position,
            });
        }

        // Recognize `private def …` / `public def …` / `protected def …` /
        // `module_function def …` as a special two-keyword form. The
        // visibility modifier is parsed and discarded; the wrapped `def`
        // becomes a normal method definition.
        if let TokenKind::Ident(name) = &self.peek().kind
            && matches!(
                name.as_str(),
                "private" | "public" | "protected" | "module_function"
            )
            && matches!(self.peek_ahead(1).kind, TokenKind::Def)
        {
            self.advance(); // consume the visibility ident
            self.skip_whitespace();
            return self.parse_function_def();
        }

        let token = self.peek().clone();
        match &token.kind {
            TokenKind::Class => {
                // `class << target; …; end` is an expression whose value is the
                // singleton class, so it may be chained (`.ancestors`, `==`, …).
                // Route it through the expression parser; `class Name` remains
                // a statement-level definition.
                if matches!(self.peek_ahead(1).kind, TokenKind::Shovel) {
                    let expr = self.parse_expression_with_lambda()?;
                    let stmt = Statement::Expression {
                        expression: expr,
                        position: token.position,
                    };
                    self.wrap_with_modifier(stmt)
                } else {
                    self.parse_class_def()
                }
            }
            TokenKind::Def => self.parse_function_def(),
            TokenKind::If => self.control_flow_statement(token.position, Self::parse_if_statement),
            TokenKind::Unless => {
                self.control_flow_statement(token.position, Self::parse_unless_statement)
            }
            TokenKind::While => {
                self.control_flow_statement(token.position, Self::parse_while_statement)
            }
            TokenKind::Until => {
                self.control_flow_statement(token.position, Self::parse_until_statement)
            }
            TokenKind::For => {
                self.control_flow_statement(token.position, Self::parse_for_statement)
            }
            TokenKind::Case => {
                self.control_flow_statement(token.position, Self::parse_case_statement)
            }
            TokenKind::Begin => {
                self.control_flow_statement(token.position, Self::parse_begin_statement)
            }
            TokenKind::Raise => self.parse_raise_statement(),
            TokenKind::Break => self.parse_break_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            TokenKind::Redo => self.parse_redo_statement(),
            TokenKind::Retry => self.parse_retry_statement(),
            TokenKind::Return => {
                let stmt = self.parse_return_statement()?;
                self.wrap_with_modifier(stmt)
            }
            TokenKind::AttrReader => self.parse_attr_reader(),
            TokenKind::AttrWriter => self.parse_attr_writer(),
            TokenKind::AttrAccessor => self.parse_attr_accessor(),
            TokenKind::Module => self.parse_module_def(),
            TokenKind::Include => self.parse_include(),
            TokenKind::Extend => self.parse_extend(),
            TokenKind::Alias => self.parse_alias(),
            TokenKind::Ident(name) if name == "undef" => self.parse_undef(),
            _ => {
                // `*rest, last = value` opens with the splat, which no
                // expression can start with, so the target list is read
                // before anything else is tried.
                if matches!(token.kind, TokenKind::Star) && self.scans_assignment_targets(0, false)
                {
                    let stmt = self.finish_multiple_assignment(None, token.position)?;
                    return self.wrap_with_modifier(stmt);
                }
                // `(a, b), c = pair, held` opens with a group of targets,
                // which reads as a parenthesized expression until the `=`
                // says what it was.
                if matches!(token.kind, TokenKind::LParen) && self.scans_grouped_targets() {
                    let stmt = self.finish_multiple_assignment(None, token.position)?;
                    return self.wrap_with_modifier(stmt);
                }
                // Try to parse as an expression or assignment (including arrow lambdas)
                let expr = self.parse_expression_with_lambda()?;

                // Check for multiple assignment: a, b, c = ...
                // Look ahead to verify there's an = after the comma-separated identifiers
                // `obj.field` names a target too, which reads as a call with
                // no arguments until the `=` turns it into a setter.
                let assignable_target = matches!(
                    expr,
                    Expression::Identifier { .. }
                        | Expression::Index { .. }
                        | Expression::InstanceVariable { .. }
                        | Expression::ClassVariable { .. }
                        | Expression::GlobalVariable { .. }
                        | Expression::ScopeResolution { .. }
                ) || matches!(&expr, Expression::MethodCall { method, arguments, trailing_block, .. }
                    if (arguments.is_empty() || method == "[]") && trailing_block.is_none());
                if assignable_target
                    && self.check(&[TokenKind::Comma])
                    && self.scans_assignment_targets(1, true)
                {
                    let stmt = self.finish_multiple_assignment(Some(expr), token.position)?;
                    return self.wrap_with_modifier(stmt);
                }

                // Check if this is an assignment
                if self.check(&[
                    TokenKind::Equal,
                    TokenKind::PlusEqual,
                    TokenKind::MinusEqual,
                    TokenKind::StarEqual,
                    TokenKind::SlashEqual,
                    TokenKind::PercentEqual,
                    TokenKind::StarStarEqual,
                    TokenKind::PipeEqual,
                    TokenKind::AmpersandEqual,
                    TokenKind::CaretEqual,
                    TokenKind::ShovelEqual,
                    TokenKind::RightShiftEqual,
                    TokenKind::LogicalOrAssign,
                    TokenKind::LogicalAndAssign,
                ]) {
                    if !is_assignable(&expr) {
                        return Err(self.error_at_current("Cannot assign to this expression"));
                    }
                    let op_token = self.advance();
                    // The value may open on the next line, which is how a
                    // long right-hand side is written.
                    self.skip_whitespace();
                    let value = self.parse_assignment_rhs()?;

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
                        TokenKind::PercentEqual => Expression::BinaryOp {
                            op: BinaryOp::Modulo,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::StarStarEqual => Expression::BinaryOp {
                            op: BinaryOp::Power,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::PipeEqual => Expression::BinaryOp {
                            op: BinaryOp::BitwiseOr,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::AmpersandEqual => Expression::BinaryOp {
                            op: BinaryOp::BitwiseAnd,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::CaretEqual => Expression::BinaryOp {
                            op: BinaryOp::Xor,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        // The shifts are methods rather than operators, so
                        // `held <<= 2` calls the one the receiver defines.
                        TokenKind::ShovelEqual | TokenKind::RightShiftEqual => {
                            Expression::MethodCall {
                                receiver: Box::new(expr.clone()),
                                method: if matches!(op_token.kind, TokenKind::ShovelEqual) {
                                    "<<".to_string()
                                } else {
                                    ">>".to_string()
                                },
                                arguments: vec![value],
                                trailing_block: None,
                                position: op_token.position,
                            }
                        }
                        TokenKind::LogicalOrAssign => Expression::BinaryOp {
                            op: BinaryOp::Or,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::LogicalAndAssign => Expression::BinaryOp {
                            op: BinaryOp::And,
                            left: Box::new(expr.clone()),
                            right: Box::new(value),
                            position: op_token.position,
                        },
                        TokenKind::Equal => value,
                        _ => unreachable!(),
                    };

                    let stmt = Statement::Assignment {
                        target: expr,
                        value: final_value,
                        position: token.position,
                    };
                    let stmt = self.fold_keyword_logic(stmt)?;
                    self.wrap_with_modifier(stmt)
                } else {
                    // It's just an expression statement
                    let stmt = Statement::Expression {
                        expression: expr,
                        position: token.position,
                    };
                    self.wrap_with_modifier(stmt)
                }
            }
        }
    }

    /// Look ahead to check if this is a multiple assignment (a, b = ...)
    /// by scanning comma-separated identifiers until we find = or something else.
    fn scans_assignment_targets(&self, mut offset: usize, had_first: bool) -> bool {
        let mut seen = had_first;
        loop {
            // Skip whitespace tokens
            let tok = self.peek_ahead(offset);
            if matches!(
                tok.kind,
                TokenKind::Newline | TokenKind::Comment(_) | TokenKind::Semicolon
            ) {
                offset += 1;
                continue;
            }
            // `first, = pair` names one target and destructures anyway, so
            // the list may end on the comma that opened this step.
            if matches!(tok.kind, TokenKind::Equal) {
                return seen;
            }
            // `*rest` takes whatever the targets around it leave, and a
            // splat with no name after it takes them and keeps none.
            let tok = if matches!(tok.kind, TokenKind::Star) {
                offset += 1;
                let next = self.peek_ahead(offset);
                if matches!(next.kind, TokenKind::Equal) {
                    return true;
                }
                if matches!(next.kind, TokenKind::Comma) {
                    seen = true;
                    offset += 1;
                    continue;
                }
                next
            } else {
                tok
            };
            // A group of targets stands where a name may, as the `(y, z)`
            // of `x, (y, z) = held, pair` does.
            if matches!(tok.kind, TokenKind::LParen) {
                let mut depth = 0;
                loop {
                    match &self.peek_ahead(offset).kind {
                        TokenKind::LParen => depth += 1,
                        TokenKind::RParen => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        TokenKind::EOF | TokenKind::Newline => return false,
                        _ => {}
                    }
                    offset += 1;
                }
                offset += 1;
                match &self.peek_ahead(offset).kind {
                    TokenKind::Equal => return true,
                    TokenKind::Comma => {
                        seen = true;
                        offset += 1;
                        continue;
                    }
                    _ => return false,
                }
            }
            // Expect an identifier (or @ivar, @@cvar, $gvar). `lambda` is a
            // method rather than syntax, so a program may name a local after
            // it and assign to that name.
            if !matches!(
                tok.kind,
                TokenKind::Ident(_)
                    | TokenKind::InstanceVar(_)
                    | TokenKind::ClassVar(_)
                    | TokenKind::GlobalVar(_)
                    | TokenKind::Lambda
            ) {
                return false;
            }
            offset += 1;
            // Skip bracket indexing (e.g., a[0])
            if matches!(self.peek_ahead(offset).kind, TokenKind::LBracket) {
                offset += 1; // skip [
                let mut depth = 1;
                while depth > 0 {
                    let inner = &self.peek_ahead(offset).kind;
                    if matches!(inner, TokenKind::LBracket) {
                        depth += 1;
                    } else if matches!(inner, TokenKind::RBracket) {
                        depth -= 1;
                    } else if matches!(inner, TokenKind::EOF) {
                        return false;
                    }
                    offset += 1;
                }
            }
            // `m::A, m::B = :a, :b` names constants under a module, which
            // are targets the same way a name is.
            while matches!(self.peek_ahead(offset).kind, TokenKind::ColonColon) {
                offset += 1;
                if matches!(self.peek_ahead(offset).kind, TokenKind::Ident(_)) {
                    offset += 1;
                } else {
                    return false;
                }
            }
            // Skip dot+method chains (e.g., obj.field)
            while matches!(self.peek_ahead(offset).kind, TokenKind::Dot) {
                offset += 1; // skip .
                if matches!(self.peek_ahead(offset).kind, TokenKind::Ident(_)) {
                    offset += 1; // skip method name
                } else {
                    return false;
                }
            }
            // After the target, expect comma or =
            let next = self.peek_ahead(offset);
            if matches!(next.kind, TokenKind::Equal) {
                return true;
            }
            if matches!(next.kind, TokenKind::Comma) {
                seen = true;
                offset += 1;
                continue;
            }
            return false;
        }
    }

    /// Whether a statement opening with `(` is a multiple assignment whose
    /// first target is a group, as `(a, b), c = pair, held` is.
    fn scans_grouped_targets(&self) -> bool {
        let mut offset = 0;
        let mut depth = 0;
        let mut commas = 0;
        loop {
            match &self.peek_ahead(offset).kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                TokenKind::Comma if depth == 1 => commas += 1,
                // `(*a) = held` names one target and takes the value apart
                // anyway, which is what the splat says.
                TokenKind::Star if depth == 1 => commas += 1,
                TokenKind::EOF | TokenKind::Newline => return false,
                _ => {}
            }
            offset += 1;
        }
        offset += 1;
        while matches!(
            self.peek_ahead(offset).kind,
            TokenKind::Comment(_) | TokenKind::Semicolon
        ) {
            offset += 1;
        }
        match &self.peek_ahead(offset).kind {
            TokenKind::Equal => commas > 0,
            TokenKind::Comma => self.scans_assignment_targets(offset + 1, true),
            _ => false,
        }
    }

    /// One target in a multiple assignment, which may carry a leading `*`
    /// marking it as the one that takes everything the others leave.
    fn parse_assignment_target(&mut self) -> Result<Expression, MetorexError> {
        // `(a, b), c = pair, held` groups targets, and a group may hold
        // further groups, which is what takes a nested Array apart.
        if self.check(&[TokenKind::LParen]) {
            let opened = self.advance();
            let mut grouped = Vec::new();
            loop {
                self.skip_whitespace();
                if self.check(&[TokenKind::RParen]) {
                    break;
                }
                grouped.push(self.parse_assignment_target()?);
                self.skip_whitespace();
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
            self.expect(TokenKind::RParen, "Expected ')' after grouped targets")?;
            return Ok(Expression::Array {
                elements: grouped,
                position: opened.position,
            });
        }
        if self.check(&[TokenKind::Star]) {
            let star = self.advance();
            self.skip_whitespace();
            // A splat with no name after it takes the values the other
            // targets leave and keeps none of them.
            if self.check(&[TokenKind::Equal, TokenKind::Comma, TokenKind::RParen]) {
                return Ok(Expression::Splat {
                    expression: Box::new(Expression::Array {
                        elements: Vec::new(),
                        position: star.position,
                    }),
                    position: star.position,
                });
            }
            let expression = self.parse_expression_with_lambda()?;
            return Ok(Expression::Splat {
                expression: Box::new(expression),
                position: star.position,
            });
        }
        self.parse_expression_with_lambda()
    }

    /// The rest of a multiple assignment, given the first target when one has
    /// already been read as an expression.
    fn finish_multiple_assignment(
        &mut self,
        first: Option<Expression>,
        position: crate::lexer::Position,
    ) -> Result<Statement, MetorexError> {
        let first = match first {
            Some(first) => first,
            None => self.parse_assignment_target()?,
        };
        let mut targets = vec![first];
        while self.match_token(&[TokenKind::Comma]) {
            self.skip_whitespace();
            // A trailing comma before the `=` names no further target.
            if self.check(&[TokenKind::Equal]) {
                break;
            }
            targets.push(self.parse_assignment_target()?);
        }
        // `(a, b) = pair` wraps the whole list in a group, which names the
        // same targets as writing them bare does.
        if targets.len() == 1
            && let Expression::Array { elements, .. } = &targets[0]
        {
            targets = elements.clone();
        }
        self.expect(TokenKind::Equal, "Expected '=' in multiple assignment")?;
        self.skip_whitespace();
        let mut values = vec![self.parse_expression_with_lambda()?];
        while self.match_token(&[TokenKind::Comma]) {
            self.skip_whitespace();
            values.push(self.parse_expression_with_lambda()?);
        }
        Ok(Statement::MultipleAssignment {
            targets,
            values,
            position,
        })
    }

    /// Check for postfix if/unless modifiers and wrap the statement.
    /// Only matches if the modifier is on the same line (no newline before it).
    /// A control-flow form read as a statement, unless a `.` follows its
    /// `end`. That marks the form as being read for its value, so it is
    /// parsed again as an expression and the chained call lands on what it
    /// answers.
    fn control_flow_statement(
        &mut self,
        position: crate::lexer::Position,
        parse: fn(&mut Self) -> Result<Statement, MetorexError>,
    ) -> Result<Statement, MetorexError> {
        let opened_at = self.stream.current_position();
        let parsed = parse(self)?;
        if !self.check(&[TokenKind::Dot]) {
            // `begin ... end while cond` and the rest of the modifiers read
            // the block they follow.
            return self.wrap_with_modifier(parsed);
        }
        self.stream.restore_position(opened_at);
        let expression = self.parse_expression_with_lambda()?;
        let stmt = Statement::Expression {
            expression,
            position,
        };
        self.wrap_with_modifier(stmt)
    }

    pub(crate) fn wrap_with_modifier(
        &mut self,
        stmt: Statement,
    ) -> Result<Statement, MetorexError> {
        // Don't consume newlines — modifier must be on the same line
        if matches!(self.peek().kind, TokenKind::Newline | TokenKind::Comment(_)) {
            return Ok(stmt);
        }
        // `stmt rescue fallback` runs the fallback when the statement raises a
        // StandardError. A modifier binds to the statement it follows, so
        // nothing may come between them: a `rescue` on its own line opens a
        // clause, and so does one after a semicolon, even on the same line.
        if self.check(&[TokenKind::Rescue])
            && self.peek().position.line == self.previous().position.line
            && !matches!(
                self.previous().kind,
                TokenKind::Semicolon | TokenKind::Newline
            )
        {
            let position = self.advance().position;
            self.skip_whitespace();
            let fallback = self.parse_expression()?;
            return Ok(Statement::Expression {
                expression: crate::ast::Expression::BeginRescue {
                    body: vec![stmt],
                    rescue_clauses: vec![crate::ast::RescueClause {
                        exception_types: vec!["StandardError".to_string()],
                        variable_name: None,
                        body: vec![Statement::Expression {
                            expression: fallback,
                            position,
                        }],
                        position,
                    }],
                    else_clause: None,
                    ensure_block: None,
                    position,
                },
                position,
            });
        }
        if self.check(&[TokenKind::If]) {
            let position = self.advance().position; // consume 'if'
            self.skip_whitespace();
            let condition = self.parse_condition_expression()?;
            Ok(Statement::If {
                condition,
                then_branch: vec![stmt],
                elsif_branches: vec![],
                else_branch: None,
                position,
            })
        } else if self.check(&[TokenKind::Unless]) {
            let position = self.advance().position; // consume 'unless'
            self.skip_whitespace();
            let condition = self.parse_condition_expression()?;
            Ok(Statement::Unless {
                condition,
                then_branch: vec![stmt],
                else_branch: None,
                position,
            })
        } else if self.check(&[TokenKind::While]) {
            let position = self.advance().position; // consume 'while'
            self.skip_whitespace();
            let condition = self.parse_condition_expression()?;
            // `begin ... end while cond` reads its condition after the body
            // has run, so the body runs at least once.
            if runs_before_the_test(&stmt) {
                return Ok(Statement::DoWhile {
                    condition,
                    body: vec![stmt],
                    position,
                });
            }
            Ok(Statement::While {
                condition,
                body: vec![stmt],
                position,
            })
        } else if self.check(&[TokenKind::Until]) {
            let position = self.advance().position; // consume 'until'
            self.skip_whitespace();
            let condition = self.parse_condition_expression()?;
            let condition = crate::ast::Expression::UnaryOp {
                op: crate::ast::UnaryOp::Not,
                operand: Box::new(condition),
                position,
            };
            if runs_before_the_test(&stmt) {
                return Ok(Statement::DoWhile {
                    condition,
                    body: vec![stmt],
                    position,
                });
            }
            Ok(Statement::While {
                condition,
                body: vec![stmt],
                position,
            })
        } else {
            Ok(stmt)
        }
    }

    /// Parse a condition expression that may contain an assignment (`x = expr`).
    /// In Ruby, `if x = foo()` assigns and tests truthiness.
    fn parse_condition_expression(
        &mut self,
    ) -> Result<crate::ast::Expression, crate::error::MetorexError> {
        let expr = self.parse_condition_operands()?;
        if self.match_token(&[crate::lexer::TokenKind::Equal]) {
            self.skip_whitespace();
            let value = self.parse_expression()?;
            let position = expr.position();
            Ok(crate::ast::Expression::BinaryOp {
                op: crate::ast::BinaryOp::Assign,
                left: Box::new(expr),
                right: Box::new(value),
                position,
            })
        } else {
            Ok(expr)
        }
    }

    /// The condition a modifier reads, where `and` and `or` join the tests the
    /// way they do in a statement of their own.
    fn parse_condition_operands(
        &mut self,
    ) -> Result<crate::ast::Expression, crate::error::MetorexError> {
        use crate::lexer::TokenKind as Kind;
        let mut held = self.parse_expression()?;
        loop {
            let op = if self.check(&[Kind::KeywordAnd]) {
                crate::ast::BinaryOp::And
            } else if self.check(&[Kind::KeywordOr]) {
                crate::ast::BinaryOp::Or
            } else {
                return Ok(held);
            };
            let position = self.advance().position;
            self.skip_whitespace();
            let right = self.parse_expression()?;
            held = crate::ast::Expression::BinaryOp {
                op,
                left: Box::new(held),
                right: Box::new(right),
                position,
            };
        }
    }

    /// Parse the right-hand side of an assignment, supporting chained assignments
    /// like `@a = @b = value`.
    /// `x = 1 and y = 2` assigns each side in turn, since `and` and `or`
    /// bind more loosely than an assignment does.
    fn fold_keyword_logic(
        &mut self,
        stmt: Statement,
    ) -> Result<Statement, crate::error::MetorexError> {
        if !self.check(&[TokenKind::KeywordAnd, TokenKind::KeywordOr]) {
            return Ok(stmt);
        }
        let Statement::Assignment {
            target,
            value,
            position,
        } = stmt
        else {
            return Ok(stmt);
        };
        let mut left = crate::ast::Expression::BinaryOp {
            op: BinaryOp::Assign,
            left: Box::new(target),
            right: Box::new(value),
            position,
        };
        while self.check(&[TokenKind::KeywordAnd, TokenKind::KeywordOr]) {
            let keyword = self.advance();
            self.skip_whitespace();
            let op = if matches!(keyword.kind, TokenKind::KeywordAnd) {
                BinaryOp::And
            } else {
                BinaryOp::Or
            };
            let right = self.parse_statement()?;
            left = crate::ast::Expression::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(statement_as_expression(right, keyword.position)),
                position: keyword.position,
            };
        }
        Ok(Statement::Expression {
            expression: left,
            position,
        })
    }

    /// The operation a compound assignment standing next in the stream
    /// applies, where one stands there at all.
    fn compound_assignment_ahead(&self) -> Option<BinaryOp> {
        let paired = [
            (TokenKind::PlusEqual, BinaryOp::Add),
            (TokenKind::MinusEqual, BinaryOp::Subtract),
            (TokenKind::StarEqual, BinaryOp::Multiply),
            (TokenKind::SlashEqual, BinaryOp::Divide),
            (TokenKind::PercentEqual, BinaryOp::Modulo),
            (TokenKind::StarStarEqual, BinaryOp::Power),
            (TokenKind::PipeEqual, BinaryOp::BitwiseOr),
            (TokenKind::AmpersandEqual, BinaryOp::BitwiseAnd),
            (TokenKind::CaretEqual, BinaryOp::Xor),
            (TokenKind::LogicalOrAssign, BinaryOp::Or),
            (TokenKind::LogicalAndAssign, BinaryOp::And),
        ];
        paired
            .into_iter()
            .find(|(kind, _)| self.check(std::slice::from_ref(kind)))
            .map(|(_, operation)| operation)
    }

    /// The shift a `<<=` or `>>=` standing next in the stream applies. The
    /// shifts are methods rather than operators, so they are named rather
    /// than folded into a binary operation.
    fn shift_assignment_ahead(&self) -> Option<&'static str> {
        if self.check(&[TokenKind::ShovelEqual]) {
            return Some("<<");
        }
        if self.check(&[TokenKind::RightShiftEqual]) {
            return Some(">>");
        }
        None
    }

    fn parse_assignment_rhs(
        &mut self,
    ) -> Result<crate::ast::Expression, crate::error::MetorexError> {
        self.assignment_rhs_depth += 1;
        let parsed = self.parse_expression_with_lambda();
        self.assignment_rhs_depth -= 1;
        let expr = parsed?;
        // `a <<= b <<= 2` works the shifts from the right the same way.
        if let Some(named) = self.shift_assignment_ahead()
            && is_assignable(&expr)
        {
            let position = self.advance().position;
            self.skip_whitespace();
            let value = self.parse_assignment_rhs()?;
            return Ok(crate::ast::Expression::BinaryOp {
                op: crate::ast::BinaryOp::Assign,
                left: Box::new(expr.clone()),
                right: Box::new(crate::ast::Expression::MethodCall {
                    receiver: Box::new(expr),
                    method: named.to_string(),
                    arguments: vec![value],
                    trailing_block: None,
                    position,
                }),
                position,
            });
        }
        // `a %= b %= 3` works the operators from the right, so a compound
        // assignment on the right of one is carried out first.
        if let Some(operation) = self.compound_assignment_ahead()
            && is_assignable(&expr)
        {
            let position = self.advance().position;
            self.skip_whitespace();
            let value = self.parse_assignment_rhs()?;
            return Ok(crate::ast::Expression::BinaryOp {
                op: crate::ast::BinaryOp::Assign,
                left: Box::new(expr.clone()),
                right: Box::new(crate::ast::Expression::BinaryOp {
                    op: operation,
                    left: Box::new(expr),
                    right: Box::new(value),
                    position,
                }),
                position,
            });
        }
        // Check for chained assignment: if the parsed expression is followed by `=`
        // and the expression is an assignable target, parse as nested assignment.
        if self.check(&[crate::lexer::TokenKind::Equal])
            && matches!(
                expr,
                crate::ast::Expression::Identifier { .. }
                    | crate::ast::Expression::InstanceVariable { .. }
                    | crate::ast::Expression::ClassVariable { .. }
                    | crate::ast::Expression::GlobalVariable { .. }
                    | crate::ast::Expression::Index { .. }
                    | crate::ast::Expression::MethodCall { .. }
            )
        {
            let position = self.advance().position;
            self.skip_whitespace();
            let value = self.parse_assignment_rhs()?;
            Ok(crate::ast::Expression::BinaryOp {
                op: crate::ast::BinaryOp::Assign,
                left: Box::new(expr),
                right: Box::new(value),
                position,
            })
        } else if self.paren_less_arg_depth == 0 && self.check(&[crate::lexer::TokenKind::Comma]) {
            // `a, b` on the right of an assignment builds an array, which is
            // how `values[0, 2] = 1, 2, 3` names its replacement.
            let position = self.peek().position;
            let mut elements = vec![expr];
            while self.match_token(&[crate::lexer::TokenKind::Comma]) {
                self.skip_whitespace();
                elements.push(self.parse_expression_with_lambda()?);
            }
            let array = crate::ast::Expression::Array { elements, position };
            self.wrap_with_rescue_modifier(array)
        } else {
            // `a = b rescue c` assigns the fallback, so the modifier binds to
            // the right-hand side rather than to the assignment.
            self.wrap_with_rescue_modifier(expr)
        }
    }
}

/// Whether a statement is the `begin ... end` block Ruby runs before it reads
/// a trailing `while` or `until` condition.
fn runs_before_the_test(stmt: &Statement) -> bool {
    matches!(
        stmt,
        Statement::Begin { .. }
            | Statement::Expression {
                expression: crate::ast::Expression::BeginRescue { .. },
                ..
            }
    )
}

/// One statement read as the expression it stands for, so a `and` or `or`
/// can hold it as an operand.
fn statement_as_expression(stmt: Statement, position: crate::lexer::Position) -> Expression {
    match stmt {
        Statement::Expression { expression, .. } => expression,
        Statement::Assignment {
            target,
            value,
            position,
        } => Expression::BinaryOp {
            op: BinaryOp::Assign,
            left: Box::new(target),
            right: Box::new(value),
            position,
        },
        other => Expression::BeginRescue {
            body: vec![other],
            rescue_clauses: Vec::new(),
            else_clause: None,
            ensure_block: None,
            position,
        },
    }
}
