//! The `each` call a `for` loop stands for.
//!
//! Ruby's `for` runs in the scope holding it rather than opening one of its
//! own, so the names it and its body bind are still readable once it has run.

use super::node::{Expression, Statement};
use super::scope_locals;
use crate::lexer::Position;

/// A `for` loop over targets the loop variable alone cannot hold, written
/// as the `each` call it stands for. The value each round hands over is
/// assigned to the targets before the body runs, so the names it binds
/// outlive the loop the way Ruby's do.
pub fn for_over_each(
    targets: Vec<Expression>,
    destructures: bool,
    iterable: Expression,
    body: Vec<Statement>,
    position: Position,
) -> Statement {
    const HANDED_OVER: &str = "__for_value";
    let mut plain_names = Vec::new();
    for target in &targets {
        let named = match target {
            Expression::Splat { expression, .. } => expression.as_ref(),
            other => other,
        };
        if let Expression::Identifier { name, .. } = named {
            plain_names.push(name.clone());
        }
    }
    let value = Expression::Identifier {
        name: HANDED_OVER.to_string(),
        position,
    };
    let assignment = if destructures {
        Statement::MultipleAssignment {
            targets,
            values: vec![value],
            position,
        }
    } else {
        Statement::Assignment {
            target: targets.into_iter().next().expect("one target"),
            value,
            position,
        }
    };
    let mut statements = vec![assignment];
    statements.extend(body);
    for name in scope_locals::collect_assigned_locals(&statements) {
        if name != HANDED_OVER && !plain_names.contains(&name) {
            plain_names.push(name);
        }
    }
    let block = Statement::Expression {
        expression: Expression::MethodCall {
            receiver: Box::new(iterable),
            method: "each".to_string(),
            arguments: Vec::new(),
            trailing_block: Some(Box::new(Expression::Lambda {
                parameters: vec![HANDED_OVER.to_string()],
                parameter_defaults: Vec::new(),
                body: statements,
                captured_vars: None,
                is_lambda: false,
                position,
            })),
            position,
        },
        position,
    };
    // A `for` loop runs in the scope holding it rather than opening one
    // of its own, so the names it and its body bind are still readable
    // once it has run.
    let declared = Statement::DeclareLocals {
        names: plain_names,
        position,
    };
    Statement::Expression {
        expression: Expression::BeginRescue {
            body: vec![declared, block],
            rescue_clauses: Vec::new(),
            else_clause: None,
            ensure_block: None,
            position,
        },
        position,
    }
}
