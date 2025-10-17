use super::helpers::pos;
use metorex::ast::Expression;
use metorex::ast::node::{ExprMatchCase, MatchPattern};

#[test]
fn test_expr_match_case_with_literal_pattern() {
    let case = ExprMatchCase {
        pattern: MatchPattern::IntLiteral(42),
        guard: None,
        body: Expression::StringLiteral {
            value: "forty-two".to_string(),
            position: pos(1, 10),
        },
        position: pos(1, 1),
    };

    assert_eq!(case.position, pos(1, 1));
    assert!(case.guard.is_none());
    match case.pattern {
        MatchPattern::IntLiteral(n) => assert_eq!(n, 42),
        _ => panic!("Expected IntLiteral pattern"),
    }
}

#[test]
fn test_expr_match_case_with_guard() {
    let case = ExprMatchCase {
        pattern: MatchPattern::Identifier("x".to_string()),
        guard: Some(Expression::BinaryOp {
            op: metorex::ast::BinaryOp::Greater,
            left: Box::new(Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 15),
            }),
            right: Box::new(Expression::IntLiteral {
                value: 0,
                position: pos(1, 19),
            }),
            position: pos(1, 15),
        }),
        body: Expression::StringLiteral {
            value: "positive".to_string(),
            position: pos(1, 25),
        },
        position: pos(1, 1),
    };

    assert!(case.guard.is_some());
    assert_eq!(case.position, pos(1, 1));
}

#[test]
fn test_expr_match_case_with_wildcard() {
    let case = ExprMatchCase {
        pattern: MatchPattern::Wildcard,
        guard: None,
        body: Expression::NilLiteral {
            position: pos(1, 10),
        },
        position: pos(1, 1),
    };

    assert!(matches!(case.pattern, MatchPattern::Wildcard));
    assert!(case.guard.is_none());
}

#[test]
fn test_expr_match_case_with_array_pattern() {
    let case = ExprMatchCase {
        pattern: MatchPattern::Array(vec![
            MatchPattern::Identifier("first".to_string()),
            MatchPattern::Identifier("second".to_string()),
        ]),
        guard: None,
        body: Expression::BinaryOp {
            op: metorex::ast::BinaryOp::Add,
            left: Box::new(Expression::Identifier {
                name: "first".to_string(),
                position: pos(1, 20),
            }),
            right: Box::new(Expression::Identifier {
                name: "second".to_string(),
                position: pos(1, 28),
            }),
            position: pos(1, 20),
        },
        position: pos(1, 1),
    };

    match &case.pattern {
        MatchPattern::Array(patterns) => assert_eq!(patterns.len(), 2),
        _ => panic!("Expected Array pattern"),
    }
}

#[test]
fn test_expression_case_simple() {
    let case_expr = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 2,
            position: pos(1, 6),
        }),
        cases: vec![
            ExprMatchCase {
                pattern: MatchPattern::IntLiteral(1),
                guard: None,
                body: Expression::StringLiteral {
                    value: "one".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 1),
            },
            ExprMatchCase {
                pattern: MatchPattern::IntLiteral(2),
                guard: None,
                body: Expression::StringLiteral {
                    value: "two".to_string(),
                    position: pos(3, 10),
                },
                position: pos(3, 1),
            },
        ],
        else_case: Some(Box::new(Expression::StringLiteral {
            value: "other".to_string(),
            position: pos(4, 6),
        })),
        position: pos(1, 1),
    };

    assert_eq!(case_expr.position(), pos(1, 1));
    assert!(!case_expr.is_literal());
    assert!(!case_expr.is_identifier());

    match case_expr {
        Expression::Case {
            expression,
            cases,
            else_case,
            ..
        } => {
            assert!(matches!(
                expression.as_ref(),
                Expression::IntLiteral { value: 2, .. }
            ));
            assert_eq!(cases.len(), 2);
            assert!(else_case.is_some());
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_expression_case_without_else() {
    let case_expr = Expression::Case {
        expression: Box::new(Expression::Identifier {
            name: "x".to_string(),
            position: pos(1, 6),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::IntLiteral(1),
            guard: None,
            body: Expression::StringLiteral {
                value: "one".to_string(),
                position: pos(2, 10),
            },
            position: pos(2, 1),
        }],
        else_case: None,
        position: pos(1, 1),
    };

    match case_expr {
        Expression::Case {
            else_case, cases, ..
        } => {
            assert!(else_case.is_none());
            assert_eq!(cases.len(), 1);
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_expression_case_with_multiple_guards() {
    let case_expr = Expression::Case {
        expression: Box::new(Expression::Identifier {
            name: "value".to_string(),
            position: pos(1, 6),
        }),
        cases: vec![
            ExprMatchCase {
                pattern: MatchPattern::Identifier("x".to_string()),
                guard: Some(Expression::BinaryOp {
                    op: metorex::ast::BinaryOp::Less,
                    left: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(2, 15),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 0,
                        position: pos(2, 19),
                    }),
                    position: pos(2, 15),
                }),
                body: Expression::StringLiteral {
                    value: "negative".to_string(),
                    position: pos(2, 25),
                },
                position: pos(2, 1),
            },
            ExprMatchCase {
                pattern: MatchPattern::Identifier("x".to_string()),
                guard: Some(Expression::BinaryOp {
                    op: metorex::ast::BinaryOp::Greater,
                    left: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(3, 15),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 0,
                        position: pos(3, 19),
                    }),
                    position: pos(3, 15),
                }),
                body: Expression::StringLiteral {
                    value: "positive".to_string(),
                    position: pos(3, 25),
                },
                position: pos(3, 1),
            },
        ],
        else_case: Some(Box::new(Expression::StringLiteral {
            value: "zero".to_string(),
            position: pos(4, 6),
        })),
        position: pos(1, 1),
    };

    match case_expr {
        Expression::Case { cases, .. } => {
            assert_eq!(cases.len(), 2);
            assert!(cases[0].guard.is_some());
            assert!(cases[1].guard.is_some());
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_expression_case_nested() {
    let inner_case = Expression::Case {
        expression: Box::new(Expression::Identifier {
            name: "y".to_string(),
            position: pos(3, 10),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::IntLiteral(1),
            guard: None,
            body: Expression::StringLiteral {
                value: "inner".to_string(),
                position: pos(4, 15),
            },
            position: pos(4, 1),
        }],
        else_case: None,
        position: pos(3, 5),
    };

    let outer_case = Expression::Case {
        expression: Box::new(Expression::Identifier {
            name: "x".to_string(),
            position: pos(1, 6),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::IntLiteral(1),
            guard: None,
            body: inner_case,
            position: pos(2, 1),
        }],
        else_case: None,
        position: pos(1, 1),
    };

    assert_eq!(outer_case.position(), pos(1, 1));
    match outer_case {
        Expression::Case { cases, .. } => {
            assert_eq!(cases.len(), 1);
            match &cases[0].body {
                Expression::Case { .. } => {
                    // Successfully found nested case
                }
                _ => panic!("Expected nested Expression::Case"),
            }
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_expression_case_with_type_patterns() {
    let case_expr = Expression::Case {
        expression: Box::new(Expression::Identifier {
            name: "obj".to_string(),
            position: pos(1, 6),
        }),
        cases: vec![
            ExprMatchCase {
                pattern: MatchPattern::Type("Integer".to_string()),
                guard: None,
                body: Expression::StringLiteral {
                    value: "int".to_string(),
                    position: pos(2, 10),
                },
                position: pos(2, 1),
            },
            ExprMatchCase {
                pattern: MatchPattern::Type("String".to_string()),
                guard: None,
                body: Expression::StringLiteral {
                    value: "str".to_string(),
                    position: pos(3, 10),
                },
                position: pos(3, 1),
            },
        ],
        else_case: None,
        position: pos(1, 1),
    };

    match case_expr {
        Expression::Case { cases, .. } => {
            assert_eq!(cases.len(), 2);
            assert!(matches!(
                cases[0].pattern,
                MatchPattern::Type(ref s) if s == "Integer"
            ));
            assert!(matches!(
                cases[1].pattern,
                MatchPattern::Type(ref s) if s == "String"
            ));
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_expression_case_with_object_pattern() {
    let case_expr = Expression::Case {
        expression: Box::new(Expression::Identifier {
            name: "point".to_string(),
            position: pos(1, 6),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Object(vec![
                ("x".to_string(), MatchPattern::Identifier("px".to_string())),
                ("y".to_string(), MatchPattern::Identifier("py".to_string())),
            ]),
            guard: None,
            body: Expression::BinaryOp {
                op: metorex::ast::BinaryOp::Add,
                left: Box::new(Expression::Identifier {
                    name: "px".to_string(),
                    position: pos(2, 20),
                }),
                right: Box::new(Expression::Identifier {
                    name: "py".to_string(),
                    position: pos(2, 25),
                }),
                position: pos(2, 20),
            },
            position: pos(2, 1),
        }],
        else_case: None,
        position: pos(1, 1),
    };

    match case_expr {
        Expression::Case { cases, .. } => {
            assert_eq!(cases.len(), 1);
            match &cases[0].pattern {
                MatchPattern::Object(pairs) => {
                    assert_eq!(pairs.len(), 2);
                    assert_eq!(pairs[0].0, "x");
                    assert_eq!(pairs[1].0, "y");
                }
                _ => panic!("Expected Object pattern"),
            }
        }
        _ => panic!("Expected Expression::Case"),
    }
}

#[test]
fn test_expression_case_clone() {
    let original = Expression::Case {
        expression: Box::new(Expression::IntLiteral {
            value: 1,
            position: pos(1, 1),
        }),
        cases: vec![ExprMatchCase {
            pattern: MatchPattern::Wildcard,
            guard: None,
            body: Expression::NilLiteral {
                position: pos(2, 1),
            },
            position: pos(2, 1),
        }],
        else_case: None,
        position: pos(1, 1),
    };

    let cloned = original.clone();
    assert_eq!(original, cloned);
}

#[test]
fn test_expr_match_case_clone() {
    let original = ExprMatchCase {
        pattern: MatchPattern::IntLiteral(42),
        guard: None,
        body: Expression::StringLiteral {
            value: "test".to_string(),
            position: pos(1, 1),
        },
        position: pos(1, 1),
    };

    let cloned = original.clone();
    assert_eq!(original, cloned);
}
