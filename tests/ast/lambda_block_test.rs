use super::helpers::pos;
use metorex::ast::{BinaryOp, Expression, Statement};

#[test]
fn test_lambda_without_captured_vars() {
    let expr = Expression::Lambda {
        parameters: vec!["x".to_string()],
        body: vec![Statement::Expression {
            expression: Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 10),
            },
            position: pos(1, 10),
        }],
        captured_vars: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_lambda_with_captured_vars() {
    // Lambda that captures 'y' from outer scope: x -> x + y
    let expr = Expression::Lambda {
        parameters: vec!["x".to_string()],
        body: vec![Statement::Expression {
            expression: Expression::BinaryOp {
                op: BinaryOp::Add,
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(1, 10),
                }),
                right: Box::new(Expression::Identifier {
                    name: "y".to_string(),
                    position: pos(1, 14),
                }),
                position: pos(1, 12),
            },
            position: pos(1, 10),
        }],
        captured_vars: Some(vec!["y".to_string()]),
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_lambda_with_multiple_captured_vars() {
    // Lambda that captures 'a', 'b', 'c' from outer scope
    let expr = Expression::Lambda {
        parameters: vec!["x".to_string()],
        body: vec![Statement::Expression {
            expression: Expression::BinaryOp {
                op: BinaryOp::Add,
                left: Box::new(Expression::Identifier {
                    name: "a".to_string(),
                    position: pos(1, 10),
                }),
                right: Box::new(Expression::BinaryOp {
                    op: BinaryOp::Add,
                    left: Box::new(Expression::Identifier {
                        name: "b".to_string(),
                        position: pos(1, 14),
                    }),
                    right: Box::new(Expression::Identifier {
                        name: "c".to_string(),
                        position: pos(1, 18),
                    }),
                    position: pos(1, 16),
                }),
                position: pos(1, 12),
            },
            position: pos(1, 10),
        }],
        captured_vars: Some(vec!["a".to_string(), "b".to_string(), "c".to_string()]),
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_lambda_implicit_return() {
    // Lambda with implicit return (last expression): x -> x * 2
    let expr = Expression::Lambda {
        parameters: vec!["x".to_string()],
        body: vec![Statement::Expression {
            expression: Expression::BinaryOp {
                op: BinaryOp::Multiply,
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(1, 10),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 2,
                    position: pos(1, 14),
                }),
                position: pos(1, 12),
            },
            position: pos(1, 10),
        }],
        captured_vars: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_lambda_with_multiple_statements() {
    // Lambda with multiple statements, last one is implicit return
    let expr = Expression::Lambda {
        parameters: vec!["x".to_string()],
        body: vec![
            Statement::Assignment {
                target: Expression::Identifier {
                    name: "temp".to_string(),
                    position: pos(2, 3),
                },
                value: Expression::BinaryOp {
                    op: BinaryOp::Multiply,
                    left: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(2, 10),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 2,
                        position: pos(2, 14),
                    }),
                    position: pos(2, 12),
                },
                position: pos(2, 3),
            },
            Statement::Expression {
                expression: Expression::Identifier {
                    name: "temp".to_string(),
                    position: pos(3, 3),
                },
                position: pos(3, 3),
            },
        ],
        captured_vars: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_lambda_no_parameters() {
    // Lambda with no parameters: -> 42
    let expr = Expression::Lambda {
        parameters: vec![],
        body: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 42,
                position: pos(1, 5),
            },
            position: pos(1, 5),
        }],
        captured_vars: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_lambda_with_instance_var_capture() {
    // Lambda that captures instance variable @count
    let expr = Expression::Lambda {
        parameters: vec!["x".to_string()],
        body: vec![Statement::Expression {
            expression: Expression::BinaryOp {
                op: BinaryOp::Add,
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(1, 10),
                }),
                right: Box::new(Expression::InstanceVariable {
                    name: "count".to_string(),
                    position: pos(1, 14),
                }),
                position: pos(1, 12),
            },
            position: pos(1, 10),
        }],
        captured_vars: Some(vec!["@count".to_string()]),
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_nested_lambdas() {
    // Outer lambda that returns an inner lambda
    let inner_lambda = Expression::Lambda {
        parameters: vec!["y".to_string()],
        body: vec![Statement::Expression {
            expression: Expression::BinaryOp {
                op: BinaryOp::Add,
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(2, 10),
                }),
                right: Box::new(Expression::Identifier {
                    name: "y".to_string(),
                    position: pos(2, 14),
                }),
                position: pos(2, 12),
            },
            position: pos(2, 10),
        }],
        captured_vars: Some(vec!["x".to_string()]),
        position: pos(2, 1),
    };

    let outer_lambda = Expression::Lambda {
        parameters: vec!["x".to_string()],
        body: vec![Statement::Expression {
            expression: inner_lambda,
            position: pos(2, 1),
        }],
        captured_vars: None,
        position: pos(1, 1),
    };

    assert_eq!(outer_lambda.position(), pos(1, 1));
}

#[test]
fn test_block_with_multiple_statements() {
    // Block statement with multiple statements
    let stmt = Statement::Block {
        statements: vec![
            Statement::Assignment {
                target: Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(2, 3),
                },
                value: Expression::IntLiteral {
                    value: 10,
                    position: pos(2, 7),
                },
                position: pos(2, 3),
            },
            Statement::Assignment {
                target: Expression::Identifier {
                    name: "y".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::IntLiteral {
                    value: 20,
                    position: pos(3, 7),
                },
                position: pos(3, 3),
            },
            Statement::Expression {
                expression: Expression::BinaryOp {
                    op: BinaryOp::Add,
                    left: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(4, 3),
                    }),
                    right: Box::new(Expression::Identifier {
                        name: "y".to_string(),
                        position: pos(4, 7),
                    }),
                    position: pos(4, 5),
                },
                position: pos(4, 3),
            },
        ],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(!stmt.is_control_flow());
    assert!(!stmt.is_definition());
}

#[test]
fn test_empty_block() {
    // Empty block statement
    let stmt = Statement::Block {
        statements: vec![],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}

#[test]
fn test_nested_blocks() {
    // Block containing another block
    let stmt = Statement::Block {
        statements: vec![
            Statement::Assignment {
                target: Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(2, 3),
                },
                value: Expression::IntLiteral {
                    value: 10,
                    position: pos(2, 7),
                },
                position: pos(2, 3),
            },
            Statement::Block {
                statements: vec![Statement::Expression {
                    expression: Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(4, 5),
                    },
                    position: pos(4, 5),
                }],
                position: pos(3, 3),
            },
        ],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
}
