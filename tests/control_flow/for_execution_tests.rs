// Unit tests for for loop control flow execution in the VM

use metorex::ast::{BinaryOp, Expression, Statement};
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

// Tests for For loops

#[test]
fn test_for_loop_over_array() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "sum".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 7),
            },
            position: pos(1, 1),
        },
        Statement::For {
            variable: "x".to_string(),
            iterable: Expression::Array {
                elements: vec![
                    Expression::IntLiteral {
                        value: 1,
                        position: pos(2, 14),
                    },
                    Expression::IntLiteral {
                        value: 2,
                        position: pos(2, 17),
                    },
                    Expression::IntLiteral {
                        value: 3,
                        position: pos(2, 20),
                    },
                    Expression::IntLiteral {
                        value: 4,
                        position: pos(2, 23),
                    },
                ],
                position: pos(2, 13),
            },
            body: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "sum".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::BinaryOp {
                    op: BinaryOp::Add,
                    left: Box::new(Expression::Identifier {
                        name: "sum".to_string(),
                        position: pos(3, 9),
                    }),
                    right: Box::new(Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(3, 15),
                    }),
                    position: pos(3, 13),
                },
                position: pos(3, 3),
            }],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let sum = vm.environment().get("sum").unwrap();
    assert_eq!(sum, Object::Int(10)); // 1+2+3+4 = 10
}

#[test]
fn test_for_loop_over_empty_array() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "count".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 9),
            },
            position: pos(1, 1),
        },
        Statement::For {
            variable: "x".to_string(),
            iterable: Expression::Array {
                elements: vec![],
                position: pos(2, 13),
            },
            body: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "count".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::BinaryOp {
                    op: BinaryOp::Add,
                    left: Box::new(Expression::Identifier {
                        name: "count".to_string(),
                        position: pos(3, 11),
                    }),
                    right: Box::new(Expression::IntLiteral {
                        value: 1,
                        position: pos(3, 19),
                    }),
                    position: pos(3, 17),
                },
                position: pos(3, 3),
            }],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let count = vm.environment().get("count").unwrap();
    assert_eq!(count, Object::Int(0)); // Should remain 0
}

#[test]
fn test_for_loop_with_break() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 10),
            },
            position: pos(1, 1),
        },
        Statement::For {
            variable: "x".to_string(),
            iterable: Expression::Array {
                elements: vec![
                    Expression::IntLiteral {
                        value: 1,
                        position: pos(2, 14),
                    },
                    Expression::IntLiteral {
                        value: 2,
                        position: pos(2, 17),
                    },
                    Expression::IntLiteral {
                        value: 3,
                        position: pos(2, 20),
                    },
                    Expression::IntLiteral {
                        value: 4,
                        position: pos(2, 23),
                    },
                    Expression::IntLiteral {
                        value: 5,
                        position: pos(2, 26),
                    },
                ],
                position: pos(2, 13),
            },
            body: vec![
                Statement::If {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::Equal,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(3, 6),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 3,
                            position: pos(3, 11),
                        }),
                        position: pos(3, 8),
                    },
                    then_branch: vec![Statement::Break {
                        position: pos(4, 5),
                    }],
                    else_branch: None,
                    position: pos(3, 3),
                },
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "result".to_string(),
                        position: pos(6, 3),
                    },
                    value: Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(6, 12),
                    },
                    position: pos(6, 3),
                },
            ],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::Int(2)); // Last value before break
}

#[test]
fn test_for_loop_with_continue() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "sum".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 7),
            },
            position: pos(1, 1),
        },
        Statement::For {
            variable: "x".to_string(),
            iterable: Expression::Array {
                elements: vec![
                    Expression::IntLiteral {
                        value: 1,
                        position: pos(2, 14),
                    },
                    Expression::IntLiteral {
                        value: 2,
                        position: pos(2, 17),
                    },
                    Expression::IntLiteral {
                        value: 3,
                        position: pos(2, 20),
                    },
                    Expression::IntLiteral {
                        value: 4,
                        position: pos(2, 23),
                    },
                    Expression::IntLiteral {
                        value: 5,
                        position: pos(2, 26),
                    },
                ],
                position: pos(2, 13),
            },
            body: vec![
                Statement::If {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::Equal,
                        left: Box::new(Expression::BinaryOp {
                            op: BinaryOp::Modulo,
                            left: Box::new(Expression::Identifier {
                                name: "x".to_string(),
                                position: pos(3, 6),
                            }),
                            right: Box::new(Expression::IntLiteral {
                                value: 2,
                                position: pos(3, 10),
                            }),
                            position: pos(3, 8),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 0,
                            position: pos(3, 15),
                        }),
                        position: pos(3, 13),
                    },
                    then_branch: vec![Statement::Continue {
                        position: pos(4, 5),
                    }],
                    else_branch: None,
                    position: pos(3, 3),
                },
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "sum".to_string(),
                        position: pos(6, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "sum".to_string(),
                            position: pos(6, 9),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(6, 15),
                        }),
                        position: pos(6, 13),
                    },
                    position: pos(6, 3),
                },
            ],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let sum = vm.environment().get("sum").unwrap();
    assert_eq!(sum, Object::Int(9)); // 1+3+5 = 9 (skips even numbers)
}

#[test]
fn test_for_loop_variable_scope() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 100,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::For {
            variable: "x".to_string(),
            iterable: Expression::Array {
                elements: vec![Expression::IntLiteral {
                    value: 42,
                    position: pos(2, 14),
                }],
                position: pos(2, 13),
            },
            body: vec![Statement::Expression {
                expression: Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(3, 3),
                },
                position: pos(3, 3),
            }],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let x = vm.environment().get("x").unwrap();
    assert_eq!(x, Object::Int(100)); // Should retain original value after loop
}

#[test]
fn test_nested_for_loops() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "result".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 10),
            },
            position: pos(1, 1),
        },
        Statement::For {
            variable: "i".to_string(),
            iterable: Expression::Array {
                elements: vec![
                    Expression::IntLiteral {
                        value: 1,
                        position: pos(2, 14),
                    },
                    Expression::IntLiteral {
                        value: 2,
                        position: pos(2, 17),
                    },
                ],
                position: pos(2, 13),
            },
            body: vec![Statement::For {
                variable: "j".to_string(),
                iterable: Expression::Array {
                    elements: vec![
                        Expression::IntLiteral {
                            value: 10,
                            position: pos(3, 16),
                        },
                        Expression::IntLiteral {
                            value: 20,
                            position: pos(3, 20),
                        },
                    ],
                    position: pos(3, 15),
                },
                body: vec![Statement::Assignment {
                    target: Expression::Identifier {
                        name: "result".to_string(),
                        position: pos(4, 5),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "result".to_string(),
                            position: pos(4, 14),
                        }),
                        right: Box::new(Expression::BinaryOp {
                            op: BinaryOp::Add,
                            left: Box::new(Expression::Identifier {
                                name: "i".to_string(),
                                position: pos(4, 23),
                            }),
                            right: Box::new(Expression::Identifier {
                                name: "j".to_string(),
                                position: pos(4, 27),
                            }),
                            position: pos(4, 25),
                        }),
                        position: pos(4, 21),
                    },
                    position: pos(4, 5),
                }],
                position: pos(3, 3),
            }],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let result = vm.environment().get("result").unwrap();
    assert_eq!(result, Object::Int(66)); // (1+10)+(1+20)+(2+10)+(2+20) = 11+21+12+22 = 66
}

#[test]
fn test_for_loop_error_on_non_iterable() {
    let mut vm = VirtualMachine::new();

    let program = vec![Statement::For {
        variable: "x".to_string(),
        iterable: Expression::IntLiteral {
            value: 42,
            position: pos(1, 13),
        },
        body: vec![Statement::Expression {
            expression: Expression::Identifier {
                name: "x".to_string(),
                position: pos(2, 3),
            },
            position: pos(2, 3),
        }],
        position: pos(1, 1),
    }];

    let result = vm.execute_program(&program);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Cannot iterate"));
}
