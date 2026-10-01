// Reading one statement, whatever it opens with.

use super::*;

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
        // `value => pattern` and `value in pattern` stand where a statement
        // does, so the test is read once the value has been.
        let Ok(Statement::Expression {
            expression,
            position,
        }) = parsed
        else {
            return parsed;
        };
        let expression = self.wrap_with_pattern_test(expression)?;
        self.wrap_with_modifier(Statement::Expression {
            expression,
            position,
        })
    }

    pub(crate) fn parse_statement_inner(&mut self) -> Result<Statement, MetorexError> {
        // Skip leading whitespace
        self.skip_whitespace();
        self.statement_start = self.stream.current_position();

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
            // Ruby's parser refuses a `BEGIN` written anywhere but the top
            // level of a code unit.
            if opens && (self.def_body_depth > 0 || self.block_body_depth > 0) {
                return Err(self.error_at_previous("BEGIN is permitted only at toplevel"));
            }
            self.advance();
            let block = self.parse_brace_block()?;
            if opens {
                let Expression::Lambda { body, .. } = block else {
                    return Err(self.error_at_previous("BEGIN expects a block"));
                };
                return Ok(Statement::BeginBlock { body, position });
            }
            return Ok(Statement::Expression {
                expression: Expression::Call {
                    callee: Box::new(Expression::Identifier {
                        name: "__end_once__".to_string(),
                        position,
                    }),
                    arguments: Vec::new(),
                    trailing_block: Some(Box::new(block)),
                    position,
                },
                position,
            });
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
                    let definition = self.parse_class_def()?;
                    self.definition_chained_onto(definition, token.position)
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
            TokenKind::Module => {
                let definition = self.parse_module_def()?;
                self.definition_chained_onto(definition, token.position)
            }
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
                    self.refuse_fixed_target(&expr)?;
                    if !is_assignable(&expr) {
                        return Err(self.error_at_current("Cannot assign to this expression"));
                    }
                    self.refuse_dynamic_constant_assignment(&expr)?;
                    if let Expression::Identifier { name, .. } = &expr
                        && names_a_numbered_parameter(name)
                    {
                        return Err(self.error_at_current(&format!(
                            "{name} is reserved for numbered parameter"
                        )));
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

                    // An `if` whose every branch jumps away answers nothing,
                    // so there is nothing for an assignment to take from it.
                    if reads_as_void(&final_value) {
                        return Err(crate::error::MetorexError::syntax_error(
                            "void value expression",
                            crate::error::SourceLocation::new(
                                token.position.line,
                                token.position.column,
                                token.position.offset,
                            ),
                        ));
                    }
                    let stmt = Statement::Assignment {
                        target: expr,
                        value: final_value,
                        position: token.position,
                    };
                    let stmt = self.fold_keyword_logic(stmt)?;
                    self.wrap_with_modifier(stmt)
                } else {
                    // It's just an expression statement. A `defined?` whose
                    // answer the next statement leaves unused is dropped,
                    // and a verbose run warns about it.
                    let expression = if matches!(expr, Expression::Defined { .. })
                        && self.another_statement_follows()
                    {
                        self.warnings.push((
                            token.position,
                            "possibly useless use of defined? in void context".to_string(),
                        ));
                        Expression::NilLiteral {
                            position: token.position,
                        }
                    } else {
                        expr
                    };
                    let stmt = Statement::Expression {
                        expression,
                        position: token.position,
                    };
                    self.wrap_with_modifier(stmt)
                }
            }
        }
    }
}
