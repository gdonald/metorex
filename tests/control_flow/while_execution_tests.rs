// Unit tests for while loop control flow execution in the VM

use metorex::ast::{BinaryOp, Expression, Statement};
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

// Tests for While loops

#[test]
fn test_while_loop_basic() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "i".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 5),
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
        Statement::While {
            condition: Expression::BinaryOp {
                op: BinaryOp::Less,
                left: Box::new(Expression::Identifier {
                    name: "i".to_string(),
                    position: pos(3, 7),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 5,
                    position: pos(3, 11),
                }),
                position: pos(3, 9),
            },
            body: vec![
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "sum".to_string(),
                        position: pos(4, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "sum".to_string(),
                            position: pos(4, 9),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "i".to_string(),
                            position: pos(4, 15),
                        }),
                        position: pos(4, 13),
                    },
                    position: pos(4, 3),
                },
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "i".to_string(),
                        position: pos(5, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "i".to_string(),
                            position: pos(5, 7),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 1,
                            position: pos(5, 11),
                        }),
                        position: pos(5, 9),
                    },
                    position: pos(5, 3),
                },
            ],
            position: pos(3, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let sum = vm.environment().get("sum").unwrap();
    assert_eq!(sum, Object::Int(10)); // 0+1+2+3+4 = 10
}

#[test]
fn test_while_loop_no_iterations() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "counter".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 11),
            },
            position: pos(1, 1),
        },
        Statement::While {
            condition: Expression::BoolLiteral {
                value: false,
                position: pos(2, 7),
            },
            body: vec![Statement::Assignment {
                target: Expression::Identifier {
                    name: "counter".to_string(),
                    position: pos(3, 3),
                },
                value: Expression::IntLiteral {
                    value: 42,
                    position: pos(3, 13),
                },
                position: pos(3, 3),
            }],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let counter = vm.environment().get("counter").unwrap();
    assert_eq!(counter, Object::Int(0)); // Should remain 0
}

#[test]
fn test_while_loop_with_break() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "i".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        Statement::While {
            condition: Expression::BoolLiteral {
                value: true,
                position: pos(2, 7),
            },
            body: vec![
                Statement::If {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::GreaterEqual,
                        left: Box::new(Expression::Identifier {
                            name: "i".to_string(),
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
                        name: "i".to_string(),
                        position: pos(6, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "i".to_string(),
                            position: pos(6, 7),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 1,
                            position: pos(6, 11),
                        }),
                        position: pos(6, 9),
                    },
                    position: pos(6, 3),
                },
            ],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let i = vm.environment().get("i").unwrap();
    assert_eq!(i, Object::Int(3));
}

#[test]
fn test_while_loop_with_continue() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        Statement::Assignment {
            target: Expression::Identifier {
                name: "i".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 5),
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
        Statement::While {
            condition: Expression::BinaryOp {
                op: BinaryOp::Less,
                left: Box::new(Expression::Identifier {
                    name: "i".to_string(),
                    position: pos(3, 7),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 5,
                    position: pos(3, 11),
                }),
                position: pos(3, 9),
            },
            body: vec![
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "i".to_string(),
                        position: pos(4, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "i".to_string(),
                            position: pos(4, 7),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 1,
                            position: pos(4, 11),
                        }),
                        position: pos(4, 9),
                    },
                    position: pos(4, 3),
                },
                Statement::If {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::Equal,
                        left: Box::new(Expression::BinaryOp {
                            op: BinaryOp::Modulo,
                            left: Box::new(Expression::Identifier {
                                name: "i".to_string(),
                                position: pos(5, 6),
                            }),
                            right: Box::new(Expression::IntLiteral {
                                value: 2,
                                position: pos(5, 10),
                            }),
                            position: pos(5, 8),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 0,
                            position: pos(5, 15),
                        }),
                        position: pos(5, 13),
                    },
                    then_branch: vec![Statement::Continue {
                        position: pos(6, 5),
                    }],
                    else_branch: None,
                    position: pos(5, 3),
                },
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "sum".to_string(),
                        position: pos(8, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "sum".to_string(),
                            position: pos(8, 9),
                        }),
                        right: Box::new(Expression::Identifier {
                            name: "i".to_string(),
                            position: pos(8, 15),
                        }),
                        position: pos(8, 13),
                    },
                    position: pos(8, 3),
                },
            ],
            position: pos(3, 1),
        },
    ];

    vm.execute_program(&program).unwrap();

    let sum = vm.environment().get("sum").unwrap();
    assert_eq!(sum, Object::Int(9)); // 1+3+5 = 9 (skips even numbers)
}
