// Unit tests for complex control flow execution in the VM

use metorex::ast::{BinaryOp, Expression, Statement};
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

// Tests for complex control flow combinations

#[test]
fn test_for_loop_in_if_branch() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "flag".to_string(),
                position: pos(1, 1),
            },
            value: Expression::BoolLiteral {
                value: true,
                position: pos(1, 8),
            },
            position: pos(1, 1),
        },
        Statement::Assignment {
            target: Expression::Identifier {
                name: "sum".to_string(),
                position: pos(2, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(2, 7),
            },
            position: pos(2, 1),
        },
        Statement::If {
            condition: Expression::Identifier {
                name: "flag".to_string(),
                position: pos(3, 4),
            },
            then_branch: vec![Statement::For {
                variable: "x".to_string(),
                iterable: Expression::Array {
                    elements: vec![
                        Expression::IntLiteral {
                            value: 1,
                            position: pos(4, 16),
                        },
                        Expression::IntLiteral {
                            value: 2,
                            position: pos(4, 19),
                        },
                        Expression::IntLiteral {
                            value: 3,
                            position: pos(4, 22),
                        },
                    ],
                    position: pos(4, 15),
                },
                body: vec![Statement::Assignment {
                    target: Expression::Identifier {
                        name: "sum".to_string(),
                        position: pos(5, 5),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "sum".to_string(),
                            position: pos(5, 11),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(5, 17),
                        }),
                        position: pos(5, 15),
                    },
                    position: pos(5, 5),
                }],
                position: pos(4, 3),
            }],
            else_branch: None,
            position: pos(3, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let sum = vm.environment().get("sum").unwrap();
    assert_eq!(sum, Object::Int(6)); // 1+2+3 = 6
}

#[test]
fn test_while_loop_in_for_loop() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "total".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 9),
            },
            position: pos(1, 1),
        },
        Statement::For {
            variable: "outer".to_string(),
            iterable: Expression::Array {
                elements: vec![
                    Expression::IntLiteral {
                        value: 2,
                        position: pos(2, 18),
                    },
                    Expression::IntLiteral {
                        value: 3,
                        position: pos(2, 21),
                    },
                ],
                position: pos(2, 17),
            },
            body: vec![
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "inner".to_string(),
                        position: pos(3, 3),
                    },
                    value: Expression::IntLiteral {
                        value: 0,
                        position: pos(3, 11),
                    },
                    position: pos(3, 3),
                },
                Statement::While {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::Less,
                        left: Box::new(Expression::Identifier {
                            name: "inner".to_string(),
                            position: pos(4, 9),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "outer".to_string(),
                            position: pos(4, 17),
                        }),
                        position: pos(4, 15),
                    },
                    body: vec![
                        Statement::Assignment {
                            target: Expression::Identifier {
                                name: "total".to_string(),
                                position: pos(5, 5),
                            },
                            value: Expression::BinaryOp {
                                op: BinaryOp::Add,
                                left: Box::new(Expression::Identifier {
                                    name: "total".to_string(),
                                    position: pos(5, 13),
                                }),
                                right: Box::new(Expression::IntLiteral {
                                    value: 1,
                                    position: pos(5, 21),
                                }),
                                position: pos(5, 19),
                            },
                            position: pos(5, 5),
                        },
                        Statement::Assignment {
                            target: Expression::Identifier {
                                name: "inner".to_string(),
                                position: pos(6, 5),
                            },
                            value: Expression::BinaryOp {
                                op: BinaryOp::Add,
                                left: Box::new(Expression::Identifier {
                                    name: "inner".to_string(),
                                    position: pos(6, 13),
                                }),
                                right: Box::new(Expression::IntLiteral {
                                    value: 1,
                                    position: pos(6, 21),
                                }),
                                position: pos(6, 19),
                            },
                            position: pos(6, 5),
                        },
                    ],
                    position: pos(4, 3),
                },
            ],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let total = vm.environment().get("total").unwrap();
    assert_eq!(total, Object::Int(5)); // outer=2: 2 iterations, outer=3: 3 iterations = 5 total
}

#[test]
fn test_control_flow_with_return() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::For {
            variable: "x".to_string(),
            iterable: Expression::Array {
                elements: vec![
                    Expression::IntLiteral {
                        value: 1,
                        position: pos(1, 14),
                    },
                    Expression::IntLiteral {
                        value: 2,
                        position: pos(1, 17),
                    },
                    Expression::IntLiteral {
                        value: 3,
                        position: pos(1, 20),
                    },
                ],
                position: pos(1, 13),
            },
            body: vec![Statement::If {
                condition: Expression::BinaryOp {
                    op: BinaryOp::Equal,
                    left: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(2, 6),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 2,
                        position: pos(2, 11),
                    }),
                    position: pos(2, 8),
                },
                then_branch: vec![Statement::Return {
                    value: Some(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(3, 12),
                    }),
                    position: pos(3, 5),
                }],
                else_branch: None,
                position: pos(2, 3),
            }],
            position: pos(1, 1),
        },
        Statement::Return {
            value: Some(Expression::IntLiteral {
                value: 999,
                position: pos(6, 8),
            }),
            position: pos(6, 1),
        },
    ];

    let result = vm.execute_program(&program).unwrap();
    assert_eq!(result, Some(Object::Int(2)));
}
