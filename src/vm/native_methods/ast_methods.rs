//! AST node serialization for the AST Inspection API.
//!
//! Converts Metorex AST nodes (Statements and Expressions) into Object::Dict
//! representations that can be inspected at runtime.

use crate::ast::{BinaryOp, Expression, Statement, UnaryOp};
use crate::object::Object;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::rc::Rc;

fn make_dict(entries: Vec<(&str, Object)>) -> Object {
    let mut map = IndexMap::new();
    for (k, v) in entries {
        map.insert(k.to_string(), v);
    }
    Object::Dict(Rc::new(RefCell::new(map)))
}

fn make_array(items: Vec<Object>) -> Object {
    Object::Array(Rc::new(RefCell::new(items)))
}

fn str_obj(s: impl Into<String>) -> Object {
    Object::string(s.into())
}

fn binary_op_str(op: &BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::Modulo => "%",
        BinaryOp::Power => "**",
        BinaryOp::Equal => "==",
        BinaryOp::CaseEqual => "===",
        BinaryOp::NotEqual => "!=",
        BinaryOp::Less => "<",
        BinaryOp::Greater => ">",
        BinaryOp::LessEqual => "<=",
        BinaryOp::GreaterEqual => ">=",
        BinaryOp::Spaceship => "<=>",
        BinaryOp::BitwiseAnd => "&",
        BinaryOp::BitwiseOr => "|",
        BinaryOp::Xor => "^",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
        BinaryOp::Assign => "=",
        BinaryOp::AddAssign => "+=",
        BinaryOp::SubtractAssign => "-=",
        BinaryOp::MultiplyAssign => "*=",
        BinaryOp::DivideAssign => "/=",
    }
}

fn unary_op_str(op: &UnaryOp) -> &'static str {
    match op {
        UnaryOp::Plus => "+",
        UnaryOp::Minus => "-",
        UnaryOp::Not => "!",
    }
}

/// Serialize an Expression to an Object::Dict.
pub fn serialize_expression(expr: &Expression) -> Object {
    match expr {
        Expression::IntLiteral { value, .. } => make_dict(vec![
            ("type", str_obj("IntLiteral")),
            ("value", Object::Int(*value)),
        ]),
        Expression::FloatLiteral { value, .. } => make_dict(vec![
            ("type", str_obj("FloatLiteral")),
            ("value", Object::Float(*value)),
        ]),
        Expression::StringLiteral { value, .. } => make_dict(vec![
            ("type", str_obj("StringLiteral")),
            ("value", str_obj(value.clone())),
        ]),
        Expression::BoolLiteral { value, .. } => make_dict(vec![
            ("type", str_obj("BoolLiteral")),
            ("value", Object::Bool(*value)),
        ]),
        Expression::NilLiteral { .. } => make_dict(vec![("type", str_obj("NilLiteral"))]),
        Expression::Symbol { value, .. } => make_dict(vec![
            ("type", str_obj("Symbol")),
            ("value", str_obj(value.clone())),
        ]),
        Expression::Identifier { name, .. } => make_dict(vec![
            ("type", str_obj("Identifier")),
            ("name", str_obj(name.clone())),
        ]),
        Expression::InstanceVariable { name, .. } => make_dict(vec![
            ("type", str_obj("InstanceVariable")),
            ("name", str_obj(name.clone())),
        ]),
        Expression::ClassVariable { name, .. } => make_dict(vec![
            ("type", str_obj("ClassVariable")),
            ("name", str_obj(name.clone())),
        ]),
        Expression::GlobalVariable { name, .. } => make_dict(vec![
            ("type", str_obj("GlobalVariable")),
            ("name", str_obj(name.clone())),
        ]),
        Expression::BinaryOp {
            op, left, right, ..
        } => make_dict(vec![
            ("type", str_obj("BinaryOp")),
            ("op", str_obj(binary_op_str(op))),
            ("left", serialize_expression(left)),
            ("right", serialize_expression(right)),
        ]),
        Expression::UnaryOp { op, operand, .. } => make_dict(vec![
            ("type", str_obj("UnaryOp")),
            ("op", str_obj(unary_op_str(op))),
            ("operand", serialize_expression(operand)),
        ]),
        Expression::Call {
            callee, arguments, ..
        } => make_dict(vec![
            ("type", str_obj("Call")),
            ("callee", serialize_expression(callee)),
            (
                "arguments",
                make_array(arguments.iter().map(serialize_expression).collect()),
            ),
        ]),
        Expression::MethodCall {
            receiver,
            method,
            arguments,
            ..
        } => make_dict(vec![
            ("type", str_obj("MethodCall")),
            ("receiver", serialize_expression(receiver)),
            ("method", str_obj(method.clone())),
            (
                "arguments",
                make_array(arguments.iter().map(serialize_expression).collect()),
            ),
        ]),
        Expression::Array { elements, .. } => make_dict(vec![
            ("type", str_obj("Array")),
            (
                "elements",
                make_array(elements.iter().map(serialize_expression).collect()),
            ),
        ]),
        Expression::Index { array, index, .. } => make_dict(vec![
            ("type", str_obj("Index")),
            ("array", serialize_expression(array)),
            ("index", serialize_expression(index)),
        ]),
        Expression::Dictionary { entries, .. } => {
            let pairs: Vec<Object> = entries
                .iter()
                .map(|(k, v)| {
                    make_dict(vec![
                        ("key", serialize_expression(k)),
                        ("value", serialize_expression(v)),
                    ])
                })
                .collect();
            make_dict(vec![
                ("type", str_obj("Dictionary")),
                ("entries", make_array(pairs)),
            ])
        }
        Expression::Lambda {
            parameters, body, ..
        } => make_dict(vec![
            ("type", str_obj("Lambda")),
            (
                "parameters",
                make_array(parameters.iter().map(|p| str_obj(p.clone())).collect()),
            ),
            (
                "body",
                make_array(body.iter().map(serialize_statement).collect()),
            ),
        ]),
        Expression::Grouped { expression, .. } => make_dict(vec![
            ("type", str_obj("Grouped")),
            ("expression", serialize_expression(expression)),
        ]),
        Expression::SelfExpr { .. } => make_dict(vec![("type", str_obj("Self"))]),
        Expression::ScopeResolution {
            namespace, name, ..
        } => make_dict(vec![
            ("type", str_obj("ScopeResolution")),
            ("namespace", serialize_expression(namespace)),
            ("name", str_obj(name.clone())),
        ]),
        Expression::InterpolatedString { .. } => {
            make_dict(vec![("type", str_obj("InterpolatedString"))])
        }
        _ => make_dict(vec![("type", str_obj("Unknown"))]),
    }
}

/// Serialize a Statement to an Object::Dict.
pub fn serialize_statement(stmt: &Statement) -> Object {
    match stmt {
        Statement::Expression { expression, .. } => serialize_expression(expression),
        Statement::Return { value, .. } => make_dict(vec![
            ("type", str_obj("Return")),
            (
                "value",
                value
                    .as_ref()
                    .map(serialize_expression)
                    .unwrap_or(Object::Nil),
            ),
        ]),
        Statement::Assignment { target, value, .. } => make_dict(vec![
            ("type", str_obj("Assignment")),
            ("target", serialize_expression(target)),
            ("value", serialize_expression(value)),
        ]),
        Statement::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => make_dict(vec![
            ("type", str_obj("If")),
            ("condition", serialize_expression(condition)),
            (
                "then_branch",
                make_array(then_branch.iter().map(serialize_statement).collect()),
            ),
            (
                "else_branch",
                else_branch
                    .as_ref()
                    .map(|b| make_array(b.iter().map(serialize_statement).collect()))
                    .unwrap_or(Object::Nil),
            ),
        ]),
        Statement::While {
            condition, body, ..
        } => make_dict(vec![
            ("type", str_obj("While")),
            ("condition", serialize_expression(condition)),
            (
                "body",
                make_array(body.iter().map(serialize_statement).collect()),
            ),
        ]),
        Statement::For {
            variable,
            iterable,
            body,
            ..
        } => make_dict(vec![
            ("type", str_obj("For")),
            ("variable", str_obj(variable.clone())),
            ("iterable", serialize_expression(iterable)),
            (
                "body",
                make_array(body.iter().map(serialize_statement).collect()),
            ),
        ]),
        Statement::MethodDef {
            name, parameters, ..
        } => make_dict(vec![
            ("type", str_obj("MethodDef")),
            ("name", str_obj(name.clone())),
            (
                "parameters",
                make_array(parameters.iter().map(|p| str_obj(p.name.clone())).collect()),
            ),
        ]),
        Statement::Raise { exception, .. } => make_dict(vec![
            ("type", str_obj("Raise")),
            (
                "exception",
                exception
                    .as_ref()
                    .map(serialize_expression)
                    .unwrap_or(Object::Nil),
            ),
        ]),
        Statement::Break { .. } => make_dict(vec![("type", str_obj("Break"))]),
        Statement::Continue { .. } => make_dict(vec![("type", str_obj("Continue"))]),
        _ => make_dict(vec![("type", str_obj("Unknown"))]),
    }
}

/// Serialize a slice of Statements to an Object::Array of Dicts.
pub fn serialize_statements(stmts: &[Statement]) -> Object {
    make_array(stmts.iter().map(serialize_statement).collect())
}
