// Coverage tests for src/resolver.rs — statement and expression resolution paths.

use metorex::ast::node::{
    ElsifBranch, Expression, InterpolationPart, MatchCase, MatchPattern, Parameter, RescueClause,
    Statement,
};
use metorex::lexer::Position;
use metorex::resolver::Resolver;

fn pos() -> Position {
    Position::default()
}

fn int_lit(v: i64) -> Expression {
    Expression::IntLiteral {
        value: v,
        position: pos(),
    }
}

fn bool_lit(v: bool) -> Expression {
    Expression::BoolLiteral {
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

fn assign(name: &str, value: Expression) -> Statement {
    Statement::Assignment {
        target: ident(name),
        value,
        position: pos(),
    }
}

// ── Statement::Unless ─────────────────────────────────────────────────────────

#[test]
fn resolver_unless_statement() {
    let mut r = Resolver::new();
    let stmt = Statement::Unless {
        condition: bool_lit(false),
        then_branch: vec![assign("x", int_lit(1))],
        else_branch: None,
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_unless_with_else_branch() {
    let mut r = Resolver::new();
    let stmt = Statement::Unless {
        condition: bool_lit(true),
        then_branch: vec![assign("x", int_lit(1))],
        else_branch: Some(vec![assign("y", int_lit(2))]),
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

// ── Statement::Break / Continue ───────────────────────────────────────────────

#[test]
fn resolver_break_and_continue() {
    let mut r = Resolver::new();
    let stmts = vec![
        Statement::Break { position: pos() },
        Statement::Continue { position: pos() },
    ];
    let result = r.resolve(&stmts);
    // Break and continue don't declare variables — no errors from resolver
    assert_eq!(
        result
            .errors
            .iter()
            .filter(|e| !e.to_string().contains("Undefined"))
            .count(),
        0
    );
}

// ── Statement::Raise ──────────────────────────────────────────────────────────

#[test]
fn resolver_raise_with_expression() {
    let mut r = Resolver::new();
    let stmt = Statement::Raise {
        exception: Some(Expression::StringLiteral {
            value: "oops".to_string(),
            position: pos(),
        }),
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_raise_without_expression() {
    let mut r = Resolver::new();
    let stmt = Statement::Raise {
        exception: None,
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

// ── Statement::Block ──────────────────────────────────────────────────────────

#[test]
fn resolver_block_statement() {
    let mut r = Resolver::new();
    let stmt = Statement::Block {
        statements: vec![assign("a", int_lit(10))],
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

// ── Statement::ModuleDef ──────────────────────────────────────────────────────

#[test]
fn resolver_module_def() {
    let mut r = Resolver::new();
    let stmt = Statement::ModuleDef {
        name: "MyMod".to_string(),
        body: vec![],
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

// ── Statement::Include / Extend ───────────────────────────────────────────────

#[test]
fn resolver_include_extend() {
    let mut r = Resolver::new();
    let stmts = vec![
        Statement::Include {
            module_name: "Enumerable".to_string(),
            position: pos(),
        },
        Statement::Extend {
            module_name: "ClassMethods".to_string(),
            position: pos(),
        },
    ];
    let result = r.resolve(&stmts);
    assert!(!result.has_errors());
}

// ── Statement::Begin with rescue/else/ensure ─────────────────────────────────

#[test]
fn resolver_begin_with_rescue_else_ensure() {
    let mut r = Resolver::new();
    let rescue = RescueClause {
        exception_types: vec![],
        variable_name: Some("ex".to_string()),
        body: vec![assign("err", ident("ex"))],
        position: pos(),
    };
    let stmt = Statement::Begin {
        body: vec![assign("x", int_lit(1))],
        rescue_clauses: vec![rescue],
        else_clause: Some(vec![assign("y", int_lit(2))]),
        ensure_block: Some(vec![assign("z", int_lit(3))]),
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    // may have warnings about unused vars, but no errors
    let errs: Vec<_> = result
        .errors
        .iter()
        .filter(|e| !e.to_string().contains("Undefined"))
        .collect();
    assert!(errs.is_empty());
}

// ── Statement::Match / CaseIn ─────────────────────────────────────────────────

#[test]
fn resolver_match_statement() {
    let mut r = Resolver::new();
    let stmt = Statement::Match {
        expression: int_lit(42),
        cases: vec![MatchCase {
            pattern: MatchPattern::IntLiteral(42),
            guard: None,
            body: vec![assign("res", int_lit(1))],
            position: pos(),
        }],
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_match_with_guard() {
    let mut r = Resolver::new();
    let stmt = Statement::Match {
        expression: int_lit(5),
        cases: vec![MatchCase {
            pattern: MatchPattern::Identifier("n".to_string()),
            guard: Some(Expression::BinaryOp {
                op: metorex::ast::BinaryOp::Greater,
                left: Box::new(ident("n")),
                right: Box::new(int_lit(0)),
                position: pos(),
            }),
            body: vec![assign("res", ident("n"))],
            position: pos(),
        }],
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

// ── Statement::If with elsif ──────────────────────────────────────────────────

#[test]
fn resolver_if_with_elsif_and_else() {
    let mut r = Resolver::new();
    let stmt = Statement::If {
        condition: bool_lit(true),
        then_branch: vec![assign("x", int_lit(1))],
        elsif_branches: vec![ElsifBranch {
            condition: bool_lit(false),
            body: vec![assign("y", int_lit(2))],
            position: pos(),
        }],
        else_branch: Some(vec![assign("z", int_lit(3))]),
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

// ── resolve_pattern variants ──────────────────────────────────────────────────

#[test]
fn resolver_pattern_array() {
    let mut r = Resolver::new();
    let stmt = Statement::Match {
        expression: int_lit(1),
        cases: vec![MatchCase {
            pattern: MatchPattern::Array(vec![
                MatchPattern::Identifier("a".to_string()),
                MatchPattern::Identifier("b".to_string()),
            ]),
            guard: None,
            body: vec![],
            position: pos(),
        }],
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_pattern_rest() {
    let mut r = Resolver::new();
    let stmt = Statement::Match {
        expression: int_lit(1),
        cases: vec![MatchCase {
            pattern: MatchPattern::Rest("rest".to_string()),
            guard: None,
            body: vec![],
            position: pos(),
        }],
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_pattern_object() {
    let mut r = Resolver::new();
    let stmt = Statement::Match {
        expression: int_lit(1),
        cases: vec![MatchCase {
            pattern: MatchPattern::Object(vec![(
                "name".to_string(),
                MatchPattern::Identifier("n".to_string()),
            )]),
            guard: None,
            body: vec![],
            position: pos(),
        }],
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_pattern_literal() {
    let mut r = Resolver::new();
    let stmt = Statement::Match {
        expression: int_lit(42),
        cases: vec![
            MatchCase {
                pattern: MatchPattern::IntLiteral(42),
                guard: None,
                body: vec![],
                position: pos(),
            },
            MatchCase {
                pattern: MatchPattern::StringLiteral("hello".to_string()),
                guard: None,
                body: vec![],
                position: pos(),
            },
            MatchCase {
                pattern: MatchPattern::BoolLiteral(true),
                guard: None,
                body: vec![],
                position: pos(),
            },
            MatchCase {
                pattern: MatchPattern::NilLiteral,
                guard: None,
                body: vec![],
                position: pos(),
            },
            MatchCase {
                pattern: MatchPattern::Wildcard,
                guard: None,
                body: vec![],
                position: pos(),
            },
        ],
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

// ── Expression resolution paths ───────────────────────────────────────────────

#[test]
fn resolver_unary_op_expression() {
    let mut r = Resolver::new();
    let stmt = assign(
        "x",
        Expression::UnaryOp {
            op: metorex::ast::UnaryOp::Minus,
            operand: Box::new(int_lit(5)),
            position: pos(),
        },
    );
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_call_expression() {
    let mut r = Resolver::new();
    // Define function first so call isn't undefined
    let func = Statement::FunctionDef {
        name: "add".to_string(),
        parameters: vec![],
        body: vec![assign("r", int_lit(0))],
        position: pos(),
    };
    let call = Statement::Expression {
        expression: Expression::Call {
            callee: Box::new(ident("add")),
            arguments: vec![int_lit(1), int_lit(2)],
            trailing_block: None,
            position: pos(),
        },
        position: pos(),
    };
    let result = r.resolve(&[func, call]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_method_call_expression() {
    let mut r = Resolver::new();
    let stmts = vec![
        assign("obj", int_lit(42)),
        Statement::Expression {
            expression: Expression::MethodCall {
                receiver: Box::new(ident("obj")),
                method: "to_s".to_string(),
                arguments: vec![],
                trailing_block: None,
                position: pos(),
            },
            position: pos(),
        },
    ];
    let result = r.resolve(&stmts);
    assert!(!result.has_errors());
}

#[test]
fn resolver_array_expression() {
    let mut r = Resolver::new();
    let stmt = assign(
        "arr",
        Expression::Array {
            elements: vec![int_lit(1), int_lit(2), int_lit(3)],
            position: pos(),
        },
    );
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_index_expression() {
    let mut r = Resolver::new();
    let stmts = vec![
        assign(
            "arr",
            Expression::Array {
                elements: vec![int_lit(1)],
                position: pos(),
            },
        ),
        assign(
            "v",
            Expression::Index {
                array: Box::new(ident("arr")),
                index: Box::new(int_lit(0)),
                position: pos(),
            },
        ),
    ];
    let result = r.resolve(&stmts);
    assert!(!result.has_errors());
}

#[test]
fn resolver_dictionary_expression() {
    let mut r = Resolver::new();
    let stmt = assign(
        "d",
        Expression::Dictionary {
            entries: vec![(
                Expression::StringLiteral {
                    value: "k".to_string(),
                    position: pos(),
                },
                int_lit(1),
            )],
            position: pos(),
        },
    );
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_lambda_expression() {
    let mut r = Resolver::new();
    let stmt = assign(
        "f",
        Expression::Lambda {
            parameters: vec!["x".to_string(), "y".to_string()],
            body: vec![assign(
                "r",
                Expression::BinaryOp {
                    op: metorex::ast::BinaryOp::Add,
                    left: Box::new(ident("x")),
                    right: Box::new(ident("y")),
                    position: pos(),
                },
            )],
            captured_vars: None,
            position: pos(),
        },
    );
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_grouped_expression() {
    let mut r = Resolver::new();
    let stmt = assign(
        "v",
        Expression::Grouped {
            expression: Box::new(int_lit(42)),
            position: pos(),
        },
    );
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_range_expression() {
    let mut r = Resolver::new();
    let stmt = assign(
        "r",
        Expression::Range {
            start: Box::new(int_lit(1)),
            end: Box::new(int_lit(10)),
            exclusive: false,
            position: pos(),
        },
    );
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_interpolated_string_expression() {
    let mut r = Resolver::new();
    let stmts = vec![
        assign(
            "name",
            Expression::StringLiteral {
                value: "World".to_string(),
                position: pos(),
            },
        ),
        assign(
            "msg",
            Expression::InterpolatedString {
                parts: vec![
                    InterpolationPart::Text("Hello, ".to_string()),
                    InterpolationPart::Expression(Box::new(ident("name"))),
                ],
                position: pos(),
            },
        ),
    ];
    let result = r.resolve(&stmts);
    assert!(!result.has_errors());
}

#[test]
fn resolver_scope_resolution_expression() {
    let mut r = Resolver::new();
    let stmt = Statement::Expression {
        expression: Expression::ScopeResolution {
            namespace: Box::new(ident("Math")),
            name: "PI".to_string(),
            position: pos(),
        },
        position: pos(),
    };
    // Math is not defined, but we just test resolution doesn't crash
    let _ = r.resolve(&[stmt]);
}

#[test]
fn resolver_global_variable_expression() {
    let mut r = Resolver::new();
    let stmt = assign(
        "g",
        Expression::GlobalVariable {
            name: "GLOBAL".to_string(),
            position: pos(),
        },
    );
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_instance_and_class_variables() {
    let mut r = Resolver::new();
    let stmts = vec![
        Statement::Expression {
            expression: Expression::InstanceVariable {
                name: "foo".to_string(),
                position: pos(),
            },
            position: pos(),
        },
        Statement::Expression {
            expression: Expression::ClassVariable {
                name: "bar".to_string(),
                position: pos(),
            },
            position: pos(),
        },
    ];
    let result = r.resolve(&stmts);
    assert!(!result.has_errors());
}

#[test]
fn resolver_if_expression() {
    let mut r = Resolver::new();
    let stmt = assign(
        "v",
        Expression::If {
            condition: Box::new(bool_lit(true)),
            then_branch: vec![assign("a", int_lit(1))],
            elsif_branches: vec![ElsifBranch {
                condition: bool_lit(false),
                body: vec![assign("b", int_lit(2))],
                position: pos(),
            }],
            else_branch: Some(vec![assign("c", int_lit(3))]),
            position: pos(),
        },
    );
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_unless_expression() {
    let mut r = Resolver::new();
    let stmt = assign(
        "v",
        Expression::Unless {
            condition: Box::new(bool_lit(false)),
            then_branch: vec![assign("a", int_lit(1))],
            else_branch: Some(vec![assign("b", int_lit(2))]),
            position: pos(),
        },
    );
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

// ── Assignment with index target ──────────────────────────────────────────────

#[test]
fn resolver_index_assignment_target() {
    let mut r = Resolver::new();
    let stmts = vec![
        assign(
            "arr",
            Expression::Array {
                elements: vec![int_lit(0)],
                position: pos(),
            },
        ),
        Statement::Assignment {
            target: Expression::Index {
                array: Box::new(ident("arr")),
                index: Box::new(int_lit(0)),
                position: pos(),
            },
            value: int_lit(99),
            position: pos(),
        },
    ];
    let result = r.resolve(&stmts);
    assert!(!result.has_errors());
}

#[test]
fn resolver_other_assignment_target() {
    let mut r = Resolver::new();
    // Global variable as assignment target
    let stmt = Statement::Assignment {
        target: Expression::GlobalVariable {
            name: "G".to_string(),
            position: pos(),
        },
        value: int_lit(42),
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

// ── FunctionDef / MethodDef with default params ───────────────────────────────

#[test]
fn resolver_function_def_with_default_param() {
    let mut r = Resolver::new();
    let mut param = Parameter::simple("x".to_string(), pos());
    param.default_value = Some(int_lit(42));
    let stmt = Statement::FunctionDef {
        name: "f".to_string(),
        parameters: vec![param],
        body: vec![],
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_method_def_with_default_param() {
    let mut r = Resolver::new();
    let mut param = Parameter::simple("x".to_string(), pos());
    param.default_value = Some(int_lit(0));
    let stmt = Statement::MethodDef {
        is_class_method: false,
        name: "greet".to_string(),
        parameters: vec![param],
        body: vec![],
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

// ── Variable redeclaration (same scope via duplicate function params) ─────────

#[test]
fn resolver_variable_redeclaration_via_duplicate_params() {
    let mut r = Resolver::new();
    // Two parameters with the same name in a function triggers a redeclaration error
    let stmt = Statement::FunctionDef {
        name: "bad".to_string(),
        parameters: vec![
            Parameter::simple("x".to_string(), pos()),
            Parameter::simple("x".to_string(), pos()),
        ],
        body: vec![],
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    // Duplicate param name should produce a redeclaration error
    assert!(result.has_errors());
}

// ── Resolver::default() (lines 753-754) ───────────────────────────────────────

#[test]
fn resolver_default_creates_new_resolver() {
    let mut r = Resolver::default();
    let stmt = assign("x", int_lit(1));
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

// ── Statement::CaseIn resolver (line 413) ─────────────────────────────────────

#[test]
fn resolver_case_in_statement() {
    let mut r = Resolver::new();
    let stmt = Statement::CaseIn {
        expression: int_lit(42),
        cases: vec![MatchCase {
            pattern: MatchPattern::IntLiteral(42),
            guard: None,
            body: vec![assign("res", int_lit(1))],
            position: pos(),
        }],
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

// ── Statement::AttrReader/Writer/Accessor resolver (line 472) ─────────────────

#[test]
fn resolver_attr_reader_writer_accessor() {
    let mut r = Resolver::new();
    let stmts = vec![
        Statement::AttrReader {
            attributes: vec!["name".to_string()],
            position: pos(),
        },
        Statement::AttrWriter {
            attributes: vec!["name".to_string()],
            position: pos(),
        },
        Statement::AttrAccessor {
            attributes: vec!["age".to_string()],
            position: pos(),
        },
    ];
    let result = r.resolve(&stmts);
    assert!(!result.has_errors());
}

// ── Statement::ModuleDef with non-empty body (line 479) ───────────────────────

#[test]
fn resolver_module_def_with_body() {
    let mut r = Resolver::new();
    let stmt = Statement::ModuleDef {
        name: "Greetable".to_string(),
        body: vec![Statement::FunctionDef {
            name: "greet".to_string(),
            parameters: vec![],
            body: vec![assign("msg", int_lit(1))],
            position: pos(),
        }],
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

// ── Expression::MethodCall with arguments (line 589) ──────────────────────────

#[test]
fn resolver_method_call_with_arguments() {
    let mut r = Resolver::new();
    let stmts = vec![
        assign("obj", int_lit(42)),
        Statement::Expression {
            expression: Expression::MethodCall {
                receiver: Box::new(ident("obj")),
                method: "clamp".to_string(),
                arguments: vec![int_lit(0), int_lit(100)],
                trailing_block: None,
                position: pos(),
            },
            position: pos(),
        },
    ];
    let result = r.resolve(&stmts);
    assert!(!result.has_errors());
}

// ── Expression::Case resolver (lines 647-676) ─────────────────────────────────

#[test]
fn resolver_case_expression() {
    use metorex::ast::node::ExprMatchCase;
    let mut r = Resolver::new();
    let stmt = assign(
        "v",
        Expression::Case {
            expression: Box::new(int_lit(42)),
            cases: vec![ExprMatchCase {
                pattern: MatchPattern::IntLiteral(42),
                guard: None,
                body: int_lit(1),
                position: pos(),
            }],
            else_case: Some(Box::new(int_lit(0))),
            position: pos(),
        },
    );
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

#[test]
fn resolver_case_expression_with_guard() {
    use metorex::ast::node::ExprMatchCase;
    let mut r = Resolver::new();
    let stmt = assign(
        "v",
        Expression::Case {
            expression: Box::new(int_lit(5)),
            cases: vec![ExprMatchCase {
                pattern: MatchPattern::Identifier("n".to_string()),
                guard: Some(Expression::BinaryOp {
                    op: metorex::ast::BinaryOp::Greater,
                    left: Box::new(ident("n")),
                    right: Box::new(int_lit(0)),
                    position: pos(),
                }),
                body: ident("n"),
                position: pos(),
            }],
            else_case: None,
            position: pos(),
        },
    );
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}

// ── Variable reassignment in same function scope (lines 227-228) ──────────────

#[test]
fn resolver_variable_reassignment_in_function_scope() {
    let mut r = Resolver::new();
    // x is declared as a parameter, then reassigned in the body
    let stmt = Statement::FunctionDef {
        name: "f".to_string(),
        parameters: vec![Parameter::simple("x".to_string(), pos())],
        body: vec![
            // Reassign x (already in scope) - triggers lines 227-228
            Statement::Assignment {
                target: ident("x"),
                value: int_lit(99),
                position: pos(),
            },
        ],
        position: pos(),
    };
    let result = r.resolve(&[stmt]);
    assert!(!result.has_errors());
}
