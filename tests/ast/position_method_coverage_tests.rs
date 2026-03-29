// Coverage tests for ast/node.rs Expression::position() and Statement::position()
// These ensure all match arms in the position() methods are exercised.

use metorex::ast::node::{ExprMatchCase, Expression, InterpolationPart, MatchPattern, Statement};
use metorex::lexer::Position;

fn pos() -> Position {
    Position::default()
}

fn int_lit(v: i64) -> Expression {
    Expression::IntLiteral {
        value: v,
        position: pos(),
    }
}

fn ident(name: &str) -> Expression {
    Expression::Identifier {
        name: name.to_string(),
        position: pos(),
    }
}

// ── Expression::position() for uncovered variants ────────────────────────────

#[test]
fn symbol_expression_position() {
    let expr = Expression::Symbol {
        value: "foo".to_string(),
        position: pos(),
    };
    let _ = expr.position();
}

#[test]
fn interpolated_string_expression_position() {
    let expr = Expression::InterpolatedString {
        parts: vec![InterpolationPart::Text("hello".to_string())],
        position: pos(),
    };
    let _ = expr.position();
}

#[test]
fn global_variable_expression_position() {
    let expr = Expression::GlobalVariable {
        name: "ARGV".to_string(),
        position: pos(),
    };
    let _ = expr.position();
}

#[test]
fn self_expr_position() {
    let expr = Expression::SelfExpr { position: pos() };
    let _ = expr.position();
}

#[test]
fn super_expression_position() {
    let expr = Expression::Super {
        arguments: vec![],
        position: pos(),
    };
    let _ = expr.position();
}

#[test]
fn range_expression_position() {
    let expr = Expression::Range {
        start: Box::new(int_lit(1)),
        end: Box::new(int_lit(10)),
        exclusive: false,
        position: pos(),
    };
    let _ = expr.position();
}

#[test]
fn case_expression_position() {
    let expr = Expression::Case {
        expression: Box::new(int_lit(1)),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::IntLiteral(1),
            guard: None,
            body: int_lit(1),
            position: pos(),
        }],
        else_case: None,
        position: pos(),
    };
    let _ = expr.position();
}

#[test]
fn scope_resolution_expression_position() {
    let expr = Expression::ScopeResolution {
        namespace: Box::new(ident("Foo")),
        name: "BAR".to_string(),
        position: pos(),
    };
    let _ = expr.position();
}

#[test]
fn if_expression_position() {
    let expr = Expression::If {
        condition: Box::new(metorex::ast::Expression::BoolLiteral {
            value: true,
            position: pos(),
        }),
        then_branch: vec![],
        elsif_branches: vec![],
        else_branch: None,
        position: pos(),
    };
    let _ = expr.position();
}

#[test]
fn unless_expression_position() {
    let expr = Expression::Unless {
        condition: Box::new(metorex::ast::Expression::BoolLiteral {
            value: false,
            position: pos(),
        }),
        then_branch: vec![],
        else_branch: None,
        position: pos(),
    };
    let _ = expr.position();
}

// ── Statement::position() for uncovered variants ──────────────────────────────

#[test]
fn unless_statement_position() {
    let stmt = Statement::Unless {
        condition: metorex::ast::Expression::BoolLiteral {
            value: false,
            position: pos(),
        },
        then_branch: vec![],
        else_branch: None,
        position: pos(),
    };
    let _ = stmt.position();
}

#[test]
fn case_in_statement_position() {
    let stmt = Statement::CaseIn {
        expression: int_lit(1),
        cases: vec![],
        position: pos(),
    };
    let _ = stmt.position();
}

#[test]
fn attr_reader_statement_position() {
    let stmt = Statement::AttrReader {
        attributes: vec!["name".to_string()],
        position: pos(),
    };
    let _ = stmt.position();
}

#[test]
fn attr_writer_statement_position() {
    let stmt = Statement::AttrWriter {
        attributes: vec!["name".to_string()],
        position: pos(),
    };
    let _ = stmt.position();
}

#[test]
fn attr_accessor_statement_position() {
    let stmt = Statement::AttrAccessor {
        attributes: vec!["name".to_string()],
        position: pos(),
    };
    let _ = stmt.position();
}

#[test]
fn module_def_statement_position() {
    let stmt = Statement::ModuleDef {
        name: "MyMod".to_string(),
        body: vec![],
        position: pos(),
    };
    let _ = stmt.position();
}

#[test]
fn include_statement_position() {
    let stmt = Statement::Include {
        module_name: "Comparable".to_string(),
        position: pos(),
    };
    let _ = stmt.position();
}

// ── Expression::is_literal() for completeness ──────────────────────────────────

#[test]
fn is_literal_for_symbol_returns_false() {
    let expr = Expression::Symbol {
        value: "foo".to_string(),
        position: pos(),
    };
    assert!(!expr.is_literal());
}

// ── Expression::is_identifier() for class variable ────────────────────────────

#[test]
fn is_identifier_for_class_variable() {
    let expr = Expression::ClassVariable {
        name: "count".to_string(),
        position: pos(),
    };
    assert!(expr.is_identifier());
}
