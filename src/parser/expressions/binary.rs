// Binary operator parsing
// Handles parsing of binary operations with proper precedence

use crate::ast::{BinaryOp, Expression};
use crate::error::MetorexError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parse logical OR (||)
    ///
    /// `or` tests what `||` tests, but it binds more loosely than a paren-less
    /// call's arguments: `check x or fallback` calls `check x` and tests what
    /// it answered, where `check x || fallback` passes the whole test as the
    /// argument. So an argument list leaves `or` for the call to be folded
    /// into afterwards.
    pub(crate) fn parse_logical_or(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_logical_and()?;

        let keyword_binds_here = self.paren_less_arg_depth == 0
            && self.assignment_rhs_depth == 0
            && self.range_operand_depth == 0;
        while self.check(&[TokenKind::LogicalOr])
            || (keyword_binds_here && self.check(&[TokenKind::KeywordOr]))
        {
            let op_token = self.advance();
            self.skip_whitespace();
            let right = self.parse_logical_and()?;
            let right = self.fold_assignment(right)?;
            expr = Expression::BinaryOp {
                op: BinaryOp::Or,
                left: Box::new(expr),
                right: Box::new(right),
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// Parse logical AND (&&). `and` binds the way `or` does, and stops at
    /// the edge of a paren-less argument list for the same reason.
    pub(crate) fn parse_logical_and(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_equality()?;

        let keyword_binds_here = self.paren_less_arg_depth == 0
            && self.assignment_rhs_depth == 0
            && self.range_operand_depth == 0;
        while self.check(&[TokenKind::LogicalAnd])
            || (keyword_binds_here && self.check(&[TokenKind::KeywordAnd]))
        {
            let op_token = self.advance();
            self.skip_whitespace();
            let right = self.parse_equality()?;
            let right = self.fold_assignment(right)?;
            expr = Expression::BinaryOp {
                op: BinaryOp::And,
                left: Box::new(expr),
                right: Box::new(right),
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// The operators that answer whether two values stand in some relation.
    /// None of them chains, so `1 == 2 == 3` is not a program.
    const RELATIONS: &'static [TokenKind] = &[
        TokenKind::EqualEqual,
        TokenKind::BangEqual,
        TokenKind::TripleEqual,
        TokenKind::Match,
        TokenKind::NotMatch,
        TokenKind::Spaceship,
    ];

    /// Parse equality operators (==, !=, ===, =~, !~, <=>)
    pub(crate) fn parse_equality(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_comparison()?;

        if self.check(Self::RELATIONS) {
            let op_token = self.advance();
            if matches!(op_token.kind, TokenKind::Match | TokenKind::NotMatch) {
                // =~ and !~ are dispatched as method calls
                self.skip_whitespace();
                let right = self.parse_comparison()?;
                // A pattern written out on the left leaves its named
                // captures behind as local variables.
                let named = matches!(op_token.kind, TokenKind::Match)
                    && matches!(expr, Expression::RegexLiteral { .. });
                if named && let Expression::RegexLiteral { pattern, .. } = &expr {
                    for name in crate::parser::named_groups(pattern) {
                        self.declare_local(&name);
                    }
                }
                expr = Expression::MethodCall {
                    receiver: Box::new(expr),
                    method: if named {
                        "__match_named__".to_string()
                    } else if op_token.kind == TokenKind::Match {
                        "=~".to_string()
                    } else {
                        "!~".to_string()
                    },
                    arguments: vec![right],
                    trailing_block: None,
                    position: op_token.position,
                };
            } else {
                let op = match op_token.kind {
                    TokenKind::EqualEqual => BinaryOp::Equal,
                    TokenKind::TripleEqual => BinaryOp::CaseEqual,
                    TokenKind::BangEqual => BinaryOp::NotEqual,
                    TokenKind::Spaceship => BinaryOp::Spaceship,
                    _ => unreachable!(),
                };
                self.skip_whitespace();
                let right = self.parse_comparison()?;
                let right = self.fold_assignment(right)?;
                expr = Expression::BinaryOp {
                    op,
                    left: Box::new(expr),
                    right: Box::new(right),
                    position: op_token.position,
                };
            }
            if self.check(Self::RELATIONS) {
                return Err(self.error_at_current("unexpected operator"));
            }
        }

        Ok(expr)
    }

    /// Parse comparison operators (<, >, <=, >=)
    pub(crate) fn parse_comparison(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_bitwise_or()?;

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
            // A comparison operator at the end of a line carries the
            // expression onto the next one.
            self.skip_whitespace();
            let right = self.parse_bitwise_or()?;
            let right = self.fold_assignment(right)?;
            expr = Expression::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// Parse `|` and `^`, which bind looser than `&` and tighter than a
    /// comparison.
    pub(crate) fn parse_bitwise_or(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_bitwise_and()?;

        while self.check(&[TokenKind::Caret, TokenKind::Pipe]) {
            let op_token = self.advance();
            let op = match op_token.kind {
                TokenKind::Caret => BinaryOp::Xor,
                _ => BinaryOp::BitwiseOr,
            };
            self.skip_whitespace();
            let right = self.parse_bitwise_and()?;
            let right = self.fold_assignment(right)?;
            expr = Expression::BinaryOp {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// Parse `&`, which binds looser than a shift and tighter than `|`.
    pub(crate) fn parse_bitwise_and(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_shift()?;

        while self.check(&[TokenKind::Ampersand]) {
            let op_token = self.advance();
            self.skip_whitespace();
            let right = self.parse_shift()?;
            let right = self.fold_assignment(right)?;
            expr = Expression::BinaryOp {
                op: BinaryOp::BitwiseAnd,
                left: Box::new(expr),
                right: Box::new(right),
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// Parse `<<` and `>>`, which bind tighter than every other operator that
    /// works on the bits of a number.
    pub(crate) fn parse_shift(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_term()?;

        while self.check(&[TokenKind::Shovel, TokenKind::RightShift]) {
            let op_token = self.advance();
            // `<<` and `>>` are method calls, so `array << value` and
            // `number >> bits` both go through dispatch.
            let method = if op_token.kind == TokenKind::Shovel {
                "<<"
            } else {
                ">>"
            };
            self.skip_whitespace();
            let right = self.parse_term()?;
            expr = Expression::MethodCall {
                receiver: Box::new(expr),
                method: method.to_string(),
                arguments: vec![right],
                trailing_block: None,
                position: op_token.position,
            };
        }

        Ok(expr)
    }

    /// Parse range operators (.., ...), which bind looser than every
    /// operator but the conditional and what follows it.
    /// Whether the first token after the line breaks and comments ahead
    /// can start a value, rather than close what is open around it.
    fn value_starts_next_line(&self) -> bool {
        let mut offset = 0;
        while matches!(
            self.peek_ahead(offset).kind,
            TokenKind::Newline | TokenKind::Comment(_)
        ) {
            offset += 1;
        }
        !matches!(
            self.peek_ahead(offset).kind,
            TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::RBrace
                | TokenKind::Comma
                | TokenKind::Semicolon
                | TokenKind::End
                | TokenKind::Else
                | TokenKind::Elsif
                | TokenKind::When
                | TokenKind::In
                | TokenKind::Rescue
                | TokenKind::Ensure
                | TokenKind::Then
                | TokenKind::Do
                | TokenKind::EOF
        )
    }

    pub(crate) fn parse_range(&mut self) -> Result<Expression, MetorexError> {
        // Beginless range: `..expr` or `...expr`
        if self.check(&[TokenKind::DotDot, TokenKind::DotDotDot]) {
            let op_token = self.advance();
            let exclusive = op_token.kind == TokenKind::DotDotDot;
            self.range_operand_depth += 1;
            let end = self.parse_logical_or();
            self.range_operand_depth -= 1;
            let end = end?;
            return Ok(Expression::Range {
                start: Box::new(Expression::NilLiteral {
                    position: op_token.position,
                }),
                end: Box::new(end),
                exclusive,
                position: op_token.position,
            });
        }

        let mut expr = self.parse_logical_or()?;

        if self.check(&[TokenKind::DotDot, TokenKind::DotDotDot]) {
            let op_token = self.advance();
            let exclusive = op_token.kind == TokenKind::DotDotDot;
            // A range that ends its line goes on to the next one when that
            // line starts a value, so `x = 1..` above `2` is `1..2`.
            if self.check(&[TokenKind::Newline]) && self.value_starts_next_line() {
                self.skip_whitespace();
            }
            // Endless range: `x..` followed by `)`, `]`, `,`, `}`, newline, or EOF.
            let is_endless = self.check(&[
                TokenKind::RParen,
                TokenKind::RBracket,
                TokenKind::RBrace,
                TokenKind::Comma,
                TokenKind::Newline,
                TokenKind::Semicolon,
            ]) || self.is_at_end();
            let end = if is_endless {
                Expression::NilLiteral {
                    position: op_token.position,
                }
            } else {
                {
                    self.range_operand_depth += 1;
                    let held = self.parse_logical_or();
                    self.range_operand_depth -= 1;
                    held?
                }
            };
            expr = Expression::Range {
                start: Box::new(expr),
                end: Box::new(end),
                exclusive,
                position: op_token.position,
            };
            // One range cannot be an end of another, so `1..2..3` is not a
            // program.
            if self.check(&[TokenKind::DotDot, TokenKind::DotDotDot]) {
                return Err(self.error_at_current("unexpected range operator"));
            }
        }

        Ok(expr)
    }

    /// Parse addition and subtraction
    pub(crate) fn parse_term(&mut self) -> Result<Expression, MetorexError> {
        let mut expr = self.parse_factor()?;

        while self.check(&[TokenKind::Plus, TokenKind::Minus]) {
            let op_token = self.advance();
            let op = match op_token.kind {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            // An operator at the end of a line carries the expression onto
            // the next one.
            self.skip_whitespace();
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
    pub(crate) fn parse_factor(&mut self) -> Result<Expression, MetorexError> {
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
}
