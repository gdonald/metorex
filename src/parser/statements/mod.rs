// Statement parsing module
// Handles parsing of all statement types

mod attributes;
mod class;
mod control_flow;
mod exception;
pub(crate) mod function;
mod patterns;

pub(crate) use crate::ast::{BinaryOp, Expression, Statement};
pub(crate) use crate::error::MetorexError;
pub(crate) use crate::lexer::TokenKind;
pub(crate) use crate::parser::Parser;

/// Whether an expression names something an assignment can write to. Ruby
/// rejects anything else while parsing, so `1 + 1 = 2` never reaches the VM.
pub(crate) fn is_assignable(expr: &Expression) -> bool {
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

mod assignment;
mod modifiers;
mod multiple_assignment;
mod reading;

pub(crate) use assignment::*;
