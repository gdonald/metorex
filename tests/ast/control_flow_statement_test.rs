use super::helpers::pos;
use metorex::ast::{BinaryOp, Expression, Statement};

#[test]
fn test_break_statement() {
    let stmt = Statement::Break {
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_control_flow());
    assert!(!stmt.is_definition());
}

#[test]
fn test_continue_statement() {
    let stmt = Statement::Continue {
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_control_flow());
    assert!(!stmt.is_definition());
}

#[test]
fn test_while_with_break() {
    let stmt = Statement::While {
        condition: Expression::BoolLiteral {
            value: true,
            position: pos(1, 7),
        },
        body: vec![
            Statement::Expression {
                expression: Expression::IntLiteral {
                    value: 1,
                    position: pos(2, 3),
                },
                position: pos(2, 3),
            },
            Statement::Break {
                position: pos(3, 3),
            },
        ],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_control_flow());
}

#[test]
fn test_while_with_continue() {
    let stmt = Statement::While {
        condition: Expression::BinaryOp {
            op: BinaryOp::Less,
            left: Box::new(Expression::Identifier {
                name: "i".to_string(),
                position: pos(1, 7),
            }),
            right: Box::new(Expression::IntLiteral {
                value: 10,
                position: pos(1, 11),
            }),
            position: pos(1, 9),
        },
        body: vec![
            Statement::If {
                condition: Expression::BinaryOp {
                    op: BinaryOp::Equal,
                    left: Box::new(Expression::BinaryOp {
                        op: BinaryOp::Modulo,
                        left: Box::new(Expression::Identifier {
                            name: "i".to_string(),
                            position: pos(2, 7),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 2,
                            position: pos(2, 11),
                        }),
                        position: pos(2, 9),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 0,
                        position: pos(2, 16),
                    }),
                    position: pos(2, 14),
                },
                then_branch: vec![Statement::Continue {
                    position: pos(3, 5),
                }],
                else_branch: None,
                position: pos(2, 3),
            },
            Statement::Expression {
                expression: Expression::IntLiteral {
                    value: 1,
                    position: pos(5, 3),
                },
                position: pos(5, 3),
            },
        ],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_nested_loops_with_break_and_continue() {
    // Outer loop with inner loop, both have break/continue
    let stmt = Statement::While {
        condition: Expression::BoolLiteral {
            value: true,
            position: pos(1, 7),
        },
        body: vec![Statement::While {
            condition: Expression::BoolLiteral {
                value: true,
                position: pos(2, 9),
            },
            body: vec![
                Statement::Break {
                    position: pos(3, 5),
                },
                Statement::Continue {
                    position: pos(4, 5),
                },
            ],
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}
