// Reading the right-hand side of an assignment.

use super::*;

impl Parser {
    /// Parse the right-hand side of an assignment, supporting chained assignments
    /// like `@a = @b = value`.
    /// `x = 1 and y = 2` assigns each side in turn, since `and` and `or`
    /// bind more loosely than an assignment does.
    pub(crate) fn fold_keyword_logic(
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
        // An `if` whose every branch jumps away answers nothing, so there is
        // nothing for an assignment to take from it.
        if reads_as_void(&value) {
            return Err(crate::error::MetorexError::syntax_error(
                "void value expression",
                crate::error::SourceLocation::new(position.line, position.column, position.offset),
            ));
        }
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
    pub(crate) fn compound_assignment_ahead(&self) -> Option<BinaryOp> {
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

    /// Refuse a constant assigned inside a method body, which Ruby calls a
    /// dynamic constant assignment, since the method may run many times.
    pub(crate) fn refuse_dynamic_constant_assignment(
        &self,
        target: &Expression,
    ) -> Result<(), MetorexError> {
        let names_a_constant = match target {
            Expression::Identifier { name, .. } => name.starts_with(char::is_uppercase),
            Expression::ScopeResolution { .. } | Expression::TopLevelConstant { .. } => true,
            _ => false,
        };
        if names_a_constant && self.def_body_depth > 0 {
            return Err(self.error_at_current("dynamic constant assignment"));
        }
        Ok(())
    }

    /// Refuse a target Ruby holds fixed: `nil`, `true`, `false`, `self`, and
    /// the globals a regexp match sets.
    pub(crate) fn refuse_fixed_target(&self, target: &Expression) -> Result<(), MetorexError> {
        let refusal = match target {
            Expression::NilLiteral { .. } => "Can't assign to nil".to_string(),
            Expression::BoolLiteral { value, .. } => format!("Can't assign to {value}"),
            Expression::Identifier { name, .. } if name == "self" => {
                "Can't change the value of self".to_string()
            }
            Expression::GlobalVariable { name, .. }
                if matches!(name.as_str(), "&" | "`" | "'" | "+")
                    || (name.starts_with(|first: char| ('1'..='9').contains(&first))
                        && name.chars().all(|held| held.is_ascii_digit())) =>
            {
                format!("Can't set variable ${name}")
            }
            _ => return Ok(()),
        };
        Err(self.error_at_current(&refusal))
    }

    /// Whether `=` or a compound assignment operator stands next in the
    /// stream.
    pub(crate) fn assignment_ahead(&self) -> bool {
        self.check(&[TokenKind::Equal])
            || self.compound_assignment_ahead().is_some()
            || self.shift_assignment_ahead().is_some()
    }

    /// The shift a `<<=` or `>>=` standing next in the stream applies. The
    /// shifts are methods rather than operators, so they are named rather
    /// than folded into a binary operation.
    pub(crate) fn shift_assignment_ahead(&self) -> Option<&'static str> {
        if self.check(&[TokenKind::ShovelEqual]) {
            return Some("<<");
        }
        if self.check(&[TokenKind::RightShiftEqual]) {
            return Some(">>");
        }
        None
    }

    pub(crate) fn parse_assignment_rhs(
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
            let reading = or_assign_reading(&operation, expr.clone());
            return Ok(crate::ast::Expression::BinaryOp {
                op: crate::ast::BinaryOp::Assign,
                left: Box::new(expr),
                right: Box::new(crate::ast::Expression::BinaryOp {
                    op: operation,
                    left: Box::new(reading),
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

    /// A `class` or `module` definition with a call written onto its `end`,
    /// as `class Held; end.name` does. The definition answers what its body
    /// answered, and the call reads from there.
    pub(crate) fn definition_chained_onto(
        &mut self,
        definition: Statement,
        position: crate::lexer::Position,
    ) -> Result<Statement, MetorexError> {
        // `class Name; end if condition` opens the class only when the
        // condition holds.
        if !self.check(&[TokenKind::Dot, TokenKind::SafeDot]) {
            return self.wrap_with_modifier(definition);
        }
        self.seeded_primary = Some(Expression::BeginRescue {
            body: vec![definition],
            rescue_clauses: Vec::new(),
            else_clause: None,
            ensure_block: None,
            position,
        });
        let expression = self.parse_expression()?;
        let statement = Statement::Expression {
            expression,
            position,
        };
        self.wrap_with_modifier(statement)
    }
}

/// Whether an expression answers nothing at all, because every way through it
/// jumps away rather than reaching a value. Ruby refuses to read one of these
/// where a value is wanted.
pub(crate) fn reads_as_void(expression: &crate::ast::Expression) -> bool {
    use crate::ast::{Expression, Statement};
    pub(crate) fn branch_is_void(body: &[Statement]) -> bool {
        match body.last() {
            Some(Statement::Return { .. } | Statement::Break { .. }) => true,
            Some(Statement::Expression { expression, .. }) => reads_as_void(expression),
            _ => false,
        }
    }
    let Expression::If {
        then_branch,
        elsif_branches,
        else_branch,
        ..
    } = expression
    else {
        return false;
    };
    // Without an else branch the conditional answers nil when the test fails,
    // so there is a value to take.
    let Some(otherwise) = else_branch else {
        return false;
    };
    branch_is_void(then_branch)
        && branch_is_void(otherwise)
        && elsif_branches.iter().all(|held| branch_is_void(&held.body))
}

/// Whether a statement is the `begin ... end` block Ruby runs before it reads
/// a trailing `while` or `until` condition.
pub(crate) fn runs_before_the_test(stmt: &Statement) -> bool {
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
pub(crate) fn statement_as_expression(
    stmt: Statement,
    position: crate::lexer::Position,
) -> Expression {
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

/// Whether `name` is one of `_1` through `_9`, which Ruby keeps for the
/// numbered parameters a block takes and refuses to let a program bind.
pub(crate) fn names_a_numbered_parameter(name: &str) -> bool {
    name.len() == 2
        && name.starts_with('_')
        && name[1..]
            .chars()
            .next()
            .is_some_and(|held| held.is_ascii_digit() && held != '0')
}

/// What `target ||= value` reads the target as. A class variable or a
/// constant that is not set yet reads as nil there rather than raising, which
/// is what `defined?(target) && target` answers.
pub(crate) fn or_assign_reading(
    operation: &BinaryOp,
    target: crate::ast::Expression,
) -> crate::ast::Expression {
    use crate::ast::Expression;
    let guarded = matches!(operation, BinaryOp::Or)
        && match &target {
            // `Scope::Name ||= value` evaluates its scope once, which a
            // guard reading the name twice would not.
            Expression::ClassVariable { .. } | Expression::TopLevelConstant { .. } => true,
            Expression::Identifier { name, .. } => name.starts_with(char::is_uppercase),
            _ => false,
        };
    if !guarded {
        return target;
    }
    let position = target.position();
    Expression::BinaryOp {
        op: BinaryOp::And,
        left: Box::new(Expression::Defined {
            expression: Box::new(target.clone()),
            position,
        }),
        right: Box::new(target),
        position,
    }
}
