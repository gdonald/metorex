// Unit tests for AST node creation and manipulation

use metorex::ast::{BinaryOp, Expression, Parameter, Statement, UnaryOp};
use metorex::lexer::Position;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

// Tests for Expression nodes

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

// Tests for Statement nodes

#[test]
fn test_expression_statement() {
    let stmt = Statement::Expression {
        expression: Expression::IntLiteral {
            value: 42,
            position: pos(1, 1),
        },
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(!stmt.is_definition());
    assert!(!stmt.is_control_flow());
}

#[test]
fn test_assignment_statement() {
    let stmt = Statement::Assignment {
        target: Expression::Identifier {
            name: "x".to_string(),
            position: pos(1, 1),
        },
        value: Expression::IntLiteral {
            value: 42,
            position: pos(1, 5),
        },
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(!stmt.is_definition());
}

#[test]
fn test_method_def() {
    let stmt = Statement::MethodDef {
        name: "add".to_string(),
        parameters: vec![
            Parameter::simple("x".to_string(), pos(1, 9)),
            Parameter::simple("y".to_string(), pos(1, 12)),
        ],
        body: vec![Statement::Expression {
            expression: Expression::BinaryOp {
                op: BinaryOp::Add,
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(2, 3),
                }),
                right: Box::new(Expression::Identifier {
                    name: "y".to_string(),
                    position: pos(2, 7),
                }),
                position: pos(2, 5),
            },
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_definition());
    assert!(!stmt.is_control_flow());
}

#[test]
fn test_class_def_no_superclass() {
    let stmt = Statement::ClassDef {
        name: "Person".to_string(),
        superclass: None,
        body: vec![],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_definition());
}

#[test]
fn test_class_def_with_superclass() {
    let stmt = Statement::ClassDef {
        name: "Employee".to_string(),
        superclass: Some("Person".to_string()),
        body: vec![],
        position: pos(1, 1),
    };
    assert!(stmt.is_definition());
}

#[test]
fn test_if_statement_no_else() {
    let stmt = Statement::If {
        condition: Expression::BoolLiteral {
            value: true,
            position: pos(1, 4),
        },
        then_branch: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 1,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        else_branch: None,
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_control_flow());
    assert!(!stmt.is_definition());
}

#[test]
fn test_if_statement_with_else() {
    let stmt = Statement::If {
        condition: Expression::BoolLiteral {
            value: true,
            position: pos(1, 4),
        },
        then_branch: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 1,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        else_branch: Some(vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 2,
                position: pos(4, 3),
            },
            position: pos(4, 3),
        }]),
        position: pos(1, 1),
    };
    assert!(stmt.is_control_flow());
}

#[test]
fn test_while_statement() {
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
        body: vec![Statement::Expression {
            expression: Expression::IntLiteral {
                value: 1,
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_control_flow());
}

#[test]
fn test_return_statement_with_value() {
    let stmt = Statement::Return {
        value: Some(Expression::IntLiteral {
            value: 42,
            position: pos(1, 8),
        }),
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(stmt.is_control_flow());
}

#[test]
fn test_return_statement_no_value() {
    let stmt = Statement::Return {
        value: None,
        position: pos(1, 1),
    };
    assert!(stmt.is_control_flow());
}

#[test]
fn test_block_statement() {
    let stmt = Statement::Block {
        statements: vec![
            Statement::Expression {
                expression: Expression::IntLiteral {
                    value: 1,
                    position: pos(1, 1),
                },
                position: pos(1, 1),
            },
            Statement::Expression {
                expression: Expression::IntLiteral {
                    value: 2,
                    position: pos(2, 1),
                },
                position: pos(2, 1),
            },
        ],
        position: pos(1, 1),
    };
    assert_eq!(stmt.position(), pos(1, 1));
    assert!(!stmt.is_control_flow());
}

// Tests for operator Display implementations

#[test]
fn test_binary_op_display() {
    assert_eq!(format!("{}", BinaryOp::Add), "+");
    assert_eq!(format!("{}", BinaryOp::Subtract), "-");
    assert_eq!(format!("{}", BinaryOp::Multiply), "*");
    assert_eq!(format!("{}", BinaryOp::Divide), "/");
    assert_eq!(format!("{}", BinaryOp::Modulo), "%");
    assert_eq!(format!("{}", BinaryOp::Equal), "==");
    assert_eq!(format!("{}", BinaryOp::NotEqual), "!=");
    assert_eq!(format!("{}", BinaryOp::Less), "<");
    assert_eq!(format!("{}", BinaryOp::Greater), ">");
    assert_eq!(format!("{}", BinaryOp::LessEqual), "<=");
    assert_eq!(format!("{}", BinaryOp::GreaterEqual), ">=");
    assert_eq!(format!("{}", BinaryOp::Assign), "=");
    assert_eq!(format!("{}", BinaryOp::AddAssign), "+=");
    assert_eq!(format!("{}", BinaryOp::SubtractAssign), "-=");
    assert_eq!(format!("{}", BinaryOp::MultiplyAssign), "*=");
    assert_eq!(format!("{}", BinaryOp::DivideAssign), "/=");
}

#[test]
fn test_unary_op_display() {
    assert_eq!(format!("{}", UnaryOp::Plus), "+");
    assert_eq!(format!("{}", UnaryOp::Minus), "-");
}

// Tests for nested expressions

#[test]
fn test_nested_binary_ops() {
    // (1 + 2) * 3
    let expr = Expression::BinaryOp {
        op: BinaryOp::Multiply,
        left: Box::new(Expression::Grouped {
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
        }),
        right: Box::new(Expression::IntLiteral {
            value: 3,
            position: pos(1, 11),
        }),
        position: pos(1, 9),
    };
    assert_eq!(expr.position(), pos(1, 9));
}

#[test]
fn test_chained_method_calls() {
    // obj.foo().bar()
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::MethodCall {
            receiver: Box::new(Expression::Identifier {
                name: "obj".to_string(),
                position: pos(1, 1),
            }),
            method: "foo".to_string(),
            arguments: vec![],
            trailing_block: None,
            position: pos(1, 1),
        }),
        method: "bar".to_string(),
        arguments: vec![],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

#[test]
fn test_complex_expression() {
    // array[i].method(x + y)
    let expr = Expression::MethodCall {
        receiver: Box::new(Expression::Index {
            array: Box::new(Expression::Identifier {
                name: "array".to_string(),
                position: pos(1, 1),
            }),
            index: Box::new(Expression::Identifier {
                name: "i".to_string(),
                position: pos(1, 7),
            }),
            position: pos(1, 1),
        }),
        method: "method".to_string(),
        arguments: vec![Expression::BinaryOp {
            op: BinaryOp::Add,
            left: Box::new(Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 17),
            }),
            right: Box::new(Expression::Identifier {
                name: "y".to_string(),
                position: pos(1, 21),
            }),
            position: pos(1, 19),
        }],
        trailing_block: None,
        position: pos(1, 1),
    };
    assert_eq!(expr.position(), pos(1, 1));
}

// Tests for Break and Continue statements

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

// Tests for Block/Lambda enhancements (Task 3.5)

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
