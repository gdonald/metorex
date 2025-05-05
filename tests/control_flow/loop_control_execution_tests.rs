// Unit tests for loop-control execution in the VM

use metorex::ast::{BinaryOp, Expression, Statement};
use metorex::lexer::Position;
use metorex::object::Object;
use metorex::vm::VirtualMachine;

// Helper function to create a test position
fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}

// Tests for Break statement

#[test]
fn test_break_in_while_loop() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        // x = 0
        Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        // while true
        Statement::While {
            condition: Expression::BoolLiteral {
                value: true,
                position: pos(2, 7),
            },
            body: vec![
                // x = x + 1
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(3, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(3, 7),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 1,
                            position: pos(3, 11),
                        }),
                        position: pos(3, 9),
                    },
                    position: pos(3, 3),
                },
                // if x >= 5: break
                Statement::If {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::GreaterEqual,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
                            position: pos(4, 6),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 5,
                            position: pos(4, 11),
                        }),
                        position: pos(4, 8),
                    },
                    then_branch: vec![Statement::Break {
                        position: pos(5, 5),
                    }],
                    elsif_branches: vec![],
                    else_branch: None,
                    position: pos(4, 3),
                },
            ],
            position: pos(2, 1),
        },
    ];

    vm.execute_program(&program).unwrap();
    assert_eq!(vm.environment().get("x"), Some(Object::Int(5)));
}

#[test]
fn test_continue_in_while_loop() {
    let mut vm = VirtualMachine::new();

    let program = vec![
        // x = 0
        Statement::Assignment {
            target: Expression::Identifier {
                name: "x".to_string(),
                position: pos(1, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(1, 5),
            },
            position: pos(1, 1),
        },
        // count = 0
        Statement::Assignment {
            target: Expression::Identifier {
                name: "count".to_string(),
                position: pos(2, 1),
            },
            value: Expression::IntLiteral {
                value: 0,
                position: pos(2, 9),
            },
            position: pos(2, 1),
        },
        // while x < 10
        Statement::While {
            condition: Expression::BinaryOp {
                op: BinaryOp::Less,
                left: Box::new(Expression::Identifier {
                    name: "x".to_string(),
                    position: pos(3, 7),
                }),
                right: Box::new(Expression::IntLiteral {
                    value: 10,
                    position: pos(3, 11),
                }),
                position: pos(3, 9),
            },
            body: vec![
                // x = x + 1
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "x".to_string(),
                        position: pos(4, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "x".to_string(),
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
                // if x % 2 == 0: continue
                Statement::If {
                    condition: Expression::BinaryOp {
                        op: BinaryOp::Equal,
                        left: Box::new(Expression::BinaryOp {
                            op: BinaryOp::Modulo,
                            left: Box::new(Expression::Identifier {
                                name: "x".to_string(),
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
                        position: pos(5, 12),
                    },
                    then_branch: vec![Statement::Continue {
                        position: pos(6, 5),
                    }],
                    elsif_branches: vec![],
                    else_branch: None,
                    position: pos(5, 3),
                },
                // count = count + 1
                Statement::Assignment {
                    target: Expression::Identifier {
                        name: "count".to_string(),
                        position: pos(7, 3),
                    },
                    value: Expression::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(Expression::Identifier {
                            name: "count".to_string(),
                            position: pos(7, 11),
                        }),
                        right: Box::new(Expression::IntLiteral {
                            value: 1,
                            position: pos(7, 19),
                        }),
                        position: pos(7, 17),
                    },
                    position: pos(7, 3),
                },
            ],
            position: pos(3, 1),
        },
    ];

    vm.execute_program(&program).unwrap();
    // count should only increment on odd numbers (1,3,5,7,9) = 5 times
    assert_eq!(vm.environment().get("count"), Some(Object::Int(5)));
}
