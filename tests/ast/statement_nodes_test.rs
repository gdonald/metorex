use super::helpers::pos;
use metorex::ast::{BinaryOp, Expression, Parameter, Statement};

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
        elsif_branches: vec![],
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
        elsif_branches: vec![],
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
