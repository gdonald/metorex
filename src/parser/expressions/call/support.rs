// What an argument list is made of once it is read.

use super::*;

/// The method a `&.` call is written as: it answers nil for a nil receiver
/// and otherwise sends the name it was given.
pub(crate) const SAFE_CALL: &str = "__mx_safe_call__";

/// Whether an expression is a string literal written in the source, whether or
/// not the source asked outright for literals that change.
pub(crate) fn names_a_string_literal(expr: &Expression) -> bool {
    match expr {
        Expression::StringLiteral { .. } => true,
        Expression::MethodCall {
            receiver, method, ..
        } => {
            method == "__mutable_literal__"
                && matches!(**receiver, Expression::StringLiteral { .. })
        }
        _ => false,
    }
}

/// Put the `**held` arguments back among the keyword arguments they were
/// written alongside, so one call carries a single set of keywords rather
/// than a hash argument and a set of keywords.
pub(crate) fn fold_keyword_splats(
    arguments: &mut Vec<Expression>,
    entries: &mut Vec<(Expression, Expression)>,
    slots: &[(usize, usize)],
    position: crate::lexer::Position,
) {
    let mut folded: Vec<(usize, Expression)> = Vec::new();
    for (at, among) in slots.iter().rev() {
        if *at < arguments.len() && matches!(arguments[*at], Expression::KeywordSplat { .. }) {
            folded.push((*among, arguments.remove(*at)));
        }
    }
    folded.reverse();
    for (moved, (among, held)) in folded.into_iter().enumerate() {
        let at = (among + moved).min(entries.len());
        entries.insert(at, (held, Expression::NilLiteral { position }));
    }
}

/// Ruby allows one block per call, so a call written with both `&arg` and a
/// literal block is refused where it is written.
pub(crate) fn refuse_two_blocks(
    arguments: &[Expression],
    has_a_literal_block: bool,
) -> Result<(), MetorexError> {
    if !has_a_literal_block
        || !arguments
            .iter()
            .any(|argument| matches!(argument, Expression::BlockArg { .. }))
    {
        return Ok(());
    }
    Err(MetorexError::runtime_error(
        "both block arg and actual block given; only one block is allowed",
        crate::error::SourceLocation::new(0, 0, 0),
    ))
}
