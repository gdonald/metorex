use super::helpers::pos;
use metorex::ast::{BinaryOp, Expression, Statement, UnaryOp};

#[test]
fn test_int_literal() {
    let expr = Expression::IntLiteral {
        value: 42,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
    assert!(expr.is_literal());
    assert!(!expr.is_identifier());
}

#[test]
fn test_float_literal() {
    let expr = Expression::FloatLiteral {
        value: 3.14,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
    assert!(expr.is_literal());
}

#[test]
fn test_string_literal() {
    let expr = Expression::StringLiteral {
        value: "hello".to_string(),
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
    assert!(expr.is_literal());
}

#[test]
fn test_bool_literal_true() {
    let expr = Expression::BoolLiteral {
        value: true,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
    assert!(expr.is_literal());
}

#[test]
fn test_bool_literal_false() {
    let expr = Expression::BoolLiteral {
        value: false,
        position: pos(1, 1),
    };
    assert!(expr.is_literal());
}

#[test]
fn test_nil_literal() {
    let expr = Expression::NilLiteral {
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
    assert!(expr.is_literal());
}

#[test]
fn test_identifier() {
    let expr = Expression::Identifier {
        name: "foo".to_string(),
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
    assert!(!expr.is_literal());
    assert!(expr.is_identifier());
}

#[test]
fn test_instance_variable() {
    let expr = Expression::InstanceVariable {
        name: "name".to_string(),
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
    assert!(expr.is_identifier());
}

#[test]
fn test_class_variable() {
    let expr = Expression::ClassVariable {
        name: "count".to_string(),
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
    assert!(expr.is_identifier());
}

#[test]
fn test_binary_op_add() {
    let left = Box::new(Expression::IntLiteral {
        value: 1,
        position: pos(1, 1),
    });
    let right = Box::new(Expression::IntLiteral {
        value: 2,
        position: pos(1, 5),
    });
    let expr = Expression::BinaryOp {
        op: BinaryOp::Add,
        left,
        right,
        position: pos(1, 3),
    };
    assert_eq!(expr.position(), pos(1, 3));
    assert!(!expr.is_literal());
}

#[test]
fn test_binary_op_subtract() {
    let left = Box::new(Expression::IntLiteral {
        value: 5,
        position: pos(1, 1),
    });
    let right = Box::new(Expression::IntLiteral {
        value: 3,
        position: pos(1, 5),
    });
    let expr = Expression::BinaryOp {
        op: BinaryOp::Subtract,
        left,
        right,
        position: pos(1, 3),
    };
    assert_eq!(expr.position(), pos(1, 3));
}

#[test]
fn test_binary_op_multiply() {
    let expr = Expression::BinaryOp {
        op: BinaryOp::Multiply,
        left: Box::new(Expression::IntLiteral {
            value: 2,
            position: pos(1, 1),
        }),
        right: Box::new(Expression::IntLiteral {
            value: 3,
            position: pos(1, 5),
        }),
        position: pos(1, 3),
    };
    assert_eq!(expr.position(), pos(1, 3));
}

#[test]
fn test_binary_op_divide() {
    let expr = Expression::BinaryOp {
        op: BinaryOp::Divide,
        left: Box::new(Expression::IntLiteral {
            value: 10,
            position: pos(1, 1),
        }),
        right: Box::new(Expression::IntLiteral {
            value: 2,
            position: pos(1, 5),
        }),
        position: pos(1, 3),
    };
    assert_eq!(expr.position(), pos(1, 3));
}

#[test]
fn test_binary_op_comparison() {
    let expr = Expression::BinaryOp {
        op: BinaryOp::Equal,
        left: Box::new(Expression::Identifier {
            name: "x".to_string(),
            position: pos(1, 1),
        }),
        right: Box::new(Expression::IntLiteral {
            value: 5,
            position: pos(1, 6),
        }),
        position: pos(1, 3),
    };
    assert_eq!(expr.position(), pos(1, 3));
}

#[test]
fn test_unary_op_minus() {
    let expr = Expression::UnaryOp {
        op: UnaryOp::Minus,
        operand: Box::new(Expression::IntLiteral {
            value: 5,
            position: pos(1, 2),
        }),
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
    assert!(!expr.is_literal());
}

#[test]
fn test_unary_op_plus() {
    let expr = Expression::UnaryOp {
        op: UnaryOp::Plus,
        operand: Box::new(Expression::IntLiteral {
            value: 5,
            position: pos(1, 2),
        }),
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_call_expression() {
    let expr = Expression::Call {
        callee: Box::new(Expression::Identifier {
            name: "foo".to_string(),
            position: pos(1, 1),
        }),
        arguments: vec![
            Expression::IntLiteral {
                value: 1,
                position: pos(1, 5),
            },
            Expression::IntLiteral {
                value: 2,
                position: pos(1, 8),
            },
        ],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_method_call() {
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::Identifier {
            name: "obj".to_string(),
            position: pos(1, 1),
        }),
        method: "foo".to_string(),
        arguments: vec![],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_array_literal() {
    let expr = Expression::Array {
        elements: vec![
            Expression::IntLiteral {
                value: 1,
                position: pos(1, 2),
            },
            Expression::IntLiteral {
                value: 2,
                position: pos(1, 5),
            },
            Expression::IntLiteral {
                value: 3,
                position: pos(1, 8),
            },
        ],
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_array_empty() {
    let expr = Expression::Array {
        elements: vec![],
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_array_index() {
    let expr = Expression::Index {
        array: Box::new(Expression::Identifier {
            name: "arr".to_string(),
            position: pos(1, 1),
        }),
        index: Box::new(Expression::IntLiteral {
            value: 0,
            position: pos(1, 5),
        }),
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_dictionary_literal() {
    let expr = Expression::Dictionary {
        entries: vec![
            (
                Expression::StringLiteral {
                    value: "name".to_string(),
                    position: pos(1, 2),
                },
                Expression::StringLiteral {
                    value: "Alice".to_string(),
                    position: pos(1, 9),
                },
            ),
            (
                Expression::StringLiteral {
                    value: "age".to_string(),
                    position: pos(2, 2),
                },
                Expression::IntLiteral {
                    value: 30,
                    position: pos(2, 8),
                },
            ),
        ],
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_dictionary_empty() {
    let expr = Expression::Dictionary {
        entries: vec![],
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_lambda_expression() {
    let expr = Expression::Lambda {
        parameters: vec!["x".to_string(), "y".to_string()],
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
        captured_vars: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_grouped_expression() {
    let expr = Expression::Grouped {
        expression: Box::new(Expression::BinaryOp {
            op: BinaryOp::Add,
            left: Box::new(Expression::IntLiteral {
                value: 1,
                position: pos(1, 2),
            }),
            right: Box::new(Expression::IntLiteral {
                value: 2,
                position: pos(1, 6),
            }),
            position: pos(1, 4),
        }),
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}
