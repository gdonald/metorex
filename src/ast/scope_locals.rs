//! Collect the set of local-variable names assigned inside a scope's body.
//!
//! Ruby pre-defines local variables as `nil` at parse time the moment the
//! parser sees an assignment to that name in the current scope, so a read
//! that runs before the assignment line returns `nil` rather than raising
//! `NameError`. Our tree-walking interpreter approximates that by walking
//! the AST once at scope entry and pre-binding every assignment target it
//! finds to `nil`.
//!
//! Walks recursively through control flow (if/while/begin/case/etc.) but
//! stops at scope boundaries — `def`, `class`, `module`, lambda/proc — so
//! variables defined in an inner method/lambda don't leak into the enclosing
//! scope.

use super::node::{Expression, RescueClause, Statement};

/// Collect every simple-identifier assignment target from `body`,
/// preserving first-seen order and skipping duplicates and constant names
/// (uppercase first letter — those go on globals/lexical scope).
pub fn collect_assigned_locals(body: &[Statement]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for stmt in body {
        walk_statement(stmt, &mut names);
    }
    names
}

fn add(names: &mut Vec<String>, name: &str) {
    if name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_uppercase() || c == '@' || c == '$')
    {
        return;
    }
    if !names.iter().any(|n| n == name) {
        names.push(name.to_string());
    }
}

fn walk_statement(stmt: &Statement, names: &mut Vec<String>) {
    match stmt {
        Statement::Expression { expression, .. } => walk_expression(expression, names),
        Statement::Assignment { target, value, .. } => {
            collect_target(target, names);
            walk_expression(value, names);
        }
        Statement::MultipleAssignment {
            targets, values, ..
        } => {
            for target in targets {
                collect_target(target, names);
            }
            for value in values {
                walk_expression(value, names);
            }
        }
        Statement::If {
            condition,
            then_branch,
            elsif_branches,
            else_branch,
            ..
        } => {
            walk_expression(condition, names);
            walk_body(then_branch, names);
            for branch in elsif_branches {
                walk_expression(&branch.condition, names);
                walk_body(&branch.body, names);
            }
            if let Some(eb) = else_branch {
                walk_body(eb, names);
            }
        }
        Statement::Unless {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            walk_expression(condition, names);
            walk_body(then_branch, names);
            if let Some(eb) = else_branch {
                walk_body(eb, names);
            }
        }
        Statement::While {
            condition, body, ..
        } => {
            walk_expression(condition, names);
            walk_body(body, names);
        }
        Statement::For {
            variable,
            iterable,
            body,
            ..
        } => {
            add(names, variable);
            walk_expression(iterable, names);
            walk_body(body, names);
        }
        Statement::Match {
            expression, cases, ..
        }
        | Statement::CaseIn {
            expression, cases, ..
        } => {
            walk_expression(expression, names);
            for case in cases {
                if let Some(guard) = &case.guard {
                    walk_expression(guard, names);
                }
                walk_body(&case.body, names);
            }
        }
        Statement::Block { statements, .. } => walk_body(statements, names),
        Statement::Begin {
            body,
            rescue_clauses,
            else_clause,
            ensure_block,
            ..
        } => walk_begin_parts(
            body,
            rescue_clauses,
            else_clause.as_deref(),
            ensure_block.as_deref(),
            names,
        ),
        Statement::Return { value, .. } => {
            if let Some(v) = value {
                walk_expression(v, names);
            }
        }
        Statement::Break { value, .. } => {
            if let Some(v) = value {
                walk_expression(v, names);
            }
        }
        Statement::Raise { exception, .. } => {
            if let Some(e) = exception {
                walk_expression(e, names);
            }
        }
        Statement::AttrReader { attributes, .. }
        | Statement::AttrWriter { attributes, .. }
        | Statement::AttrAccessor { attributes, .. } => {
            for attr in attributes {
                walk_expression(attr, names);
            }
        }
        // Scope boundaries — anything assigned inside their body belongs to the
        // inner scope, not ours.
        Statement::FunctionDef { .. }
        | Statement::MethodDef { .. }
        | Statement::ClassDef { .. }
        | Statement::ModuleDef { .. } => {}
        Statement::Redo { .. }
        | Statement::Continue { .. }
        | Statement::Include { .. }
        | Statement::Extend { .. }
        | Statement::Alias { .. } => {}
    }
}

fn walk_body(body: &[Statement], names: &mut Vec<String>) {
    for stmt in body {
        walk_statement(stmt, names);
    }
}

fn walk_begin_parts(
    body: &[Statement],
    rescue_clauses: &[RescueClause],
    else_clause: Option<&[Statement]>,
    ensure_block: Option<&[Statement]>,
    names: &mut Vec<String>,
) {
    walk_body(body, names);
    for rescue in rescue_clauses {
        if let Some(var) = &rescue.variable_name {
            add(names, var);
        }
        walk_body(&rescue.body, names);
    }
    if let Some(else_body) = else_clause {
        walk_body(else_body, names);
    }
    if let Some(ensure) = ensure_block {
        walk_body(ensure, names);
    }
}

fn collect_target(target: &Expression, names: &mut Vec<String>) {
    match target {
        Expression::Identifier { name, .. } => add(names, name),
        Expression::Splat { expression, .. } => collect_target(expression, names),
        Expression::Grouped { expression, .. } => collect_target(expression, names),
        // InstanceVariable / ClassVariable / GlobalVariable / Index assignments
        // don't introduce a new local — leave them alone.
        _ => {}
    }
}

fn walk_expression(expr: &Expression, names: &mut Vec<String>) {
    match expr {
        Expression::BinaryOp {
            op, left, right, ..
        } => {
            // Compound-assignment targets (a += 1, a ||= 1, etc.) introduce a
            // local the same way `a = ...` does.
            use super::node::BinaryOp::*;
            if matches!(
                op,
                Assign | AddAssign | SubtractAssign | MultiplyAssign | DivideAssign
            ) {
                collect_target(left, names);
            } else {
                walk_expression(left, names);
            }
            walk_expression(right, names);
        }
        Expression::UnaryOp { operand, .. } => walk_expression(operand, names),
        Expression::Call {
            callee,
            arguments,
            trailing_block,
            ..
        } => {
            walk_expression(callee, names);
            for arg in arguments {
                walk_expression(arg, names);
            }
            // Don't descend into the block body — it's a separate scope.
            // But do walk the block expression itself in case the *call site*
            // uses `&expr` (variable references in expr count as reads, not
            // writes, so this matters only for compound-write expressions,
            // which Ruby disallows here anyway).
            if let Some(b) = trailing_block {
                walk_block_arg_only(b, names);
            }
        }
        Expression::MethodCall {
            receiver,
            arguments,
            trailing_block,
            ..
        } => {
            walk_expression(receiver, names);
            for arg in arguments {
                walk_expression(arg, names);
            }
            if let Some(b) = trailing_block {
                walk_block_arg_only(b, names);
            }
        }
        Expression::Array { elements, .. } => {
            for e in elements {
                walk_expression(e, names);
            }
        }
        Expression::Index { array, index, .. } => {
            walk_expression(array, names);
            walk_expression(index, names);
        }
        Expression::Dictionary { entries, .. } => {
            for (k, v) in entries {
                walk_expression(k, names);
                walk_expression(v, names);
            }
        }
        Expression::Grouped { expression, .. }
        | Expression::Splat { expression, .. }
        | Expression::KeywordSplat { expression, .. }
        | Expression::BlockArg { expression, .. }
        | Expression::Defined { expression, .. } => walk_expression(expression, names),
        Expression::SingletonClass { target, .. } => {
            // The body opens a new scope on the singleton class, so we don't
            // walk it. The target expression is evaluated in our scope.
            walk_expression(target, names);
        }
        Expression::Super { arguments, .. } => {
            for arg in arguments {
                walk_expression(arg, names);
            }
        }
        Expression::Yield { arguments, .. } => {
            for arg in arguments {
                walk_expression(arg, names);
            }
        }
        Expression::Range { start, end, .. } => {
            walk_expression(start, names);
            walk_expression(end, names);
        }
        Expression::BeginRescue {
            body,
            rescue_clauses,
            else_clause,
            ensure_block,
            ..
        } => walk_begin_parts(
            body,
            rescue_clauses,
            else_clause.as_deref(),
            ensure_block.as_deref(),
            names,
        ),
        Expression::If {
            condition,
            then_branch,
            elsif_branches,
            else_branch,
            ..
        } => {
            walk_expression(condition, names);
            walk_body(then_branch, names);
            for branch in elsif_branches {
                walk_expression(&branch.condition, names);
                walk_body(&branch.body, names);
            }
            if let Some(eb) = else_branch {
                walk_body(eb, names);
            }
        }
        Expression::Unless {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            walk_expression(condition, names);
            walk_body(then_branch, names);
            if let Some(eb) = else_branch {
                walk_body(eb, names);
            }
        }
        Expression::Case {
            expression,
            cases,
            else_case,
            ..
        } => {
            walk_expression(expression, names);
            for case in cases {
                if let Some(guard) = &case.guard {
                    walk_expression(guard, names);
                }
                walk_expression(&case.body, names);
            }
            if let Some(ec) = else_case {
                walk_expression(ec, names);
            }
        }
        Expression::ScopeResolution { namespace, .. } => walk_expression(namespace, names),
        // Scope boundaries / leaf nodes — nothing to descend into.
        Expression::Lambda { .. } => {}
        Expression::IntLiteral { .. }
        | Expression::FloatLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::InterpolatedString { .. }
        | Expression::Symbol { .. }
        | Expression::BoolLiteral { .. }
        | Expression::NilLiteral { .. }
        | Expression::RegexLiteral { .. }
        | Expression::Identifier { .. }
        | Expression::InstanceVariable { .. }
        | Expression::ClassVariable { .. }
        | Expression::GlobalVariable { .. }
        | Expression::MagicFile { .. }
        | Expression::MagicLine { .. }
        | Expression::MagicDir { .. }
        | Expression::SelfExpr { .. } => {}
    }
}

fn walk_block_arg_only(expr: &Expression, names: &mut Vec<String>) {
    // For trailing-block expressions, the block itself is a Lambda (separate
    // scope). But the caller may have written `&some_var` or `&some_call(...)`
    // which is itself an expression evaluated *in our scope*. Walk those.
    match expr {
        Expression::Lambda { .. } => {}
        _ => walk_expression(expr, names),
    }
}
